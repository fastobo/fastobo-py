use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::mem::replace;
use std::rc::Rc;
use std::str::FromStr;
use std::string::ToString;

use pyo3::class::gc::PyVisit;
use pyo3::exceptions::IndexError;
use pyo3::exceptions::RuntimeError;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::gc::PyTraverseError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::PyGCProtocol;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PySequenceProtocol;
use pyo3::PyTypeInfo;

use fastobo::ast as obo;

use crate::error::Error;
use crate::pyfile::PyFile;
use crate::utils::AsGILRef;
use crate::utils::ClonePy;

use super::abc::AbstractFrame;
use super::header::frame::HeaderFrame;
use super::term::frame::TermFrame;
use super::typedef::frame::TypedefFrame;

// --- Module export ---------------------------------------------------------

#[pymodule(doc)]
fn module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<self::OboDoc>()?;
    m.add("__name__", "fastobo.doc")?;
    Ok(())
}

// --- Conversion Wrapper ----------------------------------------------------

#[derive(ClonePy, Debug, PartialEq, PyWrapper)]
#[wraps(AbstractFrame)]
pub enum EntityFrame {
    Term(Py<TermFrame>),
    Typedef(Py<TypedefFrame>),
}

impl FromPy<fastobo::ast::EntityFrame> for EntityFrame {
    fn from_py(frame: fastobo::ast::EntityFrame, py: Python) -> Self {
        match frame {
            fastobo::ast::EntityFrame::Term(frame) => {
                Py::new(py, TermFrame::from_py(frame, py)).map(EntityFrame::Term)
            }
            fastobo::ast::EntityFrame::Typedef(frame) => {
                Py::new(py, TypedefFrame::from_py(frame, py)).map(EntityFrame::Typedef)
            }
            _ => unimplemented!(),
        }
        .expect("could not allocate on Python heap")
    }
}

// --- OBO document ----------------------------------------------------------

/// The abstract syntax tree corresponding to an OBO document.
#[pyclass(subclass, module = "fastobo.doc")]
#[derive(Debug)]
pub struct OboDoc {
    header: Py<HeaderFrame>,
    entities: Vec<EntityFrame>,
}

impl ClonePy for OboDoc {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            header: self.header.clone_py(py),
            entities: self.entities.clone_py(py),
        }
    }
}

impl Display for OboDoc {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::OboDoc::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<obo::OboDoc> for OboDoc {
    fn from_py(mut doc: fastobo::ast::OboDoc, py: Python) -> Self {
        // Take ownership of header and entities w/o reallocation or clone.
        let header = replace(doc.header_mut(), Default::default()).into_py(py);
        let entities = replace(doc.entities_mut(), Default::default())
            .into_iter()
            .map(|frame| EntityFrame::from_py(frame, py))
            .collect();

        Self {
            header: Py::new(py, header).expect("could not move header to Python heap"),
            entities,
        }
    }
}

impl FromPy<OboDoc> for fastobo::ast::OboDoc {
    fn from_py(doc: OboDoc, py: Python) -> Self {
        let header: HeaderFrame = doc.header.as_ref(py).clone_py(py);
        doc.entities
            .iter()
            .map(|frame| fastobo::ast::EntityFrame::from_py(frame, py))
            .collect::<fastobo::ast::OboDoc>()
            .and_header(header.into_py(py))
    }
}

#[pymethods]
impl OboDoc {
    #[getter]
    fn get_header(&self) -> PyResult<Py<HeaderFrame>> {
        let py = unsafe { Python::assume_gil_acquired() };
        Ok(self.header.clone_ref(py))
    }

    #[setter]
    fn set_header(&mut self, header: &HeaderFrame) -> PyResult<()> {
        let py = unsafe { Python::assume_gil_acquired() };
        self.header = Py::new(py, header.clone_py(py))?;
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for OboDoc {
    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }
}

#[pyproto]
impl PySequenceProtocol for OboDoc {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.entities.len())
    }

    fn __getitem__(&self, index: isize) -> PyResult<PyObject> {
        let py = unsafe { Python::assume_gil_acquired() };
        if index < self.entities.len() as isize {
            let item = &self.entities[index as usize];
            Ok(item.to_object(py))
        } else {
            IndexError::into("list index out of range")
        }
    }
}
