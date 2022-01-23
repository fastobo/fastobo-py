use std::cmp::Ord;
use std::cmp::Ordering;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fmt::Write;
use std::str::FromStr;

use pyo3::class::basic::CompareOp;
use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyDate;
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

use super::super::abc::AbstractEntityClause;
use super::super::id::Ident;
use super::super::pv::PropertyValue;
use super::super::syn::Synonym;
use super::super::xref::Xref;
use super::super::xref::XrefList;
use crate::date::date_to_isodate;
use crate::date::datetime_to_isodatetime;
use crate::date::isodate_to_date;
use crate::date::isodatetime_to_datetime;
use crate::raise;
use crate::utils::AbstractClass;
use crate::utils::ClonePy;
use crate::utils::FinalClass;

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

impl IntoPy<TermClause> for fastobo::ast::TermClause {
    fn into_py(self, py: Python) -> TermClause {
        use fastobo::ast::TermClause::*;
        match self {
            IsAnonymous(b) => Py::new(py, IsAnonymousClause::new(b)).map(TermClause::IsAnonymous),
            Name(n) => Py::new(py, NameClause::new(*n)).map(TermClause::Name),
            Namespace(ns) => {
                Py::new(py, NamespaceClause::new(ns.into_py(py))).map(TermClause::Namespace)
            }
            AltId(id) => Py::new(py, AltIdClause::new(id.into_py(py))).map(TermClause::AltId),
            Def(mut def) => {
                let text = std::mem::take(def.text_mut());
                let xrefs = std::mem::take(def.xrefs_mut()).into_py(py);
                Py::new(py, DefClause::new(text, xrefs)).map(TermClause::Def)
            }
            Comment(c) => Py::new(py, CommentClause::new(*c)).map(TermClause::Comment),
            Subset(s) => Py::new(py, SubsetClause::new(s.into_py(py))).map(TermClause::Subset),
            Synonym(s) => Py::new(py, s.into_py(py))
                .map(SynonymClause::new)
                .and_then(|clause| Py::new(py, clause))
                .map(TermClause::Synonym),
            Xref(x) => Py::new(py, x.into_py(py))
                .map(XrefClause::new)
                .and_then(|clause| Py::new(py, clause))
                .map(TermClause::Xref),
            Builtin(b) => Py::new(py, BuiltinClause::new(b)).map(TermClause::Builtin),
            PropertyValue(pv) => {
                Py::new(py, PropertyValueClause::new(pv.into_py(py))).map(TermClause::PropertyValue)
            }
            IsA(id) => Py::new(py, IsAClause::new(id.into_py(py))).map(TermClause::IsA),
            IntersectionOf(r, cls) => Py::new(
                py,
                IntersectionOfClause::new(r.map(|id| id.into_py(py)), cls.into_py(py)),
            )
            .map(TermClause::IntersectionOf),
            UnionOf(cls) => {
                Py::new(py, UnionOfClause::new(cls.into_py(py))).map(TermClause::UnionOf)
            }
            EquivalentTo(cls) => {
                Py::new(py, EquivalentToClause::new(cls.into_py(py))).map(TermClause::EquivalentTo)
            }
            DisjointFrom(cls) => {
                Py::new(py, DisjointFromClause::new(cls.into_py(py))).map(TermClause::DisjointFrom)
            }
            Relationship(r, id) => {
                Py::new(py, RelationshipClause::new(r.into_py(py), id.into_py(py)))
                    .map(TermClause::Relationship)
            }
            IsObsolete(b) => Py::new(py, IsObsoleteClause::new(b)).map(TermClause::IsObsolete),
            ReplacedBy(id) => {
                Py::new(py, ReplacedByClause::new(id.into_py(py))).map(TermClause::ReplacedBy)
            }
            Consider(id) => {
                Py::new(py, ConsiderClause::new(id.into_py(py))).map(TermClause::Consider)
            }
            CreatedBy(name) => Py::new(py, CreatedByClause::new(*name)).map(TermClause::CreatedBy),
            CreationDate(dt) => {
                Py::new(py, CreationDateClause::new(*dt)).map(TermClause::CreationDate)
            }
        }
        .expect("could not allocate memory for `TermClause` in Python heap")
    }
}

// --- Base ------------------------------------------------------------------

/// A term clause, appearing in an OBO term frame.
#[pyclass(subclass, extends=AbstractEntityClause, module="fastobo.term")]
#[derive(AbstractClass)]
#[base(AbstractEntityClause)]
pub struct BaseTermClause {}

// --- IsAnonymous -----------------------------------------------------------

/// IsAnonymousClause(anonymous)
/// --
///
/// A clause declaring whether or not the current term has an anonymous id.
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Clone, ClonePy, Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct IsAnonymousClause {
    #[pyo3(get, set)]
    anonymous: bool,
}

impl IsAnonymousClause {
    pub fn new(anonymous: bool) -> Self {
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

impl IntoPy<fastobo::ast::TermClause> for IsAnonymousClause {
    fn into_py(self, _py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::from(self)
    }
}

#[pymethods]
impl IsAnonymousClause {
    #[new]
    fn __init__(anonymous: bool) -> PyClassInitializer<Self> {
        Self::new(anonymous).into()
    }

    fn raw_tag(&self) -> &str {
        "is_anonymous"
    }

    fn raw_value(&self) -> String {
        self.anonymous.to_string()
    }
}

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
#[derive(Clone, ClonePy, Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct NameClause {
    name: fastobo::ast::UnquotedString,
}

impl NameClause {
    pub fn new(name: fastobo::ast::UnquotedString) -> Self {
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
        fastobo::ast::TermClause::Name(Box::new(clause.name))
    }
}

impl IntoPy<fastobo::ast::TermClause> for NameClause {
    fn into_py(self, _py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::from(self)
    }
}

#[pymethods]
impl NameClause {
    #[new]
    fn __init__(name: String) -> PyClassInitializer<Self> {
        Self::new(fastobo::ast::UnquotedString::new(name)).into()
    }

    /// `str`: the name of the current term.
    #[getter]
    fn get_name(&self) -> &str {
        self.name.as_str()
    }

    #[setter]
    fn set_name(&mut self, name: String) {
        self.name = fastobo::ast::UnquotedString::new(name);
    }

    fn raw_tag(&self) -> &str {
        "name"
    }

    fn raw_value(&self) -> String {
        self.name.to_string()
    }
}

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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct NamespaceClause {
    #[pyo3(set)]
    namespace: Ident,
}

impl NamespaceClause {
    pub fn new(namespace: Ident) -> Self {
        Self { namespace }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for NamespaceClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        let ns: fastobo::ast::NamespaceIdent = self.namespace.into_py(py);
        fastobo::ast::TermClause::Namespace(Box::new(ns))
    }
}

#[pymethods]
impl NamespaceClause {
    #[new]
    fn __init__(namespace: Ident) -> PyClassInitializer<Self> {
        Self::new(namespace).into()
    }

    #[getter]
    /// `~fastobo.id.Ident`: the ID of the namespace this term is part of.
    fn get_namespace(&self) -> &Ident {
        &self.namespace
    }

    fn raw_tag(&self) -> &str {
        "namespace"
    }

    fn raw_value(&self) -> String {
        self.namespace.to_string()
    }
}

#[pyproto]
impl PyObjectProtocol for NamespaceClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, NamespaceClause(self.namespace))
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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct AltIdClause {
    #[pyo3(set)]
    alt_id: Ident,
}

impl AltIdClause {
    pub fn new(alt_id: Ident) -> Self {
        Self { alt_id }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for AltIdClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::AltId(Box::new(self.alt_id.into_py(py)))
    }
}

#[pymethods]
impl AltIdClause {
    #[new]
    fn __init__(alt_id: Ident) -> PyClassInitializer<Self> {
        Self::new(alt_id).into()
    }

    #[getter]
    /// `~fastobo.id.Ident`: an alternative ID used to refer to this term.
    fn get_alt_id(&self) -> &Ident {
        &self.alt_id
    }

    fn raw_tag(&self) -> &str {
        "alt_id"
    }

    fn raw_value(&self) -> String {
        self.alt_id.to_string()
    }
}

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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct DefClause {
    definition: fastobo::ast::QuotedString,
    xrefs: XrefList,
}

impl DefClause {
    pub fn new(definition: fastobo::ast::QuotedString, xrefs: XrefList) -> Self {
        Self { definition, xrefs }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for DefClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        let xrefs: fastobo::ast::XrefList = self.xrefs.into_py(py);
        let def = fastobo::ast::Definition::with_xrefs(self.definition, xrefs);
        fastobo::ast::TermClause::Def(Box::new(def))
    }
}

#[pymethods]
impl DefClause {
    #[new]
    fn __init__(definition: String, xrefs: Option<&PyAny>) -> PyResult<PyClassInitializer<Self>> {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let def = fastobo::ast::QuotedString::new(definition);
        let list = match xrefs {
            Some(x) => XrefList::collect(py, x)?,
            None => XrefList::new(Vec::new()),
        };

        Ok(Self::new(def, list).into())
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

    fn raw_tag(&self) -> &str {
        "def"
    }

    fn raw_value(&self) -> String {
        self.definition.to_string()
    }
}

#[pyproto]
impl PyObjectProtocol for DefClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        if self.xrefs.is_empty() {
            impl_repr!(self, DefClause(self.definition))
        } else {
            impl_repr!(self, DefClause(self.definition, self.xrefs))
        }
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
#[derive(Clone, ClonePy, Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct CommentClause {
    comment: fastobo::ast::UnquotedString,
}

impl CommentClause {
    pub fn new(comment: fastobo::ast::UnquotedString) -> Self {
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
        fastobo::ast::TermClause::Comment(Box::new(clause.comment))
    }
}

impl IntoPy<fastobo::ast::TermClause> for CommentClause {
    fn into_py(self, _py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::from(self)
    }
}

#[pymethods]
impl CommentClause {
    #[new]
    fn __init__(comment: String) -> PyClassInitializer<Self> {
        Self::new(fastobo::ast::UnquotedString::new(comment)).into()
    }

    #[getter]
    /// `str`: a comment relevant to this term.
    fn get_comment(&self) -> &str {
        self.comment.as_str()
    }

    #[setter]
    fn set_comment(&mut self, comment: String) {
        self.comment = fastobo::ast::UnquotedString::new(comment);
    }

    fn raw_tag(&self) -> &str {
        "comment"
    }

    fn raw_value(&self) -> String {
        self.comment.to_string()
    }
}

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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct SubsetClause {
    #[pyo3(set)]
    subset: Ident,
}

impl SubsetClause {
    pub fn new(subset: Ident) -> Self {
        Self { subset }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for SubsetClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::Subset(Box::new(self.subset.into_py(py)))
    }
}

#[pymethods]
impl SubsetClause {
    #[new]
    fn __init__(subset: Ident) -> PyClassInitializer<Self> {
        Self::new(subset).into()
    }

    #[getter]
    /// `~fastobo.id.Ident`: the ID of the subset this term is part of.
    fn get_subset(&self) -> &Ident {
        &self.subset
    }

    fn raw_tag(&self) -> &str {
        "subset"
    }

    fn raw_value(&self) -> String {
        self.subset.to_string()
    }
}

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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct SynonymClause {
    #[pyo3(set)]
    synonym: Py<Synonym>,
}

impl SynonymClause {
    pub fn new(synonym: Py<Synonym>) -> Self {
        Self { synonym }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for SynonymClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::Synonym(Box::new(
            self.synonym.as_ref(py).borrow().clone_py(py).into_py(py),
        ))
    }
}

#[pymethods]
impl SynonymClause {
    #[new]
    fn __init__(synonym: Py<Synonym>) -> PyClassInitializer<Self> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Self::new(synonym.clone_ref(py)).into()
    }

    #[getter]
    /// `~fastobo.syn.Synonym`: a possible synonym for this term.
    fn get_synonym<'py>(&self, py: Python<'py>) -> Py<Synonym> {
        self.synonym.clone_py(py)
    }

    fn raw_tag(&self) -> &str {
        "synonym"
    }

    fn raw_value(&self) -> String {
        let gil = Python::acquire_gil();
        let py = gil.python();
        format!("{}", &*self.synonym.as_ref(py).borrow())
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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct XrefClause {
    #[pyo3(get, set)]
    /// `~fastobo.xref.Xref`: a cross-reference relevant to this term.
    xref: Py<Xref>,
}

impl XrefClause {
    pub fn new(xref: Py<Xref>) -> Self {
        Self { xref }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl From<Py<Xref>> for XrefClause {
    fn from(xref: Py<Xref>) -> Self {
        Self { xref }
    }
}

impl IntoPy<fastobo::ast::TermClause> for XrefClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::Xref(Box::new(
            self.xref.as_ref(py).borrow().clone_py(py).into_py(py),
        ))
    }
}

impl IntoPy<XrefClause> for Xref {
    fn into_py(self, py: Python) -> XrefClause {
        XrefClause {
            xref: Py::new(py, self)
                .expect("could not allocate memory on Python heap for XrefClause"),
        }
    }
}

#[pymethods]
impl XrefClause {
    #[new]
    fn __init__(xref: Py<Xref>) -> PyClassInitializer<Self> {
        Self::from(xref).into()
    }

    fn raw_tag(&self) -> &str {
        "xref"
    }

    fn raw_value(&self) -> String {
        let gil = Python::acquire_gil();
        let py = gil.python();
        self.xref.as_ref(py).to_string()
    }
}

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
#[derive(Clone, ClonePy, Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct BuiltinClause {
    #[pyo3(set)]
    builtin: bool,
}

impl BuiltinClause {
    pub fn new(builtin: bool) -> Self {
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

impl IntoPy<fastobo::ast::TermClause> for BuiltinClause {
    fn into_py(self, _py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::from(self)
    }
}

#[pymethods]
impl BuiltinClause {
    #[new]
    fn __init__(builtin: bool) -> PyClassInitializer<Self> {
        Self::new(builtin).into()
    }

    /// `bool`: ``True`` if the term is built in the OBO format.
    #[getter]
    fn get_builtin(&self) -> bool {
        self.builtin
    }

    fn raw_tag(&self) -> &str {
        "builtin"
    }

    fn raw_value(&self) -> String {
        self.builtin.to_string()
    }
}

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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct PropertyValueClause {
    #[pyo3(set)]
    inner: PropertyValue,
}

impl PropertyValueClause {
    pub fn new(property_value: PropertyValue) -> Self {
        Self {
            inner: property_value,
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for PropertyValueClause {
    fn into_py(self, py: Python) -> ast::TermClause {
        ast::TermClause::PropertyValue(Box::new(self.inner.into_py(py)))
    }
}

#[pymethods]
impl PropertyValueClause {
    #[new]
    fn __init__(property_value: PropertyValue) -> PyClassInitializer<Self> {
        Self::new(property_value).into()
    }

    #[getter]
    /// `~fastobo.pv.AbstractPropertyValue`: an annotation of the term.
    fn get_property_value(&self) -> &PropertyValue {
        &self.inner
    }

    fn raw_tag(&self) -> &str {
        "property_value"
    }

    fn raw_value(&self) -> String {
        self.inner.to_string()
    }
}

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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct IsAClause {
    #[pyo3(set)]
    term: Ident,
}

impl IsAClause {
    pub fn new(term: Ident) -> Self {
        Self { term }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for IsAClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::IsA(Box::new(self.term.into_py(py)))
    }
}

#[pymethods]
impl IsAClause {
    #[new]
    fn __init__(term: Ident) -> PyClassInitializer<Self> {
        Self::new(term).into()
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the parent term.
    fn get_term(&self) -> &Ident {
        &self.term
    }

    fn raw_tag(&self) -> &str {
        "is_a"
    }

    fn raw_value(&self) -> String {
        self.term.to_string()
    }
}

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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct IntersectionOfClause {
    typedef: Option<Ident>,
    term: Ident,
}

impl IntersectionOfClause {
    pub fn new(typedef: Option<Ident>, term: Ident) -> Self {
        Self { typedef, term }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for IntersectionOfClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::IntersectionOf(
            self.typedef.map(|id| Box::new(id.into_py(py))),
            Box::new(self.term.into_py(py)),
        )
    }
}

#[pymethods]
impl IntersectionOfClause {
    #[new]
    fn __init__(typedef: Option<Ident>, term: Ident) -> PyClassInitializer<Self> {
        Self::new(typedef, term).into()
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

    fn raw_tag(&self) -> &str {
        "intersection_of"
    }

    fn raw_value(&self) -> String {
        if let Some(ref rel) = self.typedef {
            format!("{} {}", rel, &self.term)
        } else {
            format!("{}", &self.term)
        }
    }
}

#[pyproto]
impl PyObjectProtocol for IntersectionOfClause {
    fn __repr__(&self) -> PyResult<PyObject> {
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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct UnionOfClause {
    term: Ident,
}

impl UnionOfClause {
    pub fn new(term: Ident) -> Self {
        Self { term }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for UnionOfClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::UnionOf(Box::new(self.term.into_py(py)))
    }
}

#[pymethods]
impl UnionOfClause {
    #[new]
    fn __init__(id: Ident) -> PyClassInitializer<Self> {
        Self::new(id).into()
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the member term.
    fn get_term(&self) -> &Ident {
        &self.term
    }

    fn raw_tag(&self) -> &str {
        "union_of"
    }

    fn raw_value(&self) -> String {
        self.term.to_string()
    }
}

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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct EquivalentToClause {
    term: Ident,
}

impl EquivalentToClause {
    pub fn new(term: Ident) -> Self {
        Self { term }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for EquivalentToClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::EquivalentTo(Box::new(self.term.into_py(py)))
    }
}

#[pymethods]
impl EquivalentToClause {
    #[new]
    fn __init__(term: Ident) -> PyClassInitializer<Self> {
        Self::new(term).into()
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the equivalent term.
    fn get_term(&self) -> &Ident {
        &self.term
    }

    fn raw_tag(&self) -> &str {
        "equivalent_to"
    }

    fn raw_value(&self) -> String {
        self.term.to_string()
    }
}

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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct DisjointFromClause {
    #[pyo3(set)]
    term: Ident,
}

impl DisjointFromClause {
    pub fn new(term: Ident) -> Self {
        Self { term }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for DisjointFromClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::DisjointFrom(Box::new(self.term.into_py(py)))
    }
}

#[pymethods]
impl DisjointFromClause {
    #[new]
    fn __init__(term: Ident) -> PyClassInitializer<Self> {
        Self::new(term).into()
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the disjoint term.
    fn get_term(&self) -> &Ident {
        &self.term
    }

    fn raw_tag(&self) -> &str {
        "disjoint_from"
    }

    fn raw_value(&self) -> String {
        self.term.to_string()
    }
}

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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct RelationshipClause {
    #[pyo3(set)]
    typedef: Ident,
    #[pyo3(set)]
    term: Ident,
}

impl RelationshipClause {
    pub fn new(typedef: Ident, term: Ident) -> Self {
        Self { typedef, term }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for RelationshipClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::Relationship(
            Box::new(self.typedef.into_py(py)),
            Box::new(self.term.into_py(py)),
        )
    }
}

#[pymethods]
impl RelationshipClause {
    #[new]
    fn __init__(typedef: Ident, term: Ident) -> PyClassInitializer<Self> {
        Self::new(typedef, term).into()
    }

    #[getter]
    fn get_typedef<'py>(&self, py: Python<'py>) -> PyResult<Ident> {
        Ok(self.typedef.clone_py(py))
    }

    #[getter]
    fn get_term<'py>(&self, py: Python<'py>) -> PyResult<Ident> {
        Ok(self.term.clone_py(py))
    }

    fn raw_tag(&self) -> &str {
        "relationship"
    }

    fn raw_value(&self) -> String {
        format!("{} {}", self.typedef, self.term)
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
#[derive(Clone, ClonePy, Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct IsObsoleteClause {
    #[pyo3(get, set)]
    obsolete: bool,
}

impl IsObsoleteClause {
    pub fn new(obsolete: bool) -> Self {
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

impl IntoPy<fastobo::ast::TermClause> for IsObsoleteClause {
    fn into_py(self, _py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::from(self)
    }
}

#[pymethods]
impl IsObsoleteClause {
    #[new]
    fn __init__(obsolete: bool) -> PyClassInitializer<Self> {
        Self::new(obsolete).into()
    }

    fn raw_tag(&self) -> &str {
        "is_obsolete"
    }

    fn raw_value(&self) -> String {
        self.obsolete.to_string()
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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct ReplacedByClause {
    #[pyo3(set)]
    term: Ident,
}

impl ReplacedByClause {
    pub fn new(term: Ident) -> Self {
        Self { term }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for ReplacedByClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::ReplacedBy(Box::new(self.term.into_py(py)))
    }
}

#[pymethods]
impl ReplacedByClause {
    #[new]
    fn __init__(term: Ident) -> PyClassInitializer<Self> {
        Self::new(term).into()
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the replacement term.
    fn get_term(&self) -> &Ident {
        &self.term
    }

    fn raw_tag(&self) -> &str {
        "replaced_by"
    }

    fn raw_value(&self) -> String {
        self.term.to_string()
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
#[derive(Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct ConsiderClause {
    term: Ident,
}

impl ConsiderClause {
    pub fn new(term: Ident) -> Self {
        Self { term }
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
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl IntoPy<fastobo::ast::TermClause> for ConsiderClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::Consider(Box::new(self.term.into_py(py)))
    }
}

#[pymethods]
impl ConsiderClause {
    #[new]
    fn __init__(term: Ident) -> PyClassInitializer<Self> {
        Self::new(term).into()
    }

    #[getter]
    fn get_term(&self) -> &Ident {
        &self.term
    }

    fn raw_tag(&self) -> &str {
        "consider"
    }

    fn raw_value(&self) -> String {
        self.term.to_string()
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
#[derive(Clone, ClonePy, Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct CreatedByClause {
    creator: fastobo::ast::UnquotedString,
}

impl CreatedByClause {
    pub fn new(creator: fastobo::ast::UnquotedString) -> Self {
        Self { creator }
    }
}

impl Display for CreatedByClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let clause: fastobo::ast::TermClause = self.clone_py(py).into_py(py);
        clause.fmt(f)
    }
}

impl From<CreatedByClause> for fastobo::ast::TermClause {
    fn from(clause: CreatedByClause) -> Self {
        fastobo::ast::TermClause::CreatedBy(Box::new(clause.creator))
    }
}

impl IntoPy<fastobo::ast::TermClause> for CreatedByClause {
    fn into_py(self, _py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::from(self)
    }
}

#[pymethods]
impl CreatedByClause {
    #[new]
    fn __init__(creator: String) -> PyClassInitializer<Self> {
        Self::new(fastobo::ast::UnquotedString::new(creator)).into()
    }

    #[getter]
    /// `str`: the name of the creator of this term.
    fn get_creator(&self) -> &str {
        self.creator.as_str()
    }

    #[setter]
    fn set_creator(&mut self, creator: String) {
        self.creator = fastobo::ast::UnquotedString::new(creator);
    }

    fn raw_tag(&self) -> &str {
        "created_by"
    }

    fn raw_value(&self) -> String {
        self.creator.to_string()
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
/// A clause declaring the date (and optionally time) a term was created.
///
/// Arguments:
///     date (`datetime.date`): The date this term was created. If a
///         `datetime.datime` object is given, then the serialized value
///         will also include the serialized time.
///
/// Warning:
///     The timezone of the `datetime` will only be extracted down to the
///     minutes, seconds and smaller durations will be ignored. It is advised
///     to use `datetime.timezone.utc` whenever possible to preserve the
///     date and time properly.
///
/// Example:
///     >>> d1 = datetime.date(2021, 1, 23)
///     >>> print(fastobo.term.CreationDateClause(d1))
///     creation_date: 2021-01-23
///     >>> d2 = datetime.datetime(2021, 1, 23, tzinfo=datetime.timezone.utc)
///     >>> print(fastobo.term.CreationDateClause(d2))
///     creation_date: 2021-01-23T00:00:00Z
///
#[pyclass(extends=BaseTermClause, module="fastobo.term")]
#[derive(Clone, ClonePy, Debug, FinalClass)]
#[base(BaseTermClause)]
pub struct CreationDateClause {
    date: fastobo::ast::CreationDate,
}

impl CreationDateClause {
    pub fn new(date: fastobo::ast::CreationDate) -> Self {
        Self { date }
    }
}

impl Display for CreationDateClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TermClause::from(self.clone()).fmt(f)
    }
}

impl From<CreationDateClause> for fastobo::ast::TermClause {
    fn from(clause: CreationDateClause) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::CreationDate(Box::new(clause.date))
    }
}

impl IntoPy<fastobo::ast::TermClause> for CreationDateClause {
    fn into_py(self, py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::from(self)
    }
}

#[pymethods]
impl CreationDateClause {
    #[new]
    fn __init__(date: &PyAny) -> PyResult<PyClassInitializer<Self>> {
        let py = date.py();
        if let Ok(dt) = date.cast_as::<PyDateTime>() {
            let date = datetime_to_isodatetime(py, dt).map(From::from)?;
            Ok(CreationDateClause::new(date).into())
        } else {
            match date.cast_as::<PyDate>() {
                Err(e) => {
                    raise!(py, PyTypeError("expected datetime.date or datetime.datetime") from PyErr::from(e))
                }
                Ok(d) => {
                    let date = date_to_isodate(py, d).map(From::from)?;
                    Ok(CreationDateClause::new(date).into())
                }
            }
        }
    }

    #[getter]
    /// `datetime.datetime`: the date and time this term was created.
    fn get_date<'py>(&self, py: Python<'py>) -> PyResult<PyObject> {
        use fastobo::ast::CreationDate::*;
        match &self.date {
            DateTime(dt) => Ok(isodatetime_to_datetime(py, dt)?.to_object(py)),
            Date(d) => Ok(isodate_to_date(py, d)?.to_object(py)),
        }
    }

    #[setter]
    fn set_date(&mut self, datetime: &PyAny) -> PyResult<()> {
        let py = datetime.py();
        if let Ok(dt) = datetime.cast_as::<PyDateTime>() {
            self.date = From::from(datetime_to_isodatetime(py, dt)?);
        } else {
            match datetime.cast_as::<PyDate>() {
                Err(e) => {
                    raise!(py, PyTypeError("expected datetime.date or datetime.datetime") from PyErr::from(e))
                }
                Ok(d) => {
                    self.date = From::from(date_to_isodate(py, d)?);
                }
            }
        }
        Ok(())
    }

    fn raw_tag(&self) -> &str {
        "creation_date"
    }

    fn raw_value(&self) -> String {
        self.date.to_string()
    }
}

#[pyproto]
impl PyObjectProtocol for CreationDateClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let fmt = PyString::new(py, "CreationDateClause({!r})").to_object(py);
        self.get_date(py)
            .and_then(|dt| fmt.call_method1(py, "format", (dt,)))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.date)
    }
}
