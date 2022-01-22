use pyo3::prelude::*;
use pyo3::create_exception;
use pyo3::exceptions::PyValueError;
use pyo3::exceptions::PyRuntimeError;
use pyo3::exceptions::PyChildProcessError;

// --- Module export ---------------------------------------------------------

#[pymodule]
#[pyo3(name = "exceptions")]
pub fn init(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("CardinalityError", py.get_type::<self::CardinalityError>())?;
    m.add("MissingClauseError", py.get_type::<self::MissingClauseError>())?;
    m.add("DuplicateClausesError", py.get_type::<self::DuplicateClausesError>())?;
    m.add("SingleClauseError", py.get_type::<self::SingleClauseError>())?;
    m.add("DisconnectedChannelError", py.get_type::<self::DisconnectedChannelError>())?;
    m.add("__name__", "fastobo.exceptions")?;
    Ok(())
}

// --- New exceptions --------------------------------------------------------

create_exception!(exceptions, CardinalityError, PyValueError);
create_exception!(exceptions, MissingClauseError, CardinalityError);
create_exception!(exceptions, DuplicateClausesError, CardinalityError);
create_exception!(exceptions, SingleClauseError, CardinalityError);

create_exception!(exceptions, DisconnectedChannelError, PyChildProcessError);
