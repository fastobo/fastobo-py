use std::io::Error as IOError;
use std::path::Path;

use pest::error::ErrorVariant;
use pest::error::InputLocation;
use pest::error::LineColLocation;
use pyo3::exceptions::FileNotFoundError;
use pyo3::exceptions::OSError;
use pyo3::exceptions::RuntimeError;
use pyo3::exceptions::SyntaxError;
use pyo3::exceptions::ValueError;
use pyo3::PyErr;

use fastobo::parser::Rule;

/// Exact copy of `pest::error::Error` to access private fields.
struct PestError {
    /// Variant of the error
    pub variant: ErrorVariant<Rule>,
    /// Location within the input string
    pub location: InputLocation,
    /// Line/column within the input string
    pub line_col: LineColLocation,
    path: Option<String>,
    line: String,
    #[allow(dead_code)]
    continued_line: Option<String>,
}

impl PestError {
    fn message(&self) -> String {
        match self.variant {
            ErrorVariant::ParsingError {
                ref positives,
                ref negatives,
            } => Self::parsing_error_message(positives, negatives, |r| format!("{:?}", r)),
            ErrorVariant::CustomError { ref message } => message.clone(),
        }
    }

    fn parsing_error_message<F>(positives: &[Rule], negatives: &[Rule], mut f: F) -> String
    where
        F: FnMut(&Rule) -> String,
    {
        match (negatives.is_empty(), positives.is_empty()) {
            (false, false) => format!(
                "unexpected {}; expected {}",
                Self::enumerate(negatives, &mut f),
                Self::enumerate(positives, &mut f)
            ),
            (false, true) => format!("unexpected {}", Self::enumerate(negatives, &mut f)),
            (true, false) => format!("expected {}", Self::enumerate(positives, &mut f)),
            (true, true) => "unknown parsing error".to_owned(),
        }
    }

    fn enumerate<F>(rules: &[Rule], f: &mut F) -> String
    where
        F: FnMut(&Rule) -> String,
    {
        match rules.len() {
            1 => f(&rules[0]),
            2 => format!("{} or {}", f(&rules[0]), f(&rules[1])),
            l => {
                let separated = rules
                    .iter()
                    .take(l - 1)
                    .map(|r| f(r))
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("{}, or {}", separated, f(&rules[l - 1]))
            }
        }
    }
}

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
        Self {
            err,
            path: None
        }
    }
}

impl From<Error> for PyErr {
    fn from(error: Error) -> Self {
        match error.err {
            fastobo::error::Error::SyntaxError { error } => {
                match error {
                    fastobo::error::SyntaxError::ParserError { error } => {
                        // SUPER UNSAFE: check the struct has not changed when
                        //               updating! Using private fields is out of
                        //               semver so any update is dangerous.
                        let pe: PestError = unsafe { std::mem::transmute(error) };
                        let msg = pe.message();
                        let path = pe.path.unwrap_or(String::from("<stdin>"));
                        let (l, c) = match pe.line_col {
                            LineColLocation::Pos((l, c)) => (l, c),
                            LineColLocation::Span((l, c), _) => (l, c),
                        };
                        SyntaxError::py_err((msg, (path, l, c, pe.line)))
                    }
                    fastobo::error::SyntaxError::UnexpectedRule { expected, actual } => {
                        RuntimeError::py_err("unexpected rule")
                    }
                }
            }

            fastobo::error::Error::IOError { error: ioerror } => {
                let desc = ioerror.to_string();
                match ioerror.raw_os_error() {
                    Some(2) => FileNotFoundError::py_err((2, desc, error.path)),
                    Some(code) => OSError::py_err((code, desc, error.path)),
                    None => OSError::py_err((desc,))
                }
            }

            other => RuntimeError::py_err(format!("{}", other)),
        }
    }
}

impl<T> Into<pyo3::PyResult<T>> for Error {
    fn into(self) -> pyo3::PyResult<T> {
        Err(pyo3::PyErr::from(self))
    }
}

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
            fastobo_graphs::error::Error::OboSyntaxError(error) => {
                Error::from(error).into()
            }
            fastobo_graphs::error::Error::IOError(error) => {
                let desc = error.to_string();
                match error.raw_os_error() {
                    Some(code) => OSError::py_err((code, desc)),
                    None => OSError::py_err((desc,)),
                }
            }
            other => ValueError::py_err(other.to_string()),
        }
    }
}
