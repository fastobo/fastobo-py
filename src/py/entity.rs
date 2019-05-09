use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::rc::Rc;
use std::str::FromStr;
use std::string::ToString;

use pyo3::prelude::*;
use pyo3::PyTypeInfo;
use pyo3::PyNativeType;
use pyo3::types::PyAny;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::exceptions::RuntimeError;
use pyo3::exceptions::IndexError;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::PySequenceProtocol;
use pyo3::PyGCProtocol;
use pyo3::PyObjectProtocol;
use pyo3::gc::PyTraverseError;
use pyo3::class::gc::PyVisit;

use fastobo::ast as obo;

use crate::utils::AsGILRef;
use crate::utils::ClonePy;
use crate::error::Error;
use crate::pyfile::PyFile;

use super::header::frame::HeaderFrame;
use super::term::frame::TermFrame;
use super::typedef::frame::TypedefFrame;

// --- Conversion Wrapper ----------------------------------------------------

#[derive(ClonePy, Debug, PartialEq, PyWrapper)]
#[wraps(BaseEntityFrame)]
pub enum EntityFrame {
    Term(Py<TermFrame>),
    Typedef(Py<TypedefFrame>),
}

impl FromPy<fastobo::ast::EntityFrame> for EntityFrame {
    fn from_py(frame: fastobo::ast::EntityFrame, py: Python) -> Self {
        match frame {
            fastobo::ast::EntityFrame::Term(frame) =>
                Py::new(py, TermFrame::from_py(frame, py))
                    .map(EntityFrame::Term),
            fastobo::ast::EntityFrame::Typedef(frame) =>
                Py::new(py, TypedefFrame::from_py(frame, py))
                    .map(EntityFrame::Typedef),
            _ => unimplemented!(),
        }.expect("could not allocate on Python heap")
    }
}

// ---

#[pyclass(subclass)]
pub struct BaseEntityFrame {}
