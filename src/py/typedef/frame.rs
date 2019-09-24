use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fmt::Write;
use std::str::FromStr;

use pyo3::exceptions::IndexError;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyIterator;
use pyo3::types::PyString;
use pyo3::AsPyPointer;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PySequenceProtocol;
use pyo3::PyTypeInfo;

use fastobo::ast;
use fastobo::share::Cow;
use fastobo::share::Redeem;
use fastobo::share::Share;

use super::super::abc::AbstractEntityFrame;
use super::super::id::Ident;
use super::clause::TypedefClause;
use crate::utils::ClonePy;

#[pyclass(extends=AbstractEntityFrame, module="fastobo.typedef")]
#[derive(Debug, PyList)]
pub struct TypedefFrame {
    id: Ident,
    clauses: Vec<TypedefClause>,
}

impl TypedefFrame {
    pub fn new(id: Ident) -> Self {
        Self::with_clauses(id, Vec::new())
    }

    pub fn with_clauses(id: Ident, clauses: Vec<TypedefClause>) -> Self {
        Self { id, clauses }
    }
}

impl ClonePy for TypedefFrame {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            id: self.id.clone_py(py),
            clauses: self.clauses.clone_py(py),
        }
    }
}

impl Display for TypedefFrame {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefFrame::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<fastobo::ast::TypedefFrame> for TypedefFrame {
    fn from_py(frame: fastobo::ast::TypedefFrame, py: Python) -> Self {
        Self::with_clauses(
            Ident::from_py(frame.id().as_ref().clone(), py),
            frame
                .into_iter()
                .map(|line| TypedefClause::from_py(line.into_inner(), py))
                .collect(),
        )
    }
}

impl FromPy<TypedefFrame> for fastobo::ast::TypedefFrame {
    fn from_py(frame: TypedefFrame, py: Python) -> Self {
        fastobo::ast::TypedefFrame::with_clauses(
            fastobo::ast::RelationIdent::new(frame.id.into_py(py)),
            frame
                .clauses
                .iter()
                .map(|f| fastobo::ast::TypedefClause::from_py(f, py))
                .map(fastobo::ast::Line::from)
                .collect(),
        )
    }
}

impl FromPy<TypedefFrame> for fastobo::ast::EntityFrame {
    fn from_py(frame: TypedefFrame, py: Python) -> Self {
        Self::from(fastobo::ast::TypedefFrame::from_py(frame, py))
    }
}

#[pymethods]
impl TypedefFrame {
    // FIXME: should accept any iterable.
    #[new]
    fn __init__(obj: &PyRawObject, id: Ident, clauses: Option<Vec<TypedefClause>>) -> PyResult<()> {
        Ok(obj.init(Self::with_clauses(id, clauses.unwrap_or_else(Vec::new))))
    }

    #[getter]
    fn get_id(&self) -> PyResult<&Ident> {
        Ok(&self.id)
    }

    #[setter]
    fn set_id(&mut self, id: Ident) -> PyResult<()> {
        self.id = id;
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for TypedefFrame {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        PyString::new(py, "TypedefFrame({!r})")
            .to_object(py)
            .call_method1(py, "format", (&self.id,))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }
}

#[pyproto]
impl PySequenceProtocol for TypedefFrame {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.clauses.len())
    }

    fn __getitem__(&self, index: isize) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        if index < self.clauses.len() as isize {
            let item = &self.clauses[index as usize];
            Ok(item.to_object(py))
        } else {
            IndexError::into("list index out of range")
        }
    }

    fn __setitem__(&mut self, index: isize, elem: &PyAny) -> PyResult<()> {
        if index as usize > self.clauses.len() {
            return IndexError::into("list index out of range");
        }
        let clause = TypedefClause::extract(elem)?;
        self.clauses[index as usize] = clause;
        Ok(())
    }

    fn __delitem__(&mut self, index: isize) -> PyResult<()> {
        if index as usize > self.clauses.len() {
            return IndexError::into("list index out of range");
        }
        self.clauses.remove(index as usize);
        Ok(())
    }

    fn __concat__(&self, other: &PyAny) -> PyResult<Self> {
        let py = other.py();

        let iterator = PyIterator::from_object(py, other)?;
        let mut new_clauses = self.clauses.clone_py(py);
        for item in iterator {
            new_clauses.push(TypedefClause::extract(item?)?);
        }

        Ok(Self::with_clauses(self.id.clone_py(py), new_clauses))
    }
}
