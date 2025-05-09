pub mod utils;

use pyo3::{prelude::*, wrap_pymodule};

/// A Python module implemented in Rust.
#[pymodule]
fn cosrun(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(utils::utils))?;
    Ok(())
}
