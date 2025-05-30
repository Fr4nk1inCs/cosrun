use pyo3::prelude::*;

pub mod parsers;

/// Hack: workaround for https://github.com/PyO3/pyo3/issues/759
#[inline]
fn init_submodule(m: &Bound<'_, PyModule>, name: &str) -> PyResult<()> {
    Python::with_gil(|py| {
        py.import("sys")?.getattr("modules")?.set_item(name, m)
    })
}

/// A set of utilities for cosutils implemented in Rust.
#[pymodule]
mod rustlib {
    use super::*;

    #[pymodule]
    mod parsers {
        use super::*;

        #[pymodule_init]
        fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
            init_submodule(m, "cosutils.rustlib.parsers")
        }

        #[pymodule_export]
        use crate::parsers::utils::ConversionError;
        #[pymodule_export]
        use crate::parsers::utils::EvaluationError;
        #[pymodule_export]
        use crate::parsers::utils::ParseError;

        #[pymodule]
        mod nix {
            use super::*;

            #[pymodule_init]
            fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
                init_submodule(m, "cosutils.rustlib.parsers.nix")
            }

            #[pymodule_export]
            use crate::parsers::nix::eval;
            #[pymodule_export]
            use crate::parsers::nix::evals;
        }

        #[pymodule]
        mod jsonc {
            use super::*;

            #[pymodule_init]
            fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
                init_submodule(m, "cosutils.rustlib.parsers.jsonc")
            }

            #[pymodule_export]
            use crate::parsers::jsonc::load;
            #[pymodule_export]
            use crate::parsers::jsonc::loads;
        }
    }
}
