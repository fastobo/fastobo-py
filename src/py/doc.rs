use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::mem::take;
use std::rc::Rc;
use std::str::FromStr;
use std::string::ToString;

use pyo3::class::gc::PyVisit;
use pyo3::exceptions::PyIndexError;
use pyo3::gc::PyTraverseError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyIterator;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::PyTypeInfo;

use fastobo::ast as obo;
use fastobo::visit::VisitMut;

use crate::error::Error;
use crate::utils::ClonePy;
use crate::utils::EqPy;
use crate::utils::IntoPy;

use super::abc::AbstractClause;
use super::abc::AbstractFrame;
use super::header::frame::HeaderFrame;
use super::instance::frame::InstanceFrame;
use super::term::frame::TermFrame;
use super::typedef::frame::TypedefFrame;

// --- Module export ---------------------------------------------------------

#[pymodule]
#[pyo3(name = "doc")]
pub fn init<'py>(_py: Python<'py>, m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add_class::<self::OboDoc>()?;
    m.add("__name__", "fastobo.doc")?;
    Ok(())
}

// --- Conversion Wrapper ----------------------------------------------------

#[derive(ClonePy, Debug, PyWrapper, EqPy)]
#[wraps(AbstractFrame)]
pub enum EntityFrame {
    Term(Py<TermFrame>),
    Typedef(Py<TypedefFrame>),
    Instance(Py<InstanceFrame>),
}

impl IntoPy<EntityFrame> for fastobo::ast::EntityFrame {
    fn into_py(self, py: Python) -> EntityFrame {
        match self {
            fastobo::ast::EntityFrame::Term(frame) => {
                Py::new(py, frame.into_py(py)).map(EntityFrame::Term)
            }
            fastobo::ast::EntityFrame::Typedef(frame) => {
                Py::new(py, frame.into_py(py)).map(EntityFrame::Typedef)
            }
            fastobo::ast::EntityFrame::Instance(frame) => {
                Py::new(py, frame.into_py(py)).map(EntityFrame::Instance)
            }
        }
        .expect("could not allocate on Python heap")
    }
}

impl IntoPy<fastobo::ast::EntityFrame> for EntityFrame {
    fn into_py(self, py: Python) -> fastobo::ast::EntityFrame {
        match self {
            EntityFrame::Term(t) => t.borrow(py).clone_py(py).into_py(py),
            EntityFrame::Typedef(t) => t.borrow(py).clone_py(py).into_py(py),
            EntityFrame::Instance(i) => i.borrow(py).clone_py(py).into_py(py),
        }
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
#[derive(Debug, EqPy)]
pub struct OboDoc {
    #[pyo3(get, set)]
    /// `~fastobo.header.HeaderFrame`: the header containing ontology metadata.
    header: Py<HeaderFrame>,
    entities: Vec<EntityFrame>,
}

impl OboDoc {
    /// Create an empty `OboDoc` with only the given header frame.
    pub fn new(header: Py<HeaderFrame>) -> Self {
        Self::with_entities(header, Vec::new())
    }

    pub fn with_entities(header: Py<HeaderFrame>, entities: Vec<EntityFrame>) -> Self {
        Self { header, entities }
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
        let doc: fastobo::ast::OboDoc = Python::with_gil(|py| self.clone_py(py).into_py(py));
        doc.fmt(f)
    }
}

impl IntoPy<OboDoc> for fastobo::ast::OboDoc {
    fn into_py(mut self, py: Python) -> OboDoc {
        // Take ownership of header and entities w/o reallocation or clone.
        let h: HeaderFrame = take(self.header_mut()).into_py(py);
        let entities = take(self.entities_mut())
            .into_iter()
            .map(|frame| <fastobo::ast::EntityFrame as IntoPy<EntityFrame>>::into_py(frame, py))
            .collect();

        let header = Py::new(py, h).expect("could not move header to Python heap");

        OboDoc { header, entities }
    }
}

impl IntoPy<fastobo::ast::OboDoc> for OboDoc {
    fn into_py(self, py: Python) -> fastobo::ast::OboDoc {
        let header: HeaderFrame = self.header.bind(py).borrow().clone_py(py);
        self.entities
            .iter()
            .map(|frame| {
                <EntityFrame as IntoPy<fastobo::ast::EntityFrame>>::into_py(frame.clone_py(py), py)
            })
            .collect::<fastobo::ast::OboDoc>()
            .and_header(header.into_py(py))
    }
}

#[listlike(field = "entities", type = "EntityFrame")]
#[pymethods]
impl OboDoc {
    #[new]
    #[pyo3(signature = (header = None, entities = None))]
    fn __init__<'py>(
        header: Option<&HeaderFrame>,
        entities: Option<&Bound<'py, PyAny>>,
    ) -> PyResult<Self> {
        Python::with_gil(|py| {
            // extract header
            let header = header
                .map(|h| h.clone_py(py))
                .unwrap_or_else(HeaderFrame::empty);
            // create doc and extract entities
            let mut doc = OboDoc::new(Py::new(py, header)?);
            if let Some(any) = entities {
                for res in PyIterator::from_object(&any)? {
                    doc.entities.push(EntityFrame::extract_bound(&res?)?);
                }
            }
            Ok(doc)
        })
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.entities.len())
    }

    fn __getitem__<'py>(
        &self,
        py: Python<'py>,
        index: isize,
    ) -> PyResult<Bound<'py, AbstractFrame>> {
        if index < self.entities.len() as isize {
            let item = &self.entities[index as usize];
            item.into_pyobject(py)
        } else {
            Err(PyIndexError::new_err("list index out of range"))
        }
    }

    #[getter]
    fn get_header<'py>(&self, py: Python<'py>) -> PyResult<Py<HeaderFrame>> {
        Ok(self.header.clone_ref(py))
    }

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
    #[pyo3(text_signature = "(self, /)")]
    fn compact_ids(&self) -> PyResult<Self> {
        Python::with_gil(|py| {
            let mut doc: obo::OboDoc = self.clone_py(py).into_py(py);
            py.allow_threads(|| fastobo::visit::IdCompactor::new().visit_doc(&mut doc));
            Ok(doc.into_py(py))
        })
    }

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
    #[pyo3(text_signature = "(self, /)")]
    fn decompact_ids(&self) -> PyResult<Self> {
        Python::with_gil(|py| {
            let mut doc: obo::OboDoc = self.clone_py(py).into_py(py);
            py.allow_threads(|| fastobo::visit::IdDecompactor::new().visit_doc(&mut doc));
            Ok(doc.into_py(py))
        })
    }
}
