use std::fs;
use std::path::PathBuf;

use annotate_snippets::{Level, Renderer, Snippet};
use jsonc_parser::common::Range as JsoncRange;
use jsonc_parser::parse_to_value;
use jsonc_parser::JsonValue;
use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyFloat, PyInt, PyList, PyNone, PyString};
use pyo3::{PyObject, PyResult};

use crate::into_pyany;
use crate::parsers::utils::IntoRange;
use crate::parsers::utils::{ParseError, TryToPyObject};

impl IntoRange<usize> for JsoncRange {
    fn into_range(self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

impl TryToPyObject for JsonValue<'_> {
    fn try_to_pyobject(&self, py: Python<'_>) -> PyResult<PyObject> {
        let object = match self {
            JsonValue::Null => into_pyany!(PyNone::get(py)),
            JsonValue::Boolean(b) => into_pyany!(PyBool::new(py, *b)),
            JsonValue::Number(n) => {
                let number = n.to_string();
                if let Ok(int) = number.parse::<i64>() {
                    into_pyany!(PyInt::new(py, int))
                } else if let Ok(float) = number.parse::<f64>() {
                    into_pyany!(PyFloat::new(py, float))
                } else {
                    return Err(ParseError::new_err(format!(
                        "Could not parse number `{}` as either 64-bit integer \
                        or double precision floating point number",
                        number
                    )));
                }
            }
            JsonValue::String(s) => into_pyany!(PyString::new(py, s)),
            JsonValue::Array(arr) => {
                into_pyany!(PyList::new(
                    py,
                    arr.iter()
                        .map(|v| v.try_to_pyobject(py))
                        .collect::<PyResult<Vec<_>>>()
                )?)
            }
            JsonValue::Object(obj) => {
                let dict = pyo3::types::PyDict::new(py);
                for (key, value) in obj.clone().into_iter() {
                    let key_obj = PyString::new(py, &key);
                    let value_obj = value.try_to_pyobject(py)?;
                    dict.set_item(key_obj, value_obj)?;
                }
                dict.into()
            }
        };
        Ok(object)
    }
}

fn parse(content: &str, path: Option<PathBuf>) -> PyResult<JsonValue> {
    let parsed = parse_to_value(content, &Default::default());
    let path = path.as_ref().map(|p| p.to_string_lossy().to_string());

    match parsed {
        Ok(value) => Ok(value.ok_or(ParseError::new_err(
            "Parsed JSONC content is empty or invalid",
        ))?),
        Err(error) => {
            let snippet = if let Some(path) = &path {
                Snippet::source(content).fold(true).origin(path)
            } else {
                Snippet::source(content).fold(true)
            };
            let message = Renderer::styled()
                .render(Level::Error.title(&error.kind().to_string()).snippet(
                    snippet.annotation(
                        Level::Error.span(error.range().into_range()),
                    ),
                ))
                .to_string();
            Err(ParseError::new_err(message))
        }
    }
}

/// Parse a JSONC (JSON with comments) file and convert it to a Python object.
///
/// Args:
///   - path (str): The path to the JSONC file.
///
/// Returns:
///   - _JsonValue: A Python object representing a valid JSON value.
///
/// Raises:
///   - IOError: If the file cannot be read.
///   - ParseError: If the content is not valid JSONC.
#[pyfunction]
pub fn load(py: Python<'_>, path: String) -> PyResult<PyObject> {
    let path = PathBuf::from(path);
    let content = fs::read_to_string(&path).map_err(|e| {
        PyIOError::new_err(format!(
            "Failed to read file {}: {}",
            path.display(),
            e
        ))
    })?;
    parse(&content, Some(path))?.try_to_pyobject(py)
}

/// Parse a JSONC (JSON with comments) string and convert it to a Python object.
///
/// Args:
///   - content (str): The JSONC content as a string.
///
/// Returns:
///   - _JsonValue: A Python object representing a valid JSON value.
///
/// Raises:
///   - ParseError: If the content is not valid JSONC.
#[pyfunction]
pub fn loads(py: Python<'_>, expr: String) -> PyResult<PyObject> {
    parse(&expr, None)?.try_to_pyobject(py)
}
