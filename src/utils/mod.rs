use pyo3::{prelude::*, wrap_pymodule};

pub mod nix;

#[pymodule]
pub fn utils(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(nix::nix))?;
    Ok(())
}
