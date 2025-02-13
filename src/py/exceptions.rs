use pyo3::exceptions::PyChildProcessError;
use pyo3::exceptions::PyRuntimeError;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3::types::PyTuple;

// --- Macros ----------------------------------------------------------------

macro_rules! impl_pyerr {
    ($name:ident) => {
        impl $name {
            /// Creates a new [`PyErr`] of this type.
            ///
            /// [`PyErr`]: https://docs.rs/pyo3/latest/pyo3/struct.PyErr.html "PyErr in pyo3"
            #[inline]
            pub fn new_err<A>(args: A) -> pyo3::PyErr
            where
                A: pyo3::PyErrArguments + Send + Sync + 'static,
            {
                pyo3::PyErr::new::<$name, A>(args)
            }
        }
    };
}

// --- Module export ---------------------------------------------------------

#[pymodule]
#[pyo3(name = "exceptions")]
pub fn init<'py>(py: Python<'py>, m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add_class::<self::MissingClauseError>()?;
    m.add_class::<self::DuplicateClausesError>()?;
    m.add_class::<self::SingleClauseError>()?;
    m.add_class::<self::DisconnectedChannelError>()?;
    m.add("__name__", "fastobo.exceptions")?;
    Ok(())
}

// --- MissingClauseError ----------------------------------------------------

/// An error indicating a required clause is missing.
#[pyclass(module = "fastobo.exceptions", extends = PyValueError)]
pub struct MissingClauseError {
    clause: String,
    frame: Option<String>,
}

impl_pyerr!(MissingClauseError);

#[pymethods]
impl MissingClauseError {
    #[new]
    #[pyo3(signature = (clause, frame = None))]
    fn __init__(clause: String, frame: Option<String>) -> Self {
        Self { clause, frame }
    }

    fn __repr__(&self) -> String {
        match &self.frame {
            None => format!("MissingClauseError({})", self.clause.as_str()),
            Some(f) => format!("MissingClauseError({}, {})", self.clause.as_str(), f),
        }
    }

    fn __str__(&self) -> String {
        match &self.frame {
            None => format!("missing '{}' clause", &self.clause),
            Some(f) => format!("missing '{}' clause in '{}' frame", &self.clause, &f),
        }
    }
}

// --- DuplicateClausesError -------------------------------------------------

/// An error indicating a unique clause appears more than one.
#[pyclass(module = "fastobo.exceptions", extends = PyValueError)]
pub struct DuplicateClausesError {
    clause: String,
    frame: Option<String>,
}

impl_pyerr!(DuplicateClausesError);

#[pymethods]
impl DuplicateClausesError {
    #[new]
    #[pyo3(signature = (clause, frame = None))]
    fn __init__(clause: String, frame: Option<String>) -> Self {
        Self { clause, frame }
    }

    fn __repr__(&self) -> String {
        match &self.frame {
            None => format!("DuplicateClausesError({})", self.clause.as_str()),
            Some(f) => format!("DuplicateClausesError({}, {})", self.clause.as_str(), f),
        }
    }

    fn __str__(&self) -> String {
        match &self.frame {
            None => format!("duplicate '{}' clauses", &self.clause),
            Some(f) => format!("duplicate '{}' clauses in '{}' frame", &self.clause, &f),
        }
    }
}

// --- SingleClauseError -----------------------------------------------------

/// An error indicating a clause appears only once when it shouldn't.
#[pyclass(module = "fastobo.exceptions", extends = PyValueError)]
pub struct SingleClauseError {
    clause: String,
    frame: Option<String>,
}

impl_pyerr!(SingleClauseError);

#[pymethods]
impl SingleClauseError {
    #[new]
    #[pyo3(signature = (clause, frame = None))]
    fn __init__(clause: String, frame: Option<String>) -> Self {
        Self { clause, frame }
    }

    fn __repr__(&self) -> String {
        match &self.frame {
            None => format!("SingleClauseError({})", self.clause.as_str()),
            Some(f) => format!("SingleClauseError({}, {})", self.clause.as_str(), f),
        }
    }

    fn __str__(&self) -> String {
        match &self.frame {
            None => format!("single '{}' clause", &self.clause),
            Some(f) => format!("single '{}' clause in '{}' frame", &self.clause, &f),
        }
    }
}

// --- DisconnectedChannelError ----------------------------------------------

#[pyclass(module = "fastobo.exceptions", extends = PyRuntimeError)]
pub struct DisconnectedChannelError {}

impl_pyerr!(DisconnectedChannelError);

#[pymethods]
impl DisconnectedChannelError {
    #[new]
    fn __init__() -> Self {
        Self {}
    }

    fn __repr__(&self) -> String {
        String::from("DisconnectedChannelError()")
    }

    fn __str__(&self) -> String {
        String::from("disconnected thread communication channel")
    }
}
