use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fmt::Write;
use std::str::FromStr;

use pyo3::exceptions::PyIndexError;
use pyo3::exceptions::PyTypeError;
use pyo3::exceptions::PyValueError;
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

use super::super::abc::AbstractEntityFrame;
use super::super::id::Ident;
use super::clause::TypedefClause;
use crate::utils::AbstractClass;
use crate::utils::ClonePy;
use crate::utils::FinalClass;

#[pyclass(extends=AbstractEntityFrame, module="fastobo.typedef")]
#[derive(Debug, FinalClass)]
#[base(AbstractEntityFrame)]
pub struct TypedefFrame {
    #[pyo3(set)]
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
        let frame: fastobo::ast::TypedefFrame = self.clone_py(py).into_py(py);
        frame.fmt(f)
    }
}

impl IntoPy<TypedefFrame> for fastobo::ast::TypedefFrame {
    fn into_py(self, py: Python) -> TypedefFrame {
        TypedefFrame::with_clauses(
            self.id().as_ref().clone().into_py(py),
            self.into_iter()
                .map(|line| line.into_inner().into_py(py))
                .collect(),
        )
    }
}

impl IntoPy<fastobo::ast::TypedefFrame> for TypedefFrame {
    fn into_py(self, py: Python) -> fastobo::ast::TypedefFrame {
        fastobo::ast::TypedefFrame::with_clauses(
            fastobo::ast::RelationIdent::new(self.id.into_py(py)),
            self.clauses
                .iter()
                .map(|f| f.into_py(py))
                .map(|c| fastobo::ast::Line::new().and_inner(c))
                .collect(),
        )
    }
}

impl IntoPy<fastobo::ast::EntityFrame> for TypedefFrame {
    fn into_py(self, py: Python) -> fastobo::ast::EntityFrame {
        let frame: fastobo::ast::TypedefFrame = self.into_py(py);
        fastobo::ast::EntityFrame::from(frame)
    }
}

#[listlike(field = "clauses", type = "TypedefClause")]
#[pymethods]
impl TypedefFrame {
    // FIXME: should accept any iterable.
    #[new]
    fn __init__(id: Ident, clauses: Option<Vec<TypedefClause>>) -> PyClassInitializer<Self> {
        Self::with_clauses(id, clauses.unwrap_or_else(Vec::new)).into()
    }

    #[getter]
    fn get_id(&self) -> PyResult<&Ident> {
        Ok(&self.id)
    }
}

#[pyproto]
impl PyObjectProtocol for TypedefFrame {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, TypedefFrame(self.id))
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
            Err(PyIndexError::new_err("list index out of range"))
        }
    }

    fn __setitem__(&mut self, index: isize, elem: &PyAny) -> PyResult<()> {
        if index as usize > self.clauses.len() {
            return Err(PyIndexError::new_err("list index out of range"));
        }
        let clause = TypedefClause::extract(elem)?;
        self.clauses[index as usize] = clause;
        Ok(())
    }

    fn __delitem__(&mut self, index: isize) -> PyResult<()> {
        if index as usize > self.clauses.len() {
            return Err(PyIndexError::new_err("list index out of range"));
        }
        self.clauses.remove(index as usize);
        Ok(())
    }

    fn __concat__(&self, other: &PyAny) -> PyResult<Py<Self>> {
        let py = other.py();

        let iterator = PyIterator::from_object(py, other)?;
        let mut new_clauses = self.clauses.clone_py(py);
        for item in iterator {
            new_clauses.push(TypedefClause::extract(item?)?);
        }

        Py::new(py, Self::with_clauses(self.id.clone_py(py), new_clauses))
    }
}
