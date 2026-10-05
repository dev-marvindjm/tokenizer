use serde::{Serialize, Deserialize};
// use pyo3::prelude::*;
// use pyo3::types::PyType;
use crate::matchers::{Tokens, Nodes, TokenFlags, TokenClass, ACTION_VAL_U, ACTION_VAL_D, get_flags_and_token_class, get_nodes, check_category};

#[allow(unsafe_op_in_unsafe_fn)]

// #[pyclass(get_all, set_all)]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PricesTarget {
    pub open: bool,
    pub profits: Vec<f32>,
    pub stoploss: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Entry {
    EntryRange(f32, f32), // price, price
    PriceEntry(f32),    // price
    TimeEntry(String, String), // Timezone, EntryTime
    TimePrice(String, f32),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Gale {
    TimeEntry(String), // Timezone, EntryTime
    Number(i8),
    NoGale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Target {
    PeriodTime(String), // M5, H1, D
    ProfitLoss(PricesTarget),
}

// #[pyclass]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SymbolClass {
    Perpetual,
    Otc,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SymbolType {
    Crypto(String),
    Etf(String),
    Forex(String),
    Funds(String),
    Index(String),
    MoneyMarkets(String),
    Stock(String),
    Plain(String),
    Empty(String),
}

// #[pyclass]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Symbol {
    pub symbol_type: SymbolType,
    pub symbol_class: Option<SymbolClass>,
}

// #[pyclass]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    // #[pyo3(get, set)]
    pub action: Option<bool>,
    pub target: Option<Target>,
    pub symbol: Option<Symbol>,
    pub entry: Option<Entry>,
    #[serde(default)]
    pub gales: Vec<Gale>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Signals {
    pub template: String,
    pub signals: Vec<Signal>,
    /// Índice de la línea (0-based) de la primera ocurrencia del token symbol
    pub line_index: usize,
    /// Byte offset donde inicia el primer carácter del símbolo en el texto original
    pub token_start: usize,
    /// Carácter que va inmediatamente después del símbolo
    pub end_char: char,
}

impl std::ops::Deref for Signals {
    type Target = Vec<Signal>;
    fn deref(&self) -> &Self::Target {
        &self.signals
    }
}

impl Signals {
    /// Returns true if this signal collection represents binary signals, false if market signals.
    pub fn is_binary(&self) -> bool {
        self.signals.iter().any(|s| s.is_binary())
    }

    /// Returns "binary" if the signal is binary, or "market" if it's a market signal.
    pub fn template_type(&self) -> &'static str {
        if self.is_binary() {
            "binary"
        } else {
            "market"
        }
    }
}

impl PricesTarget {
    pub fn is_complete(&self) -> bool {
        self.open || !self.profits.is_empty()
    }
}

impl Entry {
    pub fn is_complete(&self) -> bool {
        match self {
            Entry::EntryRange(p1, p2) => *p1 != 0.0 && *p2 != 0.0,
            Entry::PriceEntry(p) => *p != 0.0,
            Entry::TimeEntry(_tz, t) => !t.is_empty(), // !_tz.is_empty() && !t.is_empty(),
            Entry::TimePrice(_tz, p) => *p != 0.0,
        }
    }

    /// Returns true if this Entry variant requires checking/verifying market price.
    /// `TimeEntry` does NOT depend on price. `EntryRange`, `PriceEntry`, and `TimePrice` DO depend on price.
    pub fn requires_price(&self) -> bool {
        match self {
            Entry::TimeEntry(_, _) => false,
            Entry::EntryRange(_, _) | Entry::PriceEntry(_) | Entry::TimePrice(_, _) => true,
        }
    }

    /// Returns the target entry price if this entry specifies a single price.
    pub fn target_price(&self) -> Option<f32> {
        match self {
            Entry::PriceEntry(p) => Some(*p),
            Entry::TimePrice(_, p) => Some(*p),
            _ => None,
        }
    }

    /// Returns (min_price, max_price) if this entry is an EntryRange.
    pub fn price_range(&self) -> Option<(f32, f32)> {
        match self {
            Entry::EntryRange(p1, p2) => Some((p1.min(*p2), p1.max(*p2))),
            _ => None,
        }
    }

    /// Verifies if a current market price satisfies this Entry condition.
    /// - `TimeEntry`: Always true (time-based only, does not depend on price).
    /// - `EntryRange(p1, p2)`: Checks if `min - tol <= current_price <= max + tol`.
    /// - `PriceEntry(p)` / `TimePrice(_, p)`: Checks if `current_price` satisfies `p` within tolerance,
    ///   or considering direction (e.g. for action=Some(true) buy, current <= p + tol; for action=Some(false) sell, current >= p - tol).
    pub fn is_price_valid(&self, current_price: f32, action: Option<bool>, tolerance: Option<f32>) -> bool {
        let tol = tolerance.unwrap_or(0.0001);
        match self {
            Entry::TimeEntry(_, _) => true,
            Entry::EntryRange(p1, p2) => {
                let min = p1.min(*p2) - tol;
                let max = p1.max(*p2) + tol;
                current_price >= min && current_price <= max
            }
            Entry::PriceEntry(target) | Entry::TimePrice(_, target) => {
                match action {
                    Some(true) => {
                        // Buy: Price at or below target + tolerance (good entry price)
                        current_price <= *target + tol
                    }
                    Some(false) => {
                        // Sell: Price at or above target - tolerance (good entry price)
                        current_price >= *target - tol
                    }
                    None => {
                        (current_price - *target).abs() <= tol
                    }
                }
            }
        }
    }
}

impl Target {
    pub fn is_complete(&self) -> bool {
        match self {
            Target::PeriodTime(s) => !s.is_empty(),
            Target::ProfitLoss(p) => p.is_complete(),
        }
    }
}

impl SymbolClass {
    pub fn is_complete(&self) -> bool {
        true
    }
}

impl SymbolType {
    pub fn is_complete(&self) -> bool {
        match self {
            SymbolType::Crypto(s) |
            SymbolType::Etf(s) |
            SymbolType::Forex(s) |
            SymbolType::Funds(s) |
            SymbolType::Index(s) |
            SymbolType::MoneyMarkets(s) |
            SymbolType::Plain(s) |
            SymbolType::Stock(s) => !s.is_empty(),
            SymbolType::Empty(_) => false,
        }
    }
}

impl Symbol {
    pub fn is_complete(&self) -> bool {
        self.symbol_type.is_complete()
    }
}

impl Signal {
    pub fn is_complete(&self) -> bool {
        self.action.is_some() 
            && self.target.as_ref().map_or(false, |t| t.is_complete())
            && self.symbol.as_ref().map_or(false, |s| s.is_complete())
            && self.entry.as_ref().map_or(false, |e| e.is_complete())
    }
    pub fn is_esencial(&self) -> bool {
        self.action.is_some() 
            && self.symbol.as_ref().map_or(false, |s| s.is_complete())
            && self.entry.as_ref().map_or(false, |e| e.is_complete())
    }
    pub fn copy_from(&mut self,signal:&Self) -> bool {
        if signal.target.is_some() {
            if let Some(tg) = &signal.target {
                if let Some(ref mut target) = self.target {
                    match (target,tg) {
                        (Target::PeriodTime(timeframe), Target::PeriodTime(tf)) => { *timeframe=tf.clone();},
                        _ => {},
                    }
                } else {
                    match tg {
                        Target::PeriodTime(tf) => { self.target = Some(Target::PeriodTime(tf.to_string()));},
                        _ => {},
                    }
                }
            }
        }
        if signal.entry.is_some() {
            if let Some(en) = &signal.entry {
                if let Some(ref mut entry) = self.entry {
                    match (entry,en) {
                        (Entry::TimeEntry(tz, _), Entry::TimeEntry(etz, _)) => { *tz=etz.clone(); },
                        (Entry::TimePrice(tz, _), Entry::TimePrice(etz, _)) => { *tz=etz.clone(); },
                        _ => {},
                    }
                } else {
                    match en {
                        Entry::TimeEntry(etz, _) => { self.entry = Some(Entry::TimeEntry(etz.clone(), String::new()));},
                        Entry::TimePrice(etz, _) => { self.entry = Some(Entry::TimePrice(etz.clone(), 0.0));},
                        _ => {},
                    }
                }
            }
        }
        self.is_complete()
    }
    pub fn get_timezone(&self) -> Option<&str> {
        match &self.entry {
            Some(Entry::TimeEntry(tz, _)) if !tz.is_empty() => Some(tz.as_str()),
            Some(Entry::TimePrice(tz, _)) if !tz.is_empty() => Some(tz.as_str()),
            _ => None,
        }
    }

    pub fn has_timezone(&self) -> bool {
        self.get_timezone().is_some()
    }

    pub fn add_time_zone(&mut self, timezone: String) {
        match &self.entry {
            Some(Entry::PriceEntry(p)) => self.entry = Some(Entry::TimePrice(timezone, *p)),
            Some(Entry::TimeEntry(tz, t)) if tz.is_empty() => self.entry = Some(Entry::TimeEntry(timezone, t.clone())),
            Some(Entry::TimePrice(tz, t)) if tz.is_empty() => self.entry = Some(Entry::TimePrice(timezone, *t)),
            None => self.entry = Some(Entry::TimeEntry(timezone, String::new())),
            _ => {},
        }
    }

    pub fn ensure_timezone(&mut self, timezone: &str) {
        if timezone.trim().is_empty() { return; }
        if !self.has_timezone() {
            self.add_time_zone(timezone.trim().to_string());
        }
    }
    pub fn add_timeframe(&mut self, timeframe: String) {
        if timeframe.is_empty() || timeframe.len() == 1 { return; }
        if !timeframe.chars().all(|c| c.is_alphabetic() || c.is_numeric()) {
            return;
        }
        match &self.target {
            None => self.target = Some(Target::PeriodTime(timeframe)),
            Some(Target::PeriodTime(tf)) if tf.is_empty() => self.target = Some(Target::PeriodTime(timeframe)),
            _ => {},
        }
    }

    pub fn is_binary(&self) -> bool {
        matches!(self.target, Some(Target::PeriodTime(_))) || !self.gales.is_empty()
    }

    /// Returns true if this signal's entry depends on market price verification (EntryRange, PriceEntry, TimePrice).
    pub fn requires_price_verification(&self) -> bool {
        self.entry.as_ref().map_or(false, |e| e.requires_price())
    }

    /// Verifies if a given market price satisfies the signal's entry condition.
    pub fn verify_price(&self, current_price: f32, tolerance: Option<f32>) -> bool {
        match &self.entry {
            Some(entry) => entry.is_price_valid(current_price, self.action, tolerance),
            None => true,
        }
    }
}

// #[pymethods]
impl Signal {
    // #[new]
    pub fn new() -> Self {
        Signal::default()
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    // #[classmethod]
    // pub fn from_json(_cls: &Bound<'_, PyType>, json: &str) -> Signal {
    //     serde_json::from_str(json).unwrap()
    // }
}

/// Maps a `TokenClass` to the label used for encapsulation in the text.
/// Labels use the format `field:Type` where `field` is the `Signal` struct
/// field name and `Type` is the enum variant that the token contributes to.
/// Returns `None` for classes that should not be encapsulated.
fn token_class_label(tc: &TokenClass) -> Option<&'static str> {
    match tc {
        // action: Option<bool>
        TokenClass::Action             => Some("action"),
        // target: Option<Target>
        TokenClass::Timeframe          => Some("period"),
        TokenClass::Otc                => Some("otc"),
        TokenClass::Target             => Some("target"),
        // entry: Option<Entry>  — timezone feeds into TimeEntry / TimePrice
        TokenClass::Timezone           => Some("timezone"),
        // symbol: Option<Symbol> — each TokenClass maps to a SymbolType variant
        TokenClass::SymbolCrypto      
        | TokenClass::SymbolForex       
        | TokenClass::SymbolFunds       
        | TokenClass::SymbolEtfs        
        | TokenClass::SymbolIndex       
        | TokenClass::SymbolMoney       
        | TokenClass::SymbolStocks      
        | TokenClass::SymbolCommodities
        | TokenClass::SymbolBonds
        | TokenClass::Symbol           => Some("symbol"),
        // These classes are not encapsulated
        TokenClass::Word
        | TokenClass::Indicator
        | TokenClass::Info
        | TokenClass::Result
        | TokenClass::Gale
        | TokenClass::Emoji
        | TokenClass::NewLine
        | TokenClass::Whitespace
        | TokenClass::Punctuation
        | TokenClass::Other            => None,
    }
}

// #[pyfunction]
#[allow(unused_variables, unused_assignments)]
pub fn parse_signal(texto: &str, custom_nodes: Option<&[Nodes]>, custom_symbols: Option<&[Nodes]>) -> Option<Signals> {
    let text: String = texto
        .lines() // Divide el texto en un iterador de líneas
        .filter(|line| !line.trim().is_empty()) // Descarta las vacías o con solo espacios
        .collect::<Vec<_>>() // Las junta en un vector
        .join("\n");

    // Build a mapping from each line's start byte in `text` back to its start byte
    // in the original `texto`, so positions can reference the original input.
    let orig_line_offsets: Vec<usize> = {
        let mut offsets = Vec::new();
        let mut pos = 0usize;
        for line in texto.lines() {
            if !line.trim().is_empty() {
                offsets.push(pos);
            }
            pos += line.len() + 1; // +1 for '\n' (or \r\n handled below)
        }
        // Handle \r\n line endings: subtract extra '\r' if present
        let mut corrected = Vec::new();
        let mut orig_pos = 0usize;
        for line in texto.lines() {
            if !line.trim().is_empty() {
                corrected.push(orig_pos);
            }
            // advance by the raw byte length of this line including its terminator
            orig_pos += line.len();
            // peek at the character(s) after this line in the original bytes
            let remaining = &texto.as_bytes()[orig_pos..];
            if remaining.starts_with(b"\r\n") {
                orig_pos += 2;
            } else if remaining.first().map_or(false, |&b| b == b'\n' || b == b'\r') {
                orig_pos += 1;
            }
        }
        corrected
    };

    let mut lines = text.lines().peekable();
    let mut signals = Vec::new();
    let mut signals_esencials:Vec<Signal> = Vec::new();
    let mut current_signal = Signal::default();
    let mut before_match_str = String::new();
    let mut before_tok_start: usize = 0;
    let mut before_tok_end: usize = 0;

    // Collect token replacements: (byte_start_in_texto, byte_end_in_texto, label)
    // Applied to `texto` in reverse order after parsing so that earlier offsets stay valid.
    let mut replacements: Vec<(usize, usize, &str)> = Vec::new();

    // Byte offset of the current line's start within the original `texto`.
    let mut line_index: usize = 0;
    // Byte offset within the current line at which the current token starts.
    let mut token_offset_in_line: usize = 0;
    // Byte offset (in original texto) of the first token that contributed to current_signal.
    // Byte offset (in original texto) past the last token that contributed to current_signal.
    let mut index_tp = 1;
    let mut current_target_type = "";

    // Tracking de la primera ocurrencia del token symbol
    let mut first_symbol_line: usize = 0;
    let mut first_symbol_start: usize = 0;
    let mut first_symbol_end_char: char = '\0';
    let mut found_first_symbol = false;

    let mut global_timezone:String = String::new();
    let mut global_timeframe:String = String::new();

    let mut line_contains_symbol = false;
    let mut line_contains_action = false;
    let mut line_contains_target = false;
    let mut signals_multiline = false;
    let mut nex_signals_multine = false;
    let nodes = match custom_nodes {
        Some(c_nodes) => {
            let mut n = c_nodes.to_vec();
            n.extend(get_nodes("checkers"));
            n
        }
        None => get_nodes("checkers"),
    };
    let symbols = match custom_symbols {
        Some(s_nodes) => {
            let mut s = s_nodes.to_vec();
            s.extend(get_nodes("symbols"));
            s
        }
        None => get_nodes("symbols"),
    };
    while let Some(&_) = lines.peek() {
        let mut token_line_class: TokenClass = TokenClass::Other;

        let mut next_line_contains_symbol = false;
        let mut next_line_contains_action = false;
        let mut next_line_contains_target = false;
        // for prefix index tp1. tp2. example
        let current_line = lines.next().unwrap(); // get current line
        let prefix = format!("{}.", index_tp);
        let ind1 = current_line.find(prefix.as_str()); // first index in previos line
        let next_line = if let Some(&line) =lines.peek() {line} else {&String::new()}; // check the next line current line
        let same = next_line.find(prefix.as_str());
        let prefix = format!("{}.", index_tp+1);
        let ind2 = next_line.find(prefix.as_str()); // second index in next line
        let line = if same == ind1 && index_tp == 1 {
            index_tp = -1;
            current_line.to_string()
        }else if (ind1 == ind2 || index_tp > 1) && ind1.is_some() {
            index_tp += 1;  // increment index tp
            let mut tmp = current_line.to_string();
            let ind = ind1.unwrap();
            tmp.replace_range(ind..ind+2, " ");
            tmp
        } else {
            current_line.to_string()
        };

        if !next_line.is_empty() && (signals_multiline || ( line_contains_symbol && line_contains_action && line_contains_target)) {
            let mut delim = "";
            if next_line.contains(";") {
                delim = ";";
            }else if next_line.contains(" ") {
                delim = " ";
            } else if next_line.contains("-") {
                delim = "-";
            }
            for match_str in next_line.split(delim) {
                if ACTION_VAL_D.contains(match_str) || ACTION_VAL_U.contains(match_str) {
                    next_line_contains_action = true;
                } else if !match_str.is_empty() && match_str.chars().all(|c| c.is_alphabetic() && !c.is_lowercase()) {
                    next_line_contains_symbol = true;
                } else if !match_str.is_empty() && match_str.chars().all(|c| c.is_numeric() || ":.,".contains(c)) {
                    next_line_contains_target = true;
                }
            }
        }
        // Reset per-line token offset tracker
        token_offset_in_line = 0;

        // Determine byte offset of this line's start in the original texto.
        let line_orig_offset = orig_line_offsets.get(line_index).copied().unwrap_or(0);

        let mut token_iter = line.iter_tokens();
        loop {
            let Some((name, match_str)) = token_iter.next() else { break };
            // `token_offset_in_line` is the byte start of this token within `line`.
            let tok_start = line_orig_offset + token_offset_in_line;
            let tok_end   = tok_start + match_str.len();
            token_offset_in_line += match_str.len();
            let (flags, token_class) = get_flags_and_token_class(match_str.as_str(), name, &nodes);
            match token_class {
                TokenClass::Word | TokenClass::Symbol | TokenClass::SymbolIndex if !flags.contains(TokenFlags::NEED_VAL) 
                  && (
                        token_class == TokenClass::SymbolIndex 
                     || flags.contains(TokenFlags::VAL)
                     || token_line_class == TokenClass::Symbol 
                     || token_line_class == TokenClass::Action 
                     || token_line_class == TokenClass::SymbolIndex 
                     || match_str.to_lowercase() == "otc"
                  ) => {
                    if current_signal.is_complete() {
                            signals.push(current_signal.clone());
                            current_signal = Signal::default();
                        }
                        if let Some(ref symbol) = current_signal.symbol{
                            if let SymbolType::Empty(_) = symbol.symbol_type {
                                current_signal.symbol = None;
                            }
                        }
                        if current_signal.symbol.is_none(){
                            if token_class == TokenClass::SymbolIndex{
                                let currentsym = Some(Symbol { symbol_type: SymbolType::Index(match_str.to_string()), symbol_class: None });
                                current_signal.symbol = currentsym;
                                line_contains_symbol = true;
                                if !found_first_symbol {
                                    first_symbol_line = line_index;
                                    first_symbol_start = tok_start - line_orig_offset;
                                    first_symbol_end_char = texto[tok_end..].chars().next().unwrap_or('\0');
                                    found_first_symbol = true;
                                }
                                replacements.push((tok_start, tok_end, "symbol"));
                            }else if let Some(currentsym) = get_symbol_from_text(match_str.as_str(), &get_nodes("symbols"), &nodes) {
                                current_signal.symbol = Some(currentsym);
                                line_contains_symbol = true;
                                if !found_first_symbol {
                                    first_symbol_line = line_index;
                                    first_symbol_start = tok_start - line_orig_offset;
                                    first_symbol_end_char = texto[tok_end..].chars().next().unwrap_or('\0');
                                    found_first_symbol = true;
                                }
                                replacements.push((tok_start, tok_end, "symbol"));
                            } else if let Some(currentsym) = get_symbol_from_text(before_match_str.as_str(), &get_nodes("symbols"), &nodes) {
                                current_signal.symbol = Some(currentsym);
                                line_contains_symbol = true;
                                if !found_first_symbol {
                                    first_symbol_line = line_index;
                                    first_symbol_start = before_tok_start.saturating_sub(line_orig_offset);
                                    first_symbol_end_char = texto[before_tok_end..].chars().next().unwrap_or('\0');
                                    found_first_symbol = true;
                                }
                                replacements.push((before_tok_start, before_tok_end, "symbol"));
                            }
                        } else if let Some(symbol) = current_signal.symbol.as_mut() {

                            if match_str.to_uppercase().ends_with("OTC"){
                                if let Some(class) = symbol.symbol_class.as_mut() {
                                    *class = SymbolClass::Otc;
                                    replacements.push((tok_start, tok_end, "otc"));
                                } else {
                                    symbol.symbol_class = Some(SymbolClass::Otc);
                                    replacements.push((tok_start, tok_end, "otc"));
                                }
                            }
                            let currentsym = if let Some(symbol) = get_symbol_from_text(match_str.as_str(), &get_nodes("symbols"), &nodes) {Some(symbol)} else {get_symbol_from_text(before_match_str.as_str(), &get_nodes("symbols"), &nodes)};
                            if currentsym.is_none() || !before_match_str.chars().all(|c| c.is_numeric()) || match_str.to_uppercase() != "INDEX"{
                                continue;
                            }
                            let sufix = match_str.to_uppercase();
                            match symbol.symbol_type {
                                SymbolType::Crypto(ref mut symb)       |
                                SymbolType::Forex(ref mut symb)        |
                                SymbolType::Funds(ref mut symb)        |
                                SymbolType::Etf(ref mut symb)          |
                                SymbolType::Index(ref mut symb)        |
                                SymbolType::MoneyMarkets(ref mut symb) |
                                SymbolType::Stock(ref mut symb)        |
                                SymbolType::Empty(ref mut symb)        => {
                                    let mut items: Vec<String> = symb.split(' ').map(|s| s.to_string()).collect();
                                    items.insert(1, sufix);
                                    *symb = items.join(" ");
                                },
                                _ => {},
                        }
                    }

                    token_line_class = token_class;
                },
                TokenClass::Otc if current_signal.symbol.is_some() => {
                    if let Some(symbol) = current_signal.symbol.as_mut() {
                        symbol.symbol_class = Some(SymbolClass::Otc);
                        replacements.push((tok_start, tok_end, "otc"));
                    }
                },
                TokenClass::Action | TokenClass::Otc=> {
                        if current_signal.is_complete() {
                            signals.push(current_signal.clone());
                            current_signal = Signal::default();
                        }

                        if flags.contains(TokenFlags::VALUE_UP) {
                            current_signal.action = Some(true);
                            line_contains_action = true;
                            replacements.push((tok_start, tok_end, "action"));
                        } else if flags.contains(TokenFlags::VALUE_DOWN) {
                            current_signal.action = Some(false);
                            line_contains_action = true;
                            replacements.push((tok_start, tok_end, "action"));
                        }
                        if (current_signal.symbol.is_none() || matches!(current_signal.symbol.as_ref().unwrap().symbol_type, SymbolType::Empty(_))) && !before_match_str.is_empty(){
                            if let Some(mut currentsym) = get_symbol_from_text(before_match_str.as_str(), &get_nodes("symbols"), &nodes) {
                                if token_class == TokenClass::Otc{
                                    currentsym.symbol_class = Some(SymbolClass::Otc);
                                    replacements.push((tok_start, tok_end, "otc"));
                                }
                                current_signal.symbol = Some(currentsym);
                                line_contains_symbol = true;
                                if !found_first_symbol {
                                    first_symbol_line = line_index;
                                    first_symbol_start = before_tok_start.saturating_sub(line_orig_offset);
                                    first_symbol_end_char = texto[before_tok_end..].chars().next().unwrap_or('\0');
                                    found_first_symbol = true;
                                }
                                replacements.push((before_tok_start, before_tok_end, "symbol"));
                            } else if let Some(mut currentsym) = get_symbol_from_text(match_str.as_str(), &get_nodes("symbols"), &nodes) {
                                if token_class == TokenClass::Otc{
                                    currentsym.symbol_class = Some(SymbolClass::Otc);
                                    replacements.push((tok_start, tok_end, "otc"));
                                }
                                current_signal.symbol = Some(currentsym);
                                line_contains_symbol = true;
                                if !found_first_symbol {
                                    first_symbol_line = line_index;
                                    first_symbol_start = tok_start - line_orig_offset;
                                    first_symbol_end_char = texto[tok_end..].chars().next().unwrap_or('\0');
                                    found_first_symbol = true;
                                }
                                replacements.push((tok_start, tok_end, "symbol"));
                            }
                        }
                    token_line_class = token_class;
                },
                TokenClass::Timeframe => {
                        if current_signal.is_complete() {
                            signals.push(current_signal.clone());
                            current_signal = Signal::default();
                        }
                        if flags.contains(TokenFlags::PERIOD) && flags.contains(TokenFlags::DIGIT) {
                            global_timeframe = match_str.clone();
                            current_signal.target = Some(Target::PeriodTime(match_str.to_string()));
                            replacements.push((tok_start, tok_end, "period"));
                            continue;
                        }
                        let mut timeframe = String::new();
                        let mut digitframe = String::new();
                        if flags.contains(TokenFlags::NAME) && flags.contains(TokenFlags::PARSER) {
                            let upper = match_str.to_uppercase();
                            if let Some(tf) = upper.chars().next() {
                                global_timeframe = tf.to_string();
                                replacements.push((tok_start, tok_end, "period_name"));
                                timeframe = tf.to_string();
                            }
                        }
                        if flags.contains(TokenFlags::DIGIT) && match_str.chars().all(|c| c.is_numeric()) {
                            digitframe = match_str.to_string();
                        }

                        if digitframe.is_empty() && before_match_str.chars().all(|c| c.is_numeric()) {
                            replacements.push((before_tok_start, before_tok_end, "period_digit"));
                            digitframe = before_match_str.to_string();
                            before_match_str = String::new();
                            before_tok_start = 0;
                            before_tok_end = 0;
                        }
                        let had_timeframe = !timeframe.is_empty() || !digitframe.is_empty();
                        if !current_signal.target.is_some() && had_timeframe {
                            global_timeframe = timeframe.to_string() + &digitframe;
                            current_signal.target = Some(Target::PeriodTime(timeframe + &digitframe));
                        } else if let Some(Target::PeriodTime(ref mut pt)) = current_signal.target {
                            if pt.chars().all(|c| c.is_numeric()) {
                                global_timeframe = timeframe.to_string() + pt;
                                replacements.push((tok_start, tok_end, "period_digit"));
                                *pt = timeframe.to_string() + pt;
                            } else if pt.chars().all(|c| c.is_alphabetic()) {
                                replacements.push((tok_start, tok_end, "period_name"));
                                global_timeframe.push_str(&digitframe);

                                pt.push_str(&digitframe);
                            }
                        }
                        token_line_class = token_class;
                },
                TokenClass::Timezone => {
                        let mut timezone = match_str.to_string();
                        if flags.contains(TokenFlags::SPACE){
                            timezone = timezone.replace(" ","");
                        }
                        if flags.contains(TokenFlags::VAL) {
                            if let Some(Entry::TimeEntry(ref mut tz, _)) = current_signal.entry {
                                *tz = timezone.clone();
                                global_timezone = timezone;
                            } else {
                                current_signal.entry = Some(Entry::TimeEntry(timezone.clone(), String::new()));
                                global_timezone = timezone;
                            }
                            replacements.push((tok_start, tok_end, "timezone"));
                    }
                    token_line_class = token_class;
                },
                TokenClass::Target => if flags.contains(TokenFlags::VAL) {
                    if flags.contains(TokenFlags::COLON) && !flags.contains(TokenFlags::LETTER) {
                        let time_str = if flags.contains(TokenFlags::SINGLE_QUOTATION){match_str.replace("'",":")}else{match_str};
                        let time = match time_str.split(':').map(|s| s.parse::<u32>()).collect::<Result<Vec<u32>, _>>().as_deref() {
                            Ok([h, m, s]) => format!("{:02}:{:02}:{:02}", h, m, s),
                            Ok([h, m]) => format!("{:02}:{:02}:00", h, m),
                            _ => String::new(),
                        };
                        if current_signal.is_complete() {
                            signals.push(current_signal.clone());
                            current_signal = Signal::default();
                        } else if signals_multiline && current_signal.is_esencial() {
                            signals_esencials.push(current_signal.clone());
                            current_signal = Signal::default();
                        }
                        if let Some(Entry::TimeEntry(ref mut tz, ref mut et)) = current_signal.entry {
                            if et.is_empty() {
                                *et = time;
                                line_contains_target = true;
                                replacements.push((tok_start, tok_end, "entry_time"));
                            }
                            if tz.is_empty() {
                                *tz = global_timezone.clone();
                            }
                        } else {
                            current_signal.entry = Some(Entry::TimeEntry(global_timezone.clone(), time));
                            line_contains_target = true;
                            replacements.push((tok_start, tok_end, "entry_time"));
                        }
                    } else if flags.contains(TokenFlags::LETTER) {
                        if flags.contains(TokenFlags::VALUE_UP) {
                            current_target_type = "TP";
                        } else if flags.contains(TokenFlags::VALUE_DOWN) {
                            current_target_type = "SL";
                        }
                    } else if flags.contains(TokenFlags::RANGE) || flags.contains(TokenFlags::UNDERSCORE) {
                        if current_signal.is_complete() {
                            signals.push(current_signal.clone());
                            current_signal = Signal::default();
                        }
                        let range_str = if flags.contains(TokenFlags::COMMA) {
                            match_str.replace(',', ".")
                        } else {
                            match_str.to_string()
                        };
                        let mut range: Vec<String> = Vec::new();
                        if flags.contains(TokenFlags::HYPHEN) {
                            range = range_str.split('-').map(|s| s.to_string()).collect::<Vec<String>>();
                        } else if flags.contains(TokenFlags::SLASH) {
                            range = range_str.split('/').map(|s| s.to_string()).collect::<Vec<String>>();
                        } else if flags.contains(TokenFlags::BACKSLASH) {
                            range = range_str.split('\\').map(|s| s.to_string()).collect::<Vec<String>>();
                        } else if flags.contains(TokenFlags::UNDERSCORE) {
                            range = range_str.split('_').map(|s| s.to_string()).collect::<Vec<String>>();
                        }
                        range.retain(|s| !s.trim().is_empty());

                        if range.len() == 2 {
                            if range[0].len() > range[1].len() {
                                let point = range[0].len() - range[1].len();
                                range[1] = format!("{}{}", &range[0][..point], range[1]);
                            }
                            let price1 = range[0].parse::<f32>().unwrap_or(0.0);
                            let price2 = range[1].parse::<f32>().unwrap_or(0.0);
                            if price1 > 0.0 && price2 > 0.0 {
                                current_signal.entry = Some(Entry::EntryRange(price1, price2));
                                line_contains_target = true;
                                replacements.push((tok_start, tok_end, "entry_range"));
                            }
                        }
                    } else {
                        
                        // implement better logic below
                        if match_str == "1000" { // main line checking
                            if let Some(symbol) = current_signal.symbol.as_mut() {
                                match symbol.symbol_type {
                                    SymbolType::Index(ref mut sym) => {
                                        sym.push(' ');
                                        sym.push_str(&match_str);
                                        continue;
                                    },
                                    _ => {},
                                }
                            }
                        }
                        // numerical value (price)
                        let price = if flags.contains(TokenFlags::COMMA) {match_str.replace(',', ".").parse::<f32>().unwrap_or(0.0)} else {match_str.parse::<f32>().unwrap_or(0.0)};
                        if price > 0.0 {
                            if current_target_type == "TP" {
                                
                                if let Some(Target::ProfitLoss(ref mut pt)) = current_signal.target {
                                    pt.profits.push(price);
                                    replacements.push((tok_start, tok_end, "profit_price"));
                                } else {
                                    current_signal.target = Some(Target::ProfitLoss(PricesTarget { profits: vec![price], stoploss: 0.0, open: false }));
                                    replacements.push((tok_start, tok_end, "profit_price"));
                                }
                            } else if current_target_type == "SL" {
                                if let Some(Target::ProfitLoss(ref mut pt)) = current_signal.target {
                                    pt.stoploss = price;
                                    replacements.push((tok_start, tok_end, "stoploss"));
                                } else {
                                    current_signal.target = Some(Target::ProfitLoss(PricesTarget { profits: vec![], stoploss: price, open: false }));
                                    replacements.push((tok_start, tok_end, "stoploss"));
                                }
                            } else if (current_target_type == "entry" || (current_signal.action.is_some() && current_signal.symbol.is_some())) && !match_str.contains('%') {
                                if current_signal.is_complete() {
                                    signals.push(current_signal.clone());
                                    current_signal = Signal::default();
                                }
                                if let Some(Entry::PriceEntry(p1)) = current_signal.entry {
                                    let price_prev = if flags.contains(TokenFlags::COMMA) {before_match_str.replace(',', ".").parse::<f32>().unwrap_or(0.0)} else {before_match_str.parse::<f32>().unwrap_or(0.0)};
                                    if line_contains_symbol || line_contains_action || current_target_type == "entry" || price_prev == p1 {
                                        current_signal.entry = Some(Entry::EntryRange(p1, price));
                                        line_contains_target = true;
                                        replacements.push((tok_start, tok_end, "entry_price"));
                                    }

                                } else if let Some(Entry::EntryRange(p1, _)) = current_signal.entry {
                                    if line_contains_symbol || line_contains_action || current_target_type == "entry" {
                                        current_signal.entry = Some(Entry::EntryRange(p1, price));
                                        line_contains_target = true;
                                        replacements.push((tok_start, tok_end, "entry_price"));
                                    }
                                } else if current_signal.entry.is_none() {
                                    current_signal.entry = Some(Entry::PriceEntry(price));
                                    line_contains_target = true;
                                    replacements.push((tok_start, tok_end, "entry_price"));
                                }
                                before_match_str = match_str.clone();

                            } else if !global_timezone.is_empty() {
                                current_signal.entry = Some(Entry::TimePrice(global_timezone.clone(), price));
                                line_contains_target = true;
                                replacements.push((tok_start, tok_end, "entry_price"));
                            } else if let Some(Entry::TimeEntry(tz, _)) = current_signal.entry {
                                current_signal.entry = Some(Entry::TimePrice(tz, price));
                                line_contains_target = true;
                                replacements.push((tok_start, tok_end, "entry_price"));
                            } else {
                                if let Some(Entry::PriceEntry(p1)) = current_signal.entry {
                                    current_signal.entry = Some(Entry::EntryRange(p1, price));
                                    line_contains_target = true;
                                    replacements.push((tok_start, tok_end, "entry_price"));
                                } else if let Some(Entry::EntryRange(p1, _)) = current_signal.entry {
                                    current_signal.entry = Some(Entry::EntryRange(p1, price));
                                    line_contains_target = true;
                                    replacements.push((tok_start, tok_end, "entry_price"));
                                } else if current_signal.entry.is_none() {
                                    current_signal.entry = Some(Entry::PriceEntry(price));
                                    line_contains_target = true;
                                    replacements.push((tok_start, tok_end, "entry_price"));
                                }
                            }
                        }
                    }
                    token_line_class = token_class;
                },
                TokenClass::Word if token_line_class == TokenClass::Target => {
                    if match_str.to_lowercase() == "open" {
                        if let Some(Target::ProfitLoss(ref mut pt)) = current_signal.target {
                            pt.open = true;
                            replacements.push((tok_start, tok_end, "target_open"));
                        } else {
                            current_signal.target = Some(Target::ProfitLoss(PricesTarget { profits: vec![], stoploss: 0.0, open: true }));
                            replacements.push((tok_start, tok_end, "target_open"));
                        }
                    }
                    token_line_class = token_class;
                },
                TokenClass::Symbol | TokenClass::Word => {
                    // (no position update: these are label/word tokens, not signal-bearing)

                        if flags.contains(TokenFlags::ENTRY) {
                            current_target_type = "entry";
                        }
                        if flags.contains(TokenFlags::DIGIT) {
                            before_match_str = match_str.clone();
                            before_tok_start = tok_start;
                            before_tok_end = tok_end;
                        }

                        before_match_str = match_str.clone();
                        before_tok_start = tok_start;
                        before_tok_end = tok_end;
                    token_line_class = token_class;
                },
                TokenClass::Gale | TokenClass::Indicator |TokenClass::Info| TokenClass::Result  => {
                    token_line_class = token_class;
                },
                TokenClass::Other if flags.contains(TokenFlags::DIGIT) && (token_line_class == TokenClass::Symbol || token_line_class == TokenClass::SymbolIndex) => {
                        if let Some(symb) = current_signal.symbol.as_mut() {
                            match symb.symbol_type {
                                SymbolType::Crypto(ref mut symb)       |
                                SymbolType::Forex(ref mut symb)        |
                                SymbolType::Funds(ref mut symb)        |
                                SymbolType::Etf(ref mut symb)          |
                                SymbolType::Index(ref mut symb)        |
                                SymbolType::MoneyMarkets(ref mut symb) |
                                SymbolType::Stock(ref mut symb)        |
                                SymbolType::Plain(ref mut symb)        => symb.push_str(&(" ".to_owned() + &match_str)),
                                _ => {},
                            }
                        }
                }
                _ => {
                    
                        // quietar mas arriba est word
                        if flags.contains(TokenFlags::ENTRY) {
                            current_target_type = "entry";
                        }
                        if flags.contains(TokenFlags::DIGIT) {
                            before_match_str = match_str.clone();
                            before_tok_start = tok_start;
                            before_tok_end = tok_end;
                    }
                },
            };

        } // end loop over tokens (replaces `for (name, match_str) in line.iter_tokens()`)



        line_index += 1;

        signals_multiline = line_contains_symbol && line_contains_action && line_contains_target;
        if !signals_multiline {
            line_contains_symbol = next_line_contains_symbol;
            line_contains_action = next_line_contains_action;
            line_contains_target = next_line_contains_target;
        } else if nex_signals_multine || (next_line_contains_symbol && next_line_contains_action && next_line_contains_target) {
            current_signal.add_time_zone(global_timezone.clone());
            current_signal.add_timeframe(global_timeframe.clone());
            if current_signal.is_complete() {
                signals.push(current_signal.clone());
                current_signal = Signal::default();
            } else {
                signals_esencials.push(current_signal);
                current_signal = Signal::default();
            }
        }
        nex_signals_multine = next_line_contains_symbol && next_line_contains_action && next_line_contains_target;
    }
    if current_signal.is_complete() {
        signals.push(current_signal.clone());
    }
    if signals_esencials.len() > 0 {
        for sig in signals_esencials.iter_mut() {
            if sig.copy_from(&current_signal){
                signals.push(sig.clone());
            }
        }
        signals_esencials.clear();
    }

    // ── Apply encapsulated replacements to the original text ──
    // Sort by descending start offset so that replacing later tokens first
    // does not invalidate the byte positions of earlier tokens.
    replacements.sort_by(|a, b| b.0.cmp(&a.0));
    // Apply replacements in descending order, skipping overlapping ranges.
    let mut last_start: usize = usize::MAX;
    let has_symbol = signals.iter().any(|s| s.symbol.is_some());
    let has_action = signals.iter().any(|s| s.action.is_some());
    let has_target = signals.iter().any(|s| s.target.is_some());
    let has_entry = signals.iter().any(|s| s.entry.is_some());
    let mut modified_text = texto.to_string();
    for (start, end, label) in &replacements {
        // Skip if this range overlaps with the previously applied replacement.
        if *end > last_start {
            continue;
        }
        if *end <= modified_text.len() && *start < *end {
            let encapsulated = format!("$({})", label);
            modified_text.replace_range(*start..*end, &encapsulated);
            last_start = *start;
        }
    }
    if !(has_action && (has_entry || has_target || has_symbol)) {
        return None;
    } else {
        Some(Signals {
            template: modified_text,
            signals,
            line_index: first_symbol_line,
            token_start: first_symbol_start,
            end_char: first_symbol_end_char,
            },
        )
    }
}

/// Recalculates line_index, token_start, and end_char for a specific symbol within text.
/// If target_symbol is None or not found, defaults to the symbol position detected by parse_signal.
pub fn find_symbol_position(text: &str, target_symbol: Option<&str>) -> (usize, usize, char) {
    if let Some(sym) = target_symbol {
        let sym_trimmed = sym.trim();
        if !sym_trimmed.is_empty() {
            for (line_idx, line) in text.lines().enumerate() {
                if let Some(start_pos) = line.find(sym_trimmed) {
                    let end_pos = start_pos + sym_trimmed.len();
                    let end_c = line[end_pos..].chars().next().unwrap_or('\0');
                    return (line_idx, start_pos, end_c);
                }
            }
        }
    }
    if let Some(signals) = parse_signal(text, None, None) {
        (signals.line_index, signals.token_start, signals.end_char)
    } else {
        (0, 0, '\0')
    }
}

/// Tokenizes an example_text, extracting the template pattern and calculating line_index,
/// token_start, and end_char dynamically depending on the detected or specified symbol.
pub fn tokenize_example(example_text: &str, target_symbol: Option<&str>) -> (String, usize, usize, char, Signals) {
    let signals = parse_signal(example_text, None, None).unwrap_or_default();
    let mut pattern = signals.template.clone();
    let (l_idx, t_start, e_char) = if let Some(sym) = target_symbol {
        if !sym.trim().is_empty() {
            // If custom symbol is specified and present in text, ensure it's replaced with $(symbol) in pattern
            let sym_trimmed = sym.trim();
            if example_text.contains(sym_trimmed) && !pattern.contains("$(symbol)") {
                pattern = pattern.replace(sym_trimmed, "$(symbol)");
            }
            find_symbol_position(example_text, Some(sym_trimmed))
        } else {
            (signals.line_index, signals.token_start, signals.end_char)
        }
    } else {
        (signals.line_index, signals.token_start, signals.end_char)
    };
    (pattern, l_idx, t_start, e_char, signals)
}

fn is_line_match(orig_line: &str, mod_line: &str) -> bool {
    if orig_line == mod_line {
        return true;
    }
    let tokens = orig_line.zip_tokens(mod_line);
    let mut count = 0;
    let mut static_matches = 0;
    for (orig_tok, mod_tok) in tokens {
        count += 1;
        if mod_tok.1 == orig_tok.1 {
            if !mod_tok.1.trim().is_empty() {
                static_matches += 1;
            }
        } else if !mod_tok.1.contains("$(") {
            return false;
        }
    }
    count > 0 && static_matches > 0
}

/// Analiza un texto original de señales de trading comparándolo contra un texto plantilla que contiene etiquetas `$(etiqueta)`.
///
/// # Parámetros
/// - `original_text`: Texto original/mensaje recibido con señales de trading.
/// - `modified_text`: Texto plantilla con patrones de concordancia como `$(symbol)`, `$(action)`, `$(entry_time)`, etc.
/// - `_line_index`: Índice de línea del token en la entrada (reservado para trazabilidad de posición).
/// - `_token_start`: Posición inicial de carácter del token (reservada para trazabilidad de posición).
/// - `_end_char`: Carácter delimitador final (reservado para trazabilidad de posición).
///
/// # Retorno
/// Retorna `Option<Vec<Signal>>` conteniendo la lista de señales extraídas si se encontró alguna válida, o `None` si el resultado está vacío.
pub fn parse_template(original_text: &str, modified_text: &str, line_index: Option<usize>, token_start: Option<usize>, end_char: Option<char>) -> Option<Vec<Signal>> {
    // Vector acumulador donde se almacenarán las señales de trading extraídas
    let mut signals: Vec<Signal> = Vec::new();
    // Objeto Signal temporal para ir construyendo la señal actual
    let mut current_signal: Signal = Signal::default();

    // Coleccionar las líneas del texto modificado/plantilla
    let mod_lines: Vec<&str> = modified_text.lines().collect();
    if mod_lines.is_empty() {
        return None;
    }

    let mut mod_idx = 0;
    // Recorrer el texto original línea a línea
    for original_line in original_text.lines() {
        // Reiniciar el índice de la línea plantilla si llegamos al final de la plantilla
        // o si la línea original coincide con la primera línea de la plantilla
        if mod_idx >= mod_lines.len()
            || (mod_idx > 0
                && !mod_lines[0].trim().is_empty()
                && is_line_match(original_line, mod_lines[0]))
        {
            mod_idx = 0;
        }
        if mod_idx + 1 < mod_lines.len()
            && !is_line_match(original_line, mod_lines[mod_idx])
            && is_line_match(original_line, mod_lines[mod_idx + 1])
        {
            mod_idx += 1;
        }
        let mut modified_line = mod_lines[mod_idx].to_string();
        let mut original_line = original_line.to_string();
        let current_line_idx = mod_idx;
        // mod_idx += 1;
        let mut labeled = String::new();
        if let (Some(index), Some(start), Some(end_char)) = (line_index, token_start, end_char) {
            if current_line_idx == index {
                if start < original_line.len() && original_line.is_char_boundary(start) {
                    let orig_rem = &original_line[start..];
                    let orig_len = orig_rem.find(end_char).unwrap_or(orig_rem.len());
                    let orig_val = orig_rem[..orig_len].trim();

                    let mod_rem = if start < modified_line.len() && modified_line.is_char_boundary(start) { &modified_line[start..] } else { "" };
                    let mod_len = mod_rem.find(end_char).unwrap_or(mod_rem.len());
                    let mod_val = &mod_rem[..mod_len];

                    if mod_val.contains("$(symbol)") {
                        let currentsym = get_symbol_from_text(orig_val, &get_nodes("symbols"), &get_nodes("checkers"));
                        let end_range = start + orig_len;
                        if let Some(sym) = currentsym {
                            if current_signal.symbol.is_none() {
                                current_signal.symbol = Some(sym);
                            }
                            if original_line.is_char_boundary(end_range) {
                                original_line.replace_range(start..end_range, "");
                                modified_line = modified_line.replacen("$(symbol)", "", 1);
                            }
                        } else if !orig_val.is_empty() && current_signal.symbol.is_none() {
                            current_signal.symbol = Some(Symbol {
                                symbol_type: SymbolType::Plain(orig_val.to_string()),
                                symbol_class: None,
                            });
                            if original_line.is_char_boundary(end_range) {
                                original_line.replace_range(start..end_range, "");
                                modified_line = modified_line.replacen("$(symbol)", "", 1);
                            }
                        }
                    }
                }
            }
        }

        // Iterar en paralelo sobre los tokens de la línea original y la línea plantilla
        for (original_token, modified_token) in original_line.zip_tokens(&modified_line) {
            // Si el token modificado es idéntico al original, no contiene una plantilla `$(...)`
            if modified_token.1 == original_token.1 {
                continue
            }
            // Buscar el inicio de la variable de plantilla "$("
            let start = modified_token.1.find("$(");
            if start.is_none() {
                continue
            }
            let start = start.unwrap();
            // Buscar el cierre del delimitador ")"
            let rel_end = modified_token.1[start + 2..].find(')');
            if rel_end.is_none() {
                continue
            }
            let end = start + 2 + rel_end.unwrap();
            // Extraer el nombre de la etiqueta entre "$(" y ")" (ej. "symbol", "action", etc.)
            let label = &modified_token.1[start + 2..end];
            // Extraer el valor real correspondiente desde el token del texto original
            let orig_val = original_token.1.trim();

            // Evaluar el tipo de etiqueta de plantilla encontrada
            match label {
                "symbol" => {
                    // Extraer y resolver el símbolo o activo financiero a partir del texto
                    let currentsym = get_symbol_from_text(orig_val, &get_nodes("symbols"), &get_nodes("checkers"));
                    if let Some(sym) = currentsym.as_ref() {
                        if let SymbolType::Plain(_) = sym.symbol_type {
                            if current_signal.symbol.is_none() {
                                current_signal.symbol = Some(sym.clone());
                            }
                            continue
                        }
                    }
                    // Si la señal en construcción ya está completa o tiene símbolo y acción,
                    // se guarda la señal actual y se reinicia para capturar la siguiente
                    if current_signal.symbol.is_some() && current_signal.action.is_some() {
                        signals.push(current_signal.clone());
                        current_signal = Signal::default();
                    }
                    if let Some(sym) = currentsym {
                        current_signal.symbol = Some(sym);
                    } else if !orig_val.is_empty() {
                        let clean_val = orig_val.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '-');
                        if clean_val.len() >= 3 && clean_val.chars().any(|c| c.is_alphabetic()) {
                            current_signal.symbol = Some(Symbol {
                                symbol_type: SymbolType::Plain(orig_val.to_string()),
                                symbol_class: None,
                            });
                        }
                    }
                },
                "otc" => {
                    // Verificar si el texto identifica un activo del mercado OTC (Over-The-Counter)
                    if !orig_val.to_lowercase().contains("otc") {
                        continue;
                    }
                    if current_signal.symbol.is_none() {
                        current_signal.symbol = Some(Symbol {
                            symbol_type: SymbolType::Plain(orig_val.to_string()),
                            symbol_class: Some(SymbolClass::Otc),
                        });
                    } else if let Some(ref mut sym) = current_signal.symbol {
                        sym.symbol_class = Some(SymbolClass::Otc);
                    }
                },
                "action" => {
                    // Obtener los flags y clasificación del token para determinar si es acción al alza o a la baja
                    let (flags, _) = get_flags_and_token_class(orig_val, original_token.0, &get_nodes("checkers"));
                    let detected_action = if flags.contains(TokenFlags::VALUE_UP) {
                        Some(true) // Operación de Compra / Call / Alza
                    } else if flags.contains(TokenFlags::VALUE_DOWN) {
                        Some(false) // Operación de Venta / Put / Baja
                    } else {
                        None
                    };

                    if let Some(act) = detected_action {
                        if current_signal.action.is_none() {
                            current_signal.action = Some(act);
                        } else if current_signal.action == Some(act) {
                            // Acción confirmada dentro de la misma señal (ej. Tend: Buy tras CALL)
                        } else if current_signal.symbol.is_none() {
                            current_signal.action = Some(act);
                        }
                    }
                },
                "period" => {
                    // Si la señal ya estaba completa y tenía un target, guardarla y reiniciar
                    if current_signal.is_complete() && current_signal.target.is_some() {
                        signals.push(current_signal.clone());
                        current_signal = Signal::default();
                    }
                    // Asignar el período o temporalidad (ej. M5, H1, M15)
                    current_signal.target = Some(Target::PeriodTime(orig_val.to_string()));
                }
                "period_digit" | "period_name" => {
                    // Concatenar dígitos o sufijos al valor del período de tiempo configurado
                    if let Some(Target::PeriodTime(ref mut pt)) = current_signal.target {
                        if pt.chars().all(|c| c.is_numeric()) {
                            let prefix = orig_val.chars().next().map(|c| c.to_string().to_uppercase()).unwrap_or_default();
                            *pt = prefix + pt;
                        } else if pt.chars().all(|c| c.is_alphabetic()) {
                            pt.push_str(&orig_val);
                        }
                    } else {
                        let val = if orig_val.chars().all(|c| c.is_numeric()) {
                            orig_val.to_string()
                        } else if orig_val.chars().all(|c| c.is_alphabetic()) {
                            orig_val.chars().next().map(|c| c.to_string().to_uppercase()).unwrap_or_default()
                        } else {
                            orig_val.to_string()
                        };
                        current_signal.target = Some(Target::PeriodTime(val));
                    }
                },
                "timezone" => {
                    // Remover espacios y agregar la zona horaria a la señal
                    let tz = orig_val.replace(" ", "");
                    current_signal.add_time_zone(tz);
                },
                "entry_time" | "time_entry" => {
                    // Si la señal previa ya cuenta con suficiente información, guardarla y reiniciar
                    if current_signal.is_complete() || (current_signal.symbol.is_some() && current_signal.action.is_some()) {
                        signals.push(current_signal.clone());
                        current_signal = Signal::default();
                    }
                    // Normalizar el string de la hora de entrada a formato HH:MM:SS o HH:MM:00
                    let time_str = if orig_val.contains(':') {
                        let parts: Vec<&str> = orig_val.split(':').collect();
                        match parts.as_slice() {
                            [h, m, s] => {
                                if let (Ok(h), Ok(m), Ok(s)) = (h.parse::<u32>(), m.parse::<u32>(), s.parse::<u32>()) {
                                    format!("{:02}:{:02}:{:02}", h, m, s)
                                } else {
                                    orig_val.to_string()
                                }
                            },
                            [h, m] => {
                                if let (Ok(h), Ok(m)) = (h.parse::<u32>(), m.parse::<u32>()) {
                                    format!("{:02}:{:02}:00", h, m)
                                } else {
                                    orig_val.to_string()
                                }
                            },
                            _ => orig_val.to_string(),
                        }
                    } else {
                        orig_val.to_string()
                    };

                    // Establecer o actualizar el punto de entrada por tiempo (TimeEntry)
                    if let Some(Entry::TimeEntry(ref tz, _)) = current_signal.entry {
                        current_signal.entry = Some(Entry::TimeEntry(tz.clone(), time_str));
                    } else if let Some(Entry::TimePrice(ref tz, _)) = current_signal.entry {
                        current_signal.entry = Some(Entry::TimeEntry(tz.clone(), time_str));
                    } else {
                        current_signal.entry = Some(Entry::TimeEntry(String::new(), time_str));
                    }
                },
                "entry_range" => {
                    // Parsear un rango de precios de entrada (ej: 1.1234-1.1250) o un precio individual
                    let range_str = orig_val.replace(',', ".");
                    let range: Vec<&str> = range_str.split(&['-', '/', '\\', '_'][..]).collect();
                    if range.len() == 2 {
                        let p1 = range[0].parse::<f32>().unwrap_or(0.0);
                        let p2 = range[1].parse::<f32>().unwrap_or(0.0);
                        if p1 > 0.0 && p2 > 0.0 {
                            current_signal.entry = Some(Entry::EntryRange(p1, p2));
                        }
                    } else if let Ok(price) = range_str.parse::<f32>() {
                        if price > 0.0 {
                            if let Some(Entry::PriceEntry(p1)) = current_signal.entry {
                                current_signal.entry = Some(Entry::EntryRange(p1, price));
                            } else {
                                current_signal.entry = Some(Entry::PriceEntry(price));
                            }
                        }
                    }
                },
                "entry_price" | "entry" | "price" => {
                    // Parsear un precio numérico de entrada y actualizar el tipo de entrada de la señal
                    let price = orig_val.replace(',', ".").parse::<f32>().unwrap_or(0.0);
                    if price > 0.0 {
                        if let Some(Entry::PriceEntry(p1)) = current_signal.entry {
                            current_signal.entry = Some(Entry::EntryRange(p1, price));
                        } else if let Some(Entry::TimeEntry(ref tz, _)) = current_signal.entry {
                            current_signal.entry = Some(Entry::TimePrice(tz.clone(), price));
                        } else if current_signal.entry.is_none() {
                            current_signal.entry = Some(Entry::PriceEntry(price));
                        }
                    }
                },
                "profit_price" | "profit" => {
                    // Extraer precio de Take Profit (Toma de Ganancia) y agregarlo a la lista de objetivos
                    let price = orig_val.replace(',', ".").parse::<f32>().unwrap_or(0.0);
                    if price > 0.0 {
                        if let Some(Target::ProfitLoss(ref mut pt)) = current_signal.target {
                            pt.profits.push(price);
                        } else {
                            current_signal.target = Some(Target::ProfitLoss(PricesTarget {
                                open: false,
                                profits: vec![price],
                                stoploss: 0.0,
                            }));
                        }
                    }
                },
                "stoploss" => {
                    // Extraer y asignar precio de Stop Loss (Límite de Pérdida)
                    let price = orig_val.replace(',', ".").parse::<f32>().unwrap_or(0.0);
                    if price > 0.0 {
                        if let Some(Target::ProfitLoss(ref mut pt)) = current_signal.target {
                            pt.stoploss = price;
                        } else {
                            current_signal.target = Some(Target::ProfitLoss(PricesTarget {
                                open: false,
                                profits: vec![],
                                stoploss: price,
                            }));
                        }
                    }
                },
                "gale" => {
                    // Parsear configuración de Martingala (Gale): número de gales, hora o 'NoGale'
                    if let Ok(num) = orig_val.parse::<i8>() {
                        current_signal.gales.push(Gale::Number(num));
                    } else if orig_val.contains(':') {
                        current_signal.gales.push(Gale::TimeEntry(orig_val.to_string()));
                    } else if orig_val.to_lowercase().contains("no") || orig_val == "0" {
                        current_signal.gales.push(Gale::NoGale);
                    } else {
                        if let Some(digit_char) = orig_val.chars().find(|c| c.is_ascii_digit()) {
                            if let Some(digit) = digit_char.to_digit(10) {
                                current_signal.gales.push(Gale::Number(digit as i8));
                            }
                        }
                    }
                },
                // Ignorar etiquetas que no representan propiedades directamente asignables aquí
                "indicator" | "date" | "custom" | "dismiss" | "variable" | "_" | _ => {},
            }
            labeled = label.to_string();
        }
        if labeled != "profit_price" {
            mod_idx += 1;
        }
    }

    // Agregar la última señal si tiene datos mínimos indispensables (símbolo y acción)
    if current_signal.symbol.is_some() && current_signal.action.is_some() {
        signals.push(current_signal);
    }

    // Retornar las señales extraídas o None si no se construyó ninguna
    if !signals.is_empty() {
        Some(signals)
    } else {
        None
    }
}

// #[pyfunction]
pub fn check_token_category(text: &str, category: &str) -> bool {
    check_category(text, category)
}



fn get_symbol_from_text(match_str: &str, nodes: &[Nodes], checkrs: &[Nodes]) -> Option<Symbol> {
    // let nodes = get_nodes("symbols");
    // let checkrs = get_nodes("checkers");
    // for match_str in text {
        if match_str.trim().is_empty() {
            // continue;
            return None;
        }
        let mut symbol_str = match_str.to_string();
        let mut symbol_class_str = String::new();
        
        // Clean trailing punctuation
        while symbol_str.ends_with(':') || symbol_str.ends_with(';') || symbol_str.ends_with('.') || symbol_str.ends_with(',') {
            symbol_str.pop();
        }

        let (flags, _) = get_flags_and_token_class(symbol_str.as_str(), "", checkrs);
        if flags.contains(TokenFlags::UNDERSCORE) {
            let parts: Vec<&str> = match_str.splitn(2, '_').collect();
            if parts.len() == 2 {
                symbol_str = parts[0].to_string();
                symbol_class_str = parts[1].to_string();
            }
        } else if flags.contains(TokenFlags::HYPHEN) {
            let parts: Vec<&str> = match_str.splitn(2, '-').collect();
            if parts.len() == 2 {
                symbol_str = parts[0].to_string();
                symbol_class_str = parts[1].to_string();
            }
        } else if flags.contains(TokenFlags::POINT) {
            let parts: Vec<&str> = match_str.splitn(2, '.').collect();
            if parts.len() == 2 {
                symbol_str = parts[0].to_string();
                symbol_class_str = parts[1].to_string();
            }
        } else if flags.contains(TokenFlags::SLASH) || flags.contains(TokenFlags::BACKSLASH) {
            symbol_str = match_str.replace("/", "").replace("\\", "");
        }
        let (_, tokenc) = get_flags_and_token_class(symbol_str.as_str(), &symbol_str, nodes);
        let sym_class = if symbol_class_str.contains("OTC") {
            Some(SymbolClass::Otc)
        } else if symbol_class_str.contains("P") || symbol_class_str.contains("PERP") {
            Some(SymbolClass::Perpetual)
        } else {
            None
        };

        let symb = symbol_str.to_uppercase();
        let sym_type = match tokenc {
            TokenClass::SymbolCrypto => SymbolType::Crypto(symb),
            TokenClass::SymbolForex => SymbolType::Forex(symb),
            TokenClass::SymbolFunds => SymbolType::Funds(symb),
            TokenClass::SymbolEtfs => SymbolType::Etf(symb),
            TokenClass::SymbolIndex => SymbolType::Index(symb),
            TokenClass::SymbolMoney => SymbolType::MoneyMarkets(symb),
            TokenClass::SymbolStocks => {
                if !flags.contains(TokenFlags::UPPERCASE) { 
                    return None;
                } else { 
                    SymbolType::Stock(symb)
                }
            },
            _ => {return None;},
        };

        return Some(Symbol {
            symbol_type: sym_type,
            symbol_class: sym_class,
        });
    // }
    // None
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_to_json() {
        let signal = Signal {
            action: Some(true),
            symbol: Some(Symbol {
                symbol_type: SymbolType::Crypto("BTC".to_string()),
                symbol_class: Some(SymbolClass::Perpetual),
            }),
            entry: Some(Entry::EntryRange(50000.0, 51000.0)),
            target: Some(Target::ProfitLoss(PricesTarget {
                open: false,
                profits: vec![52000.0, 53000.0],
                stoploss: 49000.0,
            })),
            gales: vec![],
        };

        // Serialize to JSON
        let json = serde_json::to_string(&signal).expect("Failed to serialize Signal");

        // Deserialize back to Signal
        let deserialized: Signal = serde_json::from_str(&json).expect("Failed to deserialize Signal");

        // Check if they are equal
        assert_eq!(signal, deserialized);
        
        // Check specific field
        if let Some(Entry::EntryRange(p1, p2)) = deserialized.entry {
            assert_eq!(p1, 50000.0);
            assert_eq!(p2, 51000.0);
        } else {
            panic!("Entry is not an EntryRange");
        }
    }

    #[test]
    fn test_signal_encapsulation() {
        // Simple single-line signal: "BTCUSDT BUY 45000"
        let text = "BTCUSDT BUY 45000\nTP 46000\nSL 44000\n";
        let signals = parse_signal(text, None, None).expect("Expected signals to be parsed");
        let modified_text = &signals.template;

        // There should be at least one parsed signal
        assert!(!signals.is_empty(), "Expected at least one signal to be parsed");

        // After parse_signal, modified_text should contain encapsulated tokens
        assert!(modified_text.contains("$(symbol)") || modified_text.contains("$(action)") || modified_text.contains("$(target)"),
            "Text should contain encapsulated tokens after parse_signal");
    }

    #[test]
    fn test_parse_template() {
        let original_text = "BTCUSDT BUY 45000\nTP 46000\nSL 44000\n";
        let modified_text = "$(symbol) $(action) $(entry_price)\nTP $(profit_price)\nSL $(stoploss)\n";

        let parsed_signals = parse_template(original_text, modified_text, None, None, None);
        assert!(parsed_signals.is_some(), "Expected parse_template to return signals");

        let signals = parsed_signals.unwrap();
        assert_eq!(signals.len(), 1);
        let signal = &signals[0];
        assert_eq!(signal.action, Some(true));
        assert!(signal.symbol.is_some());
        assert_eq!(signal.entry, Some(Entry::PriceEntry(45000.0)));
        if let Some(Target::ProfitLoss(ref pt)) = signal.target {
            assert_eq!(pt.profits, vec![46000.0]);
            assert_eq!(pt.stoploss, 44000.0);
        } else {
            panic!("Expected Target::ProfitLoss");
        }
    }

    #[test]
    fn test_parse_template_with_positional_metadata() {
        let original_text = "HEADER INFO\nEURUSD BUY 1.0850\n";
        let modified_text = "HEADER INFO\n$(symbol) $(action) $(entry_price)\n";

        // Symbol starts at line index 1, token start 0, delimited by ' '
        let parsed_signals = parse_template(original_text, modified_text, Some(1), Some(0), Some(' '));
        assert!(parsed_signals.is_some(), "Expected parse_template to return signals");

        let signals = parsed_signals.unwrap();
        assert_eq!(signals.len(), 1);
        let signal = &signals[0];
        assert_eq!(signal.action, Some(true));
        assert!(signal.symbol.is_some());
        if let Some(ref sym) = signal.symbol {
            match &sym.symbol_type {
                SymbolType::Forex(pair) => assert_eq!(pair, "EURUSD"),
                SymbolType::Plain(pair) => assert_eq!(pair, "EURUSD"),
                other => panic!("Unexpected symbol type: {:?}", other),
            }
        }
    }

    #[test]
    fn test_first_symbol_start_line_relative() {
        // Multi-line text with header line before symbol line
        let header = "HEADER INFO LINE\n";
        let symbol_line = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800\n";
        let full_text = format!("{}{}", header, symbol_line);

        let parsed = parse_signal(&full_text, None, None).expect("Expected signal to be parsed");
        assert_eq!(parsed.line_index, 1);
        // The symbol "EURUSD" starts at index 0 of line 1 (not index 28 of full_text)
        assert_eq!(parsed.token_start, 0);
        assert!(parsed.token_start <= symbol_line.len());
    }

    #[test]
    fn test_signals_is_binary_and_template_type() {
        // Binary signal with period/expiry
        let binary_text = "EURUSD CALL 5M\n14:30";
        let binary_sigs = parse_signal(binary_text, None, None).expect("Expected binary signal");
        assert!(binary_sigs.is_binary(), "Expected binary signal to be detected as binary");
        assert_eq!(binary_sigs.template_type(), "binary");

        // Market signal with TP/SL
        let market_text = "BTCUSDT BUY 45000\nTP 46000\nSL 44000";
        let market_sigs = parse_signal(market_text, None, None).expect("Expected market signal");
        assert!(!market_sigs.is_binary(), "Expected market signal not to be binary");
        assert_eq!(market_sigs.template_type(), "market");
    }

    #[test]
    fn test_parse_signal_with_custom_checkers() {
        let mut action_val_u = Nodes::new("custom_action_val_u".to_string());
        action_val_u.insert("LONG_ENTRY");
        let custom_checkers = vec![
            Nodes::new("custom_action_V".to_string()),
            action_val_u,
            Nodes::new("custom_action_val_d".to_string()),
            Nodes::new("custom_gale_val_D".to_string()),
            Nodes::new("custom_indicador_val_D".to_string()),
            Nodes::new("custom_info_val".to_string()),
            Nodes::new("custom_symbol_V".to_string()),
            Nodes::new("custom_target_V".to_string()),
            Nodes::new("custom_target_val_d".to_string()),
            Nodes::new("custom_target_val_u".to_string()),
            Nodes::new("custom_timeframe_V".to_string()),
            Nodes::new("custom_timeframe_name_parser_val_D".to_string()),
            Nodes::new("custom_timezone_V".to_string()),
            Nodes::new("custom_symbol_index_val_parser_name_D".to_string()),
        ];
        let custom_symbols = vec![];

        let text = "EURUSD LONG_ENTRY 1.0850\nTP 1.0900\nSL 1.0800";
        let parsed = parse_signal(text, Some(&custom_checkers), Some(&custom_symbols));
        assert!(parsed.is_some());
        let signals = parsed.unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].action, Some(true));
    }

    #[test]
    fn test_signal_timezone_fallback() {
        let mut sig = Signal::default();
        sig.action = Some(true);
        sig.target = Some(Target::PeriodTime("M5".to_string()));
        assert!(!sig.has_timezone());
        assert_eq!(sig.get_timezone(), None);

        // Ensure timezone assigns fallback UTC
        sig.ensure_timezone("UTC");
        assert!(sig.has_timezone());
        assert_eq!(sig.get_timezone(), Some("UTC"));

        // Second call with different tz should not overwrite existing
        sig.ensure_timezone("UTC-3");
        assert_eq!(sig.get_timezone(), Some("UTC"));
    }

    #[test]
    fn test_trade_17_single_signal_with_duplicate_action() {
        let raw_text = "📊 Divisa: GBPCHF-OTC\n⏳ EXPIRACIÓN: M1\n⏱️ ENTRADA: 19:22:00\n🟢 ARIBA / CALL\n\n🚦 Tend: Buy\n📈 Forecast: 98%\n💸 Payout: 77.0%\n\n📲 Link de registro Quotex \nRecarga con 50% más Código: TheKing50";
        let pattern = "📊 Divisa: $(symbol)\n⏳ EXPIRACIÓN: $(period)\n⏱️ ENTRADA: $(entry_time)\n$(dismiss) $(dismiss) / $(action)\n\n🚦 Tend: $(action)\n📈 Forecast: $(dismiss)%\n💸 Payout: $(dismiss)%\n\n📲 Link de registro Quotex \nRecarga con 50% más Código: TheKing50";

        let result = parse_template(raw_text, pattern, Some(0), Some(11), None);
        assert!(result.is_some(), "Expected template to parse");
        let signals = result.unwrap();
        assert_eq!(signals.len(), 1, "Expected exactly 1 signal, not ghost signals");
        let sig = &signals[0];
        assert_eq!(sig.action, Some(true));
        assert!(sig.symbol.is_some());
        if let Some(ref sym) = sig.symbol {
            assert_eq!(sym.symbol_class, Some(SymbolClass::Otc));
            match &sym.symbol_type {
                SymbolType::Forex(pair) => assert_eq!(pair, "GBPCHF"),
                other => panic!("Unexpected symbol type: {:?}", other),
            }
        }
    }

    #[test]
    fn test_trade_promo_messages_return_none() {
        let promo_msg_7 = "50START \n🔺 Código 🔺\n➡️ Recargas 50$ Recibes 75$\n➡️ Recargas 100$ Recibes 150$\nMínimo de recarga 50$ USD\n⚠️ Para utilizar el cupón debes haberte registrado con nuestro Link\n✅ Aprende a crear tu cuenta aquí👇🏽👇🏽👇🏽\n🆘 https://t.me/registropocketoption 🆘\nRegístrese e ingrese al Canal VIP 👑 24 horas de análisis en vivo 📈";
        let pattern_284 = "👑 The King 1 Minuto VIP Pocket Option 👑\n\n💷 $(symbol)\n💎 $(period)\n⌚️ $(entry_time)\n🔼 $(action)\n\n \n\nLink de registro Pocket Option";

        let result_7 = parse_template(promo_msg_7, pattern_284, None, None, None);
        assert!(result_7.is_none(), "Promo message without trade action must return None");

        let promo_msg_31 = "¡ÚLTIMO AVISO! – QUEDAN POCOS LUGARES PARA EL PRÓXIMO BOT DE COMERCIO DE QX\n\n🤖 El lanzamiento del próximo Bot de Comercio de QX está a punto de finalizar.🤖\n⚠️ Los últimos lugares para recibir la clave de acceso se están agotando MUY rápidamente.\n\n💼 EL BOT QUE TODOS QUIEREN:\n\n✔️ Copia automáticamente todas mis operaciones\n✔️ Copia cualquier sala de trading (incluidas las de pago)\n✔️ Funciona de forma autónoma 24/7, incluso con el teléfono apagado\n✔️ Velocidad increíble, sin retrasos\n✔️ Stop/valor/porcentaje totalmente configurables\n✔️ Estrategia profesional sin martingala\n\nhttps://t.me/SuporteOficialTiger";
        let pattern_291 = "✅ ENTRY CONFIRMED ✅\n\n🌎 coin: $(symbol) $(otc)\n⏳ Expiration: $(period)\n📊 Direction: $(dismiss) $(action)\n⏰ time: $(entry_time)\n\n👉 Use up to two Martingale steps in the event of a loss!\n1º GALE: ENDS IN: 17:46h\n2º GALE: ENDS IN: 17:47h\n\n📱 REGISTER HERE";

        let result_31 = parse_template(promo_msg_31, pattern_291, None, None, None);
        assert!(result_31.is_none(), "Promo message 31 without valid symbol/action must return None");
    }
}