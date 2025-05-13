use pyo3::prelude::*;
use pyo3::wrap_pymodule;

pub mod nix;

/// cosutils is a set of utilities for running C/S benchmarks
#[pymodule]
fn cosutils(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(nix::nix))?;
    Ok(())
}
