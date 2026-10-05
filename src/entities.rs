use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use crate::tokens::{parse_signal, Entry, Target, SymbolType, SymbolClass, Gale};

/// Extracts structured trading signal entities directly into native Python dictionaries and lists
/// without intermediate JSON serialization overhead.
#[pyfunction]
pub fn extract_entities(py: Python<'_>, text: &str) -> PyResult<PyObject> {
    let dict = PyDict::new_bound(py);
    let signals_opt = parse_signal(text, None, None);

    if let Some(sigs) = signals_opt {
        dict.set_item("has_signal", true)?;
        dict.set_item("is_binary", sigs.is_binary())?;
        dict.set_item("template", &sigs.template)?;

        let signals_list = PyList::empty_bound(py);
        for sig in sigs.iter() {
            let sig_dict = PyDict::new_bound(py);

            // Symbol
            if let Some(ref sym) = sig.symbol {
                let sym_dict = PyDict::new_bound(py);
                let name = match &sym.symbol_type {
                    SymbolType::Crypto(s)
                    | SymbolType::Forex(s)
                    | SymbolType::Stock(s)
                    | SymbolType::Index(s)
                    | SymbolType::Etf(s)
                    | SymbolType::Funds(s)
                    | SymbolType::MoneyMarkets(s)
                    | SymbolType::Plain(s) => s.as_str(),
                    SymbolType::Empty(s) => s.as_str(),
                };
                sym_dict.set_item("asset", name)?;
                sym_dict.set_item("is_otc", sym.symbol_class == Some(SymbolClass::Otc))?;
                sym_dict.set_item("is_perp", sym.symbol_class == Some(SymbolClass::Perpetual))?;
                sig_dict.set_item("symbol", sym_dict)?;
            } else {
                sig_dict.set_item("symbol", py.None())?;
            }

            // Action
            if let Some(action_val) = sig.action {
                sig_dict.set_item("action", if action_val { "BUY" } else { "SELL" })?;
            } else {
                sig_dict.set_item("action", py.None())?;
            }

            // Entry
            if let Some(ref entry) = sig.entry {
                let entry_dict = PyDict::new_bound(py);
                match entry {
                    Entry::PriceEntry(p) => {
                        entry_dict.set_item("type", "price")?;
                        entry_dict.set_item("prices", vec![*p])?;
                    }
                    Entry::EntryRange(p1, p2) => {
                        entry_dict.set_item("type", "range")?;
                        entry_dict.set_item("prices", vec![*p1, *p2])?;
                    }
                    Entry::TimeEntry(tz, t) => {
                        entry_dict.set_item("type", "time")?;
                        entry_dict.set_item("timezone", tz)?;
                        entry_dict.set_item("time", t)?;
                    }
                    Entry::TimePrice(tz, p) => {
                        entry_dict.set_item("type", "time_price")?;
                        entry_dict.set_item("timezone", tz)?;
                        entry_dict.set_item("prices", vec![*p])?;
                    }
                }
                sig_dict.set_item("entry", entry_dict)?;
            } else {
                sig_dict.set_item("entry", py.None())?;
            }

            // Target (Profits / SL / Timeframe)
            if let Some(ref target) = sig.target {
                match target {
                    Target::PeriodTime(tf) => {
                        sig_dict.set_item("timeframe", tf)?;
                        sig_dict.set_item("stoploss", py.None())?;
                        sig_dict.set_item("profits", PyList::empty_bound(py))?;
                    }
                    Target::ProfitLoss(pl) => {
                        sig_dict.set_item("timeframe", py.None())?;
                        if pl.stoploss > 0.0 {
                            sig_dict.set_item("stoploss", pl.stoploss)?;
                        } else {
                            sig_dict.set_item("stoploss", py.None())?;
                        }
                        let profits_list = PyList::new_bound(py, &pl.profits);
                        sig_dict.set_item("profits", profits_list)?;
                    }
                }
            } else {
                sig_dict.set_item("timeframe", py.None())?;
                sig_dict.set_item("stoploss", py.None())?;
                sig_dict.set_item("profits", PyList::empty_bound(py))?;
            }

            // Gales
            let gales_list = PyList::empty_bound(py);
            for gale in &sig.gales {
                match gale {
                    Gale::Number(n) => gales_list.append(format!("G{}", n))?,
                    Gale::TimeEntry(t) => gales_list.append(t)?,
                    Gale::NoGale => gales_list.append("NoGale")?,
                }
            }
            sig_dict.set_item("gales", gales_list)?;

            signals_list.append(sig_dict)?;
        }
        dict.set_item("signals", signals_list)?;
    } else {
        dict.set_item("has_signal", false)?;
        dict.set_item("is_binary", false)?;
        dict.set_item("template", py.None())?;
        dict.set_item("signals", PyList::empty_bound(py))?;
    }

    Ok(dict.into())
}
