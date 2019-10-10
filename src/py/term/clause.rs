use std::cmp::Ordering;
use std::cmp::Ord;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fmt::Write;
use std::str::FromStr;

use pyo3::class::basic::CompareOp;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyDateAccess;
use pyo3::types::PyDateTime;
use pyo3::types::PyString;
use pyo3::types::PyTimeAccess;
use pyo3::types::PyTzInfo;
use pyo3::AsPyPointer;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PyTypeInfo;

use fastobo::ast;
use fastobo::share::Cow;
use fastobo::share::Redeem;
use fastobo::share::Share;

use super::super::abc::AbstractEntityClause;
use super::super::id::Ident;
use super::super::pv::PropertyValue;
use super::super::syn::Synonym;
use super::super::xref::Xref;
use super::super::xref::XrefList;
use crate::date::datetime_to_isodate;
use crate::date::isodate_to_datetime;
use crate::utils::AsGILRef;
use crate::utils::ClonePy;

// --- Conversion Wrapper ----------------------------------------------------

#[derive(ClonePy, Debug, PartialEq, PyWrapper)]
#[wraps(BaseTermClause)]
pub enum TermClause {
    IsAnonymous(Py<IsAnonymousClause>),
    Name(Py<NameClause>),
    Namespace(Py<NamespaceClause>),
    AltId(Py<AltIdClause>),
    Def(Py<DefClause>),
    Comment(Py<CommentClause>),
    Subset(Py<SubsetClause>),
    Synonym(Py<SynonymClause>),
    Xref(Py<XrefClause>),
    Builtin(Py<BuiltinClause>),
    PropertyValue(Py<PropertyValueClause>),
    IsA(Py<IsAClause>),
    IntersectionOf(Py<IntersectionOfClause>),
    UnionOf(Py<UnionOfClause>),
    EquivalentTo(Py<EquivalentToClause>),
    DisjointFrom(Py<DisjointFromClause>),
    Relationship(Py<RelationshipClause>),
    IsObsolete(Py<IsObsoleteClause>),
    ReplacedBy(Py<ReplacedByClause>),
    Consider(Py<ConsiderClause>),
    CreatedBy(Py<CreatedByClause>),
    CreationDate(Py<CreationDateClause>),
}

impl FromPy<fastobo::ast::TermClause> for TermClause {
    fn from_py(clause: fastobo::ast::TermClause, py: Python) -> Self {
        use fastobo::ast::TermClause::*;
        match clause {
            IsAnonymous(b) => {
                Py::new(py, IsAnonymousClause::new(py, b)).map(TermClause::IsAnonymous)
            }
            Name(n) => Py::new(py, NameClause::new(py, n)).map(TermClause::Name),
            Namespace(ns) => Py::new(py, NamespaceClause::new(py, ns)).map(TermClause::Namespace),
            AltId(id) => Py::new(py, AltIdClause::new(py, id)).map(TermClause::AltId),
            Def(desc, xrefs) => Py::new(py, DefClause::new(py, desc, xrefs)).map(TermClause::Def),
            Comment(c) => Py::new(py, CommentClause::new(py, c)).map(TermClause::Comment),
            Subset(s) => Py::new(py, SubsetClause::new(py, s)).map(TermClause::Subset),
            Synonym(s) => Py::new(py, SynonymClause::new(py, s)).map(TermClause::Synonym),
            Xref(x) => Py::new(py, XrefClause::new(py, x)).map(TermClause::Xref),
            Builtin(b) => Py::new(py, BuiltinClause::new(py, b)).map(TermClause::Builtin),
            PropertyValue(pv) => {
                Py::new(py, PropertyValueClause::new(py, pv)).map(TermClause::PropertyValue)
            }
            IsA(id) => Py::new(py, IsAClause::new(py, id)).map(TermClause::IsA),
            IntersectionOf(r, cls) => {
                Py::new(py, IntersectionOfClause::new(py, r, cls)).map(TermClause::IntersectionOf)
            }
            UnionOf(cls) => Py::new(py, UnionOfClause::new(py, cls)).map(TermClause::UnionOf),
            EquivalentTo(cls) => {
                Py::new(py, EquivalentToClause::new(py, cls)).map(TermClause::EquivalentTo)
            }
            DisjointFrom(cls) => {
                Py::new(py, DisjointFromClause::new(py, cls)).map(TermClause::DisjointFrom)
            }
            Relationship(r, id) => {
                Py::new(py, RelationshipClause::new(py, r, id)).map(TermClause::Relationship)
            }
            IsObsolete(b) => Py::new(py, IsObsoleteClause::new(py, b)).map(TermClause::IsObsolete),
            ReplacedBy(id) => {
                Py::new(py, ReplacedByClause::new(py, id)).map(TermClause::ReplacedBy)
            }
            Consider(id) => Py::new(py, ConsiderClause::new(py, id)).map(TermClause::Consider),
            CreatedBy(name) => {
                Py::new(py, CreatedByClause::new(py, name)).map(TermClause::CreatedBy)
            }
            CreationDate(dt) => {
                Py::new(py, CreationDateClause::new(py, dt)).map(TermClause::CreationDate)
            }
        }
        .expect("could not allocate memory for `TermClause` in Python heap")
    }
}

// --- Base ------------------------------------------------------------------

/// A term clause, appearing in an OBO term frame.
#[pyclass(extends=AbstractEntityClause, module="fastobo.term")]
pub struct BaseTermClause {}

// --- IsAnonymous -----------------------------------------------------------

/// IsAnonymousClause(anonymous)
/// --
///
/// A clause declaring whether or not the current term has an anonymous id.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsAnonymousClause {
    #[pyo3(get, set)]
    anonymous: bool,
}

impl IsAnonymousClause {
    pub fn new(_py: Python, anonymous: bool) -> Self {
        Self { anonymous }
    }
}

impl Display for IsAnonymousClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TermClause::from(self.clone()).fmt(f)
    }
}

impl From<IsAnonymousClause> for fastobo::ast::TermClause {
    fn from(clause: IsAnonymousClause) -> Self {
        fastobo::ast::TermClause::IsAnonymous(clause.anonymous)
    }
}

impl FromPy<IsAnonymousClause> for fastobo::ast::TermClause {
    fn from_py(clause: IsAnonymousClause, _py: Python) -> Self {
        Self::from(clause)
    }
}

#[pymethods]
impl IsAnonymousClause {
    #[new]
    fn __init__(obj: &PyRawObject, anonymous: bool) {
        obj.init(Self::new(obj.py(), anonymous));
    }
}

impl_raw_tag!(IsAnonymousClause, "is_anonymous");
impl_raw_value!(IsAnonymousClause, "{}", self.anonymous);

#[pyproto]
impl PyObjectProtocol for IsAnonymousClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsAnonymousClause(self.anonymous))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.anonymous)
    }
}

// --- Name ------------------------------------------------------------------

/// NameClause(name)
/// --
///
/// A term clause declaring the human-readable name of this term.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Clone, ClonePy, Debug)]
pub struct NameClause {
    name: fastobo::ast::UnquotedString,
}

impl NameClause {
    pub fn new(_py: Python, name: fastobo::ast::UnquotedString) -> Self {
        Self { name }
    }
}

impl Display for NameClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TermClause::from(self.clone()).fmt(f)
    }
}

impl From<NameClause> for fastobo::ast::TermClause {
    fn from(clause: NameClause) -> Self {
        fastobo::ast::TermClause::Name(clause.name)
    }
}

impl FromPy<NameClause> for fastobo::ast::TermClause {
    fn from_py(clause: NameClause, _py: Python) -> Self {
        Self::from(clause)
    }
}

#[pymethods]
impl NameClause {
    #[new]
    fn __init__(obj: &PyRawObject, name: String) {
        obj.init(Self::new(obj.py(), fastobo::ast::UnquotedString::new(name)));
    }

    /// `str`: the name of the current term.
    #[getter]
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    #[setter]
    fn set_name(&mut self, name: String) -> PyResult<()> {
        self.name = fastobo::ast::UnquotedString::new(name);
        Ok(())
    }
}

impl_raw_tag!(NameClause, "name");
impl_raw_value!(NameClause, "{}", self.name);

#[pyproto]
impl PyObjectProtocol for NameClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, NameClause(self.name))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.name)
    }
}

// --- Namespace -------------------------------------------------------------

/// NamespaceClause(namespace)
/// --
///
/// A term clause declaring the namespace of this term.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct NamespaceClause {
    #[pyo3(set)]
    namespace: Ident,
}

impl NamespaceClause {
    pub fn new<I>(py: Python, ns: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            namespace: ns.into_py(py),
        }
    }
}

impl ClonePy for NamespaceClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            namespace: self.namespace.clone_py(py),
        }
    }
}

impl Display for NamespaceClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<NamespaceClause> for fastobo::ast::TermClause {
    fn from_py(clause: NamespaceClause, py: Python) -> Self {
        let ns = fastobo::ast::NamespaceIdent::from_py(clause.namespace, py);
        fastobo::ast::TermClause::Namespace(ns)
    }
}

#[pymethods]
impl NamespaceClause {
    #[new]
    fn __init__(obj: &PyRawObject, namespace: Ident) {
        obj.init(Self::new(obj.py(), namespace));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the ID of the namespace this term is part of.
    fn get_namespace(&self) -> &Ident {
        &self.namespace
    }
}

impl_raw_tag!(NamespaceClause, "namespace");
impl_raw_value!(NamespaceClause, "{}", self.namespace);

#[pyproto]
impl PyObjectProtocol for NamespaceClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let r = self.namespace.to_object(py).call_method0(py, "__repr__")?;
        let fmt = PyString::new(py, "NamespaceClause({!r})").to_object(py);
        fmt.call_method1(py, "format", (&self.namespace,))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.namespace)
    }
}

// --- AltId -----------------------------------------------------------------

/// AltIdClause(alt_id)
/// --
///
/// A clause defines an alternate id for this term.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct AltIdClause {
    #[pyo3(set)]
    alt_id: Ident,
}

impl AltIdClause {
    pub fn new<I>(py: Python, id: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            alt_id: id.into_py(py),
        }
    }
}

impl ClonePy for AltIdClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            alt_id: self.alt_id.clone_py(py),
        }
    }
}

impl Display for AltIdClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<AltIdClause> for fastobo::ast::TermClause {
    fn from_py(clause: AltIdClause, py: Python) -> Self {
        fastobo::ast::TermClause::AltId(clause.alt_id.into_py(py))
    }
}

#[pymethods]
impl AltIdClause {
    #[new]
    fn __init__(obj: &PyRawObject, alt_id: Ident) {
        obj.init(Self::new(obj.py(), alt_id));
    }

    #[getter]
    /// `~fastobo.id.Ident`: an alternative ID used to refer to this term.
    fn get_alt_id(&self) -> &Ident {
        &self.alt_id
    }
}

impl_raw_tag!(AltIdClause, "alt_id");
impl_raw_value!(AltIdClause, "{}", self.alt_id);

#[pyproto]
impl PyObjectProtocol for AltIdClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, AltIdClause(self.alt_id))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.alt_id)
    }
}

// --- Def -------------------------------------------------------------------

/// DefClause(definition, xrefs=None)
/// --
///
/// A clause giving a human-readable definition of the term.
///
/// Arguments:
///     definition (str): The human-readable textual definition of the
///         current term.
///     xrefs (~typing.Iterable[~fastobo.xref.Xref], optional): An iterable
///         of database cross-references describing the origin of the
///         definition, or `None`.
///
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct DefClause {
    definition: fastobo::ast::QuotedString,
    xrefs: XrefList,
}

impl DefClause {
    pub fn new<X>(py: Python, definition: fastobo::ast::QuotedString, xrefs: X) -> Self
    where
        X: IntoPy<XrefList>,
    {
        Self {
            definition,
            xrefs: xrefs.into_py(py),
        }
    }
}

impl ClonePy for DefClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            definition: self.definition.clone(),
            xrefs: self.xrefs.clone_py(py),
        }
    }
}

impl Display for DefClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<DefClause> for fastobo::ast::TermClause {
    fn from_py(clause: DefClause, py: Python) -> Self {
        fastobo::ast::TermClause::Def(clause.definition, clause.xrefs.into_py(py))
    }
}

#[pymethods]
impl DefClause {
    #[new]
    fn __init__(obj: &PyRawObject, definition: String, xrefs: Option<&PyAny>) -> PyResult<()> {
        let def = fastobo::ast::QuotedString::new(definition);
        let list = match xrefs {
            Some(x) => XrefList::collect(obj.py(), x)?,
            None => XrefList::new(obj.py(), Vec::new()),
        };
        Ok(obj.init(Self::new(obj.py(), def, list)))
    }

    #[getter]
    /// `str`: a textual definition for this term.
    fn get_definition(&self) -> &str {
        &self.definition.as_str()
    }

    #[setter]
    fn set_definition(&mut self, definition: String) {
        self.definition = fastobo::ast::QuotedString::new(definition);
    }

    #[getter]
    /// `~fastobo.xrefs.XrefList`: a list of xrefs supporting the definition.
    fn get_xrefs<'py>(&self, py: Python<'py>) -> XrefList {
        self.xrefs.clone_py(py)
    }
}

impl_raw_tag!(DefClause, "def");
impl_raw_value!(DefClause, "{}", self.definition);

#[pyproto]
impl PyObjectProtocol for DefClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, DefClause(self.definition, self.xrefs))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.definition && self.xrefs)
    }
}

// --- Comment ---------------------------------------------------------------

/// CommentClause(comment)
/// --
///
/// A clause storing a comment for this term.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Clone, ClonePy, Debug)]
pub struct CommentClause {
    comment: fastobo::ast::UnquotedString,
}

impl CommentClause {
    pub fn new(_py: Python, comment: fastobo::ast::UnquotedString) -> Self {
        Self { comment }
    }
}

impl Display for CommentClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TermClause::from(self.clone()).fmt(f)
    }
}

impl From<CommentClause> for fastobo::ast::TermClause {
    fn from(clause: CommentClause) -> Self {
        fastobo::ast::TermClause::Comment(clause.comment)
    }
}

impl FromPy<CommentClause> for fastobo::ast::TermClause {
    fn from_py(clause: CommentClause, _py: Python) -> Self {
        Self::from(clause)
    }
}

#[pymethods]
impl CommentClause {
    #[new]
    fn __init__(obj: &PyRawObject, comment: String) {
        obj.init(Self::new(
            obj.py(),
            fastobo::ast::UnquotedString::new(comment),
        ));
    }

    #[getter]
    /// `str`: a comment relevant to this term.
    fn get_comment(&self) -> &str {
        self.comment.as_str()
    }

    #[setter]
    fn set_comment(&mut self, comment: String) -> PyResult<()> {
        self.comment = fastobo::ast::UnquotedString::new(comment);
        Ok(())
    }
}

impl_raw_tag!(CommentClause, "comment");
impl_raw_value!(CommentClause, "{}", self.comment);

#[pyproto]
impl PyObjectProtocol for CommentClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, CommentClause(self.comment))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.comment)
    }
}

// --- Subset ----------------------------------------------------------------

/// SubsetClause(subset)
/// --
///
/// A clause declaring a subset to which this term belongs.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct SubsetClause {
    #[pyo3(set)]
    subset: Ident,
}

impl SubsetClause {
    pub fn new<I>(py: Python, subset: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            subset: subset.into_py(py),
        }
    }
}

impl ClonePy for SubsetClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            subset: self.subset.clone_py(py),
        }
    }
}

impl Display for SubsetClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<SubsetClause> for fastobo::ast::TermClause {
    fn from_py(clause: SubsetClause, py: Python) -> Self {
        fastobo::ast::TermClause::Subset(clause.subset.into_py(py))
    }
}

#[pymethods]
impl SubsetClause {
    #[new]
    fn __init__(obj: &PyRawObject, subset: Ident) {
        obj.init(Self::new(obj.py(), subset));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the ID of the subset this term is part of.
    fn get_subset(&self) -> &Ident {
        &self.subset
    }
}

impl_raw_tag!(SubsetClause, "subset");
impl_raw_value!(SubsetClause, "{}", self.subset);

#[pyproto]
impl PyObjectProtocol for SubsetClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, SubsetClause(self.subset))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.subset)
    }
}

// --- Synonym ---------------------------------------------------------------

/// SynonymClause(synonym)
/// --
///
/// A clause giving a synonym for this term, with some cross-references.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct SynonymClause {
    synonym: Py<Synonym>,
}

impl SynonymClause {
    pub fn new<S>(py: Python, synonym: S) -> Self
    where
        S: IntoPy<Synonym>,
    {
        Self {
            synonym: Py::new(py, synonym.into_py(py)).unwrap(),
        }
    }
}

impl ClonePy for SynonymClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            synonym: self.synonym.clone_py(py),
        }
    }
}

impl Display for SynonymClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<SynonymClause> for fastobo::ast::TermClause {
    fn from_py(clause: SynonymClause, py: Python) -> Self {
        fastobo::ast::TermClause::Synonym(
            clause.synonym.as_gil_ref(py).clone_py(py).into_py(py)
        )
    }
}

impl_raw_tag!(SynonymClause, "synonym");

#[pymethods]
impl SynonymClause {
    #[new]
    fn __init__(obj: &PyRawObject, synonym: &Synonym) {
        let s = synonym.clone_py(obj.py());
        obj.init(Self::new(obj.py(), s));
    }

    #[getter]
    /// `~fastobo.syn.Synonym`: a possible synonym for this term.
    fn get_synonym<'py>(&self, py: Python<'py>) -> Py<Synonym> {
        self.synonym.clone_py(py)
    }

    #[setter]
    fn set_synonym(&mut self, synonym: &Synonym) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        self.synonym = Py::new(py, synonym.clone_py(py))?;
        Ok(())
    }

    fn raw_value(&self) -> PyResult<String> {
        let gil = Python::acquire_gil();
        Ok(format!("{}", self.synonym.as_gil_ref(gil.python())))
    }
}

#[pyproto]
impl PyObjectProtocol for SynonymClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, SynonymClause(self.synonym))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.synonym)
    }
}

// --- Xref ------------------------------------------------------------------

/// XrefClause(xref)
/// --
///
/// A cross-reference that describes an analogous term in another vocabulary.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct XrefClause {
    xref: Py<Xref>,
}

impl XrefClause {
    pub fn new<X>(py: Python, xref: X) -> Self
    where
        X: IntoPy<Xref>,
    {
        Self::from_py(xref.into_py(py), py)
    }
}

impl ClonePy for XrefClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            xref: self.xref.clone_py(py),
        }
    }
}

impl Display for XrefClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<XrefClause> for fastobo::ast::TermClause {
    fn from_py(clause: XrefClause, py: Python) -> Self {
        fastobo::ast::TermClause::Xref(clause.xref.as_ref(py).clone_py(py).into_py(py))
    }
}

impl From<Py<Xref>> for XrefClause {
    fn from(xref: Py<Xref>) -> Self {
        Self { xref }
    }
}

impl FromPy<Xref> for XrefClause {
    fn from_py(xref: Xref, py: Python) -> Self {
        Self {
            xref: Py::new(py, xref)
                .expect("could not allocate memory on Python heap for XrefClause"),
        }
    }
}

#[pymethods]
impl XrefClause {
    #[new]
    fn __init__(obj: &PyRawObject, xref: &PyAny) -> PyResult<()> {
        Xref::from_object(obj.py(), xref).map(|x| obj.init(Self::from(x)))
    }

    #[getter]
    /// `~fastobo.xref.Xref`: a cross-reference relevant to this term.
    fn get_xref<'py>(&self, py: Python<'py>) -> Py<Xref> {
        self.xref.clone_ref(py)
    }

    #[setter]
    fn set_xref(&mut self, xref: &PyAny) -> PyResult<()> {
        self.xref = Xref::from_object(xref.py(), xref)?;
        Ok(())
    }

    pub fn raw_value(&self) -> PyResult<String> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(self.xref.as_ref(py).to_string())
    }
}

impl_raw_tag!(XrefClause, "xref");

#[pyproto]
impl PyObjectProtocol for XrefClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, XrefClause(self.xref))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.xref)
    }
}

// --- Builtin ---------------------------------------------------------------

/// BuiltinClause(builtin)
/// --
///
/// A clause declaring whether or not this term is built-in to the OBO format.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Clone, ClonePy, Debug)]
pub struct BuiltinClause {
    #[pyo3(set)]
    builtin: bool,
}

impl BuiltinClause {
    pub fn new(_py: Python, builtin: bool) -> Self {
        Self { builtin }
    }
}

impl Display for BuiltinClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TermClause::from(self.clone()).fmt(f)
    }
}

impl From<BuiltinClause> for fastobo::ast::TermClause {
    fn from(clause: BuiltinClause) -> Self {
        fastobo::ast::TermClause::Builtin(clause.builtin)
    }
}

impl FromPy<BuiltinClause> for fastobo::ast::TermClause {
    fn from_py(clause: BuiltinClause, _py: Python) -> Self {
        fastobo::ast::TermClause::from(clause)
    }
}

#[pymethods]
impl BuiltinClause {
    #[new]
    fn __init__(obj: &PyRawObject, builtin: bool) {
        obj.init(Self::new(obj.py(), builtin));
    }

    /// `bool`: ``True`` if the term is built in the OBO format.
    #[getter]
    fn get_builtin(&self) -> bool {
        self.builtin
    }
}

impl_raw_tag!(BuiltinClause, "builtin");
impl_raw_value!(BuiltinClause, "{}", self.builtin);

#[pyproto]
impl PyObjectProtocol for BuiltinClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, BuiltinClause(self.builtin))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.builtin)
    }
}

// --- PropertyValue ---------------------------------------------------------

/// PropertyValueClause(property_value)
/// --
///
/// A clause that binds a property to a value in the term.
///
/// Arguments:
///     property_value (~fastobo.pv.AbstractPropertyValue): the property value
///         to annotate the current term.
///
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct PropertyValueClause {
    inner: PropertyValue,
}

impl PropertyValueClause {
    pub fn new<P>(py: Python, property_value: P) -> Self
    where
        P: IntoPy<PropertyValue>,
    {
        Self {
            inner: property_value.into_py(py),
        }
    }
}

impl ClonePy for PropertyValueClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            inner: self.inner.clone_py(py),
        }
    }
}

impl Display for PropertyValueClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<PropertyValueClause> for fastobo::ast::TermClause {
    fn from_py(clause: PropertyValueClause, py: Python) -> ast::TermClause {
        ast::TermClause::PropertyValue(clause.inner.into_py(py))
    }
}

#[pymethods]
impl PropertyValueClause {
    #[new]
    pub fn __init__(obj: &PyRawObject, property_value: PropertyValue) {
        obj.init(Self::new(obj.py(), property_value));
    }

    #[getter]
    /// `~fastobo.pv.AbstractPropertyValue`: an annotation of the term.
    pub fn get_property_value(&self) -> &PropertyValue {
        &self.inner
    }

    #[setter]
    pub fn set_property_value(&mut self, pv: PropertyValue)  {
        self.inner = pv;
    }
}

impl_raw_tag!(PropertyValueClause, "property_value");
impl_raw_value!(PropertyValueClause, "{}", self.inner);

#[pyproto]
impl PyObjectProtocol for PropertyValueClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, PropertyValueClause(self.inner))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.inner)
    }
}

// --- IsA -------------------------------------------------------------------

/// IsAClause(term)
/// --
///
/// A clause declaring this term is a subclass of another term.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct IsAClause {
    #[pyo3(set)]
    term: Ident,
}

impl IsAClause {
    pub fn new<I>(py: Python, term: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            term: term.into_py(py),
        }
    }
}

impl ClonePy for IsAClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            term: self.term.clone_py(py),
        }
    }
}

impl Display for IsAClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<IsAClause> for fastobo::ast::TermClause {
    fn from_py(clause: IsAClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::IsA(clause.term.into_py(py))
    }
}

#[pymethods]
impl IsAClause {
    #[new]
    fn __init__(obj: &PyRawObject, term: Ident) {
        obj.init(Self::new(obj.py(), term));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the parent term.
    fn get_term(&self) -> &Ident {
        &self.term
    }
}

impl_raw_tag!(IsAClause, "is_a");
impl_raw_value!(IsAClause, "{}", self.term);

#[pyproto]
impl PyObjectProtocol for IsAClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsAClause(self.term))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.term)
    }
}

// --- IntersectionOf --------------------------------------------------------

/// IntersectionOfClause(typedef, term)
/// --
///
/// A clause stating this term is equivalent to the intersection of other terms.
///
/// Arguments:
///     typedef (~fastobo.id.Ident or None): the identifier of the composing
///         relationship, or `None` if the term is an intersection of other
///         terms.
///     term (~fastobo.id.Ident): the identifier of the composing term.
///
/// Example:
///     The following code describes the GO term ``GO:0000085`` (*G2 phase of
///     mitotic cell cycle*) as being equivalent to any term which is both
///     a subclass of ``GO:0051319`` (*G2 phase*) and has a ``part_of``
///     relationship to ``GO:0000278`` (*mitotic cell cycle*):
///
///     >>> from fastobo.term import TermFrame, IntersectionOfClause
///     >>> frame = TermFrame(fastobo.id.PrefixedIdent("GO", "0000085"))
///     >>> frame.append(IntersectionOfClause(
///     ...    typedef=None,
///     ...    term=fastobo.id.PrefixedIdent("GO", "0051319")),
///     ... )
///     >>> frame.append(IntersectionOfClause(
///     ...     typedef=fastobo.id.UnprefixedIdent("part_of"),
///     ...     term=fastobo.id.PrefixedIdent("GO", "0000278")
///     ... ))
///
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct IntersectionOfClause {
    typedef: Option<Ident>,
    term: Ident,
}

impl IntersectionOfClause {
    pub fn new<R, T>(py: Python, typedef: Option<R>, term: T) -> Self
    where
        R: IntoPy<Ident>,
        T: IntoPy<Ident>,
    {
        Self {
            typedef: typedef.map(|id| id.into_py(py)),
            term: term.into_py(py),
        }
    }
}

impl ClonePy for IntersectionOfClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
            term: self.term.clone_py(py),
        }
    }
}

impl Display for IntersectionOfClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<IntersectionOfClause> for fastobo::ast::TermClause {
    fn from_py(clause: IntersectionOfClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::IntersectionOf(
            clause.typedef.map(|id| id.into_py(py)),
            clause.term.into_py(py),
        )
    }
}

#[pymethods]
impl IntersectionOfClause {
    #[new]
    fn __init__(obj: &PyRawObject, typedef: Option<Ident>, term: Ident) {
        obj.init(Self::new(obj.py(), typedef, term))
    }

    #[getter]
    /// `str`: the identifier of the composing term.
    fn get_term(&self) -> &Ident {
        &self.term
    }

    #[getter]
    /// `str`: the identifier of the composing relationship, if any.
    fn get_typedef(&self) -> Option<&Ident> {
        self.typedef.as_ref()
    }

    pub fn raw_value(&self) -> PyResult<String> {
        if let Some(ref rel) = self.typedef {
            Ok(format!("{} {}", rel, &self.term))
        } else {
            Ok(format!("{}", &self.term))
        }
    }
}

impl_raw_tag!(IntersectionOfClause, "intersection_of");

#[pyproto]
impl PyObjectProtocol for IntersectionOfClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        // TODO
        impl_repr!(self, IntersectionOfClause(self.typedef, self.term))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef && self.term)
    }
}

// --- UnionOf ---------------------------------------------------------------

/// UnionOfClause(term)
/// --
///
/// A clause indicating the term represents the union of several other terms.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct UnionOfClause {
    term: Ident,
}

impl UnionOfClause {
    pub fn new<I>(py: Python, term: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            term: term.into_py(py),
        }
    }
}

impl ClonePy for UnionOfClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            term: self.term.clone_py(py),
        }
    }
}

impl Display for UnionOfClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<UnionOfClause> for fastobo::ast::TermClause {
    fn from_py(clause: UnionOfClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::UnionOf(clause.term.into_py(py))
    }
}

#[pymethods]
impl UnionOfClause {
    #[new]
    fn __init__(obj: &PyRawObject, id: Ident) {
        obj.init(Self::new(obj.py(), id));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the member term.
    fn get_term(&self) -> &Ident {
        &self.term
    }
}

impl_raw_tag!(UnionOfClause, "union_of");
impl_raw_value!(UnionOfClause, "{}", self.term);

#[pyproto]
impl PyObjectProtocol for UnionOfClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, UnionOfClause(self.term))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.term)
    }
}

// --- EquivalentTo ----------------------------------------------------------


/// EquivalentToClause(term)
/// --
///
/// A clause indicating the term is exactly equivalent to another term.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct EquivalentToClause {
    term: Ident,
}

impl EquivalentToClause {
    pub fn new<I>(py: Python, term: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            term: term.into_py(py),
        }
    }
}

impl ClonePy for EquivalentToClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            term: self.term.clone_py(py),
        }
    }
}

impl Display for EquivalentToClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<EquivalentToClause> for fastobo::ast::TermClause {
    fn from_py(clause: EquivalentToClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::EquivalentTo(clause.term.into_py(py))
    }
}

#[pymethods]
impl EquivalentToClause {
    #[new]
    fn __init__(obj: &PyRawObject, term: Ident) {
        obj.init(Self::new(obj.py(), term));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the equivalent term.
    fn get_term(&self) -> &Ident {
        &self.term
    }
}

impl_raw_tag!(EquivalentToClause, "equivalent_to");
impl_raw_value!(EquivalentToClause, "{}", self.term);

#[pyproto]
impl PyObjectProtocol for EquivalentToClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, EquivalentToClause(self.term))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.term)
    }
}

// --- DisjointFrom ----------------------------------------------------------

/// DisjointFromClause(term)
/// --
///
/// A clause stating this term has no instances in common with another term.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct DisjointFromClause {
    #[pyo3(set)]
    term: Ident,
}

impl DisjointFromClause {
    pub fn new<I>(py: Python, term: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            term: term.into_py(py),
        }
    }
}

impl ClonePy for DisjointFromClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            term: self.term.clone_py(py),
        }
    }
}

impl Display for DisjointFromClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<DisjointFromClause> for fastobo::ast::TermClause {
    fn from_py(clause: DisjointFromClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::DisjointFrom(clause.term.into_py(py))
    }
}

#[pymethods]
impl DisjointFromClause {
    #[new]
    fn __init__(obj: &PyRawObject, term: Ident) {
        obj.init(Self::new(obj.py(), term));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the disjoint term.
    fn get_term(&self) -> &Ident {
        &self.term
    }
}

impl_raw_tag!(DisjointFromClause, "disjoint_from");
impl_raw_value!(DisjointFromClause, "{}", self.term);

#[pyproto]
impl PyObjectProtocol for DisjointFromClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, DisjointFromClause(self.term))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.term)
    }
}

// --- Relationship ----------------------------------------------------------

/// RelationshipClause(typedef, term)
/// --
///
/// A clause describing a typed relationship between this term and another term.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct RelationshipClause {
    #[pyo3(set)]
    typedef: Ident,
    #[pyo3(set)]
    term: Ident,
}

impl RelationshipClause {
    pub fn new<R, T>(py: Python, typedef: R, term: T) -> Self
    where
        R: IntoPy<Ident>,
        T: IntoPy<Ident>,
    {
        Self {
            typedef: typedef.into_py(py),
            term: term.into_py(py),
        }
    }
}

impl ClonePy for RelationshipClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
            term: self.term.clone_py(py),
        }
    }
}

impl Display for RelationshipClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<RelationshipClause> for fastobo::ast::TermClause {
    fn from_py(clause: RelationshipClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::Relationship(clause.typedef.into_py(py), clause.term.into_py(py))
    }
}

impl_raw_tag!(RelationshipClause, "relationship");
impl_raw_value!(RelationshipClause, "{} {}", self.typedef, self.term);

#[pymethods]
impl RelationshipClause {
    #[new]
    fn __init__(obj: &PyRawObject, typedef: Ident, term: Ident) -> PyResult<()> {
        Ok(obj.init(Self::new(obj.py(), typedef, term)))
    }

    #[getter]
    fn get_typedef<'py>(&self, py: Python<'py>) -> PyResult<Ident> {
        Ok(self.typedef.clone_py(py))
    }

    #[getter]
    fn get_term<'py>(&self, py: Python<'py>) -> PyResult<Ident> {
        Ok(self.term.clone_py(py))
    }
}

#[pyproto]
impl PyObjectProtocol for RelationshipClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, RelationshipClause(self.typedef, self.term))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef && self.term)
    }
}

// --- IsObsolete ------------------------------------------------------------

/// IsObsoleteClause(obsolete)
/// --
///
/// A clause indicating whether or not this term is obsolete.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsObsoleteClause {
    #[pyo3(get, set)]
    obsolete: bool,
}

impl IsObsoleteClause {
    pub fn new(_py: Python, obsolete: bool) -> Self {
        Self { obsolete }
    }
}

impl Display for IsObsoleteClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TermClause::from(self.clone()).fmt(f)
    }
}

impl From<IsObsoleteClause> for fastobo::ast::TermClause {
    fn from(clause: IsObsoleteClause) -> Self {
        fastobo::ast::TermClause::IsObsolete(clause.obsolete)
    }
}

impl FromPy<IsObsoleteClause> for fastobo::ast::TermClause {
    fn from_py(clause: IsObsoleteClause, _py: Python) -> Self {
        fastobo::ast::TermClause::from(clause)
    }
}

impl_raw_tag!(IsObsoleteClause, "is_obsolete");
impl_raw_value!(IsObsoleteClause, "{}", self.obsolete);

#[pymethods]
impl IsObsoleteClause {
    #[new]
    fn __init__(obj: &PyRawObject, obsolete: bool) -> PyResult<()> {
        Ok(obj.init(Self::new(obj.py(), obsolete)))
    }
}

#[pyproto]
impl PyObjectProtocol for IsObsoleteClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsObsoleteClause(self.obsolete))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.obsolete)
    }
}

// --- ReplacedBy ------------------------------------------------------------

/// ReplacedByClause(term)
/// --
///
/// A clause giving a term which replaces this obsolete term.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct ReplacedByClause {
    #[pyo3(set)]
    term: Ident,
}

impl ReplacedByClause {
    pub fn new<I>(py: Python, term: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            term: term.into_py(py),
        }
    }
}

impl ClonePy for ReplacedByClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            term: self.term.clone_py(py),
        }
    }
}

impl Display for ReplacedByClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<ReplacedByClause> for fastobo::ast::TermClause {
    fn from_py(clause: ReplacedByClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::ReplacedBy(clause.term.into_py(py))
    }
}

impl_raw_tag!(ReplacedByClause, "replaced_by");
impl_raw_value!(ReplacedByClause, "{}", self.term);

#[pymethods]
impl ReplacedByClause {
    #[new]
    fn __init__(obj: &PyRawObject, term: Ident) {
        obj.init(Self::new(obj.py(), term));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the replacement term.
    fn get_term(&self) -> &Ident {
        &self.term
    }
}

#[pyproto]
impl PyObjectProtocol for ReplacedByClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, ReplacedByClause(self.term))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.term)
    }
}

// --- Consider --------------------------------------------------------------

/// ConsiderClause(term)
/// --
///
/// A clause giving a potential substitute for an obsolete term.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Debug)]
pub struct ConsiderClause {
    term: Ident,
}

impl ConsiderClause {
    pub fn new<I>(py: Python, term: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            term: term.into_py(py),
        }
    }
}

impl ClonePy for ConsiderClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            term: self.term.clone_py(py),
        }
    }
}

impl Display for ConsiderClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<ConsiderClause> for fastobo::ast::TermClause {
    fn from_py(clause: ConsiderClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::Consider(clause.term.into_py(py))
    }
}

impl_raw_tag!(ConsiderClause, "consider");
impl_raw_value!(ConsiderClause, "{}", self.term);

#[pymethods]
impl ConsiderClause {
    #[new]
    fn __init__(obj: &PyRawObject, term: Ident) -> PyResult<()> {
        Ok(obj.init(Self::new(obj.py(), term)))
    }

    #[getter]
    fn get_term(&self) -> &Ident {
        &self.term
    }
}

#[pyproto]
impl PyObjectProtocol for ConsiderClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, ConsiderClause(self.term))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.term)
    }
}

// --- CreatedBy -------------------------------------------------------------

/// CreatedByClause(creator)
/// --
///
/// A term clause stating the name of the creator of this term.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Clone, ClonePy, Debug)]
pub struct CreatedByClause {
    creator: fastobo::ast::UnquotedString,
}

impl CreatedByClause {
    pub fn new(_py: Python, creator: fastobo::ast::UnquotedString) -> Self {
        Self { creator }
    }
}

impl Display for CreatedByClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TermClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl From<CreatedByClause> for fastobo::ast::TermClause {
    fn from(clause: CreatedByClause) -> Self {
        fastobo::ast::TermClause::CreatedBy(clause.creator)
    }
}

impl FromPy<CreatedByClause> for fastobo::ast::TermClause {
    fn from_py(clause: CreatedByClause, _py: Python) -> fastobo::ast::TermClause {
        Self::from(clause)
    }
}

impl_raw_tag!(CreatedByClause, "created_by");
impl_raw_value!(CreatedByClause, "{}", self.creator);

#[pymethods]
impl CreatedByClause {
    #[new]
    fn __init__(obj: &PyRawObject, creator: String) -> PyResult<()> {
        Ok(obj.init(Self::new(obj.py(), fastobo::ast::UnquotedString::new(creator))))
    }

    #[getter]
    /// `str`: the name of the creator of this term.
    fn get_creator(&self) -> &str {
        self.creator.as_str()
    }

    #[setter]
    fn set_creator(&mut self, creator: String) -> PyResult<()> {
        self.creator = fastobo::ast::UnquotedString::new(creator);
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for CreatedByClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, CreatedByClause(self.creator))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.creator)
    }
}

// --- CreationDate ----------------------------------------------------------

/// CreationDateClause(date)
/// --
///
/// A clause declaring the date and time a term was created.
///
/// Arguments:
///     date (~datetime.datetime): the date and time this term was created.
///
/// Warning:
///     The timezone of the `datetime` will only be extracted down to the
///     minutes, seconds and smaller durations will be ignored. It is advised
///     to use `datetime.timezone.utc` whenever possible to preserve the
///     date and time properly.
///
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Clone, ClonePy, Debug)]
pub struct CreationDateClause {
    date: fastobo::ast::IsoDateTime,
}

impl CreationDateClause {
    pub fn new(_py: Python, date: fastobo::ast::IsoDateTime) -> Self {
        Self { date }
    }
}

impl Display for CreationDateClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TermClause::from(self.clone()).fmt(f)
    }
}

impl FromPy<CreationDateClause> for fastobo::ast::TermClause {
    fn from_py(clause: CreationDateClause, py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::from(clause)
    }
}

impl From<CreationDateClause> for fastobo::ast::TermClause {
    fn from(clause: CreationDateClause) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::CreationDate(clause.date)
    }
}

impl_raw_tag!(CreationDateClause, "creation_date");
impl_raw_value!(CreationDateClause, "{}", self.date);

#[pymethods]
impl CreationDateClause {
    #[new]
    fn __init__(obj: &PyRawObject, datetime: &PyDateTime) -> PyResult<()> {
        let date = datetime_to_isodate(datetime.py(), datetime)?;
        obj.init(CreationDateClause { date });
        Ok(())
    }

    #[getter]
    /// `datetime.datetime`: the date and time this term was created.
    fn get_date<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDateTime> {
        isodate_to_datetime(py, &self.date)
    }

    #[setter]
    fn set_date(&mut self, datetime: &PyDateTime) -> PyResult<()> {
        self.date = datetime_to_isodate(datetime.py(), datetime)?;
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for CreationDateClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let fmt = PyString::new(py, "CreationDateClause({!r})").to_object(py);
        self.get_date(py).and_then(|dt| fmt.call_method1(py, "format", (dt,)))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.date)
    }
}
