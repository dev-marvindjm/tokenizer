use bitflags::bitflags;
use std::sync::{LazyLock};


bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct MatcherItemFlags: u8 {
        const Negate = 0b00000001; // !
        const CaseSensitive = 0b00000010; //@
        const EndingWith = 0b00000100; //  /
        const NotInclude = 0b00001000; //.
        const Conjunction = 0b00010000; // | not increment
        const Optional = 0b00100000; // ?
        const MatchNext = 0b01000000; // & not increment
        const StartingWith = 0b10000000; //  \
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct TokenFlags: u32 {
        const POINT = 0b00000000000000000000000000000001; // .
	    const GALE_DIGIT = 0b00000000000000000000000000000010; // 1, 2
	    const HYPHEN = 0b00000000000000000000000000000100; // -
	    const UPPERCASE = 0b00000000000000000000000000001000; // if all letters contained inside are uppercase
	    const UNDERSCORE = 0b00000000000000000000000000010000; // if contain underscore
	    const SPACE = 0b00000000000000000000000000100000; // if contain space
	    const LETTER = 0b00000000000000000000000001000000; // if contain letter
	    const PERIOD_LETTER = 0b00000000000000000000000010000000; // if contain period and letter SMH only one and others digits
	    const DIGIT = 0b00000000000000000000000100000000; // if contain digit
	    const NEED_DIGIT = 0b00000000000000000000001000000000; // if name contains D
	    const VALUE_UP = 0b00000000000000000000010000000000; // if name contains u
	    const VALUE_DOWN = 0b00000000000000000000100000000000; // if name contains d
	    const VAL = 0b00000000000000000001000000000000; // if name contains val
	    const NEED_VAL = 0b00000000000000000010000000000000; // if name contains V
	    const VALUE_DIGIT = 0b00000000000000000100000000000000; // if name contains digits
	    const PERIOD = 0b00000000000000001000000000000000; // if name contains period
        const CLEAN = 0b00000000000000010000000000000000; // this will be added if contains characters not allowed
        const TIME = 0b00000000000000100000000000000000; // if name contains time
        const PRICE = 0b00000000000000100000000000000000; // if name contains price
        const RANGE = 0b00000000000001000000000000000000; // if name contains range
        const DATE = 0b00000000000010000000000000000000; // if name contains date
        const NAME = 0b00000000000100000000000000000000; // if name contains name
        const COMMA = 0b00000000001000000000000000000000;//,
        const COLON = 0b00000000010000000000000000000000; //:
        const SLASH = 0b00000000100000000000000000000000; // /
        const SINGLE_QUOTATION = 0b00000001000000000000000000000000; // '
        const STARTWITH_DIGIT = 0b00000010000000000000000000000000; // check if first is digit
        const SYMBOL_NAME = 0b00000100000000000000000000000000; // if symbol name contains others than the name like otc or perp or p after _
        const NEED_PARSE_SYMBOL = 0b00001000000000000000000000000000; // if symbol name needs to be parsed like a symbol shorterned
        const BACKSLASH = 0b00010000000000000000000000000000; // \
        const PARSER = 0b00100000000000000000000000000000; // need to by parser
        const ENTRY = 0b01000000000000000000000000000000; // need to by parser
        const NEGATIVE = 0b10000000000000000000000000000000; // need to by parser
    }
}

pub use crate::tst::Nodes;

pub fn get_nodes(tipo: &str) -> Vec<Nodes> {
    match tipo {
        "checkers" => vec![
            ACTION_V.clone(),
            ACTION_VAL_U.clone(),
            ACTION_VAL_D.clone(),
            GALE_VAL_D.clone(),
            INDICADOR_VAL_D.clone(),
            INFO_V.clone(),
            SYMBOL_V.clone(),
            // RESULT_V.clone(),
            TARGET_V.clone(),
            TARGET_VAL_D.clone(),
            TARGET_VAL_U.clone(),
            TIMEFRAME_V.clone(),
            TIMEFRAME_VAL_D.clone(),
            TIMEZONE_V.clone(),
            // TIMEZONE_VAL_D.clone(),
            SYMBOL_INDEX_VAL_D.clone(),
        ],
        "symbols" => vec![
            SYMBOL_FOREX_VAL.clone(),
            SYMBOL_CRYPTO_VAL.clone(),
            SYMBOL_INDEX_VAL.clone(),
            // SYMBOL_INDEX_VAL_D.clone(),
            SYMBOL_ETF_VAL.clone(),
            SYMBOL_FUNDS_VAL.clone(),
            SYMBOL_MONEY_VAL.clone(),
            SYMBOL_STOCK_VAL.clone(),
        ],
        &_ => vec![],
    }
}



// nodes from files embedded at compile time for iter_tokenx
static INFO_V: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/dataset/info_v.txt"), "info_val"));
pub static SYMBOL_CRYPTO_VAL: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/dataset/symbol_crypto_val.txt"), "symbol_crypto_val"));
pub static SYMBOL_ETF_VAL: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/dataset/symbol_etf_val.txt"), "symbol_etf_val"));
pub static SYMBOL_FOREX_VAL: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/dataset/symbol_forex_val.txt"), "symbol_forex_val"));
pub static SYMBOL_MONEY_VAL: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/dataset/symbol_money_val.txt"), "symbol_money_val"));
pub static SYMBOL_STOCK_VAL: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/dataset/symbol_stock_val.txt"), "symbol_stock_val"));
pub static SYMBOL_FUNDS_VAL: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/dataset/symbol_fund_val.txt"), "symbol_fund_val"));
pub static SYMBOL_INDEX_VAL: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/dataset/symbol_index_val.txt"), "symbol_index_val"));

static TIMEZONE_VAL_D: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/dataset/timezone_val_D.txt"), "timezone_val_D"));
static SYMBOL_INDEX_VAL_PARSER: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/dataset/symbol_index_val_parser.txt"), "symbol_index_val_parser"));
static ACTION_V: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/action_V.txt"), "action_V"));
pub static ACTION_VAL_U: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/action_val_d.txt"), "action_val_d"));
pub static ACTION_VAL_D: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/action_val_u.txt"), "action_val_u"));
static GALE_VAL_D: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/gale_val_D.txt"), "gale_val_D"));
static INDICADOR_VAL_D: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/indicador_val_D.txt"), "indicador_val_D"));
static SYMBOL_V: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/symbol_V.txt"), "symbol_V"));
static TARGET_ENTRY_V: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/target_entry_V.txt"), "target_entry_V"));
static TARGET_V: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/target_V.txt"), "target_V"));
static TARGET_VAL_D: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/target_val_d.txt"), "target_val_d"));
static TARGET_VAL_U: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/target_val_u.txt"), "target_val_u"));
static TIMEFRAME_V: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/timeframe_V.txt"), "timeframe_V"));
static TIMEFRAME_VAL_D: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/timeframe_name_parser_val_D.txt"), "timeframe_name_parser_val_D"));
static TIMEZONE_V: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/timezone_V.txt"), "timezone_V"));
static SYMBOL_INDEX_VAL_D: LazyLock<Nodes> = LazyLock::new(|| Nodes::from_str(include_str!("../data/cache/symbol_index_val_parser_name_D.txt"), "symbol_index_val_parser_name_D"));


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenClass {
    Action,
    Gale,
    Indicator,
    Info,
    Result,
    Symbol,
    Target,
    Timeframe,
    Timezone,
    SymbolCrypto,
    SymbolForex,
    SymbolFunds,
    SymbolEtfs,
    SymbolIndex,
    SymbolMoney,
    SymbolStocks,
    SymbolCommodities,
    SymbolBonds,
    Otc,
    NewLine,
    Whitespace,
    Punctuation,
    Emoji,
    Word,
    Other,
}

#[derive(Copy, Clone)]
pub struct Range16 {
    pub lo: u16,
    pub hi: u16,
    pub stride: u16,
}

#[derive(Copy, Clone)]
pub struct Range32 {
    pub lo: u32,
    pub hi: u32,
    pub stride: u32,
}

#[derive(Copy, Clone)]
pub struct RangeTable {
    pub r16: &'static [Range16],
    pub r32: &'static [Range32],
    pub latin_offset: usize,
}

impl RangeTable {
    pub fn is_in_table(&self, r: char) -> bool {
        let r = r as u32;
        if r <= 0xFFFF {
            let r16 = self.r16;
            if !r16.is_empty() && r16.len() > self.latin_offset && r <= r16[r16.len() - 1].hi as u32 {
                return is16(r16, r as u16);
            }
        } else {
            let r32 = self.r32;
            if !r32.is_empty() && r >= r32[0].lo {
                return is32(r32, r);
            }
        }
        false
    }
}

pub static EMOJI_CHARS: &RangeTable = &RangeTable {
    r16: &[
        Range16 { lo: 0x231a, hi: 0x231b, stride: 1 }, // reloj de arena
        Range16 { lo: 0x23e9, hi: 0x23f3, stride: 1 }, // símbolos varios
        Range16 { lo: 0x2600, hi: 0x27bf, stride: 1 }, // misceláneos y dingbats
    ],
    r32: &[
        Range32 { lo: 0x1f300, hi: 0x1f64f, stride: 1 }, // emoticonos y pictogramas
        Range32 { lo: 0x1f680, hi: 0x1f6ff, stride: 1 }, // transporte y mapas
        Range32 { lo: 0x1f7e0, hi: 0x1f7eb, stride: 1 }, // círculos de colores (incluye 🟢)
        Range32 { lo: 0x1f900, hi: 0x1f9ff, stride: 1 }, // suplemento de símbolos
        Range32 { lo: 0x1fa70, hi: 0x1faaf, stride: 1 }, // objetos recientes
    ],
    latin_offset: 0usize,
};

pub static PUNCT_CHARS: &RangeTable = &RangeTable {
    r16: &[
        Range16 { lo: 0x0021, hi: 0x0023, stride: 1 },
		Range16 { lo: 0x0025, hi: 0x002a, stride: 1 },
		Range16 { lo: 0x002c, hi: 0x002f, stride: 1 },
		Range16 { lo: 0x003a, hi: 0x003b, stride: 1 },
		Range16 { lo: 0x003f, hi: 0x0040, stride: 1 },
		Range16 { lo: 0x005b, hi: 0x005d, stride: 1 },
		Range16 { lo: 0x005f, hi: 0x007b, stride: 28 },
		Range16 { lo: 0x007d, hi: 0x00a1, stride: 36 },
		Range16 { lo: 0x00a7, hi: 0x00ab, stride: 4 },
		Range16 { lo: 0x00b6, hi: 0x00b7, stride: 1 },
		Range16 { lo: 0x00bb, hi: 0x00bf, stride: 4 },
		Range16 { lo: 0x037e, hi: 0x0387, stride: 9 },
		Range16 { lo: 0x055a, hi: 0x055f, stride: 1 },
		Range16 { lo: 0x0589, hi: 0x058a, stride: 1 },
		Range16 { lo: 0x05be, hi: 0x05c0, stride: 2 },
		Range16 { lo: 0x05c3, hi: 0x05c6, stride: 3 },
		Range16 { lo: 0x05f3, hi: 0x05f4, stride: 1 },
		Range16 { lo: 0x0609, hi: 0x060a, stride: 1 },
		Range16 { lo: 0x060c, hi: 0x060d, stride: 1 },
		Range16 { lo: 0x061b, hi: 0x061d, stride: 2 },
		Range16 { lo: 0x061e, hi: 0x061f, stride: 1 },
		Range16 { lo: 0x066a, hi: 0x066d, stride: 1 },
		Range16 { lo: 0x06d4, hi: 0x0700, stride: 44 },
		Range16 { lo: 0x0701, hi: 0x070d, stride: 1 },
		Range16 { lo: 0x07f7, hi: 0x07f9, stride: 1 },
		Range16 { lo: 0x0830, hi: 0x083e, stride: 1 },
		Range16 { lo: 0x085e, hi: 0x0964, stride: 262 },
		Range16 { lo: 0x0965, hi: 0x0970, stride: 11 },
		Range16 { lo: 0x09fd, hi: 0x0a76, stride: 121 },
		Range16 { lo: 0x0af0, hi: 0x0c77, stride: 391 },
		Range16 { lo: 0x0c84, hi: 0x0df4, stride: 368 },
		Range16 { lo: 0x0e4f, hi: 0x0e5a, stride: 11 },
		Range16 { lo: 0x0e5b, hi: 0x0f04, stride: 169 },
		Range16 { lo: 0x0f05, hi: 0x0f12, stride: 1 },
		Range16 { lo: 0x0f14, hi: 0x0f3a, stride: 38 },
		Range16 { lo: 0x0f3b, hi: 0x0f3d, stride: 1 },
		Range16 { lo: 0x0f85, hi: 0x0fd0, stride: 75 },
		Range16 { lo: 0x0fd1, hi: 0x0fd4, stride: 1 },
		Range16 { lo: 0x0fd9, hi: 0x0fda, stride: 1 },
		Range16 { lo: 0x104a, hi: 0x104f, stride: 1 },
		Range16 { lo: 0x10fb, hi: 0x1360, stride: 613 },
		Range16 { lo: 0x1361, hi: 0x1368, stride: 1 },
		Range16 { lo: 0x1400, hi: 0x166e, stride: 622 },
		Range16 { lo: 0x169b, hi: 0x169c, stride: 1 },
		Range16 { lo: 0x16eb, hi: 0x16ed, stride: 1 },
		Range16 { lo: 0x1735, hi: 0x1736, stride: 1 },
		Range16 { lo: 0x17d4, hi: 0x17d6, stride: 1 },
		Range16 { lo: 0x17d8, hi: 0x17da, stride: 1 },
		Range16 { lo: 0x1800, hi: 0x180a, stride: 1 },
		Range16 { lo: 0x1944, hi: 0x1945, stride: 1 },
		Range16 { lo: 0x1a1e, hi: 0x1a1f, stride: 1 },
		Range16 { lo: 0x1aa0, hi: 0x1aa6, stride: 1 },
		Range16 { lo: 0x1aa8, hi: 0x1aad, stride: 1 },
		Range16 { lo: 0x1b5a, hi: 0x1b60, stride: 1 },
		Range16 { lo: 0x1b7d, hi: 0x1b7e, stride: 1 },
		Range16 { lo: 0x1bfc, hi: 0x1bff, stride: 1 },
		Range16 { lo: 0x1c3b, hi: 0x1c3f, stride: 1 },
		Range16 { lo: 0x1c7e, hi: 0x1c7f, stride: 1 },
		Range16 { lo: 0x1cc0, hi: 0x1cc7, stride: 1 },
		Range16 { lo: 0x1cd3, hi: 0x2010, stride: 829 },
		Range16 { lo: 0x2011, hi: 0x2027, stride: 1 },
		Range16 { lo: 0x2030, hi: 0x2043, stride: 1 },
		Range16 { lo: 0x2045, hi: 0x2051, stride: 1 },
		Range16 { lo: 0x2053, hi: 0x205e, stride: 1 },
		Range16 { lo: 0x207d, hi: 0x207e, stride: 1 },
		Range16 { lo: 0x208d, hi: 0x208e, stride: 1 },
		Range16 { lo: 0x2308, hi: 0x230b, stride: 1 },
		Range16 { lo: 0x2329, hi: 0x232a, stride: 1 },
		Range16 { lo: 0x2768, hi: 0x2775, stride: 1 },
		Range16 { lo: 0x27c5, hi: 0x27c6, stride: 1 },
		Range16 { lo: 0x27e6, hi: 0x27ef, stride: 1 },
		Range16 { lo: 0x2983, hi: 0x2998, stride: 1 },
		Range16 { lo: 0x29d8, hi: 0x29db, stride: 1 },
		Range16 { lo: 0x29fc, hi: 0x29fd, stride: 1 },
		Range16 { lo: 0x2cf9, hi: 0x2cfc, stride: 1 },
		Range16 { lo: 0x2cfe, hi: 0x2cff, stride: 1 },
		Range16 { lo: 0x2d70, hi: 0x2e00, stride: 144 },
		Range16 { lo: 0x2e01, hi: 0x2e2e, stride: 1 },
		Range16 { lo: 0x2e30, hi: 0x2e4f, stride: 1 },
		Range16 { lo: 0x2e52, hi: 0x2e5d, stride: 1 },
		Range16 { lo: 0x3001, hi: 0x3003, stride: 1 },
		Range16 { lo: 0x3008, hi: 0x3011, stride: 1 },
		Range16 { lo: 0x3014, hi: 0x301f, stride: 1 },
		Range16 { lo: 0x3030, hi: 0x303d, stride: 13 },
		Range16 { lo: 0x30a0, hi: 0x30fb, stride: 91 },
		Range16 { lo: 0xa4fe, hi: 0xa4ff, stride: 1 },
		Range16 { lo: 0xa60d, hi: 0xa60f, stride: 1 },
		Range16 { lo: 0xa673, hi: 0xa67e, stride: 11 },
		Range16 { lo: 0xa6f2, hi: 0xa6f7, stride: 1 },
		Range16 { lo: 0xa874, hi: 0xa877, stride: 1 },
		Range16 { lo: 0xa8ce, hi: 0xa8cf, stride: 1 },
		Range16 { lo: 0xa8f8, hi: 0xa8fa, stride: 1 },
		Range16 { lo: 0xa8fc, hi: 0xa92e, stride: 50 },
		Range16 { lo: 0xa92f, hi: 0xa95f, stride: 48 },
		Range16 { lo: 0xa9c1, hi: 0xa9cd, stride: 1 },
		Range16 { lo: 0xa9de, hi: 0xa9df, stride: 1 },
		Range16 { lo: 0xaa5c, hi: 0xaa5f, stride: 1 },
		Range16 { lo: 0xaade, hi: 0xaadf, stride: 1 },
		Range16 { lo: 0xaaf0, hi: 0xaaf1, stride: 1 },
		Range16 { lo: 0xabeb, hi: 0xfd3e, stride: 20819 },
		Range16 { lo: 0xfd3f, hi: 0xfe10, stride: 209 },
		Range16 { lo: 0xfe11, hi: 0xfe19, stride: 1 },
		Range16 { lo: 0xfe30, hi: 0xfe52, stride: 1 },
		Range16 { lo: 0xfe54, hi: 0xfe61, stride: 1 },
		Range16 { lo: 0xfe63, hi: 0xfe68, stride: 5 },
		Range16 { lo: 0xfe6a, hi: 0xfe6b, stride: 1 },
		Range16 { lo: 0xff01, hi: 0xff03, stride: 1 },
		Range16 { lo: 0xff05, hi: 0xff0a, stride: 1 },
		Range16 { lo: 0xff0c, hi: 0xff0f, stride: 1 },
		Range16 { lo: 0xff1a, hi: 0xff1b, stride: 1 },
		Range16 { lo: 0xff1f, hi: 0xff20, stride: 1 },
		Range16 { lo: 0xff3b, hi: 0xff3d, stride: 1 },
		Range16 { lo: 0xff3f, hi: 0xff5b, stride: 28 },
		Range16 { lo: 0xff5d, hi: 0xff5f, stride: 2 },
		Range16 { lo: 0xff60, hi: 0xff65, stride: 1 },
    ],
    r32: &[
        Range32 { lo: 0x10100, hi: 0x10102, stride: 1 },
		Range32 { lo: 0x1039f, hi: 0x103d0, stride: 49 },
		Range32 { lo: 0x1056f, hi: 0x10857, stride: 744 },
		Range32 { lo: 0x1091f, hi: 0x1093f, stride: 32 },
		Range32 { lo: 0x10a50, hi: 0x10a58, stride: 1 },
		Range32 { lo: 0x10a7f, hi: 0x10af0, stride: 113 },
		Range32 { lo: 0x10af1, hi: 0x10af6, stride: 1 },
		Range32 { lo: 0x10b39, hi: 0x10b3f, stride: 1 },
		Range32 { lo: 0x10b99, hi: 0x10b9c, stride: 1 },
		Range32 { lo: 0x10ead, hi: 0x10f55, stride: 168 },
		Range32 { lo: 0x10f56, hi: 0x10f59, stride: 1 },
		Range32 { lo: 0x10f86, hi: 0x10f89, stride: 1 },
		Range32 { lo: 0x11047, hi: 0x1104d, stride: 1 },
		Range32 { lo: 0x110bb, hi: 0x110bc, stride: 1 },
		Range32 { lo: 0x110be, hi: 0x110c1, stride: 1 },
		Range32 { lo: 0x11140, hi: 0x11143, stride: 1 },
		Range32 { lo: 0x11174, hi: 0x11175, stride: 1 },
		Range32 { lo: 0x111c5, hi: 0x111c8, stride: 1 },
		Range32 { lo: 0x111cd, hi: 0x111db, stride: 14 },
		Range32 { lo: 0x111dd, hi: 0x111df, stride: 1 },
		Range32 { lo: 0x11238, hi: 0x1123d, stride: 1 },
		Range32 { lo: 0x112a9, hi: 0x1144b, stride: 418 },
		Range32 { lo: 0x1144c, hi: 0x1144f, stride: 1 },
		Range32 { lo: 0x1145a, hi: 0x1145b, stride: 1 },
		Range32 { lo: 0x1145d, hi: 0x114c6, stride: 105 },
		Range32 { lo: 0x115c1, hi: 0x115d7, stride: 1 },
		Range32 { lo: 0x11641, hi: 0x11643, stride: 1 },
		Range32 { lo: 0x11660, hi: 0x1166c, stride: 1 },
		Range32 { lo: 0x116b9, hi: 0x1173c, stride: 131 },
		Range32 { lo: 0x1173d, hi: 0x1173e, stride: 1 },
		Range32 { lo: 0x1183b, hi: 0x11944, stride: 265 },
		Range32 { lo: 0x11945, hi: 0x11946, stride: 1 },
		Range32 { lo: 0x119e2, hi: 0x11a3f, stride: 93 },
		Range32 { lo: 0x11a40, hi: 0x11a46, stride: 1 },
		Range32 { lo: 0x11a9a, hi: 0x11a9c, stride: 1 },
		Range32 { lo: 0x11a9e, hi: 0x11aa2, stride: 1 },
		Range32 { lo: 0x11b00, hi: 0x11b09, stride: 1 },
		Range32 { lo: 0x11c41, hi: 0x11c45, stride: 1 },
		Range32 { lo: 0x11c70, hi: 0x11c71, stride: 1 },
		Range32 { lo: 0x11ef7, hi: 0x11ef8, stride: 1 },
		Range32 { lo: 0x11f43, hi: 0x11f4f, stride: 1 },
		Range32 { lo: 0x11fff, hi: 0x12470, stride: 1137 },
		Range32 { lo: 0x12471, hi: 0x12474, stride: 1 },
		Range32 { lo: 0x12ff1, hi: 0x12ff2, stride: 1 },
		Range32 { lo: 0x16a6e, hi: 0x16a6f, stride: 1 },
		Range32 { lo: 0x16af5, hi: 0x16b37, stride: 66 },
		Range32 { lo: 0x16b38, hi: 0x16b3b, stride: 1 },
		Range32 { lo: 0x16b44, hi: 0x16e97, stride: 851 },
		Range32 { lo: 0x16e98, hi: 0x16e9a, stride: 1 },
		Range32 { lo: 0x16fe2, hi: 0x1bc9f, stride: 19645 },
		Range32 { lo: 0x1da87, hi: 0x1da8b, stride: 1 },
		Range32 { lo: 0x1e95e, hi: 0x1e95f, stride: 1 },
    ],
    latin_offset: 11usize,
};
pub static SYMBOL_CHARS: &RangeTable = &RangeTable {
    r16: &[
        Range16 { lo: 0x0024, hi: 0x002b, stride: 7 },
		Range16 { lo: 0x003c, hi: 0x003e, stride: 1 },
		Range16 { lo: 0x005e, hi: 0x0060, stride: 2 },
		Range16 { lo: 0x007c, hi: 0x007e, stride: 2 },
		Range16 { lo: 0x00a2, hi: 0x00a6, stride: 1 },
		Range16 { lo: 0x00a8, hi: 0x00a9, stride: 1 },
		Range16 { lo: 0x00ac, hi: 0x00ae, stride: 2 },
		Range16 { lo: 0x00af, hi: 0x00b1, stride: 1 },
		Range16 { lo: 0x00b4, hi: 0x00b8, stride: 4 },
		Range16 { lo: 0x00d7, hi: 0x00f7, stride: 32 },
		Range16 { lo: 0x02c2, hi: 0x02c5, stride: 1 },
		Range16 { lo: 0x02d2, hi: 0x02df, stride: 1 },
		Range16 { lo: 0x02e5, hi: 0x02eb, stride: 1 },
		Range16 { lo: 0x02ed, hi: 0x02ef, stride: 2 },
		Range16 { lo: 0x02f0, hi: 0x02ff, stride: 1 },
		Range16 { lo: 0x0375, hi: 0x0384, stride: 15 },
		Range16 { lo: 0x0385, hi: 0x03f6, stride: 113 },
		Range16 { lo: 0x0482, hi: 0x058d, stride: 267 },
		Range16 { lo: 0x058e, hi: 0x058f, stride: 1 },
		Range16 { lo: 0x0606, hi: 0x0608, stride: 1 },
		Range16 { lo: 0x060b, hi: 0x060e, stride: 3 },
		Range16 { lo: 0x060f, hi: 0x06de, stride: 207 },
		Range16 { lo: 0x06e9, hi: 0x06fd, stride: 20 },
		Range16 { lo: 0x06fe, hi: 0x07f6, stride: 248 },
		Range16 { lo: 0x07fe, hi: 0x07ff, stride: 1 },
		Range16 { lo: 0x0888, hi: 0x09f2, stride: 362 },
		Range16 { lo: 0x09f3, hi: 0x09fa, stride: 7 },
		Range16 { lo: 0x09fb, hi: 0x0af1, stride: 246 },
		Range16 { lo: 0x0b70, hi: 0x0bf3, stride: 131 },
		Range16 { lo: 0x0bf4, hi: 0x0bfa, stride: 1 },
		Range16 { lo: 0x0c7f, hi: 0x0d4f, stride: 208 },
		Range16 { lo: 0x0d79, hi: 0x0e3f, stride: 198 },
		Range16 { lo: 0x0f01, hi: 0x0f03, stride: 1 },
		Range16 { lo: 0x0f13, hi: 0x0f15, stride: 2 },
		Range16 { lo: 0x0f16, hi: 0x0f17, stride: 1 },
		Range16 { lo: 0x0f1a, hi: 0x0f1f, stride: 1 },
		Range16 { lo: 0x0f34, hi: 0x0f38, stride: 2 },
		Range16 { lo: 0x0fbe, hi: 0x0fc5, stride: 1 },
		Range16 { lo: 0x0fc7, hi: 0x0fcc, stride: 1 },
		Range16 { lo: 0x0fce, hi: 0x0fcf, stride: 1 },
		Range16 { lo: 0x0fd5, hi: 0x0fd8, stride: 1 },
		Range16 { lo: 0x109e, hi: 0x109f, stride: 1 },
		Range16 { lo: 0x1390, hi: 0x1399, stride: 1 },
		Range16 { lo: 0x166d, hi: 0x17db, stride: 366 },
		Range16 { lo: 0x1940, hi: 0x19de, stride: 158 },
		Range16 { lo: 0x19df, hi: 0x19ff, stride: 1 },
		Range16 { lo: 0x1b61, hi: 0x1b6a, stride: 1 },
		Range16 { lo: 0x1b74, hi: 0x1b7c, stride: 1 },
		Range16 { lo: 0x1fbd, hi: 0x1fbf, stride: 2 },
		Range16 { lo: 0x1fc0, hi: 0x1fc1, stride: 1 },
		Range16 { lo: 0x1fcd, hi: 0x1fcf, stride: 1 },
		Range16 { lo: 0x1fdd, hi: 0x1fdf, stride: 1 },
		Range16 { lo: 0x1fed, hi: 0x1fef, stride: 1 },
		Range16 { lo: 0x1ffd, hi: 0x1ffe, stride: 1 },
		Range16 { lo: 0x2044, hi: 0x2052, stride: 14 },
		Range16 { lo: 0x207a, hi: 0x207c, stride: 1 },
		Range16 { lo: 0x208a, hi: 0x208c, stride: 1 },
		Range16 { lo: 0x20a0, hi: 0x20c0, stride: 1 },
		Range16 { lo: 0x2100, hi: 0x2101, stride: 1 },
		Range16 { lo: 0x2103, hi: 0x2106, stride: 1 },
		Range16 { lo: 0x2108, hi: 0x2109, stride: 1 },
		Range16 { lo: 0x2114, hi: 0x2116, stride: 2 },
		Range16 { lo: 0x2117, hi: 0x2118, stride: 1 },
		Range16 { lo: 0x211e, hi: 0x2123, stride: 1 },
		Range16 { lo: 0x2125, hi: 0x2129, stride: 2 },
		Range16 { lo: 0x212e, hi: 0x213a, stride: 12 },
		Range16 { lo: 0x213b, hi: 0x2140, stride: 5 },
		Range16 { lo: 0x2141, hi: 0x2144, stride: 1 },
		Range16 { lo: 0x214a, hi: 0x214d, stride: 1 },
		Range16 { lo: 0x214f, hi: 0x218a, stride: 59 },
		Range16 { lo: 0x218b, hi: 0x2190, stride: 5 },
		Range16 { lo: 0x2191, hi: 0x2307, stride: 1 },
		Range16 { lo: 0x230c, hi: 0x2328, stride: 1 },
		Range16 { lo: 0x232b, hi: 0x2426, stride: 1 },
		Range16 { lo: 0x2440, hi: 0x244a, stride: 1 },
		Range16 { lo: 0x249c, hi: 0x24e9, stride: 1 },
		Range16 { lo: 0x2500, hi: 0x2767, stride: 1 },
		Range16 { lo: 0x2794, hi: 0x27c4, stride: 1 },
		Range16 { lo: 0x27c7, hi: 0x27e5, stride: 1 },
		Range16 { lo: 0x27f0, hi: 0x2982, stride: 1 },
		Range16 { lo: 0x2999, hi: 0x29d7, stride: 1 },
		Range16 { lo: 0x29dc, hi: 0x29fb, stride: 1 },
		Range16 { lo: 0x29fe, hi: 0x2b73, stride: 1 },
		Range16 { lo: 0x2b76, hi: 0x2b95, stride: 1 },
		Range16 { lo: 0x2b97, hi: 0x2bff, stride: 1 },
		Range16 { lo: 0x2ce5, hi: 0x2cea, stride: 1 },
		Range16 { lo: 0x2e50, hi: 0x2e51, stride: 1 },
		Range16 { lo: 0x2e80, hi: 0x2e99, stride: 1 },
		Range16 { lo: 0x2e9b, hi: 0x2ef3, stride: 1 },
		Range16 { lo: 0x2f00, hi: 0x2fd5, stride: 1 },
		Range16 { lo: 0x2ff0, hi: 0x2ffb, stride: 1 },
		Range16 { lo: 0x3004, hi: 0x3012, stride: 14 },
		Range16 { lo: 0x3013, hi: 0x3020, stride: 13 },
		Range16 { lo: 0x3036, hi: 0x3037, stride: 1 },
		Range16 { lo: 0x303e, hi: 0x303f, stride: 1 },
		Range16 { lo: 0x309b, hi: 0x309c, stride: 1 },
		Range16 { lo: 0x3190, hi: 0x3191, stride: 1 },
		Range16 { lo: 0x3196, hi: 0x319f, stride: 1 },
		Range16 { lo: 0x31c0, hi: 0x31e3, stride: 1 },
		Range16 { lo: 0x3200, hi: 0x321e, stride: 1 },
		Range16 { lo: 0x322a, hi: 0x3247, stride: 1 },
		Range16 { lo: 0x3250, hi: 0x3260, stride: 16 },
		Range16 { lo: 0x3261, hi: 0x327f, stride: 1 },
		Range16 { lo: 0x328a, hi: 0x32b0, stride: 1 },
		Range16 { lo: 0x32c0, hi: 0x33ff, stride: 1 },
		Range16 { lo: 0x4dc0, hi: 0x4dff, stride: 1 },
		Range16 { lo: 0xa490, hi: 0xa4c6, stride: 1 },
		Range16 { lo: 0xa700, hi: 0xa716, stride: 1 },
		Range16 { lo: 0xa720, hi: 0xa721, stride: 1 },
		Range16 { lo: 0xa789, hi: 0xa78a, stride: 1 },
		Range16 { lo: 0xa828, hi: 0xa82b, stride: 1 },
		Range16 { lo: 0xa836, hi: 0xa839, stride: 1 },
		Range16 { lo: 0xaa77, hi: 0xaa79, stride: 1 },
		Range16 { lo: 0xab5b, hi: 0xab6a, stride: 15 },
		Range16 { lo: 0xab6b, hi: 0xfb29, stride: 20414 },
		Range16 { lo: 0xfbb2, hi: 0xfbc2, stride: 1 },
		Range16 { lo: 0xfd40, hi: 0xfd4f, stride: 1 },
		Range16 { lo: 0xfdcf, hi: 0xfdfc, stride: 45 },
		Range16 { lo: 0xfdfd, hi: 0xfdff, stride: 1 },
		Range16 { lo: 0xfe62, hi: 0xfe64, stride: 2 },
		Range16 { lo: 0xfe65, hi: 0xfe66, stride: 1 },
		Range16 { lo: 0xfe69, hi: 0xff04, stride: 155 },
		Range16 { lo: 0xff0b, hi: 0xff1c, stride: 17 },
		Range16 { lo: 0xff1d, hi: 0xff1e, stride: 1 },
		Range16 { lo: 0xff3e, hi: 0xff40, stride: 2 },
		Range16 { lo: 0xff5c, hi: 0xff5e, stride: 2 },
		Range16 { lo: 0xffe0, hi: 0xffe6, stride: 1 },
		Range16 { lo: 0xffe8, hi: 0xffee, stride: 1 },
		Range16 { lo: 0xfffc, hi: 0xfffd, stride: 1 },
    ],
    r32: &[
        Range32 { lo: 0x10137, hi: 0x1013f, stride: 1 },
		Range32 { lo: 0x10179, hi: 0x10189, stride: 1 },
		Range32 { lo: 0x1018c, hi: 0x1018e, stride: 1 },
		Range32 { lo: 0x10190, hi: 0x1019c, stride: 1 },
		Range32 { lo: 0x101a0, hi: 0x101d0, stride: 48 },
		Range32 { lo: 0x101d1, hi: 0x101fc, stride: 1 },
		Range32 { lo: 0x10877, hi: 0x10878, stride: 1 },
		Range32 { lo: 0x10ac8, hi: 0x1173f, stride: 3191 },
		Range32 { lo: 0x11fd5, hi: 0x11ff1, stride: 1 },
		Range32 { lo: 0x16b3c, hi: 0x16b3f, stride: 1 },
		Range32 { lo: 0x16b45, hi: 0x1bc9c, stride: 20823 },
		Range32 { lo: 0x1cf50, hi: 0x1cfc3, stride: 1 },
		Range32 { lo: 0x1d000, hi: 0x1d0f5, stride: 1 },
		Range32 { lo: 0x1d100, hi: 0x1d126, stride: 1 },
		Range32 { lo: 0x1d129, hi: 0x1d164, stride: 1 },
		Range32 { lo: 0x1d16a, hi: 0x1d16c, stride: 1 },
		Range32 { lo: 0x1d183, hi: 0x1d184, stride: 1 },
		Range32 { lo: 0x1d18c, hi: 0x1d1a9, stride: 1 },
		Range32 { lo: 0x1d1ae, hi: 0x1d1ea, stride: 1 },
		Range32 { lo: 0x1d200, hi: 0x1d241, stride: 1 },
		Range32 { lo: 0x1d245, hi: 0x1d300, stride: 187 },
		Range32 { lo: 0x1d301, hi: 0x1d356, stride: 1 },
		Range32 { lo: 0x1d6c1, hi: 0x1d6db, stride: 26 },
		Range32 { lo: 0x1d6fb, hi: 0x1d715, stride: 26 },
		Range32 { lo: 0x1d735, hi: 0x1d74f, stride: 26 },
		Range32 { lo: 0x1d76f, hi: 0x1d789, stride: 26 },
		Range32 { lo: 0x1d7a9, hi: 0x1d7c3, stride: 26 },
		Range32 { lo: 0x1d800, hi: 0x1d9ff, stride: 1 },
		Range32 { lo: 0x1da37, hi: 0x1da3a, stride: 1 },
		Range32 { lo: 0x1da6d, hi: 0x1da74, stride: 1 },
		Range32 { lo: 0x1da76, hi: 0x1da83, stride: 1 },
		Range32 { lo: 0x1da85, hi: 0x1da86, stride: 1 },
		Range32 { lo: 0x1e14f, hi: 0x1e2ff, stride: 432 },
		Range32 { lo: 0x1ecac, hi: 0x1ecb0, stride: 4 },
		Range32 { lo: 0x1ed2e, hi: 0x1eef0, stride: 450 },
		Range32 { lo: 0x1eef1, hi: 0x1f000, stride: 271 },
		Range32 { lo: 0x1f001, hi: 0x1f02b, stride: 1 },
		Range32 { lo: 0x1f030, hi: 0x1f093, stride: 1 },
		Range32 { lo: 0x1f0a0, hi: 0x1f0ae, stride: 1 },
		Range32 { lo: 0x1f0b1, hi: 0x1f0bf, stride: 1 },
		Range32 { lo: 0x1f0c1, hi: 0x1f0cf, stride: 1 },
		Range32 { lo: 0x1f0d1, hi: 0x1f0f5, stride: 1 },
		Range32 { lo: 0x1f10d, hi: 0x1f1ad, stride: 1 },
		Range32 { lo: 0x1f1e6, hi: 0x1f202, stride: 1 },
		Range32 { lo: 0x1f210, hi: 0x1f23b, stride: 1 },
		Range32 { lo: 0x1f240, hi: 0x1f248, stride: 1 },
		Range32 { lo: 0x1f250, hi: 0x1f251, stride: 1 },
		Range32 { lo: 0x1f260, hi: 0x1f265, stride: 1 },
		Range32 { lo: 0x1f300, hi: 0x1f6d7, stride: 1 },
		Range32 { lo: 0x1f6dc, hi: 0x1f6ec, stride: 1 },
		Range32 { lo: 0x1f6f0, hi: 0x1f6fc, stride: 1 },
		Range32 { lo: 0x1f700, hi: 0x1f776, stride: 1 },
		Range32 { lo: 0x1f77b, hi: 0x1f7d9, stride: 1 },
		Range32 { lo: 0x1f7e0, hi: 0x1f7eb, stride: 1 },
		Range32 { lo: 0x1f7f0, hi: 0x1f800, stride: 16 },
		Range32 { lo: 0x1f801, hi: 0x1f80b, stride: 1 },
		Range32 { lo: 0x1f810, hi: 0x1f847, stride: 1 },
		Range32 { lo: 0x1f850, hi: 0x1f859, stride: 1 },
		Range32 { lo: 0x1f860, hi: 0x1f887, stride: 1 },
		Range32 { lo: 0x1f890, hi: 0x1f8ad, stride: 1 },
		Range32 { lo: 0x1f8b0, hi: 0x1f8b1, stride: 1 },
		Range32 { lo: 0x1f900, hi: 0x1fa53, stride: 1 },
		Range32 { lo: 0x1fa60, hi: 0x1fa6d, stride: 1 },
		Range32 { lo: 0x1fa70, hi: 0x1fa7c, stride: 1 },
		Range32 { lo: 0x1fa80, hi: 0x1fa88, stride: 1 },
		Range32 { lo: 0x1fa90, hi: 0x1fabd, stride: 1 },
		Range32 { lo: 0x1fabf, hi: 0x1fac5, stride: 1 },
		Range32 { lo: 0x1face, hi: 0x1fadb, stride: 1 },
		Range32 { lo: 0x1fae0, hi: 0x1fae8, stride: 1 },
		Range32 { lo: 0x1faf0, hi: 0x1faf8, stride: 1 },
		Range32 { lo: 0x1fb00, hi: 0x1fb92, stride: 1 },
		Range32 { lo: 0x1fb94, hi: 0x1fbca, stride: 1 },
    ],
    latin_offset: 10usize,
};


#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum FlowControlItem {
    Increment,// increment one or more times
    IncrementManyTimes,
    IncrementExact(u32), // increment exact times
    IncrementTimes(u32, u32), // increment between min and max times
    IncrementAtLeast(u32), // increment as many times as possible (the u32 means minimum)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MatcherItemType {
    TextSequence(String),
    Set(String),
    Range(Vec<[char; 2]>),
    SetAndRange(String, Vec<[char; 2]>),
    Function(fn(char) -> bool),
    Group(Vec<MatcherItem>),
    OrItem(Vec<MatcherItem>),
    AndItem(Vec<MatcherItem>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatcherItem {
    item_flags: MatcherItemFlags,
    item_check: MatcherItemType,
    flow_control: FlowControlItem,
}

impl MatcherItem {
    pub fn check(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<usize> {

        let mut temp_chars = chars.clone();
        let is_case_sensitive = self.item_flags.contains(MatcherItemFlags::CaseSensitive);

        let matched_chars = match &self.item_check {
            MatcherItemType::Function(f) => {
                if let Some(&c) = temp_chars.peek() {
                    let check_c = if is_case_sensitive { c } else { c.to_ascii_lowercase() };
                    if f(check_c) { temp_chars.next(); Some(1) } else { None }
                } else { None }
            }
            MatcherItemType::Set(s) => {
                if let Some(&c) = temp_chars.peek() {
                    let check_c = if is_case_sensitive { c } else { c.to_ascii_lowercase() };
                    if s.contains(check_c) { temp_chars.next(); Some(1) } else { None }
                } else { None }
            }
            MatcherItemType::SetAndRange(s, r) => {
                if let Some(&c) = temp_chars.peek() {
                    let check_c = if is_case_sensitive { c } else { c.to_ascii_lowercase() };
                    if s.contains(check_c) || r.iter().any(|range| check_c >= range[0] && check_c <= range[1]) {
                        temp_chars.next();
                        Some(1)
                    } else { None }
                } else { None }
            }
            MatcherItemType::Range(r) => {
                if let Some(&c) = temp_chars.peek() {
                    let check_c = if is_case_sensitive { c } else { c.to_ascii_lowercase() };
                    if r.iter().any(|range| check_c >= range[0] && check_c <= range[1]) { temp_chars.next(); Some(1) } else { None }
                } else { None }
            }
            MatcherItemType::TextSequence(s) => {
                let mut ok = true;
                let mut c_count = 0;
                for expected_c in s.chars() {
                    if let Some(c) = temp_chars.next() {
                        c_count += 1;
                        let matches = if is_case_sensitive {
                            c == expected_c
                        } else {
                            c.to_ascii_lowercase() == expected_c.to_ascii_lowercase()
                        };
                        if !matches {
                            ok = false;
                            break;
                        }
                    } else {
                        ok = false;
                        break;
                    }
                }
                if ok { Some(c_count) } else { None }
            }
            MatcherItemType::Group(items) => {
                let mut ok = true;
                let mut c_count = 0;
                for item in items {
                    if let Some(c) = item.match_many(&mut temp_chars) {
                        c_count += c;
                    } else {
                        ok = false;
                        break;
                    }
                }
                if ok { Some(c_count) } else { None }
            }
            MatcherItemType::OrItem(items) => {
                let mut ok = false;
                let mut c_count = 0;
                for item in items {
                    let mut clone_chars = temp_chars.clone();
                    if let Some(c) = item.match_many(&mut clone_chars) {
                        temp_chars = clone_chars;
                        c_count = c;
                        ok = true;
                        break;
                    }
                }
                if ok { Some(c_count) } else { None }
            }
            MatcherItemType::AndItem(items) => {
                let mut ok = true;
                let mut max_chars = temp_chars.clone();
                let mut max_c_count = 0;
                for item in items {
                    let mut clone_chars = temp_chars.clone();
                    if let Some(c) = item.match_many(&mut clone_chars) {
                        max_chars = clone_chars;
                        max_c_count = c;
                    } else {
                        ok = false;
                        break;
                    }
                }
                if ok { temp_chars = max_chars; Some(max_c_count) } else { None }
            }
        };

        if self.item_flags.intersects(MatcherItemFlags::Negate) {
            if matched_chars.is_some() {
                None
            } else {
                let mut c_clone = chars.clone();
                if let Some(_) = c_clone.next() {
                    if !self.item_flags.intersects(MatcherItemFlags::NotInclude) {
                        *chars = c_clone;
                        Some(1)
                    } else {
                        Some(0)
                    }
                } else {
                    None
                }
            }
        } else {
            if let Some(c) = matched_chars {
                if !self.item_flags.intersects(MatcherItemFlags::NotInclude) {
                    *chars = temp_chars;
                    Some(c)
                } else {
                    Some(0)
                }
            } else {
                None
            }
        }
    }

    pub fn match_many(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<usize> {
        let (min, max) = match self.flow_control {
            FlowControlItem::IncrementExact(min) => (min, min),
            FlowControlItem::IncrementAtLeast(min) => (min, u32::MAX),
            FlowControlItem::IncrementTimes(min, max) => (min, max),
            FlowControlItem::Increment => (1, 1),
            FlowControlItem::IncrementManyTimes => (1, u32::MAX),
        };

        let mut count = 0;
        let mut chars_count = 0;
        let mut temp_chars = chars.clone();

        while count < max {
            let mut char_clone = temp_chars.clone();
            if let Some(c) = self.check(&mut char_clone) {
                temp_chars = char_clone;
                count += 1;
                chars_count += c;
            } else {
                break;
            }
        }

        if count < min && !self.item_flags.intersects(MatcherItemFlags::Optional) {
            return None;
        }

        // Check boundaries
        if self.item_flags.intersects(MatcherItemFlags::EndingWith) {
            if let Some(&c) = temp_chars.peek() {
                if !c.is_whitespace() && !is_symbol(c) && !is_emoji(c) && ! ";".contains(c) {
                    return None;
                }
            }
        }

        *chars = temp_chars;
        Some(chars_count)
    }
}

pub fn get_name<'a>(nodes: &'a [Nodes], text: &'a str) -> &'a str {
    for node in nodes.iter() {
        if node.contains(text) {
            return &node.name;
        }
    }
    ""
}



pub fn get_flags_and_token_class(text: &str, name: &str, nodes: &[Nodes]) -> (TokenFlags, TokenClass) {
    let mut flags = TokenFlags::empty();
    let mut token_class = TokenClass::Other;
    let mut name_from_string = get_name(nodes, text);
    if name_from_string.is_empty() {
        name_from_string = name;
    }

    for s in name_from_string.split('_') {
        match s {
            "action" => {token_class = TokenClass::Action;},
            "gale" => {token_class = TokenClass::Gale;},
            "indicator" | "indicador" => {token_class = TokenClass::Indicator;},
            "info" => {token_class = TokenClass::Info;},
            "result" => {token_class = TokenClass::Result;},
            "symbol" => {token_class = TokenClass::Symbol;},
            "target" => {token_class = TokenClass::Target;},
            "timeframe" => {token_class = TokenClass::Timeframe;},
            "timezone" => {token_class = TokenClass::Timezone;},
            "whitespace" => {token_class = TokenClass::Whitespace;},
            "punctuation" => {token_class = TokenClass::Punctuation;},
            "newline" => {token_class = TokenClass::NewLine;},
            "crypto" => {token_class = TokenClass::SymbolCrypto;},
            "forex" => {token_class = TokenClass::SymbolForex;},
            "fund" => {token_class = TokenClass::SymbolFunds;},
            "etf" => {token_class = TokenClass::SymbolEtfs;},
            "index" => {token_class = TokenClass::SymbolIndex;},
            "money" => {token_class = TokenClass::SymbolMoney;},
            "stock" => {token_class = TokenClass::SymbolStocks;},
            "commodities" => {token_class = TokenClass::SymbolCommodities;}, // not implemented yet
            "bonds" => {token_class = TokenClass::SymbolBonds;}, // not implemented yet
            "otc" => {token_class = TokenClass::Otc;},
            "emoji" => {token_class = TokenClass::Emoji;},
            "word" => {token_class = TokenClass::Word;},
            "D" => flags.insert( TokenFlags::NEED_DIGIT ),
            "u" => flags.insert( TokenFlags::VALUE_UP ),
            "d" => flags.insert( TokenFlags::VALUE_DOWN ),
            "val" => flags.insert( TokenFlags::VAL ),
            "V" => flags.insert( TokenFlags::NEED_VAL ),
            "digits" => flags.insert( TokenFlags::VALUE_DIGIT ),
            "digit" => flags.insert( TokenFlags::DIGIT ),
            "period" => flags.insert( TokenFlags::PERIOD ),
            "clean" => flags.insert( TokenFlags::CLEAN ),
            "time" => flags.insert( TokenFlags::TIME ),
            "price" => flags.insert( TokenFlags::PRICE ),
            "range" => flags.insert( TokenFlags::RANGE ),
            "date" => flags.insert( TokenFlags::DATE ),
            "name" => flags.insert( TokenFlags::NAME ),
            "parser" => flags.insert( TokenFlags::PARSER ),
            "entry" => flags.insert( TokenFlags::ENTRY ),
            "no" => flags.insert( TokenFlags::NEGATIVE ),
            _ => {},
        }
    }


    let mut has_letter = false;
    let mut all_upper = true;
    let mut has_digit = false;

    if let Some(first_char) = text.chars().next() {
        if first_char.is_ascii_digit() {
            flags.insert(TokenFlags::STARTWITH_DIGIT);
        }
    }
    let upper = text.to_uppercase();
    if  upper == "INDEX" {
        flags.insert(TokenFlags::VAL);
        flags.insert(TokenFlags::NAME);
    } else if upper == "ENTRY"{
        flags.insert(TokenFlags::ENTRY);
    } else if upper == "OTC" {
        token_class = TokenClass::Otc;
    }
    for c in text.chars() {
        match c {
            '.'=> { flags.insert(TokenFlags::POINT); }
            '-'=> { flags.insert(TokenFlags::HYPHEN); }
            '_'=> { flags.insert(TokenFlags::UNDERSCORE); }
            ' '=> { flags.insert(TokenFlags::SPACE); }
            ','=> { flags.insert(TokenFlags::COMMA); }
            ':'=> { flags.insert(TokenFlags::COLON); }
            '/'=> { flags.insert(TokenFlags::SLASH); }
            '\''=> { flags.insert(TokenFlags::SINGLE_QUOTATION); }
            '\\'=> { flags.insert(TokenFlags::BACKSLASH); }
            'h'|'s'|'m'|'M'|'H'|'S'=> { flags.insert(TokenFlags::PERIOD_LETTER)}
            _ => { }
        }
        if c.is_alphabetic() {
            has_letter = true;
            if !c.is_uppercase() {
                all_upper = false;
            }
        } else if c.is_ascii_digit() {
            has_digit = true;
        }
    }

    if has_letter {
        flags.insert(TokenFlags::LETTER);
        if all_upper {
            flags.insert(TokenFlags::UPPERCASE);
        }
    }
    if has_digit {
        flags.insert(TokenFlags::DIGIT);
    }

    (flags, token_class)
}

// this function convert the pattern into a MatcherItem
// flag starting items or ending items are: ! @ $ _ | & ^
// flow control ending items are: + * ? {n} {n,m} {n,} {n}? {n,m}? {n,}?
// item types are: | check if prev not match, & check if prev match otherwise skip,
// [] is a set of characters this shold match if char is inside this can contains ranges like a-z, A-Z, 0-9,
// <> is a sequence of characters this shold match if full sequence is inside the text
// () is a group of characters this is new pattern like all above
// ! this is a flag that means negate
// @ this is a flag that means casesensitive
// $ this is a flag that means ending with the befores lletters
// _ this is a flag that means match the text but no include in the result
// | this is a flag that means if prev not match one of the next with this flags must match
// & this is a flag that means if prev match all next with this flags must match
// ^ this is a flag that means the text showld start or the word showld start with the token
// ? this is a flag that means if prev match one of the next with this flags match
// +*{n}{n,m}{n,}{n} are flow control characters
// \ is a escape character that are in same level of the flags characters
// ' is a escape character that are in same level of the flags characters
// for adding any of befores chars as text add ' this space char
pub fn parser(pattern: &str) -> Vec<MatcherItem> {
    let mut patterns: Vec<MatcherItem> = Vec::new();
    let mut chars = pattern.chars().peekable();

    while let Some(_) = chars.peek() {
        let mut item_flags = MatcherItemFlags::empty();
        let mut flow_control = FlowControlItem::Increment;

        // 1. Helper to parse flags from current position
        fn parse_start_flags(chars: &mut std::iter::Peekable<std::str::Chars>, flags: &mut MatcherItemFlags) {
            while let Some(&c) = chars.peek() {
                match c {
                    '!' => { flags.insert(MatcherItemFlags::Negate); chars.next(); }
                    '@' => { flags.insert(MatcherItemFlags::CaseSensitive); chars.next(); }
                    '_' => { flags.insert(MatcherItemFlags::NotInclude); chars.next(); }
                    '|' => { flags.insert(MatcherItemFlags::Conjunction); chars.next(); }
                    '&' => { flags.insert(MatcherItemFlags::MatchNext); chars.next(); }
                    '^' => { flags.insert(MatcherItemFlags::StartingWith); chars.next(); }
                    _ => break,
                }
            }
        }
        fn parse_end_flags(chars: &mut std::iter::Peekable<std::str::Chars>, flags: &mut MatcherItemFlags) {
            while let Some(&c) = chars.peek() {
                match c {
                    '$' => { flags.insert(MatcherItemFlags::EndingWith); chars.next(); }
                    '?' => { flags.insert(MatcherItemFlags::Optional); chars.next(); }
                    _ => break,
                }
            }
        }
        // Parse prefix flags
        parse_start_flags(&mut chars, &mut item_flags);

        if chars.peek().is_none() {
            break;
        }

        // 2. Parse Item Type
        let item_check = match chars.next().unwrap() {
            '\\' => {
                let next_c = chars.next().unwrap_or('.');
                match next_c {
                    'a' => MatcherItemType::Function(|c| c.is_alphanumeric()),
                    'A' => MatcherItemType::Function(|c| !c.is_alphanumeric()),
                    'd' => MatcherItemType::Function(|c| c.is_digit(10)),
                    'D' => MatcherItemType::Function(|c| !c.is_digit(10)),
                    'e' => MatcherItemType::Function(|c| is_emoji(c)), // traits unicode implementation
                    's' => MatcherItemType::Function(|c| c.is_whitespace()),
                    'S' => MatcherItemType::Function(|c| !c.is_whitespace()),
                    'u' => MatcherItemType::Function(|c| c.is_uppercase()),
                    'U' => MatcherItemType::Function(|c| !c.is_uppercase()),
                    'l' => MatcherItemType::Function(|c| c.is_alphabetic()),
                    '.' => MatcherItemType::Function(|_| true),
                    'n' => MatcherItemType::Function(|c| c.is_alphabetic() || "_^-:$./=+!@".contains(c)),
                    'N' => MatcherItemType::Function(|c| !c.is_alphabetic() && !"_^-:$./=+!@".contains(c)),
                    'r' => MatcherItemType::Function(|c| c.is_digit(10) || "/\\.,-_".contains(c)),
                    'p' => MatcherItemType::Function(|c| c.is_digit(10) || ".,".contains(c)),
                    'P' => MatcherItemType::Function(|c| is_punct(c)),
                    'w' => MatcherItemType::Function(|c| c.is_alphabetic() || c == '_'),
                    'W' => MatcherItemType::Function(|c| !(c.is_alphabetic() || c == '_')),
                    escaped => MatcherItemType::TextSequence(escaped.to_string()),
                }
            }
            '\'' => {
                let next_c = chars.next().unwrap_or('\'');
                MatcherItemType::TextSequence(next_c.to_string())
            }
            '[' => {
                let mut content = String::new();
                let mut ranges = Vec::new();
                let mut negate_set = false;
                if chars.peek() == Some(&'^') {
                    negate_set = true;
                    chars.next();
                }
                while let Some(c) = chars.next() {
                    if chars.peek() == Some(&'-') && !( "[\\\']".contains(c)){
                        chars.next();
                        let next_c = chars.next().unwrap();
                        if next_c == ']' {
                            content.push(c);
                            content.push('-');
                            break;
                        }else if (c == '\'' || c == '\\') && next_c == '-' {
                            content.push(next_c)
                        }else if c < next_c{
                            ranges.push([c, next_c]);
                        } else {
                            content.push(c);
                            content.push('-');
                            content.push(next_c);
                        }
                        continue;
                    }
                    if c == ']' {
                        break;
                    }
                    content.push(c);
                }
                if negate_set {
                    item_flags.insert(MatcherItemFlags::Negate);
                }
                if content != "" && ranges.len() > 0 {
                    MatcherItemType::SetAndRange(content, ranges)
                } else if content != "" {
                    MatcherItemType::Set(content)
                } else {
                    MatcherItemType::Range(ranges)
                }
            }
            '<' => {
                let mut content = String::new();
                while let Some(c) = chars.next() {
                    if c == '>' {
                        break;
                    }
                    content.push(c);
                }
                MatcherItemType::TextSequence(content)
            }
            '(' => {
                let mut content = String::new();
                let mut depth = 1;
                while let Some(c) = chars.next() {
                    if c == '(' {
                        depth += 1;
                    } else if c == ')' {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    content.push(c);
                }
                let items = parser(&content);
                MatcherItemType::Group(items)
            }

            c => MatcherItemType::TextSequence(c.to_string()),
        };

        // Parse suffix flags (before quantifier)
        parse_end_flags(&mut chars, &mut item_flags);

        // 3. Parse Quantifier
        if let Some(&c) = chars.peek() {
            match c {
                '*' => {
                    flow_control = FlowControlItem::IncrementAtLeast(0);
                    item_flags.insert(MatcherItemFlags::Optional);
                    chars.next();
                }
                '+' => {
                    flow_control = FlowControlItem::IncrementManyTimes;
                    chars.next();
                    if chars.peek() == Some(&'?') {
                        item_flags.insert(MatcherItemFlags::Optional);
                        chars.next();
                    }
                }
                '?' => {
                    flow_control = FlowControlItem::IncrementTimes(0, 1);
                    item_flags.insert(MatcherItemFlags::Optional);
                    chars.next();
                }
                '{' => {
                    chars.next();
                    let mut content = String::new();
                    while let Some(c) = chars.next() {
                        if c == '}' {
                            if chars.peek() == Some(&'?') {
                                item_flags.insert(MatcherItemFlags::Optional);
                                chars.next();
                            }
                            break;
                        }
                        content.push(c);
                    }
                    let parts: Vec<&str> = content.split(',').collect();
                    if parts.len() == 1 {
                        if let Ok(n) = parts[0].parse() {
                            flow_control = FlowControlItem::IncrementExact(n);
                        }
                    } else if parts.len() == 2 {
                        let min = parts[0].parse().unwrap_or(0);
                        if parts[1].is_empty() {
                            flow_control = FlowControlItem::IncrementAtLeast(min);
                        } else {
                            let max = parts[1].parse().unwrap_or(min);
                            flow_control = FlowControlItem::IncrementTimes(min, max);
                        }
                    }
                }
                _ => {}
            }
        }

        // Parse suffix flags (after quantifier)
        parse_end_flags(&mut chars, &mut item_flags);

        // concatenating last TextSequence item with current item
        if let Some(last) = patterns.last_mut() {
            if last.item_flags.is_empty()
            && item_flags.is_empty()
            && flow_control == FlowControlItem::Increment
            && last.flow_control == FlowControlItem::Increment {
                match (&last.item_check, &item_check) {
                    (MatcherItemType::TextSequence(last_text), MatcherItemType::TextSequence(current_text)) => {
                        last.item_check = MatcherItemType::TextSequence(last_text.to_string() + current_text);
                        continue;
                    }
                    _ => {}
                }
            }
        }

        let item = MatcherItem {
            item_flags,
            item_check,
            flow_control,
        };

        let item: MatcherItem = if item_flags.intersects(MatcherItemFlags::MatchNext) { // si es next
                let some_item: MatcherItem = match patterns.pop() {
                    Some(last) => match &last.item_check {
                        MatcherItemType::AndItem(items) => {
                            let mut ret: Vec<MatcherItem> = items.to_vec();
                            ret.push(item);
                            MatcherItem {
                                item_flags: MatcherItemFlags::empty(),
                                item_check: MatcherItemType::AndItem(ret),
                                flow_control: FlowControlItem::Increment,
                            }
                        },
                        _ => MatcherItem{
                            item_flags: MatcherItemFlags::empty(),
                            item_check: MatcherItemType::AndItem(vec![last, item]),
                            flow_control: FlowControlItem::Increment,
                        },
                    },
                    None => MatcherItem{ // manage as error
                        item_flags: MatcherItemFlags::empty(),
                        item_check: MatcherItemType::AndItem(vec![item]),
                        flow_control: FlowControlItem::Increment,
                    },
                };
                some_item
            } else if item_flags.intersects(MatcherItemFlags::Conjunction) { // si es next
                let some_item: MatcherItem = match patterns.pop() {
                    Some(last) => match &last.item_check {
                        MatcherItemType::OrItem(items) => { // updating the last OrItem
                            let mut ret: Vec<MatcherItem> = items.to_vec();
                            ret.push(item);
                            MatcherItem {
                                item_flags: MatcherItemFlags::empty(),
                                item_check: MatcherItemType::OrItem(ret),
                                flow_control: FlowControlItem::Increment,
                            }
                        },
                        _ => MatcherItem{
                            item_flags: MatcherItemFlags::empty(),
                            item_check: MatcherItemType::OrItem(vec![last, item]),
                            flow_control: FlowControlItem::Increment,
                        },
                    },
                    None => MatcherItem{ // manage as error
                        item_flags: MatcherItemFlags::empty(),
                        item_check: MatcherItemType::OrItem(vec![item]),
                        flow_control: FlowControlItem::Increment,
                    },
                };
                some_item
            } else {
                item
            };
        patterns.push(item);
    }
    patterns
}

fn is16(ranges: &[Range16], r: u16) -> bool {
    if ranges.len() <= 18 || r <= '\u{00FF}' as u16 {
        for rg in ranges {
            if r < rg.lo {
                return false;
            }
            if r <= rg.hi {
                return rg.stride == 1 || (r - rg.lo) % rg.stride == 0;
            }
        }
        return false;
    }

    // binary search
    let mut lo = 0;
    let mut hi = ranges.len();
    while lo < hi {
        let m = lo + (hi - lo) / 2;
        let rg = &ranges[m];
        if rg.lo <= r && r <= rg.hi {
            return rg.stride == 1 || (r - rg.lo) % rg.stride == 0;
        }
        if r < rg.lo {
            hi = m;
        } else {
            lo = m + 1;
        }
    }
    false
}

fn is32(ranges: &[Range32], r: u32) -> bool {
    if ranges.len() <= 18 {
        for rg in ranges {
            if r < rg.lo {
                return false;
            }
            if r <= rg.hi {
                return rg.stride == 1 || (r - rg.lo) % rg.stride == 0;
            }
        }
        return false;
    }

    // binary search
    let mut lo = 0;
    let mut hi = ranges.len();
    while lo < hi {
        let m = lo + (hi - lo) / 2;
        let rg = &ranges[m];
        if rg.lo <= r && r <= rg.hi {
            return rg.stride == 1 || (r - rg.lo) % rg.stride == 0;
        }
        if r < rg.lo {
            hi = m;
        } else {
            lo = m + 1;
        }
    }
    false
}

fn is_emoji(c: char) -> bool {
    EMOJI_CHARS.is_in_table(c)
}

fn is_symbol(c: char) -> bool {
    "$+<=>^`|~¢£¤¥¦¨©¬®¯°±´¸×÷".contains(c) || SYMBOL_CHARS.is_in_table(c)
}

fn is_punct(c: char) -> bool {
    "!\"#$%&'()*,-./:;?@[\\]_{}¡§«¶·»¿".contains(c) || PUNCT_CHARS.is_in_table(c)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patterns {
    patterns: Vec<MatcherItem>,
    pub name: &'static str,
}

impl Patterns {
    pub fn new(name: &'static str, pattern: &str) -> Self {
        Self {
            patterns: parser(pattern),
            name,
        }
    }

    // pub fn get_patterns(&self) -> &[MatcherItem] {
    //     &self.patterns
    // }

    pub fn match_chars(&self, chars: &mut std::iter::Peekable<std::str::Chars>) -> Option<(&'static str, String)> {
        let mut temp_chars = chars.clone();
        let mut consumed_count = 0;
        for pattern in self.patterns.iter() {
            if let Some(c) = pattern.match_many(&mut temp_chars) {
                consumed_count += c;
            } else {
                return None;
            }
        }

        let matched_string: String = chars.clone().take(consumed_count).collect();

        *chars = temp_chars;
        Some((self.name, matched_string))
    }
}

// specific and valid tokes patterns
static SYMBOL_TOKEN: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("symbol_val","\\a+([_.-]P|o\\w+?\\n*)|([:$/=+!@^]\\w+\\n*)"),);
static TIMEFRAME_DIGITS_PERIOD_VAL: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("timeframe_digits_period_val","\\d+[smh]$"),);
static DATE_VAL: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("date_val","\\d{1,2}[/-]\\d{1,2}[/-]\\d{2,4}"),);
static TARGET_TIME_VAL_2: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("target_time_val_clean","\\d{1,2}d[':]\\d${1,2}h[':]\\d${1,2}m"),);
static TARGET_TIME_VAL: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("target_time_val","\\d{1,2}[:'](\\d{2}[:'](\\d{2}_[^0-9:'])|\\d${2})|((\\d{2}_[^0-9:'])|\\d${2})"),);
static TARGET_PRICE_RANGE_VAL: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("target_price_range_val","(\\d+\\p\\d+)|\\d{4,}\\r+"),);
static TARGET_PRICE_VAL: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("target_price_val","(\\d+\\p\\d+)|\\d{4,}"),);
static TIMEFRAME_PERIOD_DIGITS_VAL: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("timeframe_period_digits_val","[smh]\\d+"),);
static DIGIT: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("digit","\\d+"),);
static INDEX: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("index","\\d._(\\d+\\p\\d+)"),);
static TIMEZONE_TIME_VAL: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("timezone_name_digit_time_val","(<UTC>|<GMT>)\\s*[+-]?\\s*[01]?[0-9](:([034][05]))?"),);
static TIMEZONE_DIGIT_VAL: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("timezone_digits_val","[+-]?[01]?[0-9](:([034][05]))?"),);
static TIMEFRAME_PERIOD_NAME_VAL: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("timeframe_period_parser_name_val","(<min>[uú]<t>[oe]s?)"),);
static GALE_ORDINAL: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("gale_digit_val","\\d+[ºª]"),);
static TOKEN_NAME: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("token_name","<$(>\\w+\\)"),);
static TOKEN_OTC: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("token_name","\\(?<$(otc)>\\)?"),);
// general and commons patterns
static WORD: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("word","\\w+"),);
static OTC_WORD: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("otc","\\(?< - >?<otc>\\)?"),);
static WHITE_SPACE: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("whitespace"," "),);
static NEW_LINE: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("newline","\n"),);
static PUNCTUATION: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("punctuation","\\P"),);
static EMOJI: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("emoji","\\e"),);
static SINGLE_CHAR: LazyLock<Patterns> = LazyLock::new (|| Patterns::new("single_char","\\."),);

pub fn check_category(text: &str, category: &str) -> bool {
    let mut chars = text.chars().peekable();
    match category {
        "simbolo" | "symbol" => {
            SYMBOL_TOKEN.match_chars(&mut chars).is_some() && (
                SYMBOL_CRYPTO_VAL.contains(text) || SYMBOL_ETF_VAL.contains(text) ||
                SYMBOL_FOREX_VAL.contains(text) || SYMBOL_MONEY_VAL.contains(text) ||
                SYMBOL_STOCK_VAL.contains(text) || SYMBOL_FUNDS_VAL.contains(text) ||
                SYMBOL_INDEX_VAL.contains(text) || SYMBOL_V.contains(text)
            )
        }
        "accion" | "action" => {
            ACTION_V.contains(text) || ACTION_VAL_U.contains(text) || ACTION_VAL_D.contains(text)
        }
        "info" => {
            INFO_V.contains(text)
        }
        "timezone" => {
            let matches_pattern = TIMEZONE_TIME_VAL.match_chars(&mut chars.clone()).is_some() ||
                                TIMEZONE_DIGIT_VAL.match_chars(&mut chars).is_some();
            matches_pattern && (TIMEZONE_VAL_D.contains(text) || TIMEZONE_V.contains(text))
        }
        "price" | "precio" => {
            let matches_pattern = TARGET_PRICE_VAL.match_chars(&mut chars.clone()).is_some() ||
                                TARGET_PRICE_RANGE_VAL.match_chars(&mut chars).is_some();
            matches_pattern || (
                TARGET_V.contains(text) || TARGET_VAL_D.contains(text) ||
                TARGET_VAL_U.contains(text) || TARGET_ENTRY_V.contains(text)
            )
        }
        "gale" => {
            GALE_VAL_D.contains(text)
        }
        "indicador" | "indicator" => {
            INDICADOR_VAL_D.contains(text)
        }
        "timeframe" => {
            let matches_pattern = TIMEFRAME_DIGITS_PERIOD_VAL.match_chars(&mut chars.clone()).is_some() ||
                                TIMEFRAME_PERIOD_DIGITS_VAL.match_chars(&mut chars.clone()).is_some() ||
                                TIMEFRAME_PERIOD_NAME_VAL.match_chars(&mut chars).is_some();
            matches_pattern && (TIMEFRAME_V.contains(text) || TIMEFRAME_VAL_D.contains(text))
        }
        _ => false,
    }
}






static PATTERNS_DIGIT: LazyLock<Vec<Patterns>> = LazyLock::new(|| vec![
    SYMBOL_TOKEN.clone(),
    TARGET_TIME_VAL_2.clone(),
    TARGET_TIME_VAL.clone(),
    TIMEFRAME_DIGITS_PERIOD_VAL.clone(),
    DATE_VAL.clone(),
    TARGET_PRICE_RANGE_VAL.clone(),
    // INDEX.clone(),
    TARGET_PRICE_VAL.clone(),
    GALE_ORDINAL.clone(),
    TIMEFRAME_PERIOD_DIGITS_VAL.clone(),
    DIGIT.clone(),
    TIMEZONE_DIGIT_VAL.clone(),

]);

static PATTERNS_HMS: LazyLock<Vec<Patterns>> = LazyLock::new(|| vec![
    TIMEFRAME_PERIOD_DIGITS_VAL.clone(),
    TIMEFRAME_PERIOD_NAME_VAL.clone(),
    SYMBOL_TOKEN.clone(),
    WORD.clone(),
    WHITE_SPACE.clone(),
    NEW_LINE.clone(),
    PUNCTUATION.clone(),
    EMOJI.clone(),
    SINGLE_CHAR.clone(),
]);

static PATTERNS_UG: LazyLock<Vec<Patterns>> = LazyLock::new(|| vec![
    TIMEZONE_TIME_VAL.clone(),
    SYMBOL_TOKEN.clone(),
    WORD.clone(),
    WHITE_SPACE.clone(),
    NEW_LINE.clone(),
    PUNCTUATION.clone(),
    EMOJI.clone(),
    SINGLE_CHAR.clone(),
]);

static PATTERNS_X: LazyLock<Vec<Patterns>> = LazyLock::new(|| vec![
    Patterns::new("xincrement","x\\d+"),
    SYMBOL_TOKEN.clone(),
    WORD.clone(),
    WHITE_SPACE.clone(),
    NEW_LINE.clone(),
    PUNCTUATION.clone(),
    EMOJI.clone(),
    SINGLE_CHAR.clone(),
]);

static PATTERNS_DEFAULT: LazyLock<Vec<Patterns>> = LazyLock::new(|| vec![
    SYMBOL_TOKEN.clone(),
    WORD.clone(),
    TOKEN_NAME.clone(),
    WHITE_SPACE.clone(),
    NEW_LINE.clone(),
    PUNCTUATION.clone(),
    EMOJI.clone(),
    SINGLE_CHAR.clone(),
]);

fn get_patterns_by_first_char(first: char) -> &'static [Patterns] {
    if first.is_digit(10) {
        return &**PATTERNS_DIGIT;
    }
    match first.to_ascii_lowercase() {
        'h' | 'm' | 's' => &**PATTERNS_HMS,
        'u' | 'g' => &**PATTERNS_UG,
        'x' => &**PATTERNS_X,
        _ => &**PATTERNS_DEFAULT,
    }
}


pub struct TokenIterator<'a> {
    remaining: &'a str,
}

impl<'a> TokenIterator<'a> {
    pub fn new(remaining: &'a str) -> Self {
        Self { remaining }
    }

    pub fn zip_tokens<'b>(self, other: TokenIterator<'b>) -> ZipTokenIterator<'a, 'b> {
        ZipTokenIterator::from_iters(self, other)
    }

    pub fn zip_str<'b>(self, other: &'b str) -> ZipTokenIterator<'a, 'b> {
        ZipTokenIterator::from_iters(self, TokenIterator::new(other))
    }
}

impl<'a> Iterator for TokenIterator<'a> {
    type Item = (&'a str, String);

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        let first = self.remaining.chars().next().unwrap();
        let patterns = get_patterns_by_first_char(first);
        let mut chars = self.remaining.chars().peekable();

        for pattern in patterns {
            if let Some((name, matched_string)) = pattern.match_chars(&mut chars) {
                // Prevent infinite looping on empty matches
                if !matched_string.is_empty() {
                    let len = matched_string.len();
                    if len <= self.remaining.len() && self.remaining.is_char_boundary(len) {
                        self.remaining = &self.remaining[len..];
                        return Some((name, matched_string));
                    }
                }
            }
        }

        // If no known pattern matched, return the rest as "unknown"
        let ret = self.remaining.to_string();
        self.remaining = "";
        Some(("", ret))
    }
}

pub struct ZipTokenIterator<'a, 'b> {
    iter1: TokenIterator<'a>,
    iter2: TokenIterator<'b>,
}

impl<'a, 'b> ZipTokenIterator<'a, 'b> {
    pub fn new(str1: &'a str, str2: &'b str) -> Self {
        Self {
            iter1: TokenIterator::new(str1),
            iter2: TokenIterator::new(str2),
        }
    }

    pub fn from_iters(iter1: TokenIterator<'a>, iter2: TokenIterator<'b>) -> Self {
        Self { iter1, iter2 }
    }
}

impl<'a, 'b> Iterator for ZipTokenIterator<'a, 'b> {
    type Item = ((&'a str, String), (&'b str, String));

    fn next(&mut self) -> Option<Self::Item> {
        match (self.iter1.next(), self.iter2.next()) {
            (Some(t1), Some(t2)) => Some((t1, t2)),
            _ => None,
        }
    }
}

pub fn zip_tokens<'a, 'b>(s1: &'a str, s2: &'b str) -> ZipTokenIterator<'a, 'b> {
    ZipTokenIterator::new(s1, s2)
}

pub trait Tokens<'a> {
    fn iter_tokens(&'a self) -> TokenIterator<'a>;
    fn zip_tokens<'b>(&'a self, other: &'b str) -> ZipTokenIterator<'a, 'b>;
}

impl<'a> Tokens<'a> for str {
    fn iter_tokens(&'a self) -> TokenIterator<'a> {
        TokenIterator { remaining: self }
    }

    fn zip_tokens<'b>(&'a self, other: &'b str) -> ZipTokenIterator<'a, 'b> {
        ZipTokenIterator::new(self, other)
    }
}

impl<'a> Tokens<'a> for String {
    fn iter_tokens(&'a self) -> TokenIterator<'a> {
        TokenIterator { remaining: self.as_str() }
    }

    fn zip_tokens<'b>(&'a self, other: &'b str) -> ZipTokenIterator<'a, 'b> {
        ZipTokenIterator::new(self.as_str(), other)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parser() {
        let patterns = parser("SELL'_TSLA'@799");
        // assert_eq!(patterns.len(), 1);
        assert!(patterns[0].item_flags.is_empty());
        assert_eq!(patterns[0].flow_control, FlowControlItem::Increment);
        assert_eq!(patterns[0].item_check, MatcherItemType::TextSequence("SELL_TSLA@799".to_string()));
    }
    #[test]
    fn test_pattern_match() {
        let patterns = Patterns::new("target_time_val","(\\d+\\p\\d+)|\\d{4,}");
        let mut chars = "2000.00".chars().peekable();
        let result = patterns.match_chars(&mut chars);
        assert_eq!(result, Some(("target_time_val".as_ref(), "2000.00".to_string())));
        let patterns = Patterns::new("date_val","\\d{1,2}[/-]\\d{1,2}[/-]\\d{2,4}");
        let mut chars = "12/12/2024".chars().peekable();
        let result = patterns.match_chars(&mut chars);
        assert_eq!(result, Some(("date_val".as_ref(), "12/12/2024".to_string())));
    }

        use super::Nodes;
    use std::env;
    #[test]
    fn test_insert_and_contains() {
        unsafe {
            env::set_var("RUST_BACKTRACE", "1");
        }
        let mut root = Nodes::new("test".to_string());
        root.insert_from_file(".\\data\\cache\\action_v.txt");
        assert!(root.contains("TEND"));
    }

    #[test]
    fn test_from_files() {
        let info_v = Nodes::from_file(".\\data\\dataset\\info_v.txt");
        let symbol_crypto_val = Nodes::from_file( ".\\data\\dataset\\symbol_crypto_val.txt" );
        let symbol_etf_val = Nodes::from_file( ".\\data\\dataset\\symbol_etf_val.txt" );
        let symbol_forex_val = Nodes::from_file( ".\\data\\dataset\\symbol_forex_val.txt" );
        let symbol_money_val = Nodes::from_file( ".\\data\\dataset\\symbol_money_val.txt" );
        // let symbol_bonds_val = Nodes::from_file( ".\\data\\dataset\\symbol_bonds_val.txt" );
        let symbol_stock_val = Nodes::from_file( ".\\data\\dataset\\symbol_stock_val.txt" );
        let symbol_funds_val = Nodes::from_file( ".\\data\\dataset\\symbol_fund_val.txt" );
        // let symbol_commodities_val = Nodes::from_file( ".\\data\\dataset\\symbol_commodities_val.txt" );
        let symbol_index_val = Nodes::from_file( ".\\data\\dataset\\symbol_index_val.txt" );
        // let action_v = Node::from_file(".\\data\\dataset\\action_v.txt");
        // let gale_v = Node::from_file(".\\data\\dataset\\gale_v.txt");
        // let indicator_v = Node::from_file(".\\data\\dataset\\indicator_v.txt");
        // let result_v = Node::from_file(".\\data\\dataset\\result_v.txt");
        // let symbol_v = Node::from_file(".\\data\\dataset\\symbol_v.txt");
        // let target_v = Node::from_file(".\\data\\dataset\\target_v.txt");
        // let timeframe_v = Node::from_file(".\\data\\dataset\\timeframe_v.txt");
        // let timezone_v = Node::from_file(".\\data\\dataset\\timezone_v.txt");
    }

    #[test]
    fn test_create_nodes() {
        let mut nodes = Nodes::new("test".to_string());
        nodes.insert("test");
        // println!("{}", nodes.contains("test"));
        // println!("{:#?}", nodes.nodes);
        assert!(nodes.contains("test"));
    }
    #[test]
    fn test_not_include_lookahead() {
        // Pattern: "A" followed by lookahead "B" (using '_' flag for NotInclude)
        let patterns = Patterns::new("lookahead", "A_B");
        let mut chars = "AB".chars().peekable();
        let result = patterns.match_chars(&mut chars);

        // Should match "A" because "B" follows, but "B" should NOT be consumed or included.
        assert_eq!(result, Some(("lookahead", "A".to_string())));

        // Check if 'B' is still there in the iterator
        assert_eq!(chars.next(), Some('B'));

        // Pattern: "A" followed by negative lookahead "B" (using '!_B')
        let patterns_neg = Patterns::new("neg_lookahead", "A!_B");

        // "AC" should match "A" because "C" is not "B"
        let mut chars2 = "AC".chars().peekable();
        let result2 = patterns_neg.match_chars(&mut chars2);
        assert_eq!(result2, Some(("neg_lookahead", "A".to_string())));
        assert_eq!(chars2.next(), Some('C'));

        // "AB" should NOT match because "B" IS "B"
        let mut chars3 = "AB".chars().peekable();
        let result3 = patterns_neg.match_chars(&mut chars3);
        assert_eq!(result3, None);
    }

    #[test]
    fn test_check_category() {
        // Test action (assuming TEND is in action_V.txt)
        assert!(check_category("TEND", "accion"));

        // Test invalid category
        assert!(!check_category("TEND", "invalid"));
    }

    #[test]
    fn test_zip_token_iterator() {
        let str1 = "SELL TSLA @799";
        let str2 = "BUY AAPL @150";

        let mut zipped = zip_tokens(str1, str2);
        let first_pair = zipped.next();
        assert!(first_pair.is_some());

        let zipped_vec: Vec<_> = str1.zip_tokens(str2).collect();
        assert!(!zipped_vec.is_empty());
    }
}


