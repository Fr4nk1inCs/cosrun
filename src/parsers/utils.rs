use std::ops::Range;

use annotate_snippets::{Annotation, Snippet};
use pyo3::exceptions::PyValueError;
use pyo3::{create_exception, PyErr, PyObject, PyResult, Python};

create_exception!(parsers, ParseError, PyValueError);
create_exception!(parsers, EvaluationError, PyValueError);
create_exception!(parsers, ConversionError, PyValueError);

pub trait IntoRange<T> {
    fn into_range(self) -> Range<T>;
}

pub trait IntoAnnotation<'a> {
    fn into_annotation(self) -> (Option<Annotation<'a>>, String);
}

pub trait IntoPyErr {
    fn into_pyerr(self, snippet: Snippet) -> PyErr;
}

#[macro_export]
macro_rules! into_pyany {
    ($obj:expr) => {
        $obj.to_owned().into_any().unbind()
    };
}

pub trait TryToPyObject {
    fn try_to_pyobject(&self, py: Python<'_>) -> PyResult<PyObject>;
}
