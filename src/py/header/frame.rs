use std::iter::FromIterator;
use std::iter::IntoIterator;

use fastobo::ast as obo;
use pyo3::class::gc::PyVisit;
use pyo3::exceptions::IndexError;
use pyo3::exceptions::RuntimeError;
use pyo3::exceptions::TypeError;
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

#[pyclass(extends=AbstractFrame, module="fastobo.header")]
#[derive(Debug, PyList)]
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

impl FromPy<obo::HeaderFrame> for HeaderFrame {
    fn from_py(frame: fastobo::ast::HeaderFrame, py: Python) -> Self {
        frame
            .into_iter()
            .map(|clause| HeaderClause::from_py(clause, py))
            .collect()
    }
}

impl FromPy<HeaderFrame> for obo::HeaderFrame {
    fn from_py(frame: HeaderFrame, py: Python) -> Self {
        frame
            .clauses
            .into_iter()
            .map(|clause| obo::HeaderClause::from_py(clause, py))
            .collect()
    }
}

impl ToPyObject for HeaderFrame {
    fn to_object(&self, py: Python) -> PyObject {
        IntoPy::into_py(PyList::new(py, &self.clauses), py)
    }
}

#[pymethods]
impl HeaderFrame {
    #[new]
    pub fn __init__(obj: &PyRawObject, clauses: Option<&PyAny>) -> PyResult<()> {
        if let Some(c) = clauses {
            let mut vec = Vec::new();
            for item in PyIterator::from_object(c.py(), c)? {
                vec.push(HeaderClause::extract(item?)?);
            }
            Ok(obj.init(Self::new(vec)))
        } else {
            Ok(obj.init(Self::new(Vec::new())))
        }
    }
}

#[pyproto]
impl PyObjectProtocol for HeaderFrame {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let fmt = PyString::new(py, "HeaderFrame({!r})").to_object(py);
        fmt.call_method1(py, "format", (self.to_object(py),))
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
            IndexError::into("list index out of range")
        }
    }
    fn __setitem__(&mut self, index: isize, elem: &PyAny) -> PyResult<()> {
        if index as usize > self.clauses.len() {
            return IndexError::into("list index out of range");
        }
        let clause = HeaderClause::extract(elem)?;
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
            new_clauses.push(HeaderClause::extract(item?)?);
        }

        Ok(Self::new(new_clauses))
    }
}
