use std::iter::FromIterator;
use std::iter::IntoIterator;

use fastobo::ast as obo;
use pyo3::class::gc::PyVisit;
use pyo3::exceptions::PyIndexError;
use pyo3::gc::PyTraverseError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyIterator;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::AsPyPointer;
use pyo3::PyGCProtocol;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PySequenceProtocol;
use pyo3::PyTypeInfo;

use super::super::abc::AbstractFrame;
use super::clause::BaseHeaderClause;
use super::clause::HeaderClause;
use crate::utils::ClonePy;
use crate::utils::FinalClass;
use crate::utils::AbstractClass;

#[pyclass(extends=AbstractFrame, module="fastobo.header")]
#[derive(Debug, FinalClass)]
#[base(AbstractFrame)]
pub struct HeaderFrame {
    clauses: Vec<HeaderClause>,
}

impl HeaderFrame {
    pub fn empty() -> Self {
        Self::new(Vec::new())
    }

    pub fn new(clauses: Vec<HeaderClause>) -> Self {
        Self { clauses }
    }
}

impl ClonePy for HeaderFrame {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            clauses: self.clauses.clone_py(py),
        }
    }
}

impl FromIterator<HeaderClause> for HeaderFrame {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = HeaderClause>,
    {
        Self::new(iter.into_iter().collect())
    }
}

impl IntoPy<HeaderFrame> for fastobo::ast::HeaderFrame {
    fn into_py(self, py: Python) -> HeaderFrame {
        self
            .into_iter()
            .map(|clause| clause.into_py(py))
            .collect()
    }
}

impl IntoPy<obo::HeaderFrame> for HeaderFrame {
    fn into_py(self, py: Python) -> obo::HeaderFrame {
        self
            .clauses
            .into_iter()
            .map(|clause| clause.into_py(py))
            .collect()
    }
}

impl ToPyObject for HeaderFrame {
    fn to_object(&self, py: Python) -> PyObject {
        IntoPy::into_py(PyList::new(py, &self.clauses), py)
    }
}

#[listlike(field = "clauses", type = "HeaderClause")]
#[pymethods]
impl HeaderFrame {
    #[new]
    pub fn __init__(clauses: Option<&PyAny>) -> PyResult<PyClassInitializer<Self>> {
        let mut vec = Vec::new();
        if let Some(c) = clauses {
            for item in PyIterator::from_object(c.py(), c)? {
                vec.push(HeaderClause::extract(item?)?);
            }
        }
        Ok(Self::new(vec).into())
    }
}

#[pyproto]
impl PyObjectProtocol for HeaderFrame {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, HeaderFrame(self))
    }

    fn __str__(&self) -> PyResult<String> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let frame: obo::HeaderFrame = self.clone_py(py).into_py(py);
        Ok(frame.to_string())
    }
}

// FIXME(@althonos)
#[pyproto]
impl PySequenceProtocol for HeaderFrame {
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
        let clause = HeaderClause::extract(elem)?;
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
            new_clauses.push(HeaderClause::extract(item?)?);
        }

        let init = PyClassInitializer::from(AbstractFrame {})
            .add_subclass(Self::new(new_clauses));
        Py::new(py, init)
    }
}
