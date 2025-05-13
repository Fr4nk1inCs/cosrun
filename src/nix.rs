use annotate_snippets::{Annotation, Level, Renderer, Snippet};
use codemap::Span;
use pyo3::create_exception;
use pyo3::exceptions::{PyIOError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyFloat, PyInt, PyList, PyNone, PyString};
use pyo3::PyObject;
use pyo3::{pyfunction, PyResult};
use rnix::parser::ParseError as RnixParseError;
use std::iter::zip;
use std::ops::Range;
use std::path::PathBuf;
use std::str::from_utf8;
use std::{fs, rc::Rc};
use tvix_eval::{
    Error as TvixError, ErrorKind as TvixErrorKind, Value as TvixValue,
};
use tvix_eval::{EvalIO, EvalMode, Evaluation, StdIO};

create_exception!(nix, ParseError, PyValueError);
create_exception!(nix, EvaluateError, PyValueError);
create_exception!(nix, ConvertionError, PyValueError);

trait IntoRange<T> {
    fn into_range(self) -> Range<T>;
}

impl IntoRange<usize> for Span {
    fn into_range(self) -> Range<usize> {
        // parse the usize from the debug string: Pos(usize)
        let start = format!("{:?}", self.low())
            .strip_prefix("Pos(")
            .unwrap()
            .strip_suffix(")")
            .unwrap()
            .parse::<u32>()
            .unwrap();
        let end = format!("{:?}", self.high())
            .strip_prefix("Pos(")
            .unwrap()
            .strip_suffix(")")
            .unwrap()
            .parse::<u32>()
            .unwrap();
        Range {
            start: (start - 1) as usize,
            end: (end - 1) as usize,
        }
    }
}

trait IntoAnnotation<'a> {
    fn into_annotation(self) -> (Option<Annotation<'a>>, String);
}

impl<'a> IntoAnnotation<'a> for &RnixParseError {
    fn into_annotation(self) -> (Option<Annotation<'a>>, String) {
        match self {
            RnixParseError::Unexpected(range) => (
                Some(Level::Error.span(Range::<usize>::from(*range))),
                "error node".into(),
            ),
            RnixParseError::UnexpectedExtra(range) => (
                Some(Level::Error.span(Range::<usize>::from(*range))),
                "unexpected token at".into(),
            ),
            RnixParseError::UnexpectedWanted(got, range, kinds) => (
                Some(Level::Error.span(Range::<usize>::from(*range))),
                format!("expect any of {:?}, found {:?}", kinds, got),
            ),
            RnixParseError::UnexpectedDoubleBind(range) => (
                Some(Level::Error.span(Range::<usize>::from(*range))),
                "unexpected double bind".into(),
            ),
            RnixParseError::UnexpectedEOF => {
                (None, "unexpected EOF".to_string())
            }
            RnixParseError::UnexpectedEOFWanted(kinds) => {
                (None, format!("unexpected EOF, expected any of {:?}", kinds))
            }
            RnixParseError::DuplicatedArgs(range, ident) => (
                Some(Level::Error.span(Range::<usize>::from(*range))),
                format!("duplicated argument {}", ident),
            ),
            RnixParseError::RecursionLimitExceeded => {
                (None, "recursion limit exceeded".to_string())
            }
            _ => (None, "unknown error".to_string()),
        }
    }
}

trait IntoPyErr {
    fn into_pyerr(self, snippet: Snippet) -> PyErr;
}

impl IntoPyErr for TvixError {
    fn into_pyerr(self, snippet: Snippet) -> PyErr {
        let renderer = Renderer::styled();
        match self.kind {
            TvixErrorKind::ParseErrors(errors) => {
                let mut annotations = Vec::new();
                let mut anno_messages = Vec::new();
                let mut messages = Vec::new();

                for error in errors {
                    let (annotation, message) = error.into_annotation();
                    if let Some(annotation) = annotation {
                        annotations.push(annotation);
                        anno_messages.push(message);
                    } else {
                        messages.push(message);
                    }
                }

                let annotations = zip(annotations, anno_messages.iter())
                    .map(|(a, m)| a.label(m));
                let message = Level::Error
                    .title("failed to parse Nix code")
                    .snippet(snippet.annotations(annotations));
                let message = renderer.render(message).to_string();
                ParseError::new_err(message)
            }
            TvixErrorKind::NativeError { gen_type: _, err } => {
                err.into_pyerr(snippet)
            }
            TvixErrorKind::BytecodeError(err) => err.into_pyerr(snippet),
            _ => {
                let range = self.span.into_range();
                let title = self.to_string();
                let message = Level::Error
                    .title(&title)
                    .snippet(snippet.annotation(Level::Error.span(range)));
                let message = renderer.render(message).to_string();
                EvaluateError::new_err(message)
            }
        }
    }
}

/// Parse and evaluate a nix expression
fn eval_expr(expr: &str, location: Option<PathBuf>) -> PyResult<TvixValue> {
    // FIXME: This is a hack to make the evaluation result to be a JSON object
    let builder = Evaluation::builder_pure()
        .io_handle(Rc::new(StdIO) as Rc<dyn EvalIO>)
        .mode(EvalMode::Strict);
    let eval = builder.build();

    let result = eval.evaluate(expr, location.clone());

    if let Some(value) = result.value {
        Ok(value)
    } else {
        // Error message
        if result.errors.is_empty() {
            Err(EvaluateError::new_err(
                "No error is throwed but evaluation failed".to_string(),
            ))
        } else {
            let location = if let Some(location) = &location {
                location.to_string_lossy().to_string()
            } else {
                "tempfile".to_string()
            };

            let error = result.errors[0].clone();
            let snippet = Snippet::source(expr).origin(&location).fold(true);
            Err(error.into_pyerr(snippet))
        }
    }
}

trait TryToPy {
    fn try_to_object(&self, py: Python<'_>) -> PyResult<PyObject>;
    fn try_to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>>;
}

macro_rules! into_pyany {
    ($obj:expr) => {
        $obj.to_owned().into_any().unbind()
    };
}

impl TryToPy for TvixValue {
    fn try_to_object(&self, py: Python<'_>) -> PyResult<PyObject> {
        let object = match self {
            TvixValue::Null => into_pyany!(PyNone::get(py)),
            TvixValue::Bool(b) => into_pyany!(PyBool::new(py, *b)),
            TvixValue::Integer(i) => into_pyany!(PyInt::new(py, *i)),
            TvixValue::Float(f) => into_pyany!(PyFloat::new(py, *f)),
            TvixValue::String(s) => {
                into_pyany!(PyString::new(py, &s.to_string()))
            }
            TvixValue::Path(s) => {
                let converted = s.clone().into_os_string().into_string().map_err(|_| {
                    ConvertionError::new_err(
                        "Failed to convert path to string, try wrap your path as `\"${path}\"`",
                    )
                })?;
                into_pyany!(PyString::new(py, &converted))
            }

            TvixValue::List(l) => {
                let converted = l
                    .into_iter()
                    .map(|v| v.try_to_object(py))
                    .collect::<PyResult<Vec<_>>>()?;
                into_pyany!(PyList::new(py, converted)?)
            }
            TvixValue::Attrs(attrs) => {
                let dict = PyDict::new(py);
                for (k, v) in attrs.iter() {
                    let key = from_utf8(k.as_bytes()).map_err(|e| {
                        ConvertionError::new_err(format!(
                            "Failed to convert bytes to string ({}) on {}",
                            e, k
                        ))
                    })?;
                    let value = v.try_to_object(py)?;
                    dict.set_item(key, value)?;
                }
                into_pyany!(dict)
            }
            TvixValue::Thunk(thunk) => {
                if thunk.is_evaluated() {
                    thunk.value().try_to_object(py)?
                } else {
                    Err(ConvertionError::new_err(format!(
                        "Cannot convert nix thunk to python object: {}",
                        self
                    )))?
                }
            }
            _ => Err(ConvertionError::new_err(format!(
                "Cannot convert nix type {} to python object",
                self
            )))?,
        };
        Ok(object)
    }

    fn try_to_dict(&self, py: Python<'_>) -> PyResult<Py<PyDict>> {
        let dict = PyDict::new(py);
        match self {
            TvixValue::Attrs(attrs) => {
                for (k, v) in attrs.iter() {
                    let key = from_utf8(k.as_bytes()).map_err(|e| {
                        ConvertionError::new_err(format!(
                            "Failed to convert bytes to string ({}) on {}",
                            e, k
                        ))
                    })?;
                    let value = v.try_to_object(py)?;
                    dict.set_item(key, value)?;
                }
                Ok(dict.to_owned().unbind())
            }
            TvixValue::Thunk(thunk) => {
                if thunk.is_evaluated() {
                    Ok(thunk.value().try_to_dict(py)?)
                } else {
                    Err(ConvertionError::new_err(format!(
                        "Cannot convert nix thunk to python object: {}",
                        self
                    )))?
                }
            }
            _ => Err(ConvertionError::new_err(format!(
                "Expected a nix attrs, but got {}",
                self
            ))),
        }
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
        PyIOError::new_err(format!(
            "Failed to read file {}: {}",
            path.display(),
            e
        ))
    })?;
    eval_expr(&content, Some(path.clone()))?.try_to_dict(py)
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
pub fn evals(
    py: Python<'_>,
    content: String,
    dir: Option<String>,
) -> PyResult<Py<PyDict>> {
    let path = dir.map(|d| PathBuf::from(d).join("virtual.nix"));
    eval_expr(&content, path)?.try_to_dict(py)
}

#[pymodule]
pub fn nix(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(eval, m)?)?;
    m.add_function(wrap_pyfunction!(evals, m)?)?;
    Ok(())
}
