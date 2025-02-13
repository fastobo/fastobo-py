use std::io::Error as IOError;
use std::path::Path;

use pyo3::exceptions::PyFileNotFoundError;
use pyo3::exceptions::PyOSError;
use pyo3::exceptions::PyRuntimeError;
use pyo3::exceptions::PySyntaxError;
use pyo3::exceptions::PyValueError;
use pyo3::PyErr;

use fastobo::ast as obo;
use fastobo::syntax::pest::error::ErrorVariant;
use fastobo::syntax::pest::error::InputLocation;
use fastobo::syntax::pest::error::LineColLocation;
use fastobo::syntax::Rule;

use crate::py::exceptions::DisconnectedChannelError;
use crate::py::exceptions::DuplicateClausesError;
use crate::py::exceptions::MissingClauseError;
use crate::py::exceptions::SingleClauseError;

// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! raise(
    ($py:expr, $error_type:ident ($msg:expr) from $inner:expr ) => ({
        let err = $error_type::new_err($msg).to_object($py);
        err.call_method1(
            $py,
            "__setattr__",
            ("__cause__".to_object($py), $inner.to_object($py)),
        )?;
        return Err(PyErr::from_value(err.bind($py).clone()))
    })
);

// ---------------------------------------------------------------------------

/// A wrapper to convert `fastobo::error::Error` into a `PyErr`.
pub struct Error {
    err: fastobo::error::Error,
    path: Option<String>,
}

impl Error {
    pub fn with_path<S: Into<String>>(mut self, path: S) -> Self {
        self.path = Some(path.into());
        self
    }
}

impl From<Error> for fastobo::error::Error {
    fn from(error: Error) -> Self {
        error.err
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        fastobo::error::Error::from(err).into()
    }
}

impl From<fastobo::error::SyntaxError> for Error {
    fn from(err: fastobo::error::SyntaxError) -> Self {
        fastobo::error::Error::from(err).into()
    }
}

impl From<fastobo::error::Error> for Error {
    fn from(err: fastobo::error::Error) -> Self {
        Self { err, path: None }
    }
}

impl From<Error> for PyErr {
    fn from(error: Error) -> Self {
        match error.err {
            fastobo::error::Error::SyntaxError { error } => match error {
                fastobo::error::SyntaxError::ParserError { error } => {
                    let msg = error.variant.message().into_owned();
                    let path = error
                        .path()
                        .map(String::from)
                        .unwrap_or_else(|| String::from("<stdin>"));
                    let line = error.line().to_string();
                    let (l, c) = match error.line_col {
                        LineColLocation::Pos((l, c)) => (l, c),
                        LineColLocation::Span((l, c), _) => (l, c),
                    };
                    PySyntaxError::new_err((msg, (path, l, c, line)))
                }
                fastobo::error::SyntaxError::UnexpectedRule { expected, actual } => {
                    PyRuntimeError::new_err("unexpected rule")
                }
            },

            fastobo::error::Error::IOError { error: ioerror } => {
                let desc = ioerror.to_string();
                match ioerror.raw_os_error() {
                    Some(2) => PyFileNotFoundError::new_err((2, desc, error.path)),
                    Some(code) => PyOSError::new_err((code, desc, error.path)),
                    None => PyOSError::new_err((desc,)),
                }
            }

            fastobo::error::Error::CardinalityError { id, inner } => {
                let idstr = id.map(|ident| ident.to_string());
                match inner {
                    fastobo::error::CardinalityError::MissingClause { name } => {
                        MissingClauseError::new_err((name, idstr))
                    }
                    fastobo::error::CardinalityError::DuplicateClauses { name } => {
                        DuplicateClausesError::new_err((name, idstr))
                    }
                    fastobo::error::CardinalityError::SingleClause { name } => {
                        SingleClauseError::new_err((name, idstr))
                    }
                }
            }

            fastobo::error::Error::ThreadingError { error } => match error {
                fastobo::error::ThreadingError::DisconnectedChannel => {
                    DisconnectedChannelError::new_err(())
                }
            }, // other => PyRuntimeError::new_err(format!("{}", other)),
        }
    }
}

impl<T> Into<pyo3::PyResult<T>> for Error {
    fn into(self) -> pyo3::PyResult<T> {
        Err(pyo3::PyErr::from(self))
    }
}

// ---------------------------------------------------------------------------

/// A wrapper to convert `fastobo_graphs::error::Error` into a `PyErr`.
pub struct GraphError(fastobo_graphs::error::Error);

impl From<fastobo_graphs::error::Error> for GraphError {
    fn from(e: fastobo_graphs::error::Error) -> Self {
        GraphError(e)
    }
}

impl From<GraphError> for PyErr {
    fn from(err: GraphError) -> Self {
        match err.0 {
            fastobo_graphs::error::Error::OboSyntaxError(error) => Error::from(error).into(),
            fastobo_graphs::error::Error::IOError(error) => {
                let desc = error.to_string();
                match error.raw_os_error() {
                    Some(code) => PyOSError::new_err((code, desc)),
                    None => PyOSError::new_err((desc,)),
                }
            }
            other => PyValueError::new_err(other.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------

/// A wrapper to convert `fastobo_graphs::error::Error` into a `PyErr`.
pub struct OwlError(fastobo_owl::Error);

impl From<fastobo_owl::Error> for OwlError {
    fn from(e: fastobo_owl::Error) -> Self {
        OwlError(e)
    }
}

impl From<OwlError> for PyErr {
    fn from(err: OwlError) -> Self {
        match err.0 {
            fastobo_owl::Error::Cardinality(error) => {
                Error::from(fastobo::error::Error::CardinalityError {
                    id: Some(obo::Ident::from(obo::UnprefixedIdent::new("header"))),
                    inner: error,
                })
                .into()
            }
            fastobo_owl::Error::Syntax(error) => Error::from(error).into(),
        }
    }
}
