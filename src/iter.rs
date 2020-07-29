use std::convert::TryFrom;
use std::convert::TryInto;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Error as IoError;
use std::io::Read;
use std::iter::Iterator;
use std::path::Path;
use std::path::PathBuf;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::ops::DerefMut;

use pyo3::exceptions::OSError;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::prelude::*;
use pyo3::types::PyString;
use pyo3::types::PyBytes;
use pyo3::PyIterProtocol;
use pyo3::PyObjectProtocol;
use pyo3::PyErrValue;
use pyo3::AsPyPointer;
use pyo3::PyGCProtocol;

use fastobo::parser::Parser;
use fastobo::parser::ThreadedParser;
use fastobo::parser::SequentialParser;

use crate::error::Error;
use crate::py::header::frame::HeaderFrame;
use crate::py::doc::EntityFrame;
use crate::pyfile::PyFileGILRead;
use crate::transmute_file_error;
use crate::utils::ClonePy;

// ---------------------------------------------------------------------------

/// An enum providing `Read` for either Python file-handles or filesystem files.
pub enum Handle {
    FsFile(File, PathBuf),
    PyFile(PyFileGILRead),
}

impl Handle {
    fn handle(&self) -> PyObject {
        let gil = Python::acquire_gil();
        let py = gil.python();
        match self {
            Handle::FsFile(_, path) =>  path.display().to_string().to_object(py),
            Handle::PyFile(f) => {
                f.file().lock().unwrap().to_object(py)
            }
        }
    }
}

impl TryFrom<PathBuf> for Handle {
    type Error = std::io::Error;
    fn try_from(p: PathBuf) -> Result<Self, Self::Error> {
        let file = File::open(&p)?;
        Ok(Handle::FsFile(file, p))
    }
}

impl Read for Handle {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        match self {
            Handle::FsFile(f, _) => f.read(buf),
            Handle::PyFile(f) => f.read(buf),
        }
    }
}

// ---------------------------------------------------------------------------

/// An enum providing the same API for the sequential and threaded parsers from `fastobo`.
pub enum InternalParser<B: BufRead> {
    Sequential(SequentialParser<B>),
    Threaded(ThreadedParser<B>),
}

impl<B: BufRead> InternalParser<B> {
    pub fn with_thread_count(stream: B, n: i16) -> Result<Self, PyErr> {
        match n {
            0 => Ok(InternalParser::Threaded(ThreadedParser::new(stream))),
            1 => Ok(InternalParser::Sequential(Parser::new(stream))),
            n if n < 0 => {
                ValueError::into("threads count must be positive or null")
            },
            n => {
                let t = std::num::NonZeroUsize::new(n as usize).unwrap();
                Ok(InternalParser::Threaded(ThreadedParser::with_threads(stream, t)))
            },
        }
    }

    pub fn try_into_doc(&mut self) -> Result<fastobo::ast::OboDoc, fastobo::error::Error> {
        match self {
            InternalParser::Sequential(parser) => parser.try_into(),
            InternalParser::Threaded(parser) => parser.try_into(),
        }
    }
}

impl<B: BufRead> AsMut<B> for InternalParser<B> {
    fn as_mut(&mut self) -> &mut B {
        match self {
            InternalParser::Sequential(parser) => parser.as_mut(),
            InternalParser::Threaded(parser) => parser.as_mut(),
        }
    }
}

impl<B: BufRead> AsRef<B> for InternalParser<B> {
    fn as_ref(&self) -> &B {
        match self {
            InternalParser::Sequential(parser) => parser.as_ref(),
            InternalParser::Threaded(parser) => parser.as_ref(),
        }
    }
}

impl<B: BufRead> From<B> for InternalParser<B> {
    fn from(stream: B) -> Self {
        Self::Sequential(SequentialParser::from(stream))
    }
}

impl<B: BufRead> Iterator for InternalParser<B> {
    type Item = fastobo::error::Result<fastobo::ast::Frame>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            InternalParser::Sequential(parser) => parser.next(),
            InternalParser::Threaded(parser) => parser.next(),
        }
    }
}

impl<B: BufRead> Parser<B> for InternalParser<B> {
    fn new(stream: B) -> Self {
        InternalParser::Sequential(SequentialParser::new(stream))
    }

    fn with_threads(stream: B, threads: NonZeroUsize) -> Self {
        if threads.get() == 1 {
            Self::new(stream)
        } else {
            InternalParser::Threaded(ThreadedParser::with_threads(stream, threads))
        }
    }

    fn ordered(&mut self, ordered: bool) -> &mut Self {
        match self {
            InternalParser::Sequential(parser) => {
                parser.ordered(ordered);
            }
            InternalParser::Threaded(parser) => {
                parser.ordered(ordered);
            }
        };
        self
    }

    fn into_inner(self) -> B {
        match self {
            InternalParser::Sequential(parser) => parser.into_inner(),
            InternalParser::Threaded(parser) => parser.into_inner(),
        }
    }
}

// ---------------------------------------------------------------------------

// FIXME: May cause memory leaks?
/// An iterator over the frames of an OBO document.
///
/// See help(fastobo.iter) for more information.
#[pyclass(module = "fastobo")]
pub struct FrameReader {
    inner: InternalParser<BufReader<Handle>>,
    header: Py<HeaderFrame>,
}

impl FrameReader {
    fn new(handle: BufReader<Handle>, ordered: bool, threads: i16) -> PyResult<Self> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let mut inner = InternalParser::with_thread_count(handle, threads)?;
        inner.ordered(ordered);
        let frame = inner
            .next()
            .unwrap()
            .map_err(Error::from)?
            .into_header_frame()
            .unwrap();
        let header = Py::new(py, HeaderFrame::from_py(frame, py))?;

        Ok(Self { inner, header })
    }

    pub fn from_path<P: AsRef<Path>>(path: P, ordered: bool, threads: i16) -> PyResult<Self> {
        let p = path.as_ref();
        match Handle::try_from(p.to_owned()) {
            Ok(inner) => Self::new(BufReader::new(inner), ordered, threads),
            Err(e) => Error::from(e).with_path(p.display().to_string()).into(),
        }
    }

    pub fn from_handle(obj: &PyAny, ordered: bool, threads: i16) -> PyResult<Self> {
        match PyFileGILRead::from_ref(obj).map(Handle::PyFile) {
            Ok(inner) => Self::new(BufReader::new(inner), ordered, threads),
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
        fmt.call_method1(py, "format", (&self.inner.as_ref().get_ref().handle(),))
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
            Some(Err(e)) => {
                let gil = Python::acquire_gil();
                let py = gil.python();
                if PyErr::occurred(py) {
                    Err(PyErr::fetch(py))
                } else {
                    Err(Error::from(e).into())
                }
            }
        }
    }
}
