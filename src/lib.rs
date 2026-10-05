pub mod tst;
pub mod matchers;
pub mod tokens;
pub mod tokenizer;
pub mod templates;
pub mod entities;

use pyo3::prelude::*;

/// Python module entry point for the high-performance native trading token engine
#[pymodule]
fn rust_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<tokenizer::Token>()?;
    m.add_class::<templates::ParsedSignal>()?;
    m.add_function(wrap_pyfunction!(tokenizer::tokenize_text, m)?)?;
    m.add_function(wrap_pyfunction!(templates::match_template, m)?)?;
    m.add_function(wrap_pyfunction!(entities::extract_entities, m)?)?;
    Ok(())
}
