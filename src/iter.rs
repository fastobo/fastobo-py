use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error as IoError;
use std::io::Read;
use std::iter::Iterator;
use std::path::Path;
use std::path::PathBuf;
use std::ops::Deref;
use std::ops::DerefMut;

use pyo3::exceptions::TypeError;
use pyo3::exceptions::OSError;
use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3::types::PyBytes;
use pyo3::PyIterProtocol;
use pyo3::PyObjectProtocol;
use pyo3::PyErrValue;
use pyo3::AsPyPointer;
use pyo3::PyGCProtocol;

use crate::error::Error;
use crate::py::header::frame::HeaderFrame;
use crate::py::doc::EntityFrame;
use crate::pyfile::PyFileGILRead;
use crate::transmute_file_error;
use crate::utils::ClonePy;

// ---------------------------------------------------------------------------

/// A wrapper for a path on the local filesystem
struct FsFile {
    file: File,
    path: PathBuf,
}

impl FsFile {
    fn open<P: AsRef<Path>>(path: P) -> Result<Self, IoError> {
        let p = path.as_ref();
        File::open(p)
            .map(|f| FsFile {
                file: f,
                path: p.to_owned()
            })
    }
}

impl Read for FsFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        self.file.read(buf)
    }
}

// ---------------------------------------------------------------------------

trait Handle: BufRead {
    fn handle(&self) -> PyObject;
    fn into_err(&mut self) -> Option<PyErr>;
}

impl Handle for BufReader<FsFile> {
    fn handle(&self) -> PyObject {
        let gil = Python::acquire_gil();
        let py = gil.python();
        PyString::new(py, &self.get_ref().path.display().to_string())
            .to_object(py)
    }

    fn into_err(&mut self) -> Option<PyErr> {
        None
    }
}

impl Handle for BufReader<PyFileGILRead> {
    fn handle(&self) -> PyObject {
        let gil = Python::acquire_gil();
        let py = gil.python();
        self.get_ref().file().lock().unwrap().to_object(py)
    }

    fn into_err(&mut self) -> Option<PyErr> {
        self.get_mut().err_mut().take()
    }
}

// ---------------------------------------------------------------------------

// FIXME: May cause memory leaks.
/// An iterator over the frames of an OBO document.
///
/// See help(fastobo.iter) for more information.
#[pyclass(module = "fastobo")]
pub struct FrameReader {
    inner: fastobo::parser::SequentialReader<Box<dyn Handle>>,
    header: Py<HeaderFrame>,
}

impl FrameReader {
    fn new(reader: Box<dyn Handle>, ordered: bool) -> PyResult<Self> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut inner = fastobo::parser::SequentialReader::new(reader);
        inner.ordered(ordered);

        let frame = inner
            .next()
            .unwrap()
            .map_err(Error::from)?
            .into_header_frame()
            .unwrap();
        let header = PyCell::new(py, HeaderFrame::from_py(frame, py))
            .map(Py::from)?;

        Ok(Self { inner, header })
    }

    pub fn from_path<P: AsRef<Path>>(path: P, ordered: bool) -> PyResult<Self> {
        let p = path.as_ref();
        match FsFile::open(p) {
            Ok(inner) => Self::new(Box::new(BufReader::new(inner)), ordered),
            Err(e) => Error::from(e).with_path(p.display().to_string()).into(),
        }
    }

    pub fn from_handle(obj: &PyAny, ordered: bool) -> PyResult<Self> {
        match PyFileGILRead::from_ref(obj) {
            Ok(inner) => Self::new(Box::new(BufReader::new(inner)), ordered),
            Err(e) => Err(e),
        }
    }
}

#[pymethods]
impl FrameReader {
    fn header<'py>(&self, py: Python<'py>) -> Py<HeaderFrame> {
        self.header.clone_py(py)
    }
}

#[pyproto]
impl PyObjectProtocol for FrameReader {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let fmt = PyString::new(py, "fastobo.iter({!r})").to_object(py);
        fmt.call_method1(py, "format", (&self.inner.as_ref().handle(),))
    }
}

#[pyproto]
impl PyIterProtocol for FrameReader {
    fn __iter__(slf: PyRefMut<'p, Self>) -> PyResult<PyRefMut<'p, Self>> {
        Ok(slf)
    }

    fn __next__(mut slf: PyRefMut<'p, Self>) -> PyResult<Option<EntityFrame>> {
        match slf.deref_mut().inner.next() {
            None => Ok(None),
            Some(Ok(frame)) => {
                let gil = Python::acquire_gil();
                let entity = frame.into_entity_frame().unwrap();
                Ok(Some(EntityFrame::from_py(entity, gil.python())))
            },
            Some(Err(e)) => slf
                .deref_mut()
                .inner
                .as_mut()
                .into_err()
                .unwrap_or_else(|| Error::from(e).into())
                .into(),
        }
    }
}
