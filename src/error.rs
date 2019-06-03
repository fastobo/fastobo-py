use std::io::Error as IOError;
use std::path::Path;

use pest::error::ErrorVariant;
use pest::error::InputLocation;
use pest::error::LineColLocation;
use pyo3::exceptions::OSError;
use pyo3::exceptions::RuntimeError;
use pyo3::exceptions::SyntaxError;
use pyo3::PyErr;

use fastobo::parser::Rule;

/// Exact copy of `pest::error::Error` to access privat fields.
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
pub struct Error(fastobo::error::Error);

impl From<Error> for fastobo::error::Error {
    fn from(err: Error) -> Self {
        err.0
    }
}

impl From<fastobo::error::Error> for Error {
    fn from(err: fastobo::error::Error) -> Self {
        Self(err)
    }
}

impl From<Error> for PyErr {
    fn from(err: Error) -> Self {
        match err.0 {
            fastobo::error::Error::ParserError { error } => {
                // SUPER UNSAFE: check the struct as not changed when
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

            fastobo::error::Error::IOError { error } => {
                let desc = <std::io::Error as std::error::Error>::description(&error).to_string();
                match error.raw_os_error() {
                    Some(code) => OSError::py_err((code, desc)),
                    None => OSError::py_err((desc,)),
                }
            }

            _ => RuntimeError::py_err("todo"),
        }
    }
}

impl<T> Into<pyo3::PyResult<T>> for Error {
    fn into(self) -> pyo3::PyResult<T> {
        Err(pyo3::PyErr::from(self))
    }
}
