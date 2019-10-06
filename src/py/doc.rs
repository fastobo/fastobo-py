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
use pyo3::types::PyIterator;
use pyo3::PyGCProtocol;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PySequenceProtocol;
use pyo3::PyTypeInfo;

use fastobo::ast as obo;
use fastobo::visit::VisitMut;

use crate::error::Error;
use crate::utils::AsGILRef;
use crate::utils::ClonePy;

use super::abc::AbstractFrame;
use super::header::frame::HeaderFrame;
use super::term::frame::TermFrame;
use super::typedef::frame::TypedefFrame;
use super::instance::frame::InstanceFrame;

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
    Instance(Py<InstanceFrame>),
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
            fastobo::ast::EntityFrame::Instance(frame) => {
                Py::new(py, InstanceFrame::from_py(frame, py)).map(EntityFrame::Instance)
            },
        }
        .expect("could not allocate on Python heap")
    }
}

// --- OBO document ----------------------------------------------------------

/// OboDoc(header=None, entities=None)
/// --
///
/// The abstract syntax tree corresponding to an OBO document.
///
/// Arguments:
///     header (~fastobo.header.HeaderFrame, optional): the header to use in
///         the document. If `None` is given, an empty header is used instead.
///     entities (collections.abc.Iterable, optional): an iterable of entity
///         frames, either `TermFrame`, `TypedefFrame` or `InstanceFrame`.
///
#[pyclass(module = "fastobo.doc")]
#[derive(Debug, PyList)]
pub struct OboDoc {
    header: Py<HeaderFrame>,
    entities: Vec<EntityFrame>,
}

impl OboDoc {
    pub fn with_header(header: Py<HeaderFrame>) -> Self {
        Self {
            header,
            entities: Vec::new(),
        }
    }
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
    #[new]
    fn __init__(obj: &PyRawObject, header: Option<&HeaderFrame>, entities: Option<&PyAny>) -> PyResult<()> {
        let py = obj.py();

        // extract header
        let header = header
            .map(|h| h.clone_py(py))
            .unwrap_or_else(HeaderFrame::empty);

        // create doc and extract entities
        let mut doc = OboDoc::with_header(Py::new(py, header)?);
        if let Some(any) = entities {
            for res in PyIterator::from_object(py, &any.to_object(py))? {
                doc.entities.push(EntityFrame::extract(res?)?);
            }
        }

        Ok(obj.init(doc))
    }

    /// `~fastobo.header.HeaderFrame`: the header containing ontology metadata.
    #[getter]
    fn get_header<'py>(&self, py: Python<'py>) -> PyResult<Py<HeaderFrame>> {
        Ok(self.header.clone_ref(py))
    }

    #[setter]
    fn set_header(&mut self, header: &HeaderFrame) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        self.header = Py::new(py, header.clone_py(py))?;
        Ok(())
    }

    /// compact_ids(self, /)
    /// --
    ///
    /// Create a semantically equivalent OBO document with compact identifiers.
    ///
    /// The OBO specification describes how to perform an URI decompaction
    /// using either ID spaces declared in the document header, builtin ID
    /// spaces, or a default rule using the `purl.obolibrary.org` domain.
    /// By applying the reverse operation, a new ontology can be created with
    /// compact identifiers. Some URLs may not have a compact representation
    /// if they don't correspond to any decompaction rule.
    ///
    /// Example:
    ///     >>> doc = fastobo.loads(textwrap.dedent(
    ///     ...     """
    ///     ...     idspace: MassBank http://www.massbank.jp/jsp/FwdRecord.jsp?id=
    ///     ...
    ///     ...     [Term]
    ///     ...     id: http://purl.obolibrary.org/obo/CHEBI_27958
    ///     ...     xref: http://www.massbank.jp/jsp/FwdRecord.jsp?id=EA281701
    ///     ...     """
    ///     ... ))
    ///     >>> compact_doc = doc.compact_ids()
    ///     >>> print(compact_doc[0])
    ///     [Term]
    ///     id: CHEBI:27958
    ///     xref: MassBank:EA281701
    ///     <BLANKLINE>
    ///
    /// See Also:
    ///     The `Translation of Identifiers
    ///     <http://owlcollab.github.io/oboformat/doc/obo-syntax.html#5.9>`_
    ///     section of the OBO format version 1.4 specification.
    ///
    fn compact_ids(&self) -> PyResult<Self> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut doc = obo::OboDoc::from_py(self.clone_py(py), py);
        fastobo::visit::IdCompactor::new().visit_doc(&mut doc);
        Ok(doc.into_py(py))
    }

    /// decompact_ids(self, /)
    /// --
    ///
    /// Create a semantically equivalent OBO document with IRI identifiers.
    ///
    /// The OBO specification describes how to perform an URI decompaction
    /// using either ID spaces declared in the document header, builtin ID
    /// spaces, or a default rule using the `purl.obolibrary.org` domain.
    ///
    /// Example:
    ///     >>> doc = fastobo.loads(textwrap.dedent(
    ///     ...     """
    ///     ...     idspace: MassBank http://www.massbank.jp/jsp/FwdRecord.jsp?id=
    ///     ...
    ///     ...     [Term]
    ///     ...     id: CHEBI:27958
    ///     ...     xref: MassBank:EA281701
    ///     ...     """
    ///     ... ))
    ///     >>> url_doc = doc.decompact_ids()
    ///     >>> print(url_doc[0])
    ///     [Term]
    ///     id: http://purl.obolibrary.org/obo/CHEBI_27958
    ///     xref: http://www.massbank.jp/jsp/FwdRecord.jsp?id=EA281701
    ///     <BLANKLINE>
    ///
    /// See Also:
    ///     The `Translation of Identifiers
    ///     <http://owlcollab.github.io/oboformat/doc/obo-syntax.html#5.9>`_
    ///     section of the OBO format version 1.4 specification.
    ///
    fn decompact_ids(&self) -> PyResult<Self> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let mut doc = obo::OboDoc::from_py(self.clone_py(py), py);
        fastobo::visit::IdDecompactor::new().visit_doc(&mut doc);
        Ok(doc.into_py(py))
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
        let gil = Python::acquire_gil();
        let py = gil.python();
        if index < self.entities.len() as isize {
            let item = &self.entities[index as usize];
            Ok(item.to_object(py))
        } else {
            IndexError::into("list index out of range")
        }
    }
}
