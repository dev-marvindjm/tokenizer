use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use crate::matchers::{Tokens, ACTION_VAL_D, ACTION_VAL_U};
use crate::tokens::parse_signal;

#[pyclass(get_all, set_all)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub kind: String,
    pub value: String,
    pub start: usize,
    pub end: usize,
    pub line: usize,
}

#[pymethods]
impl Token {
    #[new]
    #[pyo3(signature = (kind, value, start, end, line))]
    pub fn new(kind: String, value: String, start: usize, end: usize, line: usize) -> Self {
        Self {
            kind,
            value,
            start,
            end,
            line,
        }
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("kind", &self.kind)?;
        dict.set_item("value", &self.value)?;
        dict.set_item("start", self.start)?;
        dict.set_item("end", self.end)?;
        dict.set_item("line", self.line)?;
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "Token(kind='{}', value='{}', start={}, end={}, line={})",
            self.kind, self.value, self.start, self.end, self.line
        )
    }
}

/// Tokenizes text into a stream of normalized trading tokens
pub fn tokenize_text_internal(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let parsed_signals = parse_signal(text, None, None);

    // Keep track of extracted signal components for semantic tagging
    let mut extracted_symbol: Option<String> = None;
    let mut extracted_action: Option<bool> = None;
    let mut extracted_entry_prices: Vec<f32> = Vec::new();
    let mut extracted_sl: Option<f32> = None;
    let mut extracted_tps: Vec<f32> = Vec::new();
    let mut extracted_timeframe: Option<String> = None;
    let mut extracted_time: Option<String> = None;

    if let Some(ref sigs) = parsed_signals {
        for sig in sigs.iter() {
            if let Some(ref s) = sig.symbol {
                match &s.symbol_type {
                    crate::tokens::SymbolType::Crypto(v)
                    | crate::tokens::SymbolType::Forex(v)
                    | crate::tokens::SymbolType::Stock(v)
                    | crate::tokens::SymbolType::Index(v)
                    | crate::tokens::SymbolType::Etf(v)
                    | crate::tokens::SymbolType::Funds(v)
                    | crate::tokens::SymbolType::MoneyMarkets(v)
                    | crate::tokens::SymbolType::Plain(v) => {
                        extracted_symbol = Some(v.to_uppercase());
                    }
                    _ => {}
                }
            }
            if sig.action.is_some() {
                extracted_action = sig.action;
            }
            if let Some(ref entry) = sig.entry {
                match entry {
                    crate::tokens::Entry::PriceEntry(p) => extracted_entry_prices.push(*p),
                    crate::tokens::Entry::EntryRange(p1, p2) => {
                        extracted_entry_prices.push(*p1);
                        extracted_entry_prices.push(*p2);
                    }
                    crate::tokens::Entry::TimeEntry(_, t) => extracted_time = Some(t.clone()),
                    crate::tokens::Entry::TimePrice(_, p) => extracted_entry_prices.push(*p),
                }
            }
            if let Some(ref target) = sig.target {
                match target {
                    crate::tokens::Target::PeriodTime(pt) => extracted_timeframe = Some(pt.clone()),
                    crate::tokens::Target::ProfitLoss(pl) => {
                        if pl.stoploss > 0.0 {
                            extracted_sl = Some(pl.stoploss);
                        }
                        for p in &pl.profits {
                            extracted_tps.push(*p);
                        }
                    }
                }
            }
        }
    }

    let mut char_offset = 0;
    for (line_idx, line) in text.lines().enumerate() {
        let line_len = line.len();
        let mut col_offset = 0;

        for (pattern_name, token_val) in line.iter_tokens() {
            let tok_len = token_val.len();
            let start = char_offset + col_offset;
            let end = start + tok_len;
            col_offset += tok_len;

            let val_trimmed = token_val.trim();
            if val_trimmed.is_empty() {
                continue;
            }

            let upper = val_trimmed.to_uppercase();

            // Semantic classification
            let kind = if let Some(ref sym) = extracted_symbol {
                if upper == *sym || upper == sym.replace('/', "") || upper.contains(sym) {
                    "SYMBOL"
                } else {
                    classify_word(&upper, pattern_name, &extracted_entry_prices, extracted_sl, &extracted_tps, extracted_timeframe.as_deref(), extracted_time.as_deref())
                }
            } else {
                classify_word(&upper, pattern_name, &extracted_entry_prices, extracted_sl, &extracted_tps, extracted_timeframe.as_deref(), extracted_time.as_deref())
            };

            tokens.push(Token {
                kind: kind.to_string(),
                value: token_val,
                start,
                end,
                line: line_idx,
            });
        }

        // Account for \n or \r\n line ending
        char_offset += line_len;
        if text.as_bytes().get(char_offset) == Some(&b'\r') {
            char_offset += 1;
        }
        if text.as_bytes().get(char_offset) == Some(&b'\n') {
            char_offset += 1;
        }
    }

    tokens
}

fn classify_word(
    upper: &str,
    pattern_name: &str,
    entry_prices: &[f32],
    sl: Option<f32>,
    tps: &[f32],
    timeframe: Option<&str>,
    time: Option<&str>,
) -> &'static str {
    // Action checks
    if ACTION_VAL_U.contains(upper) || matches!(upper, "BUY" | "CALL" | "COMPRA" | "LONG" | "UP") {
        return "ACTION_BUY";
    }
    if ACTION_VAL_D.contains(upper) || matches!(upper, "SELL" | "PUT" | "VENTA" | "SHORT" | "DOWN") {
        return "ACTION_SELL";
    }

    // SL / TP labels
    if matches!(upper, "SL" | "STOPLOSS" | "STOP" | "STOP_LOSS") {
        return "SL";
    }
    if upper.starts_with("TP") || matches!(upper, "TARGET" | "TAKEPROFIT" | "TAKE_PROFIT") {
        return "TP";
    }

    // Entry labels
    if matches!(upper, "ENTRY" | "EP" | "PRICE" | "PRECIO" | "ENTRADA" | "@") {
        return "PRICE_ENTRY";
    }

    // Check specific extracted values
    if let Ok(val) = upper.parse::<f32>() {
        if let Some(stoploss) = sl {
            if (val - stoploss).abs() < 0.0001 {
                return "SL";
            }
        }
        if tps.iter().any(|tp| (val - *tp).abs() < 0.0001) {
            return "TP";
        }
        if entry_prices.iter().any(|ep| (val - *ep).abs() < 0.0001) {
            return "PRICE_ENTRY";
        }
    }

    // Timeframe / Expiration
    if let Some(tf) = timeframe {
        if upper == tf {
            return "TIMEFRAME";
        }
    }
    if let Some(t) = time {
        if upper == t {
            return "EXPIRATION";
        }
    }

    // Timeframe / Expiration
    if upper.starts_with('M') && upper[1..].chars().all(|c| c.is_ascii_digit()) && upper.len() > 1 {
        return "TIMEFRAME";
    }
    if upper.starts_with('H') && upper[1..].chars().all(|c| c.is_ascii_digit()) && upper.len() > 1 {
        return "TIMEFRAME";
    }
    if upper.ends_with("MIN") || upper.ends_with('M') || upper.ends_with('H') || upper.ends_with('D') {
        let prefix = &upper[..upper.len() - if upper.ends_with("MIN") { 3 } else { 1 }];
        if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
            return "TIMEFRAME";
        }
    }

    // Gale
    if upper.starts_with('G') && upper[1..].chars().all(|c| c.is_ascii_digit()) || matches!(upper, "GALE" | "MG" | "MARTINGALE") {
        return "GALE";
    }

    // Pattern based classification
    match pattern_name {
        "target_time_val" | "target_time_val_clean" => "EXPIRATION",
        "timeframe_digits_period_val" | "timeframe_period_digits_val" | "timeframe_period_parser_name_val" => "TIMEFRAME",
        "target_price_val" | "target_price_range_val" => "PRICE",
        "date_val" => "DATE",
        "emoji" => "EMOJI",
        "punctuation" => "PUNCTUATION",
        "digit" => "DIGIT",
        _ => {
            if upper.chars().all(|c| c.is_ascii_digit() || c == '.' || c == ',') && upper.chars().any(|c| c.is_ascii_digit()) {
                "PRICE"
            } else {
                "TEXT"
            }
        }
    }
}

#[pyfunction]
pub fn tokenize_text(text: &str) -> Vec<Token> {
    tokenize_text_internal(text)
}
