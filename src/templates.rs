use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use crate::tokens::{parse_template, parse_signal, Signal, Entry, Target, SymbolType};

#[pyclass(get_all, set_all)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ParsedSignal {
    pub symbol: Option<String>,
    pub action: Option<String>,
    pub entry_prices: Vec<f32>,
    pub stoploss: Option<f32>,
    pub takeprofits: Vec<f32>,
    pub timeframe: Option<String>,
    pub expiration: Option<String>,
    pub is_binary: bool,
    pub template_name: Option<String>,
    pub matched_pattern: Option<String>,
}

#[pymethods]
impl ParsedSignal {
    #[new]
    #[pyo3(signature = (symbol=None, action=None, entry_prices=None, stoploss=None, takeprofits=None, timeframe=None, expiration=None, is_binary=false, template_name=None, matched_pattern=None))]
    pub fn new(
        symbol: Option<String>,
        action: Option<String>,
        entry_prices: Option<Vec<f32>>,
        stoploss: Option<f32>,
        takeprofits: Option<Vec<f32>>,
        timeframe: Option<String>,
        expiration: Option<String>,
        is_binary: bool,
        template_name: Option<String>,
        matched_pattern: Option<String>,
    ) -> Self {
        Self {
            symbol,
            action,
            entry_prices: entry_prices.unwrap_or_default(),
            stoploss,
            takeprofits: takeprofits.unwrap_or_default(),
            timeframe,
            expiration,
            is_binary,
            template_name,
            matched_pattern,
        }
    }

    pub fn to_dict(&self, py: Python<'_>) -> PyResult<PyObject> {
        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("symbol", &self.symbol)?;
        dict.set_item("action", &self.action)?;
        dict.set_item("entry_prices", &self.entry_prices)?;
        dict.set_item("stoploss", self.stoploss)?;
        dict.set_item("takeprofits", &self.takeprofits)?;
        dict.set_item("timeframe", &self.timeframe)?;
        dict.set_item("expiration", &self.expiration)?;
        dict.set_item("is_binary", self.is_binary)?;
        dict.set_item("template_name", &self.template_name)?;
        dict.set_item("matched_pattern", &self.matched_pattern)?;
        Ok(dict.into())
    }

    fn __repr__(&self) -> String {
        format!(
            "ParsedSignal(symbol={:?}, action={:?}, entry={:?}, sl={:?}, tp={:?}, tf={:?}, binary={})",
            self.symbol, self.action, self.entry_prices, self.stoploss, self.takeprofits, self.timeframe, self.is_binary
        )
    }
}

impl ParsedSignal {
    pub fn from_signal(sig: &Signal, is_binary: bool, tpl_name: Option<String>, pattern: Option<String>) -> Self {
        let mut symbol = None;
        if let Some(ref s) = sig.symbol {
            symbol = match &s.symbol_type {
                SymbolType::Crypto(v)
                | SymbolType::Forex(v)
                | SymbolType::Stock(v)
                | SymbolType::Index(v)
                | SymbolType::Etf(v)
                | SymbolType::Funds(v)
                | SymbolType::MoneyMarkets(v)
                | SymbolType::Plain(v) => Some(v.clone()),
                SymbolType::Empty(v) => {
                    if v.is_empty() { None } else { Some(v.clone()) }
                }
            };
        }

        let action = sig.action.map(|a| if a { "BUY".to_string() } else { "SELL".to_string() });

        let mut entry_prices = Vec::new();
        let mut expiration = None;
        if let Some(ref entry) = sig.entry {
            match entry {
                Entry::PriceEntry(p) => entry_prices.push(*p),
                Entry::EntryRange(p1, p2) => {
                    entry_prices.push(*p1);
                    entry_prices.push(*p2);
                }
                Entry::TimeEntry(_, t) => expiration = Some(t.clone()),
                Entry::TimePrice(_, p) => entry_prices.push(*p),
            }
        }

        let mut stoploss = None;
        let mut takeprofits = Vec::new();
        let mut timeframe = None;

        if let Some(ref target) = sig.target {
            match target {
                Target::PeriodTime(pt) => timeframe = Some(pt.clone()),
                Target::ProfitLoss(pl) => {
                    if pl.stoploss > 0.0 {
                        stoploss = Some(pl.stoploss);
                    }
                    for p in &pl.profits {
                        takeprofits.push(*p);
                    }
                }
            }
        }

        Self {
            symbol,
            action,
            entry_prices,
            stoploss,
            takeprofits,
            timeframe,
            expiration,
            is_binary,
            template_name: tpl_name,
            matched_pattern: pattern,
        }
    }
}

/// Matches a raw text against a given template pattern syntax
#[pyfunction]
#[pyo3(signature = (text, pattern, template_name=None))]
pub fn match_template(text: &str, pattern: &str, template_name: Option<&str>) -> Option<ParsedSignal> {
    if pattern.trim().is_empty() {
        return None;
    }

    if let Some(matched_signals) = parse_template(text, pattern, None, None, None) {
        if let Some(first_sig) = matched_signals.first() {
            let is_binary = first_sig.is_binary();
            return Some(ParsedSignal::from_signal(
                first_sig,
                is_binary,
                template_name.map(|s| s.to_string()),
                Some(pattern.to_string()),
            ));
        }
    }

    // Fallback: if template matching was not strict, try parse_signal to see if structure is valid
    if let Some(signals) = parse_signal(text, None, None) {
        if let Some(first_sig) = signals.first() {
            if first_sig.is_esencial() {
                let is_binary = signals.is_binary();
                return Some(ParsedSignal::from_signal(
                    first_sig,
                    is_binary,
                    template_name.map(|s| s.to_string()),
                    Some(signals.template.clone()),
                ));
            }
        }
    }

    None
}
