use pyo3::exceptions::{PyIOError, PyUnicodeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyNone, PyString};
use pyo3::PyObject;
use pyo3::{pyfunction, PyResult};
use std::path::PathBuf;
use std::str::from_utf8;
use std::{fs, rc::Rc};
use tvix_eval::{Error as TvixError, ErrorKind as TvixErrorKind, Value as TvixValue};
use tvix_eval::{EvalIO, EvalMode, Evaluation, StdIO};

fn display_tvix_error(e: &TvixError) -> String {
    format!(
        "{}{}",
        e,
        match e.kind {
            TvixErrorKind::ParseErrors(ref errors) => errors
                .iter()
                .map(|e| format!("\n    {}", e))
                .collect::<Vec<_>>()
                .join(""),
            _ => "".to_string(),
        }
    )
}

/// Parse and evaluate a nix expression
fn eval_expr(expr: &str, location: Option<PathBuf>) -> Result<TvixValue, String> {
    // FIXME: This is a hack to make the evaluation result to be a JSON object
    let builder = Evaluation::builder_pure()
        .io_handle(Rc::new(StdIO) as Rc<dyn EvalIO>)
        .mode(EvalMode::Strict);
    let eval = builder.build();

    let result = eval.evaluate(expr, location.clone());

    if let Some(value) = result.value {
        Ok(value)
    } else {
        Err(result
            .errors
            .iter()
            .map(display_tvix_error)
            .collect::<Vec<_>>()
            .join("\n  "))
    }
}

/// Convert TVixValue to PyObject
fn tvix_to_pyobject(py: Python<'_>, value: &TvixValue) -> PyResult<PyObject> {
    let object = match value {
        TvixValue::Null => PyNone::get(py).to_owned().into_any().unbind(),
        TvixValue::Bool(b) => PyBool::new(py, *b).to_owned().into_any().unbind(),
        TvixValue::Integer(i) => PyInt::new(py, *i).to_owned().into_any().unbind(),
        TvixValue::Float(f) => PyFloat::new(py, *f).to_owned().into_any().unbind(),
        TvixValue::String(s) => PyString::new(py, &s.to_string())
            .to_owned()
            .into_any()
            .unbind(),
        TvixValue::Path(s) => PyString::new(
            py,
            &(s.clone().into_os_string().into_string().map_err(|_| {
                PyValueError::new_err(
                    "Failed to convert path to string, try wrap your path as `${path}`",
                )
            })?),
        )
        .to_owned()
        .into_any()
        .unbind(),
        TvixValue::List(l) => {
            let converted = l
                .into_iter()
                .map(|v| tvix_to_pyobject(py, v))
                .collect::<PyResult<Vec<_>>>()?;
            PyList::new(py, converted)?.to_owned().into_any().unbind()
        }
        TvixValue::Attrs(attrs) => {
            let dict = PyDict::new(py);
            for (k, v) in attrs.iter() {
                let key = from_utf8(k.as_bytes()).map_err(|e| {
                    PyUnicodeError::new_err(format!(
                        "Failed to convert bytes to string ({}) on {}",
                        e, k
                    ))
                })?;
                let value = tvix_to_pyobject(py, v)?;
                dict.set_item(key, value)?;
            }
            dict.to_owned().into_any().unbind()
        }
        TvixValue::Thunk(t) => {
            if t.is_evaluated() {
                tvix_to_pyobject(py, &t.value())?
            } else {
                Err(PyValueError::new_err(format!(
                    "Cannot convert nix thunk to python object: {}",
                    value
                )))?
            }
        }
        _ => Err(PyValueError::new_err(format!(
            "Cannot convert nix type {} to python object",
            value
        )))?,
    };
    Ok(object)
}

/// Convert TvixValue to PyDict
fn tvix_to_pydict(py: Python<'_>, value: &TvixValue) -> PyResult<Py<PyDict>> {
    let dict = PyDict::new(py);
    if let TvixValue::Attrs(attrs) = value {
        for (k, v) in attrs.iter() {
            let key = from_utf8(k.as_bytes()).map_err(|e| {
                PyUnicodeError::new_err(format!(
                    "Failed to convert bytes to string ({}) on {}",
                    e, k
                ))
            })?;
            let value = tvix_to_pyobject(py, v)?;
            dict.set_item(key, value)?;
        }
        Ok(dict.to_owned().unbind())
    } else {
        Err(PyValueError::new_err(format!(
            "Expected a nix attrs, but got {}",
            value
        )))
    }
}

/// Evaluate a nix file and convert it to python dictionary
/// The expression must be evaluated as attrset.
///
/// Args:
///   path (str): The path to the nix file.
///
/// Returns:
///   dict: The evaluated nix expression as a python dictionary.
#[pyfunction]
pub fn eval(py: Python<'_>, path: String) -> PyResult<Py<PyDict>> {
    let path = PathBuf::from(path);
    let content = fs::read_to_string(&path).map_err(|e| {
        PyIOError::new_err(format!("Failed to read file {}: {}", path.display(), e))
    })?;
    let value = eval_expr(&content, Some(path.clone())).map_err(|e| {
        PyValueError::new_err(format!(
            "Failed to evaluate nix expression in {}\n  {}",
            path.display(),
            e
        ))
    })?;

    tvix_to_pydict(py, &value).map_err(|e| {
        PyValueError::new_err(format!(
            "Failed to convert nix expression to python dict:\n  {}",
            e
        ))
    })
}

/// Evaluate a nix expression and convert it to python dictionary.
/// The expression must be evaluated as attrset.
///
/// Args:
///    content (str): The nix expression to evaluate.
///    dir (str): The base directory to evaluate the expression in, we will
///               create a vitrual nix file as if the content is in the file
///
/// Returns:
///   dict: The evaluated nix expression as a python dictionary.
#[pyfunction]
pub fn evals(py: Python<'_>, content: String, dir: Option<String>) -> PyResult<Py<PyDict>> {
    let path = dir.map(|d| PathBuf::from(d).join("virtual.nix"));
    let value = eval_expr(&content, path.clone()).map_err(|e| {
        PyValueError::new_err(format!(
            "Failed to evaluate nix expression '''\n{}'''\nErrors:\n{}",
            content, e
        ))
    })?;

    tvix_to_pydict(py, &value).map_err(|e| {
        PyValueError::new_err(format!(
            "Failed to convert nix expression to python dict:\n  {}",
            e
        ))
    })
}

#[pymodule]
pub fn nix(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    m.add_function(wrap_pyfunction!(evals, m)?)?;
    Ok(())
}
