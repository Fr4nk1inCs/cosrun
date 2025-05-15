use pyo3::prelude::*;

pub mod nix;

/// A set of utilities for cosutils implemented in Rust.
#[pymodule]
fn rustlib(py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    nix::export(py, m)?;
    Ok(())
}
