use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fmt::Write;
use std::str::FromStr;

use pyo3::exceptions::PyIndexError;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyIterator;
use pyo3::types::PyString;
use pyo3::AsPyPointer;
use pyo3::PyTypeInfo;

use fastobo::ast;

use super::super::abc::AbstractEntityFrame;
use super::super::id::Ident;
use super::clause::BaseTermClause;
use super::clause::TermClause;
use crate::utils::AbstractClass;
use crate::utils::ClonePy;
use crate::utils::EqPy;
use crate::utils::FinalClass;
use crate::utils::IntoPy;

#[pyclass(extends=AbstractEntityFrame, module="fastobo.term")]
#[derive(Debug, FinalClass, EqPy)]
#[base(AbstractEntityFrame)]
pub struct TermFrame {
    #[pyo3(set)]
    id: Ident,
    clauses: Vec<TermClause>,
}

impl TermFrame {
    pub fn new(id: Ident) -> Self {
        Self::with_clauses(id, Vec::new())
    }

    pub fn with_clauses(id: Ident, clauses: Vec<TermClause>) -> Self {
        Self { id, clauses }
    }
}

impl ClonePy for TermFrame {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            id: self.id.clone_py(py),
            clauses: self.clauses.clone_py(py),
        }
    }
}

impl Display for TermFrame {
    // FIXME: no clone
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let frame: fastobo::ast::TermFrame = Python::with_gil(|py| self.clone_py(py).into_py(py));
        frame.fmt(f)
    }
}

impl IntoPy<TermFrame> for fastobo::ast::TermFrame {
    fn into_py(self, py: Python) -> TermFrame {
        TermFrame::with_clauses(
            self.id().as_ref().clone().into_py(py),
            self.into_iter()
                .map(|line| line.into_inner().into_py(py))
                .collect(),
        )
    }
}

impl IntoPy<fastobo::ast::TermFrame> for TermFrame {
    fn into_py(self, py: Python) -> fastobo::ast::TermFrame {
        fastobo::ast::TermFrame::with_clauses(
            fastobo::ast::ClassIdent::new(self.id.into_py(py)),
            self.clauses
                .iter()
                .map(|f| f.clone_py(py).into_py(py))
                .map(|c| fastobo::ast::Line::new().and_inner(c))
                .collect(),
        )
    }
}

impl IntoPy<fastobo::ast::EntityFrame> for TermFrame {
    fn into_py(self, py: Python) -> fastobo::ast::EntityFrame {
        let frame: fastobo::ast::TermFrame = self.into_py(py);
        frame.into()
    }
}

#[listlike(field = "clauses", type = "TermClause")]
#[pymethods]
impl TermFrame {
    // FIXME: should accept any iterable.
    #[new]
    #[pyo3(signature = (id, clauses = None))]
    fn __init__<'py>(
        id: Ident,
        clauses: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<PyClassInitializer<Self>> {
        if let Some(clauses) = clauses {
            match clauses.extract() {
                Ok(c) => Ok(Self::with_clauses(id, c).into()),
                Err(_) => Err(PyTypeError::new_err("Expected list of `TermClause`")),
            }
        } else {
            Ok(Self::new(id).into())
        }
    }

    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, TermFrame(self.id))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.clauses.len())
    }

    fn __getitem__<'py>(&self, index: isize) -> PyResult<Py<BaseTermClause>> {
        if index < self.clauses.len() as isize {
            let item = &self.clauses[index as usize];
            Python::with_gil(|py| Ok(item.into_pyobject(py)?.unbind()))
        } else {
            Err(PyIndexError::new_err("list index out of range"))
        }
    }

    fn __setitem__<'py>(&mut self, index: isize, elem: &Bound<'py, PyAny>) -> PyResult<()> {
        if index as usize > self.clauses.len() {
            return Err(PyIndexError::new_err("list index out of range"));
        }
        let clause = TermClause::extract_bound(elem)?;
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

    fn __concat__<'py>(&self, other: &Bound<'py, PyAny>) -> PyResult<Py<Self>> {
        let py = other.py();

        let iterator = PyIterator::from_object(other)?;
        let mut new_clauses = self.clauses.clone_py(py);
        for item in iterator {
            new_clauses.push(TermClause::extract_bound(&item?)?);
        }

        Py::new(py, Self::with_clauses(self.id.clone_py(py), new_clauses))
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the term frame.
    fn get_id(&self) -> PyResult<&Ident> {
        Ok(&self.id)
    }
}
