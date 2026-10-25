use serde::{Serialize, Deserialize};
// use pyo3::prelude::*;
// use pyo3::types::PyType;
use crate::matchers::{Tokens, Nodes, TokenFlags, TokenClass, ACTION_VAL_U, ACTION_VAL_D, get_flags_and_token_class, get_nodes, get_nodes_ref, check_category};

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
                            }else if let Some(currentsym) = get_symbol_from_text(match_str.as_str(), get_nodes_ref("symbols"), &nodes) {
                                current_signal.symbol = Some(currentsym);
                                line_contains_symbol = true;
                                if !found_first_symbol {
                                    first_symbol_line = line_index;
                                    first_symbol_start = tok_start - line_orig_offset;
                                    first_symbol_end_char = texto[tok_end..].chars().next().unwrap_or('\0');
                                    found_first_symbol = true;
                                }
                                replacements.push((tok_start, tok_end, "symbol"));
                            } else if let Some(currentsym) = get_symbol_from_text(before_match_str.as_str(), get_nodes_ref("symbols"), &nodes) {
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
                            let currentsym = if let Some(symbol) = get_symbol_from_text(match_str.as_str(), get_nodes_ref("symbols"), &nodes) {Some(symbol)} else {get_symbol_from_text(before_match_str.as_str(), get_nodes_ref("symbols"), &nodes)};
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
                            if let Some(mut currentsym) = get_symbol_from_text(before_match_str.as_str(), get_nodes_ref("symbols"), &nodes) {
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
                            } else if let Some(mut currentsym) = get_symbol_from_text(match_str.as_str(), get_nodes_ref("symbols"), &nodes) {
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

#[derive(Debug, Default, Clone)]
pub struct TemplateRequirements {
    pub has_symbol: bool,
    pub has_action: bool,
    pub has_entry: bool,
    pub has_stoploss: bool,
    pub has_profit: bool,
    pub has_period: bool,
    pub has_time: bool,
}

impl TemplateRequirements {
    pub fn from_pattern(pattern: &str) -> Self {
        let mut req = Self::default();
        for line in pattern.lines() {
            let mut rem = line;
            while let Some(start) = rem.find("$(") {
                if let Some(end) = rem[start + 2..].find(')') {
                    let label = &rem[start + 2..start + 2 + end];
                    match label {
                        "symbol" => req.has_symbol = true,
                        "action" => req.has_action = true,
                        "entry_price" | "entry_range" | "entry" | "price" => req.has_entry = true,
                        "stoploss" => req.has_stoploss = true,
                        "profit_price" | "profit" => req.has_profit = true,
                        "period" | "period_digit" | "period_name" => req.has_period = true,
                        "entry_time" | "time_entry" => req.has_time = true,
                        _ => {}
                    }
                    rem = &rem[start + 2 + end + 1..];
                } else {
                    break;
                }
            }
        }
        req
    }

    pub fn is_satisfied_by(&self, signal: &Signal) -> bool {
        if self.has_symbol && signal.symbol.is_none() {
            return false;
        }
        if self.has_action && signal.action.is_none() {
            return false;
        }
        if self.has_entry && signal.entry.is_none() {
            return false;
        }
        if self.has_stoploss {
            match &signal.target {
                Some(Target::ProfitLoss(pt)) if pt.stoploss > 0.0 => {},
                _ => return false,
            }
        }
        if self.has_profit {
            match &signal.target {
                Some(Target::ProfitLoss(pt)) if pt.open || !pt.profits.is_empty() => {},
                _ => return false,
            }
        }
        if self.has_period {
            match &signal.target {
                Some(Target::PeriodTime(pt)) if !pt.is_empty() => {},
                _ => return false,
            }
        }
        if self.has_time {
            match &signal.entry {
                Some(Entry::TimeEntry(_, t)) if !t.is_empty() => {},
                Some(Entry::TimePrice(_, _)) => {},
                _ => return false,
            }
        }
        true
    }
}

fn is_line_match(orig_line: &str, mod_line: &str) -> bool {
    let o_trim = orig_line.trim();
    let m_trim = mod_line.trim();
    if o_trim == m_trim {
        return true;
    }
    if m_trim.is_empty() {
        return o_trim.is_empty();
    }
    if o_trim.is_empty() {
        return false;
    }

    if !mod_line.contains("$(") {
        return o_trim == m_trim;
    }

    let mut matched_static = 0;
    let mut matched_placeholders = 0;
    let mut has_static = false;

    for (mod_tok, orig_tok) in mod_line.zip_tokens(orig_line) {
        if mod_tok.1.contains("$(") {
            matched_placeholders += 1;
        } else if mod_tok.1 == orig_tok.1 {
            if !mod_tok.1.trim().is_empty() {
                matched_static += 1;
            }
        } else {
            return false;
        }
    }

    for tok in mod_line.iter_tokens() {
        if !tok.1.contains("$(") && !tok.1.trim().is_empty() {
            has_static = true;
            break;
        }
    }

    if has_static {
        matched_static > 0
    } else {
        matched_placeholders > 0
    }
}

#[derive(Debug, Clone)]
enum TemplateSegment {
    Literal(String),
    Tag(String),
}

fn collapse_spaces(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_whitespace() || c == '\u{00a0}' {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out
}

fn is_tp_template_line(tpl_line: &str) -> bool {
    if tpl_line.contains("$(profit_price)") || tpl_line.contains("$(profit)") || tpl_line.contains("$(index)") {
        return true;
    }
    if tpl_line.contains("$(") {
        let checkers = get_nodes_ref("checkers");
        for (name, tok) in tpl_line.iter_tokens() {
            if !tok.contains("$(") && !tok.trim().is_empty() {
                let (flags, tc) = get_flags_and_token_class(tok.as_str(), name, checkers);
                if tc == TokenClass::Target && flags.contains(TokenFlags::VALUE_UP) {
                    return true;
                }
            }
        }
    }
    false
}

fn is_main_template_line(tpl_line: &str) -> bool {
    let req = TemplateRequirements::from_pattern(tpl_line);
    req.has_symbol && req.has_action && (req.has_entry || req.has_time)
}

fn remove_ci_substring(haystack: &str, needle_lower: &str) -> String {
    let mut result = haystack.to_string();
    loop {
        let lower = result.to_lowercase();
        if let Some(pos) = lower.find(needle_lower) {
            let end = pos + needle_lower.len();
            if result.is_char_boundary(pos) && result.is_char_boundary(end) {
                result.replace_range(pos..end, "");
                continue;
            }
        }
        break;
    }
    result
}

fn normalize_tp_index_in_template(tpl_line: &str) -> String {
    let mut s = tpl_line.replace("$(profit_price)1", "TP$(index)");
    let lower = s.to_lowercase();
    if let Some(tp_pos) = lower.find("tp") {
        let after_tp = tp_pos + 2;
        let rem = &s[after_tp..];
        // Caso: TP.$(dismiss). o TP.1. o TP.$(index).
        let rem_trim_start = rem.trim_start_matches(|c: char| c == ' ' || c == '\u{00a0}');
        let ws_len = rem.len() - rem_trim_start.len();
        if rem_trim_start.starts_with('.') {
            let after_dot = &rem_trim_start[1..];
            if let Some(rest) = after_dot.strip_prefix("$(dismiss).")
                .or_else(|| after_dot.strip_prefix("$(index)."))
            {
                s = format!("{}TP.$(index).{}", &s[..tp_pos], rest);
                return s;
            }
            let digits_len = after_dot.chars().take_while(|c| c.is_ascii_digit()).count();
            if digits_len > 0 && after_dot[digits_len..].starts_with('.') {
                let rest = &after_dot[digits_len + 1..];
                s = format!("{}TP.$(index).{}", &s[..tp_pos], rest);
                return s;
            }
        }
        // Caso: TP $(dismiss) / TP$(dismiss) / TP $(index) / TP$(index)
        if let Some(rest) = rem_trim_start.strip_prefix("$(dismiss)")
            .or_else(|| rem_trim_start.strip_prefix("$(index)"))
        {
            s = format!("{}{}$(index){}", &s[..tp_pos], &s[tp_pos..after_tp], rest);
            return s;
        }
        // Caso: TP1: / TP 1: / Tp2- / Tp-
        let digits_len = rem_trim_start.chars().take_while(|c| c.is_ascii_digit()).count();
        if digits_len > 0 {
            let rest = &rem_trim_start[digits_len..];
            s = format!("{}{}$(index){}", &s[..tp_pos], &s[tp_pos..after_tp], rest);
            return s;
        } else if ws_len == 0 && rem_trim_start.starts_with('-') {
            s = format!("{}{}$(index){}", &s[..tp_pos], &s[tp_pos..after_tp], rem_trim_start);
            return s;
        }
    }
    s
}

fn preprocess_template_line(tpl_line: &str, orig_line: &str) -> String {
    let mut t = collapse_spaces(tpl_line.trim());
    let orig_has_otc = orig_line.to_lowercase().contains("otc");

    // Regla 2: Si el texto original no contiene "otc" (en cualquier case),
    // obviar completamente $(otc) de los cálculos y extracción.
    if !orig_has_otc && t.to_lowercase().contains("$(otc)") {
        t = remove_ci_substring(&t, "($(otc))");
        t = remove_ci_substring(&t, "-$(otc)");
        t = remove_ci_substring(&t, "_$(otc)");
        t = remove_ci_substring(&t, "$(otc)");
        t = collapse_spaces(t.trim());
    }

    // Regla 3: Normalizar índice de Take Profit (TP$(index) / TP$(dismiss) / TP1)
    if is_tp_template_line(&t) {
        t = normalize_tp_index_in_template(&t);
    }

    t
}

fn parse_template_segments(tpl_line: &str) -> Vec<TemplateSegment> {
    let mut segments = Vec::new();
    let mut rem = tpl_line;
    while let Some(start) = rem.find("$(") {
        if let Some(rel_end) = rem[start + 2..].find(')') {
            if start > 0 {
                segments.push(TemplateSegment::Literal(rem[..start].to_string()));
            }
            let label = &rem[start + 2..start + 2 + rel_end];
            segments.push(TemplateSegment::Tag(label.to_string()));
            rem = &rem[start + 2 + rel_end + 1..];
        } else {
            break;
        }
    }
    if !rem.is_empty() {
        segments.push(TemplateSegment::Literal(rem.to_string()));
    }
    segments
}

/// Busca `lit` dentro de `haystack` de forma case-insensitive y tolerante a espacios múltiples.
/// Retorna `(start_byte_in_haystack, end_byte_in_haystack)`.
fn find_literal_ci(haystack: &str, lit: &str, must_start_at_zero: bool) -> Option<(usize, usize)> {
    if lit.is_empty() {
        return Some((0, 0));
    }
    let lit_trimmed = lit.trim();
    if lit_trimmed.is_empty() {
        // El literal es solo espacio(s)
        if must_start_at_zero {
            let ws_bytes: usize = haystack.chars().take_while(|c| c.is_whitespace() || *c == '\u{00a0}').map(|c| c.len_utf8()).sum();
            return if ws_bytes > 0 { Some((0, ws_bytes)) } else { None };
        } else {
            for (idx, c) in haystack.char_indices() {
                if c.is_whitespace() || c == '\u{00a0}' {
                    let ws_bytes: usize = haystack[idx..].chars().take_while(|ch| ch.is_whitespace() || *ch == '\u{00a0}').map(|ch| ch.len_utf8()).sum();
                    return Some((idx, idx + ws_bytes));
                }
            }
            return None;
        }
    }

    let leading_ws = lit.starts_with(|c: char| c.is_whitespace() || c == '\u{00a0}');
    let trailing_ws = lit.ends_with(|c: char| c.is_whitespace() || c == '\u{00a0}');

    for strict_ws in [true, false] {
        for (start_idx, _) in haystack.char_indices() {
            if must_start_at_zero && start_idx > 0 {
                break;
            }
            let mut h_idx = start_idx;
            if leading_ws {
                let ws: usize = haystack[h_idx..].chars().take_while(|c| c.is_whitespace() || *c == '\u{00a0}').map(|c| c.len_utf8()).sum();
                if strict_ws && ws == 0 && start_idx > 0 {
                    continue;
                }
                h_idx += ws;
            }

            let mut h_chars = haystack[h_idx..].char_indices().peekable();
            let mut l_chars = lit_trimmed.chars().peekable();
            let mut matched = true;
            let mut end_idx = h_idx;

            while let Some(lc) = l_chars.next() {
                if lc.is_whitespace() || lc == '\u{00a0}' {
                    while let Some(&nlc) = l_chars.peek() {
                        if nlc.is_whitespace() || nlc == '\u{00a0}' {
                            l_chars.next();
                        } else {
                            break;
                        }
                    }
                    let mut consumed_ws = false;
                    while let Some(&(rel_i, hc)) = h_chars.peek() {
                        if hc.is_whitespace() || hc == '\u{00a0}' {
                            consumed_ws = true;
                            end_idx = h_idx + rel_i + hc.len_utf8();
                            h_chars.next();
                        } else {
                            break;
                        }
                    }
                    if !consumed_ws {
                        matched = false;
                        break;
                    }
                } else if let Some((rel_i, hc)) = h_chars.next() {
                    if !lc.eq_ignore_ascii_case(&hc) && lc.to_lowercase().to_string() != hc.to_lowercase().to_string() {
                        matched = false;
                        break;
                    }
                    end_idx = h_idx + rel_i + hc.len_utf8();
                } else {
                    matched = false;
                    break;
                }
            }

            if matched {
                if trailing_ws {
                    let ws: usize = haystack[end_idx..].chars().take_while(|c| c.is_whitespace() || *c == '\u{00a0}').map(|c| c.len_utf8()).sum();
                    if strict_ws && ws == 0 && end_idx < haystack.len() {
                        continue;
                    }
                    end_idx += ws;
                }
                return Some((start_idx, end_idx));
            }
        }
    }
    None
}

fn is_indicator_line_match(orig_line: &str, tpl_line: &str) -> bool {
    let checkers = get_nodes_ref("checkers");
    let orig_toks: Vec<(&str, String)> = orig_line.iter_tokens().filter(|t| !t.1.trim().is_empty()).collect();
    let tpl_toks: Vec<(&str, String)> = tpl_line.iter_tokens().filter(|t| !t.1.trim().is_empty()).collect();
    if orig_toks.is_empty() || tpl_toks.is_empty() {
        return false;
    }
    let mut has_indicator = false;
    for (name, val) in &tpl_toks {
        let (_, tc) = get_flags_and_token_class(val.as_str(), name, checkers);
        if tc == TokenClass::Indicator {
            has_indicator = true;
            break;
        }
    }
    if !has_indicator {
        return false;
    }
    let orig_words: Vec<String> = orig_toks.iter()
        .filter(|(n, v)| {
            let (flags, tc) = get_flags_and_token_class(v.as_str(), n, checkers);
            tc != TokenClass::Emoji && !flags.contains(TokenFlags::DIGIT)
        })
        .map(|(_, v)| v.to_lowercase())
        .collect();
    let tpl_words: Vec<String> = tpl_toks.iter()
        .filter(|(n, v)| {
            let (flags, tc) = get_flags_and_token_class(v.as_str(), n, checkers);
            tc != TokenClass::Emoji && !flags.contains(TokenFlags::DIGIT)
        })
        .map(|(_, v)| v.to_lowercase())
        .collect();
    !tpl_words.is_empty() && orig_words == tpl_words
}

fn apply_template_label(
    label: &str,
    raw_val: &str,
    signals: &mut Vec<Signal>,
    current_signal: &mut Signal,
    same_line_entry_count: &mut usize,
) -> bool {
    let orig_val = raw_val.trim();
    let checkers = get_nodes_ref("checkers");
    let symbols_nodes = get_nodes_ref("symbols");

    match label {
        "symbol" => {
            if orig_val.is_empty() {
                return false;
            }
            let lower_val = orig_val.to_lowercase();
            let has_otc_in_val = lower_val.contains("otc");
            let cleaned_for_sym = if has_otc_in_val {
                let mut tmp = remove_ci_substring(orig_val, "(otc)");
                tmp = remove_ci_substring(&tmp, "-otc");
                tmp = remove_ci_substring(&tmp, "_otc");
                tmp = remove_ci_substring(&tmp, "otc");
                tmp.trim().to_string()
            } else {
                orig_val.to_string()
            };

            let mut currentsym = get_symbol_from_text(&cleaned_for_sym, symbols_nodes, checkers)
                .or_else(|| get_symbol_from_text(orig_val, symbols_nodes, checkers));

            if currentsym.is_none() {
                // Intentar buscar en sub-tokens si viene acompañado (ej. "EUR/HUF OTC" o "#INJ/USDT")
                for (_, tok) in cleaned_for_sym.iter_tokens() {
                    let t_trim = tok.trim();
                    if !t_trim.is_empty() && !t_trim.eq_ignore_ascii_case("otc") {
                        if let Some(s) = get_symbol_from_text(t_trim, symbols_nodes, checkers) {
                            currentsym = Some(s);
                            break;
                        }
                    }
                }
            }

            if let Some(ref mut sym) = currentsym {
                if has_otc_in_val {
                    sym.symbol_class = Some(SymbolClass::Otc);
                }
            }

            if let Some(sym) = currentsym.as_ref() {
                if let SymbolType::Plain(_) = sym.symbol_type {
                    if current_signal.symbol.is_none() {
                        current_signal.symbol = Some(sym.clone());
                    }
                    return true;
                }
            }

            if current_signal.symbol.is_some()
                && current_signal.action.is_some()
                && (current_signal.entry.as_ref().map_or(false, |e| e.is_complete())
                    || current_signal.target.as_ref().map_or(false, |t| t.is_complete()))
            {
                let prev = current_signal.clone();
                signals.push(prev.clone());
                *current_signal = Signal::default();
                current_signal.copy_from(&prev);
            }

            if let Some(sym) = currentsym {
                current_signal.symbol = Some(sym);
                true
            } else {
                let clean_val = cleaned_for_sym.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '-');
                if clean_val.len() >= 3 && clean_val.chars().any(|c| c.is_alphabetic()) {
                    current_signal.symbol = Some(Symbol {
                        symbol_type: SymbolType::Plain(clean_val.to_string()),
                        symbol_class: if has_otc_in_val { Some(SymbolClass::Otc) } else { None },
                    });
                    true
                } else {
                    false
                }
            }
        }
        "otc" => {
            let (flags, tc) = get_flags_and_token_class(orig_val, "", checkers);
            if tc != TokenClass::Otc && !orig_val.to_lowercase().contains("otc") && !flags.contains(TokenFlags::SYMBOL_NAME) {
                return false;
            }
            if current_signal.symbol.is_none() {
                current_signal.symbol = Some(Symbol {
                    symbol_type: SymbolType::Plain(orig_val.to_string()),
                    symbol_class: Some(SymbolClass::Otc),
                });
            } else if let Some(ref mut sym) = current_signal.symbol {
                sym.symbol_class = Some(SymbolClass::Otc);
            }
            true
        }
        "action" => {
            if orig_val.is_empty() {
                return false;
            }
            let (flags, _) = get_flags_and_token_class(orig_val, "", checkers);
            let mut detected_action = if flags.contains(TokenFlags::VALUE_UP) {
                Some(true)
            } else if flags.contains(TokenFlags::VALUE_DOWN) {
                Some(false)
            } else {
                None
            };

            if detected_action.is_none() {
                for (t_name, t_str) in orig_val.iter_tokens() {
                    let (t_flags, _) = get_flags_and_token_class(t_str.trim(), t_name, checkers);
                    if t_flags.contains(TokenFlags::VALUE_UP) {
                        detected_action = Some(true);
                        break;
                    } else if t_flags.contains(TokenFlags::VALUE_DOWN) {
                        detected_action = Some(false);
                        break;
                    }
                }
            }

            if let Some(act) = detected_action {
                if current_signal.action.is_none() || current_signal.symbol.is_none() {
                    current_signal.action = Some(act);
                }
                true
            } else if current_signal.action.is_some() && !orig_val.is_empty() {
                // Línea secundaria de tendencia (ej. "🚦 Tend: neutral" cuando la acción principal ya fue extraída)
                true
            } else {
                false
            }
        }
        "period" => {
            if orig_val.is_empty() {
                return false;
            }
            if current_signal.is_complete() && current_signal.target.is_some() {
                signals.push(current_signal.clone());
                *current_signal = Signal::default();
            }
            current_signal.target = Some(Target::PeriodTime(orig_val.to_string()));
            true
        }
        "period_digit" | "period_name" => {
            if orig_val.is_empty() {
                return false;
            }
            if let Some(Target::PeriodTime(ref mut pt)) = current_signal.target {
                if pt.chars().all(|c| c.is_numeric()) {
                    let prefix = orig_val.chars().next().map(|c| c.to_string().to_uppercase()).unwrap_or_default();
                    *pt = prefix + pt;
                } else if pt.chars().all(|c| c.is_alphabetic()) {
                    pt.push_str(orig_val);
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
            true
        }
        "timezone" => {
            if orig_val.is_empty() {
                return false;
            }
            let tz = orig_val.replace(' ', "");
            current_signal.add_time_zone(tz);
            true
        }
        "entry_time" | "time_entry" => {
            if orig_val.is_empty() {
                return false;
            }
            if current_signal.is_complete()
                || (current_signal.entry.as_ref().map_or(false, |e| e.is_complete())
                    && current_signal.symbol.is_some()
                    && current_signal.action.is_some())
            {
                let prev = current_signal.clone();
                signals.push(prev.clone());
                *current_signal = Signal::default();
                current_signal.copy_from(&prev);
            }
            let (flags, _) = get_flags_and_token_class(orig_val, "", checkers);
            let normalized = if flags.contains(TokenFlags::SINGLE_QUOTATION) {
                orig_val.replace('\'', ":")
            } else {
                orig_val.to_string()
            };
            let time_str = if normalized.contains(':') {
                let parts: Vec<&str> = normalized.split(':').collect();
                match parts.as_slice() {
                    [h, m, s] => {
                        let s_clean = s.trim_end_matches(|c: char| !c.is_ascii_digit());
                        if let (Ok(h), Ok(m), Ok(s)) = (h.parse::<u32>(), m.parse::<u32>(), s_clean.parse::<u32>()) {
                            format!("{:02}:{:02}:{:02}", h, m, s)
                        } else {
                             return false;
                        }
                    }
                    [h, m] => {
                        let m_clean = m.trim_end_matches(|c: char| !c.is_ascii_digit());
                        if let (Ok(h), Ok(m)) = (h.parse::<u32>(), m_clean.parse::<u32>()) {
                            format!("{:02}:{:02}:00", h, m)
                        } else {
                            return false;
                        }
                    }
                    _ => return false,
                }
            } else {
                return false;
            };

            if let Some(Entry::TimeEntry(ref tz, _)) = current_signal.entry {
                current_signal.entry = Some(Entry::TimeEntry(tz.clone(), time_str));
            } else if let Some(Entry::TimePrice(ref tz, _)) = current_signal.entry {
                current_signal.entry = Some(Entry::TimeEntry(tz.clone(), time_str));
            } else {
                current_signal.entry = Some(Entry::TimeEntry(String::new(), time_str));
            }
            true
        }
        "entry_range" => {
            let (flags, _) = get_flags_and_token_class(orig_val, "", checkers);
            let range_str = if flags.contains(TokenFlags::COMMA) {
                orig_val.replace(',', ".")
            } else {
                orig_val.to_string()
            };
            let range: Vec<&str> = range_str
                .split(&['-', '/', '\\', '_'][..])
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if range.len() == 2 {
                let p1 = range[0].parse::<f32>().unwrap_or(0.0);
                let p2 = range[1].parse::<f32>().unwrap_or(0.0);
                if p1 > 0.0 && p2 > 0.0 {
                    current_signal.entry = Some(Entry::EntryRange(p1, p2));
                    return true;
                }
            } else if let Ok(price) = range_str.trim().parse::<f32>() {
                if price > 0.0 {
                    if let Some(Entry::PriceEntry(p1)) = current_signal.entry {
                        current_signal.entry = Some(Entry::EntryRange(p1, price));
                    } else {
                        current_signal.entry = Some(Entry::PriceEntry(price));
                    }
                    return true;
                }
            }
            false
        }
        "entry_price" | "entry" | "price" => {
            let (flags, _) = get_flags_and_token_class(orig_val, "", checkers);
            let clean_num = if flags.contains(TokenFlags::COMMA) {
                orig_val.replace(',', ".")
            } else {
                orig_val.to_string()
            };
            let price = clean_num.trim_matches(|c: char| !c.is_ascii_digit() && c != '.').parse::<f32>().unwrap_or(0.0);
            if price > 0.0 {
                *same_line_entry_count += 1;
                if let Some(Entry::PriceEntry(p1)) = current_signal.entry {
                    if *same_line_entry_count > 1 {
                        current_signal.entry = Some(Entry::EntryRange(p1, price));
                    }
                } else if let Some(Entry::TimeEntry(ref tz, ref t)) = current_signal.entry {
                    if t.is_empty() {
                        current_signal.entry = Some(Entry::TimePrice(tz.clone(), price));
                    }
                } else if current_signal.entry.is_none() {
                    current_signal.entry = Some(Entry::PriceEntry(price));
                }
                true
            } else {
                false
            }
        }
        "profit_price" | "profit" => {
            if orig_val.eq_ignore_ascii_case("open") {
                if let Some(Target::ProfitLoss(ref mut pt)) = current_signal.target {
                    pt.open = true;
                } else {
                    current_signal.target = Some(Target::ProfitLoss(PricesTarget {
                        open: true,
                        profits: vec![],
                        stoploss: 0.0,
                    }));
                }
                true
            } else {
                let (flags, _) = get_flags_and_token_class(orig_val, "", checkers);
                let clean_num = if flags.contains(TokenFlags::COMMA) {
                    orig_val.replace(',', ".")
                } else {
                    orig_val.to_string()
                };
                let price = clean_num.trim_matches(|c: char| !c.is_ascii_digit() && c != '.').parse::<f32>().unwrap_or(0.0);
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
                    true
                } else {
                    false
                }
            }
        }
        "stoploss" => {
            let (flags, tc) = get_flags_and_token_class(orig_val, "", checkers);
            let clean_num = if flags.contains(TokenFlags::COMMA) {
                orig_val.replace(',', ".")
            } else {
                orig_val.to_string()
            };
            let price = clean_num.trim_matches(|c: char| !c.is_ascii_digit() && c != '.').parse::<f32>().unwrap_or(0.0);
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
                true
            } else {
                // Si la plantilla etiquetó la palabra "SL" como $(stoploss) antes del precio
                tc == TokenClass::Target && flags.contains(TokenFlags::VALUE_DOWN)
            }
        }
        "gale" => {
            let (flags, _) = get_flags_and_token_class(orig_val, "", checkers);
            if let Ok(num) = orig_val.parse::<i8>() {
                current_signal.gales.push(Gale::Number(num));
                true
            } else if flags.contains(TokenFlags::COLON) {
                current_signal.gales.push(Gale::TimeEntry(orig_val.to_string()));
                true
            } else if flags.contains(TokenFlags::NEGATIVE) || orig_val.to_lowercase().contains("no") || orig_val == "0" {
                current_signal.gales.push(Gale::NoGale);
                true
            } else if let Some(digit_char) = orig_val.chars().find(|c| c.is_ascii_digit()) {
                if let Some(digit) = digit_char.to_digit(10) {
                    current_signal.gales.push(Gale::Number(digit as i8));
                    return true;
                }
                false
            } else {
                false
            }
        }
        "index" => {
            orig_val.is_empty() || orig_val.chars().all(|c| c.is_ascii_digit() || c == '.' || c.is_whitespace())
        }
        "_" => !orig_val.contains(char::is_whitespace),
        "_line" | "_lines" => true,
        "indicator" | "date" | "custom" | "dismiss" | "variable" | _ => !orig_val.is_empty(),
    }
}

fn is_ignored_label(lbl: &str) -> bool {
    matches!(lbl, "_" | "_line" | "_lines")
}

fn strip_tag_with_delimiters(tpl_line: &str, tag: &str) -> String {
    let mut s = tpl_line.to_string();
    if !s.contains(tag) {
        return s;
    }
    s = remove_ci_substring(&s, &format!("({})", tag));
    s = remove_ci_substring(&s, &format!("[{}]", tag));
    s = remove_ci_substring(&s, &format!("- {}", tag));
    s = remove_ci_substring(&s, &format!("-{}", tag));
    s = remove_ci_substring(&s, &format!("/ {}", tag));
    s = remove_ci_substring(&s, &format!("/{}", tag));
    s = remove_ci_substring(&s, &format!("| {}", tag));
    s = remove_ci_substring(&s, &format!("|{}", tag));
    s = remove_ci_substring(&s, &format!("_{}", tag));
    s = remove_ci_substring(&s, tag);
    collapse_spaces(s.trim())
}

fn strip_optional_underscore(tpl_line: &str) -> String {
    strip_tag_with_delimiters(tpl_line, "$(_)")
}

fn strip_all_optional_tags(tpl_line: &str) -> String {
    let s = strip_tag_with_delimiters(tpl_line, "$(_lines)");
    let s = strip_tag_with_delimiters(&s, "$(_line)");
    strip_tag_with_delimiters(&s, "$(_)")
}

/// Elimina del template exactamente `n` etiquetas `$(_)` (las de más a la derecha),
/// junto con sus delimitadores, como si nunca hubieran existido. Las demás se conservan.
fn strip_optional_underscore_n(tpl_line: &str, n: usize) -> String {
    const MARK: &str = "\u{1}";
    let total = tpl_line.matches("$(_)").count();
    if n >= total {
        return strip_optional_underscore(tpl_line);
    }
    // Protege las primeras (total - n) apariciones con un marcador temporal
    let keep = total - n;
    let mut protected = String::new();
    let mut rest = tpl_line;
    for _ in 0..keep {
        if let Some(p) = rest.find("$(_)") {
            protected.push_str(&rest[..p]);
            protected.push_str(MARK);
            rest = &rest[p + 4..];
        }
    }
    protected.push_str(rest);
    let stripped = strip_optional_underscore(&protected);
    collapse_spaces(stripped.replace(MARK, "$(_)").trim())
}

fn is_optional_template_line(tpl_line: &str) -> bool {
    let t = tpl_line.trim();
    if !t.contains("$(_)") && !t.contains("$(_line)") && !t.contains("$(_lines)") {
        return false;
    }
    let req = TemplateRequirements::from_pattern(t);
    !req.has_symbol && !req.has_action && !req.has_entry && !req.has_stoploss && !req.has_profit && !req.has_period && !req.has_time
}

fn is_multiline_optional_template_line(tpl_line: &str) -> bool {
    tpl_line.contains("$(_lines)") && is_optional_template_line(tpl_line)
}

fn is_pure_wildcard_line(tpl_line: &str) -> bool {
    strip_all_optional_tags(tpl_line).is_empty()
}

/// Compara una línea original contra una línea de template y extrae los valores
/// de las etiquetas `$(etiqueta)` hacia `signals` / `current_signal`.
///
/// # Parámetros
/// - `orig_line`: línea del mensaje original.
/// - `tpl_line`: línea del template (literales + etiquetas `$(...)`).
/// - `end_char_new_line`: indica si la línea termina en salto de línea; con `Some(false)`
///   la etiqueta `$(symbol)` final solo captura hasta el primer espacio.
/// - `signals`: lista de señales acumuladas, modificada por `apply_template_label`.
/// - `current_signal`: señal en construcción, modificada por `apply_template_label`.
///
/// # Retorno
/// - `Some((total_items, matched_items))`: ítems evaluados (literales y etiquetas, sin contar
///   `$(_)`) y cuántos coincidieron.
/// - `None`: la línea no coincide con el template.
///
/// # Funcionamiento
/// El template se divide en segmentos (literales y etiquetas). Los literales deben aparecer
/// al inicio de la posición actual; las etiquetas capturan el texto hasta el siguiente literal
/// (o hasta el final de la línea). Al terminar no debe quedar texto sin consumir.
fn match_and_extract_line_core(
    orig_line: &str,
    tpl_line: &str,
    end_char_new_line: Option<bool>,
    signals: &mut Vec<Signal>,
    current_signal: &mut Signal,
) -> Option<(usize, usize)> {
    // Normaliza espacios de la línea original y preprocesa el template
    let o_clean = collapse_spaces(orig_line.trim());
    let t_clean = preprocess_template_line(tpl_line, orig_line);

    // Template vacío: solo coincide con una línea vacía; línea vacía con template no vacío no coincide
    if t_clean.is_empty() {
        return if o_clean.is_empty() { Some((0, 0)) } else { None };
    }
    if o_clean.is_empty() {
        return None;
    }

    // Si la línea del template no tiene ninguna etiqueta $(etiqueta)
    if !t_clean.contains("$(") {
        // Comparación literal sin distinguir mayúsculas/minúsculas
        if o_clean.eq_ignore_ascii_case(&t_clean) || o_clean.to_lowercase() == t_clean.to_lowercase() {
            return Some((1, 1));
        }
        // Líneas indicadoras (p. ej. encabezados) con coincidencia flexible
        if is_indicator_line_match(&o_clean, &t_clean) {
            return Some((1, 1));
        }
        return None;
    }

    let segments = parse_template_segments(&t_clean);
    let mut pos = 0usize; // posición actual (bytes) en o_clean
    let mut total_items = 0usize; // ítems evaluados
    let mut matched_items = 0usize; // ítems que coincidieron
    let mut same_line_entry_count = 0usize; // contador de entradas en la misma línea
    let mut idx = 0usize; // índice del segmento actual

    while idx < segments.len() {
        match &segments[idx] {
            TemplateSegment::Literal(lit) => {
                // Un literal debe aparecer exactamente en la posición actual
                total_items += 1;
                if let Some((m_start, m_end)) = find_literal_ci(&o_clean[pos..], lit, true) {
                    if m_start == 0 {
                        matched_items += 1;
                        pos += m_end;
                        idx += 1;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
            TemplateSegment::Tag(_) => {
                // Agrupar tags consecutivos sin literal intermedio (ej. $(period_digit)$(period_name))
                let tag_start = idx;
                while idx < segments.len() && matches!(segments[idx], TemplateSegment::Tag(_)) {
                    idx += 1;
                }
                let tag_slice = &segments[tag_start..idx];

                // Buscar el siguiente literal estático que delimita a este grupo de tags
                let (captured, next_pos, lit_matched) = if idx < segments.len() {
                    if let TemplateSegment::Literal(next_lit) = &segments[idx] {
                        // Caso especial: si el único tag es $(index) y next_lit empieza después de dígitos opcionales.
                        // Se intenta capturar espacios + dígitos antes del literal; si no se puede,
                        // el índice queda vacío y el literal debe estar justo en la posición actual.
                        if tag_slice.len() == 1 {
                            if let TemplateSegment::Tag(lbl) = &tag_slice[0] {
                                if lbl == "index" {
                                    let rem = &o_clean[pos..];
                                    let lead_ws: usize = rem
                                        .chars()
                                        .take_while(|c| *c == ' ' || *c == '\u{00a0}')
                                        .map(|c| c.len_utf8())
                                        .sum();
                                    let digit_bytes: usize = rem[lead_ws..]
                                        .chars()
                                        .take_while(|c| c.is_ascii_digit())
                                        .map(|c| c.len_utf8())
                                        .sum();
                                    let cand_with_digits = if digit_bytes > 0 { lead_ws + digit_bytes } else { 0 };
                                    if cand_with_digits > 0 {
                                        if let Some((m_start, m_end)) = find_literal_ci(&rem[cand_with_digits..], next_lit, true) {
                                            if m_start == 0 {
                                                idx += 1;
                                                (&rem[..cand_with_digits], pos + cand_with_digits + m_end, true)
                                            } else if let Some((0, m_end0)) = find_literal_ci(rem, next_lit, true) {
                                                idx += 1;
                                                ("", pos + m_end0, true)
                                            } else {
                                                return None;
                                            }
                                        } else if let Some((0, m_end0)) = find_literal_ci(rem, next_lit, true) {
                                            idx += 1;
                                            ("", pos + m_end0, true)
                                        } else {
                                            return None;
                                        }
                                    } else if let Some((0, m_end0)) = find_literal_ci(rem, next_lit, true) {
                                        idx += 1;
                                        ("", pos + m_end0, true)
                                    } else {
                                        return None;
                                    }
                                } else if let Some((m_start, m_end)) = find_literal_ci(&o_clean[pos..], next_lit, false) {
                                    // Otra etiqueta: captura todo hasta el siguiente literal
                                    idx += 1;
                                    (&o_clean[pos..pos + m_start], pos + m_end, true)
                                } else {
                                    return None;
                                }
                            } else {
                                unreachable!()
                            }
                        } else if let Some((m_start, m_end)) = find_literal_ci(&o_clean[pos..], next_lit, false) {
                            idx += 1;
                            (&o_clean[pos..pos + m_start], pos + m_end, true)
                        } else {
                            return None;
                        }
                    } else {
                        unreachable!()
                    }
                } else {
                    // Fin de la línea: la etiqueta captura el resto del texto.
                    // Si la línea no termina en salto y la etiqueta es $(symbol), solo toma el primer token.
                    let rem = &o_clean[pos..];
                    if end_char_new_line == Some(false) && tag_slice.len() == 1 {
                        if let TemplateSegment::Tag(lbl) = &tag_slice[0] {
                            if lbl == "symbol" {
                                let token_end = rem.find(' ').unwrap_or(rem.len());
                                (&rem[..token_end], pos + rem.len(), false)
                            } else {
                                (rem, o_clean.len(), false)
                            }
                        } else {
                            (rem, o_clean.len(), false)
                        }
                    } else {
                        (rem, o_clean.len(), false)
                    }
                };

                // El literal delimitador también cuenta como ítem coincidente
                if lit_matched {
                    total_items += 1;
                    matched_items += 1;
                }

                if tag_slice.len() == 1 {
                    if let TemplateSegment::Tag(lbl) = &tag_slice[0] {
                        if !is_ignored_label(lbl) {
                            total_items += 1;
                        }
                        if apply_template_label(lbl, captured, signals, current_signal, &mut same_line_entry_count) {
                            if !is_ignored_label(lbl) {
                                matched_items += 1;
                            }
                        } else {
                            return None;
                        }
                    }
                } else if tag_slice.len() == 2 {
                    // Caso 2: dos etiquetas consecutivas; se divide el texto capturado en dos partes
                    // (dígitos / letras para period_digit / period_name, o por el primer espacio)
                    if let (TemplateSegment::Tag(l1), TemplateSegment::Tag(l2)) = (&tag_slice[0], &tag_slice[1]) {
                        if !is_ignored_label(l1) {
                            total_items += 1;
                        }
                        if !is_ignored_label(l2) {
                            total_items += 1;
                        }
                        let cap_trim = captured.trim();
                        let split_idx = if l1 == "period_digit" {
                            cap_trim.chars().take_while(|c| c.is_ascii_digit()).map(|c| c.len_utf8()).sum()
                        } else if l1 == "period_name" {
                            cap_trim.chars().take_while(|c| c.is_alphabetic()).map(|c| c.len_utf8()).sum()
                        } else {
                            cap_trim.find(' ').unwrap_or(cap_trim.len())
                        };
                        let (part1, part2) = cap_trim.split_at(split_idx);
                        if apply_template_label(l1, part1.trim(), signals, current_signal, &mut same_line_entry_count)
                            && apply_template_label(l2, part2.trim(), signals, current_signal, &mut same_line_entry_count)
                        {
                            if !is_ignored_label(l1) {
                                matched_items += 1;
                            }
                            if !is_ignored_label(l2) {
                                matched_items += 1;
                            }
                        } else {
                            return None;
                        }
                    }
                } else {
                    // Caso 3: tres o más etiquetas; todas reciben el mismo texto capturado
                    for seg in tag_slice {
                        if let TemplateSegment::Tag(lbl) = seg {
                            if !is_ignored_label(lbl) {
                                total_items += 1;
                            }
                            if apply_template_label(lbl, captured, signals, current_signal, &mut same_line_entry_count) {
                                if !is_ignored_label(lbl) {
                                    matched_items += 1;
                                }
                            } else {
                                return None;
                            }
                        }
                    }
                }

                // Avanza más allá del texto capturado y del literal delimitador
                pos = next_pos;
            }
        }
    }

    // Verificar que no haya quedado texto estático sobrante sin consumir en orig_line
    // (el resultado es el par de contadores usado para puntuar la coincidencia)
    if !o_clean[pos..].trim().is_empty() {
        return None;
    }

    Some((total_items, matched_items))
}

fn match_and_extract_line(
    orig_line: &str,
    tpl_line: &str,
    end_char_new_line: Option<bool>,
    signals: &mut Vec<Signal>,
    current_signal: &mut Signal,
) -> Option<(usize, usize)> {
    if tpl_line.contains("$(_)") || tpl_line.contains("$(_line)") || tpl_line.contains("$(_lines)") {
        let backup_signals = signals.clone();
        let backup_current_signal = current_signal.clone();

        // Compara la cantidad de tokens del template con la de la línea original.
        // Si el template excede en N tokens, se eliminan N etiquetas $(_) del template
        // (como si no existieran) en lugar de omitir tokens de la línea.
        // Si no excede, $(_) consume (y omite) el token correspondiente.
        let tpl_tokens = preprocess_template_line(tpl_line, orig_line).split_whitespace().count();
        let orig_tokens = orig_line.split_whitespace().count();
        let excess = tpl_tokens.saturating_sub(orig_tokens);

        // Candidatos en orden de preferencia: (template a usar)
        let stripped = strip_optional_underscore_n(tpl_line, excess);
        let all_stripped = strip_all_optional_tags(tpl_line);
        let mut candidates: Vec<String> = if excess > 0 && tpl_line.contains("$(_)") {
            // Primero sin las $(_) sobrantes; si falla, el template original
            vec![stripped, tpl_line.to_string()]
        } else {
            // Primero consumiendo $(_) / $(_line) / $(_lines); si falla, sin ninguna etiqueta opcional
            vec![tpl_line.to_string(), strip_optional_underscore(tpl_line)]
        };
        if !candidates.contains(&all_stripped) {
            candidates.push(all_stripped);
        }

        for cand in candidates {
            if cand.is_empty() {
                continue;
            }
            if let Some(res) = match_and_extract_line_core(orig_line, &cand, end_char_new_line, signals, current_signal) {
                return Some(res);
            }
            // Restaurar estado antes del siguiente intento
            *signals = backup_signals.clone();
            *current_signal = backup_current_signal.clone();
        }

        return None;
    }

    match_and_extract_line_core(orig_line, tpl_line, end_char_new_line, signals, current_signal)
}

/// Analiza un texto original de señales de trading comparándolo contra un texto plantilla que contiene etiquetas `$(etiqueta)`.
/// Retorna `Option<Vec<Signal>>` conteniendo la lista de señales extraídas si existe un 100% de coincidencia, o `None`.
pub fn parse_template(original_text: &str, template_text: &str, end_char_new_line: Option<bool>) -> Option<Vec<Signal>> {
    let orig_lines: Vec<&str> = original_text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    let raw_tpl_lines: Vec<&str> = template_text.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();

    if orig_lines.is_empty() || raw_tpl_lines.is_empty() {
        return None;
    }

    // Expandir cualquier línea que contenga $(_lines) junto con otros tokens en la misma línea:
    // convierte la parte de la línea en $(_line) y agrega una línea separada $(_lines) para el resto de líneas.
    let mut expanded_tpl_lines: Vec<String> = Vec::new();
    for &line in &raw_tpl_lines {
        if line.contains("$(_lines)") && !is_pure_wildcard_line(line) {
            let trimmed = line.trim();
            if trimmed.starts_with("$(_lines)") {
                expanded_tpl_lines.push("$(_lines)".to_string());
                let rest = trimmed.replacen("$(_lines)", "$(_line)", 1);
                expanded_tpl_lines.push(rest);
            } else {
                let replaced = trimmed.replace("$(_lines)", "$(_line)");
                expanded_tpl_lines.push(replaced);
                expanded_tpl_lines.push("$(_lines)".to_string());
            }
        } else {
            expanded_tpl_lines.push(line.to_string());
        }
    }
    let tpl_lines: Vec<&str> = expanded_tpl_lines.iter().map(|s| s.as_str()).collect();

    let mut signals: Vec<Signal> = Vec::new();
    let mut current_signal = Signal::default();
    let mut total_items = 0usize;
    let mut matched_items = 0usize;
    let mut visited_tpl_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();

    let first_is_main = is_main_template_line(tpl_lines[0]);

    if first_is_main && tpl_lines.len() > 1 {
        // Dividir orig_lines en bloques por cada línea que coincida con la línea principal tpl_lines[0]
        let mut block_starts = vec![0usize];
        for i in 1..orig_lines.len() {
            let mut probe_sigs = Vec::new();
            let mut probe_sig = Signal::default();
            if match_and_extract_line(orig_lines[i], tpl_lines[0], end_char_new_line, &mut probe_sigs, &mut probe_sig).is_some()
                && probe_sig.symbol.is_some()
                && probe_sig.action.is_some()
                && probe_sig.entry.is_some()
            {
                block_starts.push(i);
            }
        }

        let has_required_subsequent = tpl_lines[1..].iter().any(|l| !is_optional_template_line(l));

        for b_idx in 0..block_starts.len() {
            let start = block_starts[b_idx];
            let end = if b_idx + 1 < block_starts.len() {
                block_starts[b_idx + 1]
            } else {
                orig_lines.len()
            };
            let block = &orig_lines[start..end];
            if block.len() < 2 && has_required_subsequent {
                return None;
            }

            let mut sig_local = Signal::default();
            // 1. Evaluar primera línea como línea principal (symbol + action + entry)
            let (t0, m0) = match_and_extract_line(block[0], tpl_lines[0], end_char_new_line, &mut signals, &mut sig_local)?;
            if t0 == 0 || m0 < t0 || sig_local.symbol.is_none() || sig_local.action.is_none() || sig_local.entry.is_none() {
                return None;
            }
            visited_tpl_lines.insert(0);
            total_items += t0;
            matched_items += m0;

            // 2. Iterar las líneas restantes en reversa desde la última hacia la línea 1
            let mut orig_idx = block.len() as isize - 1;
            let mut tpl_idx = tpl_lines.len() as isize - 1;

            while orig_idx >= 1 && tpl_idx >= 1 {
                let u_orig = orig_idx as usize;
                let u_tpl = tpl_idx as usize;

                if is_optional_template_line(tpl_lines[u_tpl]) {
                    let mut matches_earlier = false;
                    let mut look = u_tpl as isize - 1;
                    while look >= 1 {
                        if !is_pure_wildcard_line(tpl_lines[look as usize]) {
                            let mut probe_sigs = Vec::new();
                            let mut probe_sig = sig_local.clone();
                            if match_and_extract_line(block[u_orig], tpl_lines[look as usize], end_char_new_line, &mut probe_sigs, &mut probe_sig).is_some() {
                                matches_earlier = true;
                                break;
                            }
                        }
                        if !is_optional_template_line(tpl_lines[look as usize]) {
                            break;
                        }
                        look -= 1;
                    }
                    if matches_earlier {
                        tpl_idx -= 1;
                        continue;
                    }
                }

                if let Some((t_cnt, m_cnt)) = match_and_extract_line(
                    block[u_orig],
                    tpl_lines[u_tpl],
                    end_char_new_line,
                    &mut signals,
                    &mut sig_local,
                ) {
                    visited_tpl_lines.insert(u_tpl);
                    total_items += t_cnt;
                    matched_items += m_cnt;

                    if is_multiline_optional_template_line(tpl_lines[u_tpl]) {
                        // $(_lines) consume la línea actual y se mantiene activa para más líneas
                        orig_idx -= 1;
                    } else if is_tp_template_line(tpl_lines[u_tpl]) {
                        orig_idx -= 1;
                        // Continuar iterando sobre múltiples líneas de Take Profit sin avanzar tpl_idx
                        while orig_idx >= 1 {
                            let next_u_orig = orig_idx as usize;
                            if let Some((tp_t, tp_m)) = match_and_extract_line(
                                block[next_u_orig],
                                tpl_lines[u_tpl],
                                end_char_new_line,
                                &mut signals,
                                &mut sig_local,
                            ) {
                                total_items += tp_t;
                                matched_items += tp_m;
                                orig_idx -= 1;
                            } else {
                                break;
                            }
                        }
                        tpl_idx -= 1;
                        // Si el template tenía más de una línea TP consecutiva ya absorbida, marcarla visitada
                        while tpl_idx >= 1 && is_tp_template_line(tpl_lines[tpl_idx as usize]) {
                            if orig_idx >= 1 && match_and_extract_line(
                                block[orig_idx as usize],
                                tpl_lines[tpl_idx as usize],
                                end_char_new_line,
                                &mut Vec::new(),
                                &mut sig_local.clone(),
                            ).is_some() {
                                break;
                            }
                            visited_tpl_lines.insert(tpl_idx as usize);
                            tpl_idx -= 1;
                        }
                    } else {
                        orig_idx -= 1;
                        tpl_idx -= 1;
                    }
                } else if is_optional_template_line(tpl_lines[u_tpl]) {
                    tpl_idx -= 1;
                } else {
                    return None;
                }
            }

            while tpl_idx >= 1 && is_optional_template_line(tpl_lines[tpl_idx as usize]) {
                tpl_idx -= 1;
            }

            // Si quedaron líneas sin coincidir en el bloque o en el template, no hay 100% de coincidencia
            if orig_idx >= 1 || tpl_idx >= 1 {
                return None;
            }

            // Reordenar los take profits (fueron leídos desde el último hasta el primero en reversa)
            if let Some(Target::ProfitLoss(ref mut pt)) = sig_local.target {
                pt.profits.reverse();
            }
            signals.push(sig_local);
        }
    } else {
        // Iteración hacia adelante (soporta múltiples señales en el mismo texto)
        let mut orig_idx = 0usize;
        let mut tpl_idx = 0usize;

        while orig_idx < orig_lines.len() {
            if tpl_idx < tpl_lines.len() {
                if is_optional_template_line(tpl_lines[tpl_idx]) {
                    let mut matches_future = false;
                    for lookahead in (tpl_idx + 1)..tpl_lines.len() {
                        if !is_pure_wildcard_line(tpl_lines[lookahead]) {
                            let mut probe_sigs = Vec::new();
                            let mut probe_sig = current_signal.clone();
                            if match_and_extract_line(orig_lines[orig_idx], tpl_lines[lookahead], end_char_new_line, &mut probe_sigs, &mut probe_sig).is_some() {
                                matches_future = true;
                                break;
                            }
                        }
                        if !is_optional_template_line(tpl_lines[lookahead]) {
                            break;
                        }
                    }
                    if matches_future {
                        tpl_idx += 1;
                        continue;
                    }

                    // Si el resto del template es puramente opcional y esta línea inicia una nueva señal, saltar al reinicio
                    let remaining_all_optional = ((tpl_idx + 1)..tpl_lines.len()).all(|i| is_optional_template_line(tpl_lines[i]));
                    if remaining_all_optional && tpl_idx > 0 && current_signal.symbol.is_some() && current_signal.action.is_some() {
                        let mut matches_restart = false;
                        for cand_idx in 0..tpl_idx {
                            if !is_pure_wildcard_line(tpl_lines[cand_idx])
                                && (cand_idx == 0
                                    || tpl_lines[cand_idx].contains("$(symbol)")
                                    || tpl_lines[cand_idx].contains("$(entry_time)")
                                    || tpl_lines[cand_idx].contains("$(period)"))
                            {
                                let mut probe_signals = Vec::new();
                                let mut probe_sig = Signal::default();
                                if match_and_extract_line(orig_lines[orig_idx], tpl_lines[cand_idx], end_char_new_line, &mut probe_signals, &mut probe_sig).is_some() {
                                    matches_restart = true;
                                    break;
                                }
                            }
                        }
                        if matches_restart {
                            tpl_idx = tpl_lines.len();
                            continue;
                        }
                    }
                }

                if let Some((t_cnt, m_cnt)) = match_and_extract_line(
                    orig_lines[orig_idx],
                    tpl_lines[tpl_idx],
                    end_char_new_line,
                    &mut signals,
                    &mut current_signal,
                ) {
                    visited_tpl_lines.insert(tpl_idx);
                    total_items += t_cnt;
                    matched_items += m_cnt;

                    if is_multiline_optional_template_line(tpl_lines[tpl_idx]) {
                        // $(_lines) consume todas las líneas siguientes hasta que coincida una línea posterior o reinicio
                        orig_idx += 1;
                    } else if is_tp_template_line(tpl_lines[tpl_idx]) {
                        orig_idx += 1;
                        while orig_idx < orig_lines.len() {
                            if let Some((tp_t, tp_m)) = match_and_extract_line(
                                orig_lines[orig_idx],
                                tpl_lines[tpl_idx],
                                end_char_new_line,
                                &mut signals,
                                &mut current_signal,
                            ) {
                                total_items += tp_t;
                                matched_items += tp_m;
                                orig_idx += 1;
                            } else {
                                break;
                            }
                        }
                        tpl_idx += 1;
                        while tpl_idx < tpl_lines.len() && is_tp_template_line(tpl_lines[tpl_idx]) {
                            if orig_idx < orig_lines.len() && match_and_extract_line(
                                orig_lines[orig_idx],
                                tpl_lines[tpl_idx],
                                end_char_new_line,
                                &mut Vec::new(),
                                &mut current_signal.clone(),
                            ).is_some() {
                                break;
                            }
                            visited_tpl_lines.insert(tpl_idx);
                            tpl_idx += 1;
                        }
                    } else {
                        orig_idx += 1;
                        tpl_idx += 1;
                    }
                    continue;
                } else if is_optional_template_line(tpl_lines[tpl_idx]) {
                    tpl_idx += 1;
                    continue;
                }
            }

            // Verificar si inicia una nueva señal repitiendo desde el inicio del bloque de señal en el template
            if tpl_idx > 0 && current_signal.symbol.is_some() && current_signal.action.is_some() {
                let mut restart_found = None;
                for cand_idx in 0..tpl_idx {
                    if cand_idx == 0
                        || tpl_lines[cand_idx].contains("$(symbol)")
                        || tpl_lines[cand_idx].contains("$(entry_time)")
                        || tpl_lines[cand_idx].contains("$(period)")
                    {
                        let mut probe_signals = Vec::new();
                        let mut probe_sig = Signal::default();
                        if match_and_extract_line(orig_lines[orig_idx], tpl_lines[cand_idx], end_char_new_line, &mut probe_signals, &mut probe_sig).is_some() {
                            restart_found = Some(cand_idx);
                            break;
                        }
                    }
                }
                if let Some(r_idx) = restart_found {
                    let prev = current_signal.clone();
                    signals.push(prev.clone());
                    current_signal = Signal::default();
                    current_signal.copy_from(&prev);
                    tpl_idx = r_idx;
                    continue;
                }
            }

            return None;
        }
    }

    if current_signal.symbol.is_some() && current_signal.action.is_some() {
        signals.push(current_signal);
    }

    if signals.is_empty() || total_items == 0 || matched_items < total_items {
        return None;
    }

    // Verificar que todas las líneas no opcionales del template fueron visitadas
    for (idx, line) in tpl_lines.iter().enumerate() {
        if !visited_tpl_lines.contains(&idx) && !is_optional_template_line(line) {
            return None;
        }
    }

    // Verificar que cada señal extraída satisfaga todos los requisitos del template
    let template_req = TemplateRequirements::from_pattern(template_text);
    if !signals.iter().all(|s| template_req.is_satisfied_by(s)) {
        return None;
    }

    let ratio = (matched_items as f32 / total_items as f32) * 100.0;
    let limit: f32 = std::env::var("TOKENIZER_TEMPLATE_MATCH_LIMIT")
        .ok()
        .and_then(|v| v.parse::<f32>().ok())
        .unwrap_or(95.0);
    if ratio >= limit {
        Some(signals)
    } else {
        None
    }
}

/// Analiza un texto original retornando las señales extraídas junto con el score de coincidencia (0.0 - 100.0).
pub fn parse_template_scored(original_text: &str, modified_text: &str, line_index: Option<usize>, token_start: Option<usize>, end_char: Option<char>) -> Option<(Vec<Signal>, f32)> {
    // Vector acumulador donde se almacenarán las señales de trading extraídas
    let mut signals: Vec<Signal> = Vec::new();
    // Objeto Signal temporal para ir construyendo la señal actual
    let mut current_signal: Signal = Signal::default();

    // Contadores para el ratio de acierto
    let mut total_template_tokens = 0usize;
    let mut matched_tokens = 0usize;

    // Coleccionar las líneas del texto modificado/plantilla
    let mod_lines: Vec<&str> = modified_text.lines().collect();
    if mod_lines.is_empty() {
        return None;
    }

    let mut mod_idx = 0;
    let mut visited_mod_lines: std::collections::HashSet<usize> = std::collections::HashSet::new();

    // Recorrer el texto original línea a línea
    for original_line in original_text.lines() {
        let orig_trimmed = original_line.trim();
        if orig_trimmed.is_empty() {
            if mod_idx < mod_lines.len() && mod_lines[mod_idx].trim().is_empty() {
                visited_mod_lines.insert(mod_idx);
                mod_idx += 1;
            }
            continue;
        }

        // Buscar si coincide con mod_idx o con alguna línea posterior en mod_lines
        let mut target_idx = None;
        if mod_idx < mod_lines.len() && is_line_match(original_line, mod_lines[mod_idx]) {
            target_idx = Some(mod_idx);
        } else {
            for lookahead in (mod_idx + 1)..mod_lines.len() {
                if is_line_match(original_line, mod_lines[lookahead]) {
                    // Permitir saltar si las líneas intermedias eran opcionales (vacías o sin placeholders esenciales)
                    let can_skip = mod_lines[(mod_idx + 1)..lookahead].iter().all(|l| {
                        let req = TemplateRequirements::from_pattern(l);
                        !req.has_symbol && !req.has_action && !req.has_entry && !req.has_stoploss && !req.has_profit && !req.has_period && !req.has_time
                    });
                    if can_skip {
                        target_idx = Some(lookahead);
                    }
                    break;
                }
            }
        }

        if let Some(idx) = target_idx {
            mod_idx = idx;
        } else if mod_idx > 0 && is_line_match(original_line, mod_lines[0]) {
            mod_idx = 0;
        }

        if mod_idx >= mod_lines.len() {
            continue;
        }

        visited_mod_lines.insert(mod_idx);
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
                        let currentsym = get_symbol_from_text(orig_val, get_nodes_ref("symbols"), get_nodes_ref("checkers"));
                        let end_range = start + orig_len;
                        if let Some(sym) = currentsym {
                            total_template_tokens += 1;
                            matched_tokens += 1;
                            if current_signal.symbol.is_none() {
                                current_signal.symbol = Some(sym);
                                // agrega valor al ratio
                            }
                            if original_line.is_char_boundary(end_range) {
                                original_line.replace_range(start..end_range, "");
                                modified_line = modified_line.replacen("$(symbol)", "", 1);
                            }
                        } else if !orig_val.is_empty() && current_signal.symbol.is_none() {
                            total_template_tokens += 1;
                            matched_tokens += 1;
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

        let mod_toks: Vec<(&str, String)> = modified_line.iter_tokens().collect();
        let orig_toks: Vec<(&str, String)> = original_line.iter_tokens().collect();
        let zipped_len = std::cmp::min(mod_toks.len(), orig_toks.len());

        // Iterar en paralelo sobre los tokens de la línea original y la línea plantilla
        for (modified_token, original_token) in modified_line.zip_tokens(&original_line) {
            // print!("original_token: {:?} {:?} ", original_token, modified_token);
            total_template_tokens += 1;
            // Si el token modificado es idéntico al original, no contiene una plantilla `$(...)`
            if modified_token.1 == original_token.1 {
                // agrega valor al ratio
                matched_tokens += 1;
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
                    let currentsym = get_symbol_from_text(orig_val, get_nodes_ref("symbols"), get_nodes_ref("checkers"));
                    if let Some(sym) = currentsym.as_ref() {
                        if let SymbolType::Plain(_) = sym.symbol_type {
                            if current_signal.symbol.is_none() {
                                current_signal.symbol = Some(sym.clone());
                                // agrega valor al ratio
                                matched_tokens += 1;
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
                        // agrega valor al ratio
                        matched_tokens += 1;
                    } else if !orig_val.is_empty() {
                        let clean_val = orig_val.trim_matches(|c: char| !c.is_alphanumeric() && c != '/' && c != '-');
                        if clean_val.len() >= 3 && clean_val.chars().any(|c| c.is_alphabetic()) {
                            current_signal.symbol = Some(Symbol {
                                symbol_type: SymbolType::Plain(orig_val.to_string()),
                                symbol_class: None,
                            });
                            // agrega valor al ratio
                            matched_tokens += 1;
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
                        // agrega valor al ratio
                        matched_tokens += 1;
                    } else if let Some(ref mut sym) = current_signal.symbol {
                        sym.symbol_class = Some(SymbolClass::Otc);
                        // agrega valor al ratio
                        matched_tokens += 1;
                    }
                },
                "action" => {
                    // Obtener los flags y clasificación del token para determinar si es acción al alza o a la baja
                    let (flags, _) = get_flags_and_token_class(orig_val, original_token.0, get_nodes_ref("checkers"));
                    let detected_action = if flags.contains(TokenFlags::VALUE_UP) {
                        Some(true) // Operación de Compra / Call / Alza
                        // agrega valor al ratio
                    } else if flags.contains(TokenFlags::VALUE_DOWN) {
                        Some(false) // Operación de Venta / Put / Baja
                        // agrega valor al ratio
                    } else {
                        None
                    };

                    if let Some(act) = detected_action {
                        if current_signal.action.is_none() {
                            current_signal.action = Some(act);
                            // agrega valor al ratio
                            matched_tokens += 1;
                        } else if current_signal.action == Some(act) {
                            // Acción confirmada dentro de la misma señal (ej. Tend: Buy tras CALL)
                            matched_tokens += 1;
                        } else if current_signal.symbol.is_none() {
                            current_signal.action = Some(act);
                            // agrega valor al ratio
                            matched_tokens += 1;
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
                    // agrega valor al ratio
                    matched_tokens += 1;
                }
                "period_digit" | "period_name" => {
                    // Concatenar dígitos o sufijos al valor del período de tiempo configurado
                    if let Some(Target::PeriodTime(ref mut pt)) = current_signal.target {
                        if pt.chars().all(|c| c.is_numeric()) {
                            let prefix = orig_val.chars().next().map(|c| c.to_string().to_uppercase()).unwrap_or_default();
                            *pt = prefix + pt;
                            // agrega valor al ratio
                            matched_tokens += 1;
                        } else if pt.chars().all(|c| c.is_alphabetic()) {
                            pt.push_str(&orig_val);
                            // agrega valor al ratio
                            matched_tokens += 1;
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
                        // agrega valor al ratio
                        matched_tokens += 1;
                    }
                },
                "timezone" => {
                    // Remover espacios y agregar la zona horaria a la señal
                    let tz = orig_val.replace(" ", "");
                    current_signal.add_time_zone(tz);
                    // agrega valor al ratio
                    matched_tokens += 1;
                },
                "entry_time" | "time_entry" => {
                    // Si la señal previa ya cuenta con suficiente información y ya tiene punto de entrada, guardarla y reiniciar
                    if current_signal.is_complete() || (current_signal.entry.is_some() && current_signal.symbol.is_some() && current_signal.action.is_some()) {
                        signals.push(current_signal.clone());
                        current_signal = Signal::default();
                    }
                    // Normalizar el string de la hora de entrada a formato HH:MM:SS o HH:MM:00
                    let time_str = if orig_val.contains(':') {
                        let parts: Vec<&str> = orig_val.split(':').collect();
                        match parts.as_slice() {
                            [h, m, s] => {
                                if let (Ok(h), Ok(m), Ok(s)) = (h.parse::<u32>(), m.parse::<u32>(), s.parse::<u32>()) {
                                    // agrega valor al ratio
                                    format!("{:02}:{:02}:{:02}", h, m, s)
                                } else {
                                    orig_val.to_string()
                                }
                            },
                            [h, m] => {
                                if let (Ok(h), Ok(m)) = (h.parse::<u32>(), m.parse::<u32>()) {
                                    // agrega valor al ratio
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
                    // agrega valor al ratio
                    if !orig_val.is_empty() {
                        matched_tokens += 1;
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
                            // agrega valor al ratio
                            matched_tokens += 1;
                        }
                    } else if let Ok(price) = range_str.parse::<f32>() {
                        if price > 0.0 {
                            if let Some(Entry::PriceEntry(p1)) = current_signal.entry {
                                current_signal.entry = Some(Entry::EntryRange(p1, price));
                            } else {
                                current_signal.entry = Some(Entry::PriceEntry(price));
                            }
                            // agrega valor al ratio
                            matched_tokens += 1;
                        }
                    }
                },
                "entry_price" | "entry" | "price" => {
                    // Parsear un precio numérico de entrada y actualizar el tipo de entrada de la señal
                    let price = orig_val.replace(',', ".").parse::<f32>().unwrap_or(0.0);
                    if price > 0.0 {
                        if let Some(Entry::PriceEntry(p1)) = current_signal.entry {
                            current_signal.entry = Some(Entry::EntryRange(p1, price));
                            // agrega valor al ratio
                            matched_tokens += 1;
                        } else if let Some(Entry::TimeEntry(ref tz, _)) = current_signal.entry {
                            current_signal.entry = Some(Entry::TimePrice(tz.clone(), price));
                            // agrega valor al ratio
                            matched_tokens += 1;
                        } else if current_signal.entry.is_none() {
                            current_signal.entry = Some(Entry::PriceEntry(price));
                            // agrega valor al ratio
                            matched_tokens += 1;
                        }
                    }
                },
                "profit_price" | "profit" => {
                    // Extraer precio de Take Profit (Toma de Ganancia) y agregarlo a la lista de objetivos
                    if orig_val.to_lowercase() == "open" {
                        if let Some(Target::ProfitLoss(ref mut pt)) = current_signal.target {
                            pt.open = true;
                        } else {
                            current_signal.target = Some(Target::ProfitLoss(PricesTarget {
                                open: true,
                                profits: vec![],
                                stoploss: 0.0,
                            }));
                        }
                        matched_tokens += 1;
                    } else {
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
                            // agrega valor al ratio
                            matched_tokens += 1;
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
                        // agrega valor al ratio
                        matched_tokens += 1;
                    }
                },
                "gale" => {
                    // Parsear configuración de Martingala (Gale): número de gales, hora o 'NoGale'
                    let mut matched_gale = false;
                    if let Ok(num) = orig_val.parse::<i8>() {
                        current_signal.gales.push(Gale::Number(num));
                        matched_gale = true;
                    } else if orig_val.contains(':') {
                        current_signal.gales.push(Gale::TimeEntry(orig_val.to_string()));
                        matched_gale = true;
                    } else if orig_val.to_lowercase().contains("no") || orig_val == "0" {
                        current_signal.gales.push(Gale::NoGale);
                        matched_gale = true;
                    } else {
                        if let Some(digit_char) = orig_val.chars().find(|c| c.is_ascii_digit()) {
                            if let Some(digit) = digit_char.to_digit(10) {
                                current_signal.gales.push(Gale::Number(digit as i8));
                                matched_gale = true;
                            }
                        }
                    }
                    if matched_gale {
                        // agrega valor al ratio
                        matched_tokens += 1;
                    }
                },
                "_" | "_line" | "_lines" => {
                    // Las etiquetas _, _line y _lines no se toman en cuenta para el porcentaje final
                    total_template_tokens = total_template_tokens.saturating_sub(1);
                },
                // Ignorar etiquetas que no representan propiedades directamente asignables aquí
                "indicator" | "date" | "custom" | "dismiss" | "variable" | _ => {
                    // agraga valor al ratio
                    if !orig_val.is_empty() {
                        matched_tokens += 1;
                    }
                },
                
            }
            labeled = label.to_string();
        }
        if labeled != "profit_price" && labeled != "_lines" {
            mod_idx += 1;
        }

        // Contabilizar tokens remanentes no-vacíos de la plantilla que no se llegaron a zip en esta línea
        for extra_mod in &mod_toks[zipped_len..] {
            if !extra_mod.1.trim().is_empty()
                && !extra_mod.1.contains("$(_)")
                && !extra_mod.1.contains("$(_line)")
                && !extra_mod.1.contains("$(_lines)")
            {
                total_template_tokens += 1;
            }
        }
    }

    // Agregar la última señal si tiene datos mínimos indispensables (símbolo y acción)
    if current_signal.symbol.is_some() && current_signal.action.is_some() {
        signals.push(current_signal);
    }

    if signals.is_empty() {
        return None;
    }

    // Verificar que todas las señales extraídas cumplan con los requerimientos del template
    let template_req = TemplateRequirements::from_pattern(modified_text);
    if !signals.iter().all(|s| template_req.is_satisfied_by(s)) {
        return None;
    }

    // Contabilizar líneas esenciales no visitadas de la plantilla
    for (idx, m_line) in mod_lines.iter().enumerate() {
        if !visited_mod_lines.contains(&idx) {
            let line_req = TemplateRequirements::from_pattern(m_line);
            if line_req.has_symbol || line_req.has_action || line_req.has_entry || line_req.has_stoploss || line_req.has_profit || line_req.has_period || line_req.has_time {
                for tok in m_line.iter_tokens() {
                    if !tok.1.trim().is_empty()
                        && !tok.1.contains("$(_)")
                        && !tok.1.contains("$(_line)")
                        && !tok.1.contains("$(_lines)")
                    {
                        total_template_tokens += 1;
                    }
                }
            }
        }
    }

    let ratio = if total_template_tokens > 0 {
        (matched_tokens as f32 / (total_template_tokens as f32)) * 100.0
    } else {
        0.0
    };

    if ratio >= 98.0 {
        return Some((signals, ratio));
    }

    None
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

        let parsed_signals = parse_template(original_text, modified_text, None);
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

        let parsed_signals = parse_template(original_text, modified_text, Some(false));
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

        let result = parse_template(raw_text, pattern, Some(true));
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
    fn test_single_tp_and_dismiss_templates() {
        // ID 23 test: Tp$(dismiss)-
        let ex23 = "Gold buy now @4102\n\nSL:4001\n\nTp1-4106";
        let pat23 = "$(symbol) $(action) now @$(entry_price)\n\nSL:$(stoploss)\n\nTp$(dismiss)-$(profit_price)";
        let res23 = parse_template(ex23, pat23, None);
        assert!(res23.is_some());

        // ID 8 test: single TP and SL
        let ex8 = "🕯 #XAUUSD SELL 4534\n\nTP 4530\n\n🔴 SL_4546";
        let pat8 = "🕯 #$(symbol) $(action) $(entry_price)\n\nTP $(profit_price)\n\n🔴 SL_$(stoploss)";
        let res8 = parse_template(ex8, pat8, None);
        assert!(res8.is_some());

        // ID 16 test: single TP
        let ex16 = "🕯 #XAUUSD BUY 4040\n\nTP 4044\n\n🔴 SL_4026";
        let pat16 = "🕯 #$(symbol) $(action) $(entry_price)\n\nTP $(profit_price)\n\n🔴 SL_$(stoploss)";
        let res16 = parse_template(ex16, pat16, None);
        assert!(res16.is_some());

        // ID 10 test: TP.$(dismiss).
        let ex10 = "💥XAUUSD NOW SELL\u{00a0} 4095💯💯\n\n\u{00a0} 🔽TP.1. 4090\n\n❌SL.TP. 4103";
        let pat10 = "💥XAUUSD $(symbol) $(action)\u{00a0} $(entry_price)💯💯\n\n\u{00a0} 🔽TP.$(dismiss). $(profit_price)\n\n❌SL.TP. $(stoploss)";
        let res10 = parse_template(ex10, pat10, None);
        assert!(res10.is_some());

        // Multi-TP con TP$(index) e iteración en reversa desde la última línea
        let ex_multi_tp = "🔔  GOLD  🎯| 🔴 SELL |  4103.50\n    ⛔️SL  4109.50\n    ✅ TP1  4101.50\n    ✅ TP2  4098.50\n    ✅ TP3  4093.50";
        let pat_multi_tp = "🔔  $(symbol)  🎯| $(dismiss) $(action) |  $(entry_price)\n    ⛔️SL  $(stoploss)\n    ✅ TP$(index)  $(profit_price)";
        let res_multi = parse_template(ex_multi_tp, pat_multi_tp, None).expect("Expected multi-TP with TP$(index) to match");
        assert_eq!(res_multi.len(), 1);
        if let Some(Target::ProfitLoss(ref pt)) = res_multi[0].target {
            assert_eq!(pt.profits, vec![4101.5, 4098.5, 4093.5]);
            assert_eq!(pt.stoploss, 4109.5);
        } else {
            panic!("Expected ProfitLoss target");
        }
    }

    #[test]
    fn test_trade_promo_messages_return_none() {
        let promo_msg_7 = "50START \n🔺 Código 🔺\n➡️ Recargas 50$ Recibes 75$\n➡️ Recargas 100$ Recibes 150$\nMínimo de recarga 50$ USD\n⚠️ Para utilizar el cupón debes haberte registrado con nuestro Link\n✅ Aprende a crear tu cuenta aquí👇🏽👇🏽👇🏽\n🆘 https://t.me/registropocketoption 🆘\nRegístrese e ingrese al Canal VIP 👑 24 horas de análisis en vivo 📈";
        let pattern_284 = "👑 The King 1 Minuto VIP Pocket Option 👑\n\n💷 $(symbol)\n💎 $(period)\n⌚️ $(entry_time)\n🔼 $(action)\n\n \n\nLink de registro Pocket Option";

        let result_7 = parse_template(promo_msg_7, pattern_284, None);
        assert!(result_7.is_none(), "Promo message without trade action must return None");

        let promo_msg_31 = "¡ÚLTIMO AVISO! – QUEDAN POCOS LUGARES PARA EL PRÓXIMO BOT DE COMERCIO DE QX\n\n🤖 El lanzamiento del próximo Bot de Comercio de QX está a punto de finalizar.🤖\n⚠️ Los últimos lugares para recibir la clave de acceso se están agotando MUY rápidamente.\n\n💼 EL BOT QUE TODOS QUIEREN:\n\n✔️ Copia automáticamente todas mis operaciones\n✔️ Copia cualquier sala de trading (incluidas las de pago)\n✔️ Funciona de forma autónoma 24/7, incluso con el teléfono apagado\n✔️ Velocidad increíble, sin retrasos\n✔️ Stop/valor/porcentaje totalmente configurables\n✔️ Estrategia profesional sin martingala\n\nhttps://t.me/SuporteOficialTiger";
        let pattern_291 = "✅ ENTRY CONFIRMED ✅\n\n🌎 coin: $(symbol) $(otc)\n⏳ Expiration: $(period)\n📊 Direction: $(dismiss) $(action)\n⏰ time: $(entry_time)\n\n👉 Use up to two Martingale steps in the event of a loss!\n1º GALE: ENDS IN: 17:46h\n2º GALE: ENDS IN: 17:47h\n\n📱 REGISTER HERE";

        let result_31 = parse_template(promo_msg_31, pattern_291, None);
        assert!(result_31.is_none(), "Promo message 31 without valid symbol/action must return None");
    }

    #[test]
    fn test_template_with_otc_and_parentheses_otc() {
        // Test template con $(otc)
        let signal_otc = "EURUSD OTC\nCALL\n12:00:00\n";
        let template_otc = "$(symbol) $(otc)\n$(action)\n$(entry_time)\n";
        let result = parse_template(signal_otc, template_otc, None);
        assert!(result.is_some(), "Expected template with $(otc) to match");
        let signals = result.unwrap();
        assert_eq!(signals.len(), 1);
        assert_eq!(signals[0].action, Some(true));
        assert_eq!(signals[0].symbol.as_ref().unwrap().symbol_class, Some(SymbolClass::Otc));

        // Test template con $(otc) cuando el texto NO contiene otc (debe ser obviado y dar 100% match)
        let signal_no_otc = "EURUSD\nCALL\n12:00:00\n";
        let result_no_otc = parse_template(signal_no_otc, template_otc, None);
        assert!(result_no_otc.is_some(), "Expected template with $(otc) to omit $(otc) when text has no otc");
        let signals_no_otc = result_no_otc.unwrap();
        assert_eq!(signals_no_otc.len(), 1);
        assert_eq!(signals_no_otc[0].symbol.as_ref().unwrap().symbol_class, None);

        // Test template con ($(otc)) entre paréntesis y minúsculas (case-insensitive)
        let signal_paren_otc = "EURUSD (otc)\nCALL\n12:00:00\n";
        let template_paren_otc = "$(symbol) ($(otc))\n$(action)\n$(entry_time)\n";
        let result_paren = parse_template(signal_paren_otc, template_paren_otc, None);
        assert!(result_paren.is_some(), "Expected template with ($(otc)) to match");
        let signals_paren = result_paren.unwrap();
        assert_eq!(signals_paren.len(), 1);
        assert_eq!(signals_paren[0].action, Some(true));
        assert_eq!(signals_paren[0].symbol.as_ref().unwrap().symbol_class, Some(SymbolClass::Otc));
    }

    #[test]
    fn test_parse_template_against_database_examples() {
        let dump_str = include_str!("../data/templates_test_dump.json");
        let v: serde_json::Value = serde_json::from_str(dump_str).expect("Valid JSON");
        let list = v.as_array().expect("Array");

        let mut total_tested = 0;
        let mut matched = 0;
        let mut failures = Vec::new();

        for item in list {
            let id = item["id"].as_i64().unwrap();
            let is_active = item["is_active"].as_i64().unwrap() == 1;
            let name = item["name"].as_str().unwrap();
            let pattern = item["pattern_syntax"].as_str().unwrap();
            let example = item["example_text"].as_str().unwrap();

            total_tested += 1;
            let parsed = parse_template(example, pattern, None);
            if let Some(sigs) = parsed {
                matched += 1;
                println!("ID {}: OK ({} signals) [{}]", id, sigs.len(), name);
            } else {
                failures.push((id, is_active, name.to_string()));
                println!("ID {}: FAILED (is_active={}) [{}]", id, is_active, name);
            }
        }

        println!("\n=== RESUMEN TEST BASE DE DATOS ===");
        println!("Total testeados: {}", total_tested);
        println!("Exitosos: {}", matched);
        println!("Fallidos: {}", failures.len());
        for (id, is_active, name) in &failures {
            println!(" - Falló ID {}: {} (activo={})", id, name, is_active);
        }
        assert!(failures.is_empty(), "Expected 100% of database examples to match their templates");
    }

    #[test]
    fn test_parse_template_with_optional_underscore_token() {
        // 1. Token $(_) al final de la línea: presente y ausente
        let pat_end_tok = "$(symbol) $(action) $(entry_price) $(_)\nTP $(profit_price)\nSL $(stoploss)\n";
        let sig_end_present = "EURUSD BUY 1.0850 NOW\nTP 1.0900\nSL 1.0800\n";
        let sig_end_absent = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800\n";

        let res_present = parse_template(sig_end_present, pat_end_tok, None);
        assert!(res_present.is_some(), "Expected template with $(_) present at end of line to match 100%");
        let sigs_p = res_present.unwrap();
        assert_eq!(sigs_p.len(), 1);
        assert_eq!(sigs_p[0].action, Some(true));

        let res_absent = parse_template(sig_end_absent, pat_end_tok, None);
        assert!(res_absent.is_some(), "Expected template with $(_) absent at end of line to match 100%");
        let sigs_a = res_absent.unwrap();
        assert_eq!(sigs_a.len(), 1);
        assert_eq!(sigs_a[0].action, Some(true));

        // 2. Token $(_) al inicio de la línea: presente y ausente
        let pat_start = "$(_) $(symbol) $(action) $(entry_price)\nTP $(profit_price)\nSL $(stoploss)\n";
        let sig_start_present = "ALERTA EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800\n";
        let sig_start_absent = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800\n";

        let res_start_p = parse_template(sig_start_present, pat_start, None);
        assert!(res_start_p.is_some(), "Expected template with $(_) present at start of line to match");
        let res_start_a = parse_template(sig_start_absent, pat_start, None);
        assert!(res_start_a.is_some(), "Expected template with $(_) absent at start of line to match");

        // 3. Token $(_) en medio de la línea: presente y ausente
        let pat_mid = "$(symbol) $(_) $(action) $(entry_price)\nTP $(profit_price)\nSL $(stoploss)\n";
        let sig_mid_present = "EURUSD OTC BUY 1.0850\nTP 1.0900\nSL 1.0800\n";
        let sig_mid_absent = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800\n";

        let res_mid_p = parse_template(sig_mid_present, pat_mid, None);
        assert!(res_mid_p.is_some(), "Expected template with $(_) present in middle of line to match");
        let res_mid_a = parse_template(sig_mid_absent, pat_mid, None);
        assert!(res_mid_a.is_some(), "Expected template with $(_) absent in middle of line to match");

        // 4. Token $(_) con delimitadores: ($(_))
        let pat_delim = "$(symbol) ($(_)) $(action) $(entry_price)\nTP $(profit_price)\nSL $(stoploss)\n";
        let sig_delim_present = "EURUSD (OTC) BUY 1.0850\nTP 1.0900\nSL 1.0800\n";
        let sig_delim_absent = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800\n";

        let res_delim_p = parse_template(sig_delim_present, pat_delim, None);
        assert!(res_delim_p.is_some(), "Expected template with ($(_)) present to match");
        let res_delim_a = parse_template(sig_delim_absent, pat_delim, None);
        assert!(res_delim_a.is_some(), "Expected template with ($(_)) absent to match");

        // 5. Línea con un único token opcional $(_): presente y ausente; falla con múltiples tokens
        let pat_one_tok_line = "$(symbol) $(action) $(entry_price)\n$(_)\nTP $(profit_price)\nSL $(stoploss)\n";
        let sig_one_tok_present = "EURUSD BUY 1.0850\nVIP\nTP 1.0900\nSL 1.0800\n";
        let sig_one_tok_absent = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800\n";
        let sig_multi_tok_line = "EURUSD BUY 1.0850\nEXTRA INFO CANAL VIP\nTP 1.0900\nSL 1.0800\n";

        assert!(parse_template(sig_one_tok_present, pat_one_tok_line, None).is_some(), "Expected single-token line to match $(_)");
        assert!(parse_template(sig_one_tok_absent, pat_one_tok_line, None).is_some(), "Expected absent optional $(_) line to match");
        assert!(parse_template(sig_multi_tok_line, pat_one_tok_line, None).is_none(), "Expected multi-token line NOT to match single $(_)");

        // 6. Línea completa opcional $(_line): presente (múltiples palabras) y ausente; falla con múltiples líneas
        let pat_line = "$(symbol) $(action) $(entry_price)\n$(_line)\nTP $(profit_price)\nSL $(stoploss)\n";
        let sig_line_present = "EURUSD BUY 1.0850\nEXTRA INFO CANAL VIP\nTP 1.0900\nSL 1.0800\n";
        let sig_line_absent = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800\n";
        let sig_two_lines = "EURUSD BUY 1.0850\nEXTRA INFO 1\nEXTRA INFO 2\nTP 1.0900\nSL 1.0800\n";

        assert!(parse_template(sig_line_present, pat_line, None).is_some(), "Expected template with optional $(_line) present to match");
        assert!(parse_template(sig_line_absent, pat_line, None).is_some(), "Expected template with optional $(_line) absent to match");
        assert!(parse_template(sig_two_lines, pat_line, None).is_none(), "Expected single $(_line) NOT to match two extra lines");

        // 7. Múltiples líneas restantes opcionales $(_lines) al final o en medio: 0, 1 y múltiples líneas
        let pat_lines_end = "$(symbol) $(action) $(entry_price)\nTP $(profit_price)\nSL $(stoploss)\n$(_lines)\n";
        let sig_lines_0 = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800\n";
        let sig_lines_1 = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800\nEXTRA FOOTER LINK\n";
        let sig_lines_3 = "EURUSD BUY 1.0850\nTP 1.0900\nSL 1.0800\nFOOTER LINE 1\nFOOTER LINE 2\nFOOTER LINE 3\n";

        assert!(parse_template(sig_lines_0, pat_lines_end, None).is_some(), "Expected $(_lines) with 0 lines to match");
        assert!(parse_template(sig_lines_1, pat_lines_end, None).is_some(), "Expected $(_lines) with 1 line to match");
        assert!(parse_template(sig_lines_3, pat_lines_end, None).is_some(), "Expected $(_lines) with 3 lines to match");

        // 8. $(_lines) en modo forward (primera línea no es main)
        let pat_fwd_lines = "$(symbol) $(action)\nENTRY $(entry_price)\nTP $(profit_price)\nSL $(stoploss)\n$(_lines)\n";
        let sig_fwd_lines_3 = "EURUSD BUY\nENTRY 1.0850\nTP 1.0900\nSL 1.0800\nNOTA 1\nNOTA 2\nNOTA 3\n";
        assert!(parse_template(sig_fwd_lines_3, pat_fwd_lines, None).is_some(), "Expected forward $(_lines) with 3 lines to match");

        // 9. parse_template_scored también excluye $(_), $(_line) y $(_lines) del porcentaje
        let scored_p = parse_template_scored(sig_end_present, pat_end_tok, None, None, None);
        assert!(scored_p.is_some());
        let (_, score_p) = scored_p.unwrap();
        assert!(score_p >= 98.0, "Score should be >= 98% with $(_) token present: {}", score_p);
    }
}