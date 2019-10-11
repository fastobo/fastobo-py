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
use pyo3::types::PyDateTime;
use pyo3::types::PyString;
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
#[wraps(BaseTypedefClause)]
pub enum TypedefClause {
    IsAnonymous(Py<IsAnonymousClause>),
    Name(Py<NameClause>),
    Namespace(Py<NamespaceClause>),
    AltId(Py<AltIdClause>),
    Def(Py<DefClause>),
    Comment(Py<CommentClause>),
    Subset(Py<SubsetClause>),
    Synonym(Py<SynonymClause>),
    Xref(Py<XrefClause>),
    PropertyValue(Py<PropertyValueClause>),
    Domain(Py<DomainClause>),
    Range(Py<RangeClause>),
    Builtin(Py<BuiltinClause>),
    HoldsOverChain(Py<HoldsOverChainClause>),
    IsAntiSymmetric(Py<IsAntiSymmetricClause>),
    IsCyclic(Py<IsCyclicClause>),
    IsReflexive(Py<IsReflexiveClause>),
    IsSymmetric(Py<IsSymmetricClause>),
    IsAsymmetric(Py<IsAsymmetricClause>),
    IsTransitive(Py<IsTransitiveClause>),
    IsFunctional(Py<IsFunctionalClause>),
    IsInverseFunctional(Py<IsInverseFunctionalClause>),
    IsA(Py<IsAClause>),
    IntersectionOf(Py<IntersectionOfClause>),
    UnionOf(Py<UnionOfClause>),
    EquivalentTo(Py<EquivalentToClause>),
    DisjointFrom(Py<DisjointFromClause>),
    InverseOf(Py<InverseOfClause>),
    TransitiveOver(Py<TransitiveOverClause>),
    EquivalentToChain(Py<EquivalentToChainClause>),
    DisjointOver(Py<DisjointOverClause>),
    Relationship(Py<RelationshipClause>),
    IsObsolete(Py<IsObsoleteClause>),
    ReplacedBy(Py<ReplacedByClause>),
    Consider(Py<ConsiderClause>),
    CreatedBy(Py<CreatedByClause>),
    CreationDate(Py<CreationDateClause>),
    ExpandAssertionTo(Py<ExpandAssertionToClause>),
    ExpandExpressionTo(Py<ExpandExpressionToClause>),
    IsMetadataTag(Py<IsMetadataTagClause>),
    IsClassLevel(Py<IsClassLevelClause>),
}

// TODO
impl FromPy<fastobo::ast::TypedefClause> for TypedefClause {
    fn from_py(clause: fastobo::ast::TypedefClause, py: Python) -> Self {
        use fastobo::ast::TypedefClause::*;
        match clause {
            IsAnonymous(b) => {
                Py::new(py, IsAnonymousClause::new(py, b)).map(TypedefClause::IsAnonymous)
            }
            Name(n) => Py::new(py, NameClause::new(py, n)).map(TypedefClause::Name),
            Namespace(ns) => {
                Py::new(py, NamespaceClause::new(py, ns)).map(TypedefClause::Namespace)
            }
            AltId(id) => Py::new(py, AltIdClause::new(py, id)).map(TypedefClause::AltId),
            Def(desc, xrefs) => {
                Py::new(py, DefClause::new(py, desc, xrefs)).map(TypedefClause::Def)
            }
            Comment(c) => Py::new(py, CommentClause::new(py, c)).map(TypedefClause::Comment),
            Subset(s) => Py::new(py, SubsetClause::new(py, s)).map(TypedefClause::Subset),
            Synonym(s) => Py::new(py, SynonymClause::new(py, s)).map(TypedefClause::Synonym),
            Xref(x) => Py::new(py, XrefClause::new(py, x)).map(TypedefClause::Xref),
            PropertyValue(pv) => {
                Py::new(py, PropertyValueClause::new(py, pv)).map(TypedefClause::PropertyValue)
            }
            Domain(id) => Py::new(py, DomainClause::new(py, id)).map(TypedefClause::Domain),
            Range(id) => Py::new(py, RangeClause::new(py, id)).map(TypedefClause::Range),
            Builtin(b) => Py::new(py, BuiltinClause::new(py, b)).map(TypedefClause::Builtin),
            HoldsOverChain(r1, r2) => Py::new(py, HoldsOverChainClause::new(py, r1, r2))
                .map(TypedefClause::HoldsOverChain),
            IsAntiSymmetric(b) => {
                Py::new(py, IsAntiSymmetricClause::new(py, b)).map(TypedefClause::IsAntiSymmetric)
            }
            IsCyclic(b) => Py::new(py, IsCyclicClause::new(py, b)).map(TypedefClause::IsCyclic),
            IsReflexive(b) => {
                Py::new(py, IsReflexiveClause::new(py, b)).map(TypedefClause::IsReflexive)
            }
            IsSymmetric(b) => {
                Py::new(py, IsSymmetricClause::new(py, b)).map(TypedefClause::IsSymmetric)
            }
            IsAsymmetric(b) => {
                Py::new(py, IsAsymmetricClause::new(py, b)).map(TypedefClause::IsAsymmetric)
            }
            IsTransitive(b) => {
                Py::new(py, IsTransitiveClause::new(py, b)).map(TypedefClause::IsTransitive)
            }
            IsFunctional(b) => {
                Py::new(py, IsFunctionalClause::new(py, b)).map(TypedefClause::IsFunctional)
            }
            IsInverseFunctional(b) => Py::new(py, IsInverseFunctionalClause::new(py, b))
                .map(TypedefClause::IsInverseFunctional),
            IsA(id) => Py::new(py, IsAClause::new(py, id)).map(TypedefClause::IsA),
            IntersectionOf(r) => {
                Py::new(py, IntersectionOfClause::new(py, r)).map(TypedefClause::IntersectionOf)
            }
            UnionOf(cls) => Py::new(py, UnionOfClause::new(py, cls)).map(TypedefClause::UnionOf),
            EquivalentTo(cls) => {
                Py::new(py, EquivalentToClause::new(py, cls)).map(TypedefClause::EquivalentTo)
            }
            DisjointFrom(cls) => {
                Py::new(py, DisjointFromClause::new(py, cls)).map(TypedefClause::DisjointFrom)
            }
            TransitiveOver(r) => {
                Py::new(py, TransitiveOverClause::new(py, r)).map(TypedefClause::TransitiveOver)
            }
            EquivalentToChain(r1, r2) => Py::new(py, EquivalentToChainClause::new(py, r1, r2))
                .map(TypedefClause::EquivalentToChain),
            DisjointOver(r) => {
                Py::new(py, DisjointOverClause::new(py, r)).map(TypedefClause::DisjointOver)
            }
            InverseOf(cls) => {
                Py::new(py, InverseOfClause::new(py, cls)).map(TypedefClause::InverseOf)
            }
            Relationship(r, id) => {
                Py::new(py, RelationshipClause::new(py, r, id)).map(TypedefClause::Relationship)
            }
            IsObsolete(b) => {
                Py::new(py, IsObsoleteClause::new(py, b)).map(TypedefClause::IsObsolete)
            }
            ReplacedBy(id) => {
                Py::new(py, ReplacedByClause::new(py, id)).map(TypedefClause::ReplacedBy)
            }
            Consider(id) => Py::new(py, ConsiderClause::new(py, id)).map(TypedefClause::Consider),
            CreatedBy(name) => {
                Py::new(py, CreatedByClause::new(py, name)).map(TypedefClause::CreatedBy)
            }
            CreationDate(dt) => {
                Py::new(py, CreationDateClause::new(py, dt)).map(TypedefClause::CreationDate)
            }
            ExpandAssertionTo(d, xrefs) => Py::new(py, ExpandAssertionToClause::new(py, d, xrefs))
                .map(TypedefClause::ExpandAssertionTo),
            ExpandExpressionTo(d, xrefs) => {
                Py::new(py, ExpandExpressionToClause::new(py, d, xrefs))
                    .map(TypedefClause::ExpandExpressionTo)
            }
            IsMetadataTag(b) => {
                Py::new(py, IsMetadataTagClause::new(py, b)).map(TypedefClause::IsMetadataTag)
            }
            IsClassLevel(b) => {
                Py::new(py, IsClassLevelClause::new(py, b)).map(TypedefClause::IsClassLevel)
            }
        }
        .expect("could not allocate memory for `TypedefClause` in Python heap")
    }
}

// --- Base ------------------------------------------------------------------

#[pyclass(extends=AbstractEntityClause, module="fastobo.typedef")]
pub struct BaseTypedefClause {}

// --- IsAnonymous -----------------------------------------------------------

/// IsAnonymousClause(anonymous)
/// --
///
/// A clause declaring whether or not the relationship has an anonymous id.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<IsAnonymousClause> for fastobo::ast::TypedefClause {
    fn from(clause: IsAnonymousClause) -> Self {
        fastobo::ast::TypedefClause::IsAnonymous(clause.anonymous)
    }
}

impl FromPy<IsAnonymousClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsAnonymousClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::IsAnonymous(clause.anonymous)
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
/// A clause declaring the human-readable name of this relationship.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<NameClause> for fastobo::ast::TypedefClause {
    fn from(clause: NameClause) -> Self {
        fastobo::ast::TypedefClause::Name(clause.name)
    }
}

impl FromPy<NameClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: NameClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::from(clause)
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
    fn get_name(&self) -> PyResult<&str> {
        Ok(self.name.as_str())
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
/// A term clause declaring the namespace of this relationship.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<NamespaceClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: NamespaceClause, py: Python) -> Self {
        let ns = fastobo::ast::NamespaceIdent::from_py(clause.namespace, py);
        fastobo::ast::TypedefClause::Namespace(ns)
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
    fn get_namespace(&self) -> PyResult<&Ident> {
        Ok(&self.namespace)
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
/// A clause defines an alternate id for this relationship.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct AltIdClause {
    #[pyo3(set)]
    alt_id: Ident,
}

impl AltIdClause {
    pub fn new<I>(py: Python, alt_id: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self { alt_id: alt_id.into_py(py) }
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
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<AltIdClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: AltIdClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::AltId(clause.alt_id.into_py(py))
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
    fn get_alt_id(&self) -> PyResult<&Ident> {
        Ok(&self.alt_id)
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
/// A clause giving a human-readable definition of the relationship.
///
/// Arguments:
///     definition (str): The human-readable textual definition of the
///         current relationship.
///     xrefs (~typing.Iterable[~fastobo.xref.Xref], optional): An iterable
///         of database cross-references describing the origin of the
///         definition, or `None`.
///
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<DefClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: DefClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::Def(clause.definition, clause.xrefs.into_py(py))
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
    fn get_definition(&self) -> PyResult<&str> {
        Ok(&self.definition.as_str())
    }

    #[setter]
    fn set_definition(&mut self, definition: String) {
        self.definition = fastobo::ast::QuotedString::new(definition);
    }

    #[getter]
    /// `~fastobo.xrefs.XrefList`: a list of xrefs supporting the definition.
    fn get_xrefs<'py>(&self, py: Python<'py>) -> PyResult<XrefList> {
        Ok(self.xrefs.clone_py(py))
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
/// A clause storing a comment for this relationship.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<CommentClause> for fastobo::ast::TypedefClause {
    fn from(clause: CommentClause) -> Self {
        fastobo::ast::TypedefClause::Comment(clause.comment)
    }
}

impl FromPy<CommentClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: CommentClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::Comment(clause.comment)
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
    fn get_comment(&self) -> PyResult<&str> {
        Ok(self.comment.as_str())
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
/// A clause declaring a subset to which this relationship belongs.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<SubsetClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: SubsetClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::Subset(clause.subset.into_py(py))
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
    fn get_subset(&self) -> PyResult<&Ident> {
        Ok(&self.subset)
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
/// A clause giving a synonym for this relation, with some cross-references.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<SynonymClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: SynonymClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::Synonym(
            clause.synonym.as_gil_ref(py).clone_py(py).into_py(py)
        )
    }
}

#[pymethods]
impl SynonymClause {
    #[new]
    fn __init__(obj: &PyRawObject, synonym: &Synonym) {
        let s = synonym.clone_py(obj.py());
        obj.init(Self::new(obj.py(), s));
    }

    #[getter]
    /// `~fastobo.syn.Synonym`: a possible synonym for this term.
    fn get_synonym<'py>(&self, py: Python<'py>) -> PyResult<Py<Synonym>> {
        Ok(self.synonym.clone_py(py))
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
        let py = gil.python();
        Ok(format!("{}", self.synonym.as_gil_ref(py)))
    }
}

impl_raw_tag!(SynonymClause, "synonym");

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
/// A cross-reference describing an analogous relation in another vocabulary.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<XrefClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: XrefClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::Xref(clause.xref.as_ref(py).clone_py(py).into_py(py))
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
    fn get_xref<'py>(&self, py: Python<'py>) -> PyResult<Py<Xref>> {
        Ok(self.xref.clone_ref(py))
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

// --- PropertyValue ---------------------------------------------------------

/// PropertyValueClause(property_value)
/// --
///
/// A clause that binds a property to a value in the relationship.
///
/// Arguments:
///     property_value (~fastobo.pv.AbstractPropertyValue): the property value
///         to annotate the current relationship.
///
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<PropertyValueClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: PropertyValueClause, py: Python) -> ast::TypedefClause {
        ast::TypedefClause::PropertyValue(clause.inner.into_py(py))
    }
}

#[pymethods]
impl PropertyValueClause {
    #[new]
    pub fn __init__(obj: &PyRawObject, pv: PropertyValue) {
        obj.init(Self::new(obj.py(), pv));
    }

    #[getter]
    /// `~fastobo.pv.AbstractPropertyValue`: an annotation of the relation.
    pub fn property_value(&self) -> &PropertyValue {
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

// --- Domain ----------------------------------------------------------------

/// DomainClause(domain)
/// --
///
/// A clause declaring the domain of the relationship, if any.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct DomainClause {
    #[pyo3(set)]
    domain: Ident,
}

impl DomainClause {
    pub fn new<I>(py: Python, domain: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            domain: domain.into_py(py),
        }
    }
}

impl ClonePy for DomainClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            domain: self.domain.clone_py(py),
        }
    }
}

impl Display for DomainClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<DomainClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: DomainClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::Domain(clause.domain.into_py(py))
    }
}

#[pymethods]
impl DomainClause {
    #[new]
    fn __init__(obj: &PyRawObject, domain: Ident) {
        obj.init(Self::new(obj.py(), domain));
    }

    /// `~fastobo.id.Ident`: the identifier of the domain of the relation.
    #[getter]
    fn get_domain(&self) -> &Ident {
        &self.domain
    }
}

impl_raw_tag!(DomainClause, "domain");
impl_raw_value!(DomainClause, "{}", self.domain);

#[pyproto]
impl PyObjectProtocol for DomainClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, DomainClause(self.domain))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.domain)
    }
}

// --- Range -----------------------------------------------------------------

/// RangeClause(range)
/// --
///
/// A clause declaring the range of the relationship, if any.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct RangeClause {
    #[pyo3(set)]
    range: Ident,
}

impl RangeClause {
    pub fn new<I>(py: Python, range: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            range: range.into_py(py),
        }
    }
}

impl ClonePy for RangeClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            range: self.range.clone_py(py),
        }
    }
}

impl Display for RangeClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<RangeClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: RangeClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::Range(clause.range.into_py(py))
    }
}

#[pymethods]
impl RangeClause {
    #[new]
    fn __init__(obj: &PyRawObject, range: Ident) {
        obj.init(Self::new(obj.py(), range));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the range of the typedef.
    fn get_range(&self) -> &Ident {
        &self.range
    }
}

impl_raw_tag!(RangeClause, "range");
impl_raw_value!(RangeClause, "{}", self.range);

#[pyproto]
impl PyObjectProtocol for RangeClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, RangeClause(self.range))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.range)
    }
}

// --- Builtin ---------------------------------------------------------------

/// BuiltinClause(builtin)
/// --
///
/// A clause declaring whether this relation is built-in to the OBO format.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<BuiltinClause> for fastobo::ast::TypedefClause {
    fn from(clause: BuiltinClause) -> Self {
        fastobo::ast::TypedefClause::Builtin(clause.builtin)
    }
}

impl FromPy<BuiltinClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: BuiltinClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::from(clause)
    }
}

#[pymethods]
impl BuiltinClause {
    #[new]
    fn __init__(obj: &PyRawObject, builtin: bool) {
        obj.init(Self::new(obj.py(), builtin));
    }

    /// `bool`: ``True`` if the relationship is built in the OBO format.
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

// --- HoldsOverChain --------------------------------------------------------

/// HoldsOverChainClause(first, last)
/// --
///
/// An extension of the `transitive_over` tag for property chains.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct HoldsOverChainClause {
    #[pyo3(set)]
    first: Ident,
    #[pyo3(set)]
    last: Ident,
}

impl HoldsOverChainClause {
    pub fn new<R1, R2>(py: Python, first: R1, last: R2) -> Self
    where
        R1: IntoPy<Ident>,
        R2: IntoPy<Ident>,
    {
        Self {
            first: first.into_py(py),
            last: last.into_py(py),
        }
    }
}

impl ClonePy for HoldsOverChainClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            first: self.first.clone_py(py),
            last: self.last.clone_py(py),
        }
    }
}

impl Display for HoldsOverChainClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<HoldsOverChainClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: HoldsOverChainClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::HoldsOverChain(
            clause.first.into_py(py),
            clause.last.into_py(py),
        )
    }
}

#[pymethods]
impl HoldsOverChainClause {
    #[new]
    fn __init__(obj: &PyRawObject, first: Ident, last: Ident) {
        obj.init(Self::new(obj.py(), first, last));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the first typedef of the chain.
    fn get_first(&self) -> &Ident {
        &self.first
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the last typedef of the chain.
    fn get_last(&self) -> &Ident {
        &self.last
    }
}

impl_raw_tag!(HoldsOverChainClause, "holds_over_chain");
impl_raw_value!(HoldsOverChainClause, "{} {}", self.first, self.last);

#[pyproto]
impl PyObjectProtocol for HoldsOverChainClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, HoldsOverChainClause(self.first, self.last))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.first && self.last)
    }
}

// --- IsAntiSymmetric -------------------------------------------------------

/// IsAntiSymmetricClause(anti_symmetric)
/// --
///
/// A clause declaring whether the relationship if anti-symmetric or not.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsAntiSymmetricClause {
    #[pyo3(set)]
    anti_symmetric: bool,
}

impl IsAntiSymmetricClause {
    pub fn new(_py: Python, anti_symmetric: bool) -> Self {
        Self { anti_symmetric }
    }
}

impl Display for IsAntiSymmetricClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<IsAntiSymmetricClause> for fastobo::ast::TypedefClause {
    fn from(clause: IsAntiSymmetricClause) -> Self {
        fastobo::ast::TypedefClause::IsAntiSymmetric(clause.anti_symmetric)
    }
}

impl FromPy<IsAntiSymmetricClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsAntiSymmetricClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::from(clause)
    }
}

#[pymethods]
impl IsAntiSymmetricClause {
    #[new]
    fn __init__(obj: &PyRawObject, anti_symmetric: bool) {
        obj.init(Self::new(obj.py(), anti_symmetric));
    }

    #[getter]
    fn get_anti_symmetric(&self) -> bool {
        self.anti_symmetric
    }
}

impl_raw_tag!(IsAntiSymmetricClause, "is_anti_symmetric");
impl_raw_value!(IsAntiSymmetricClause, "{}", self.anti_symmetric);

#[pyproto]
impl PyObjectProtocol for IsAntiSymmetricClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsAntiSymmetricClause(self.anti_symmetric))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.anti_symmetric)
    }
}

// --- IsCyclic --------------------------------------------------------------

/// IsCyclicClause(cyclic)
/// --
///
/// A clause declaring whether the relationship if cyclic or not.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsCyclicClause {
    #[pyo3(set)]
    cyclic: bool,
}

impl IsCyclicClause {
    pub fn new(_py: Python, cyclic: bool) -> Self {
        Self { cyclic }
    }
}

impl Display for IsCyclicClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<IsCyclicClause> for fastobo::ast::TypedefClause {
    fn from(clause: IsCyclicClause) -> Self {
        fastobo::ast::TypedefClause::IsCyclic(clause.cyclic)
    }
}

impl FromPy<IsCyclicClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsCyclicClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::from(clause)
    }
}

#[pymethods]
impl IsCyclicClause {
    #[new]
    fn __init__(obj: &PyRawObject, cyclic: bool) {
        obj.init(Self::new(obj.py(), cyclic));
    }

    #[getter]
    fn get_cyclic(&self) -> bool {
        self.cyclic
    }
}

impl_raw_tag!(IsCyclicClause, "is_cyclic");
impl_raw_value!(IsCyclicClause, "{}", self.cyclic);

#[pyproto]
impl PyObjectProtocol for IsCyclicClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsCyclicClause(self.cyclic))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.cyclic)
    }
}

// --- IsReflexive -----------------------------------------------------------

/// IsReflexiveClause(reflexive)
/// --
///
/// A clause declaring whether the relationship if reflexive or not.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsReflexiveClause {
    #[pyo3(set)]
    reflexive: bool,
}

impl IsReflexiveClause {
    pub fn new(_py: Python, reflexive: bool) -> Self {
        Self { reflexive }
    }
}

impl Display for IsReflexiveClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<IsReflexiveClause> for fastobo::ast::TypedefClause {
    fn from(clause: IsReflexiveClause) -> Self {
        fastobo::ast::TypedefClause::IsReflexive(clause.reflexive)
    }
}

impl FromPy<IsReflexiveClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsReflexiveClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::from(clause)
    }
}

#[pymethods]
impl IsReflexiveClause {
    #[new]
    fn __init__(obj: &PyRawObject, reflexive: bool) {
        obj.init(Self::new(obj.py(), reflexive));
    }

    #[getter]
    fn get_reflexive(&self) -> bool {
        self.reflexive
    }
}

impl_raw_tag!(IsReflexiveClause, "is_reflexive");
impl_raw_value!(IsReflexiveClause, "{}", self.reflexive);

#[pyproto]
impl PyObjectProtocol for IsReflexiveClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsReflexiveClause(self.reflexive))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.reflexive)
    }
}

// --- IsSymmetric -----------------------------------------------------------

/// IsSymmetricClause(symmetric)
/// --
///
/// A clause declaring whether the relationship if symmetric or not.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsSymmetricClause {
    symmetric: bool,
}

impl IsSymmetricClause {
    pub fn new(_py: Python, symmetric: bool) -> Self {
        Self { symmetric }
    }
}

impl Display for IsSymmetricClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<IsSymmetricClause> for fastobo::ast::TypedefClause {
    fn from(clause: IsSymmetricClause) -> Self {
        fastobo::ast::TypedefClause::IsSymmetric(clause.symmetric)
    }
}

impl FromPy<IsSymmetricClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsSymmetricClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::from(clause)
    }
}

#[pymethods]
impl IsSymmetricClause {
    #[new]
    fn __init__(obj: &PyRawObject, symmetric: bool) {
        obj.init(Self::new(obj.py(), symmetric));
    }

    #[getter]
    fn get_symmetric(&self) -> bool {
        self.symmetric
    }
}

impl_raw_tag!(IsSymmetricClause, "is_symmetric");
impl_raw_value!(IsSymmetricClause, "{}", self.symmetric);

#[pyproto]
impl PyObjectProtocol for IsSymmetricClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsSymmetricClause(self.symmetric))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.symmetric)
    }
}

// --- IsAsymmetric -----------------------------------------------------------

/// IsAsymmetricClause(asymmetric)
/// --
///
/// A clause declaring whether the relationship is asymmetric or not.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsAsymmetricClause {
    #[pyo3(set)]
    asymmetric: bool,
}

impl IsAsymmetricClause {
    pub fn new(_py: Python, asymmetric: bool) -> Self {
        Self { asymmetric }
    }
}

impl Display for IsAsymmetricClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<IsAsymmetricClause> for fastobo::ast::TypedefClause {
    fn from(clause: IsAsymmetricClause) -> Self {
        fastobo::ast::TypedefClause::IsAsymmetric(clause.asymmetric)
    }
}

impl FromPy<IsAsymmetricClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsAsymmetricClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::from(clause)
    }
}

#[pymethods]
impl IsAsymmetricClause {
    #[new]
    fn __init__(obj: &PyRawObject, asymmetric: bool) {
        obj.init(Self::new(obj.py(), asymmetric));
    }

    #[getter]
    fn get_asymmetric(&self) -> bool {
        self.asymmetric
    }
}

impl_raw_tag!(IsAsymmetricClause, "is_asymmetric");
impl_raw_value!(IsAsymmetricClause, "{}", self.asymmetric);

#[pyproto]
impl PyObjectProtocol for IsAsymmetricClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsAsymmetricClause(self.asymmetric))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.asymmetric)
    }
}

// --- IsTransitive ----------------------------------------------------------

/// IsTransitiveClause(transitive)
/// --
///
/// A clause declaring whether the relationship if transitive or not.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsTransitiveClause {
    #[pyo3(set)]
    transitive: bool,
}

impl IsTransitiveClause {
    pub fn new(_py: Python, transitive: bool) -> Self {
        Self { transitive }
    }
}

impl Display for IsTransitiveClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<IsTransitiveClause> for fastobo::ast::TypedefClause {
    fn from(clause: IsTransitiveClause) -> Self {
        fastobo::ast::TypedefClause::IsTransitive(clause.transitive)
    }
}

impl FromPy<IsTransitiveClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsTransitiveClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::from(clause)
    }
}

#[pymethods]
impl IsTransitiveClause {
    #[new]
    fn __init__(obj: &PyRawObject, transitive: bool) {
        obj.init(Self::new(obj.py(), transitive));
    }

    #[getter]
    fn get_transitive(&self) -> bool {
        self.transitive
    }
}

impl_raw_tag!(IsTransitiveClause, "is_transitive");
impl_raw_value!(IsTransitiveClause, "{}", self.transitive);

#[pyproto]
impl PyObjectProtocol for IsTransitiveClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsTransitiveClause(self.transitive))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.transitive)
    }
}

// --- IsFunctional ----------------------------------------------------------

/// IsFunctionalClause(functional)
/// --
///
/// A clause declaring whether the relationship if functional or not.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsFunctionalClause {
    #[pyo3(set)]
    functional: bool,
}

impl IsFunctionalClause {
    pub fn new(_py: Python, functional: bool) -> Self {
        Self { functional }
    }
}

impl Display for IsFunctionalClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<IsFunctionalClause> for fastobo::ast::TypedefClause {
    fn from(clause: IsFunctionalClause) -> Self {
        fastobo::ast::TypedefClause::IsFunctional(clause.functional)
    }
}

impl FromPy<IsFunctionalClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsFunctionalClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::from(clause)
    }
}

#[pymethods]
impl IsFunctionalClause {
    #[new]
    fn __init__(obj: &PyRawObject, functional: bool) {
        obj.init(Self::new(obj.py(), functional));
    }

    #[getter]
    fn get_functional(&self) -> bool {
        self.functional
    }
}

impl_raw_tag!(IsFunctionalClause, "is_functional");
impl_raw_value!(IsFunctionalClause, "{}", self.functional);

#[pyproto]
impl PyObjectProtocol for IsFunctionalClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsFunctionalClause(self.functional))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.functional)
    }
}

// --- IsInverseFunctional ---------------------------------------------------

/// IsInverseFunctionalClause(functional)
/// --
///
/// A clause declaring whether the relationship if inverse-functional or not.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsInverseFunctionalClause {
    #[pyo3(set)]
    inverse_functional: bool,
}

impl IsInverseFunctionalClause {
    pub fn new(_py: Python, inverse_functional: bool) -> Self {
        Self { inverse_functional }
    }
}

impl Display for IsInverseFunctionalClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<IsInverseFunctionalClause> for fastobo::ast::TypedefClause {
    fn from(clause: IsInverseFunctionalClause) -> Self {
        fastobo::ast::TypedefClause::IsInverseFunctional(clause.inverse_functional)
    }
}

impl FromPy<IsInverseFunctionalClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsInverseFunctionalClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::from(clause)
    }
}

#[pymethods]
impl IsInverseFunctionalClause {
    #[new]
    fn __init__(obj: &PyRawObject, inverse_functional: bool) {
        obj.init(Self::new(obj.py(), inverse_functional));
    }

    #[getter]
    fn get_inverse_functional(&self) -> bool {
        self.inverse_functional
    }
}

impl_raw_tag!(IsInverseFunctionalClause, "is_inverse_functional");
impl_raw_value!(IsInverseFunctionalClause, "{}", self.inverse_functional);

#[pyproto]
impl PyObjectProtocol for IsInverseFunctionalClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsInverseFunctionalClause(self.inverse_functional))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.inverse_functional)
    }
}

// --- IsA -------------------------------------------------------------------

/// IsAClause(typedef)
/// --
///
/// A clause declaring this relation is a subproperty of another relation.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct IsAClause {
    #[pyo3(set)]
    typedef: Ident,
}

impl IsAClause {
    pub fn new<I>(py: Python, typedef: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self { typedef: typedef.into_py(py) }
    }
}

impl Display for IsAClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl ClonePy for IsAClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
        }
    }
}

impl FromPy<IsAClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsAClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::IsA(clause.typedef.into_py(py))
    }
}

#[pymethods]
impl IsAClause {
    #[new]
    fn __init__(obj: &PyRawObject, typedef: Ident) {
        obj.init(Self::new(obj.py(), typedef));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the parent term.
    fn get_typedef(&self) -> &Ident {
        &self.typedef
    }
}

impl_raw_tag!(IsAClause, "is_a");
impl_raw_value!(IsAClause, "{}", self.typedef);

#[pyproto]
impl PyObjectProtocol for IsAClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsAClause(self.typedef))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef)
    }
}

// --- IntersectionOf --------------------------------------------------------

/// IntersectionOfClause(typedef)
/// --
///
/// Declares this relation is equivalent to the intersection of other relations.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct IntersectionOfClause {
    #[pyo3(set)]
    typedef: Ident,
}

impl IntersectionOfClause {
    pub fn new<R>(py: Python, typedef: R) -> Self
    where
        R: IntoPy<Ident>,
    {
        Self {
            typedef: typedef.into_py(py),
        }
    }
}

impl ClonePy for IntersectionOfClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
        }
    }
}

impl Display for IntersectionOfClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<IntersectionOfClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IntersectionOfClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::IntersectionOf(clause.typedef.into_py(py))
    }
}

#[pymethods]
impl IntersectionOfClause {
    #[new]
    fn __init__(obj: &PyRawObject, typedef: Ident) {
        obj.init(Self::new(obj.py(), typedef))
    }

    #[getter]
    fn get_typedef(&self) -> &Ident {
        &self.typedef
    }
}

impl_raw_tag!(IntersectionOfClause, "intersection_of");
impl_raw_value!(IntersectionOfClause, "{}", self.typedef);

#[pyproto]
impl PyObjectProtocol for IntersectionOfClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IntersectionOfClause(self.typedef))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef)
    }
}

// --- UnionOf ---------------------------------------------------------------

/// UnionOfClause(typedef)
/// --
///
/// Declares the relation represents the union of several other relations.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct UnionOfClause {
    #[pyo3(set)]
    typedef: Ident,
}

impl UnionOfClause {
    pub fn new<I>(py: Python, typedef: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            typedef: typedef.into_py(py),
        }
    }
}

impl ClonePy for UnionOfClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
        }
    }
}

impl Display for UnionOfClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<UnionOfClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: UnionOfClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::UnionOf(clause.typedef.into_py(py))
    }
}

#[pymethods]
impl UnionOfClause {
    #[new]
    fn __init__(obj: &PyRawObject, typedef: Ident) {
        obj.init(Self::new(obj.py(), typedef));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the member typedef.
    fn get_typedef(&self) -> &Ident {
        &self.typedef
    }
}

impl_raw_tag!(UnionOfClause, "union_of");
impl_raw_value!(UnionOfClause, "{}", self.typedef);

#[pyproto]
impl PyObjectProtocol for UnionOfClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, UnionOfClause(self.typedef))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef)
    }
}

// --- EquivalentTo ----------------------------------------------------------

/// EquivalentToClause(typedef)
/// --
///
/// A clause indicating the relation is exactly equivalent to another relation.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct EquivalentToClause {
    #[pyo3(set)]
    typedef: Ident,
}

impl EquivalentToClause {
    pub fn new<I>(py: Python, typedef: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            typedef: typedef.into_py(py),
        }
    }
}

impl ClonePy for EquivalentToClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
        }
    }
}

impl Display for EquivalentToClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<EquivalentToClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: EquivalentToClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::EquivalentTo(clause.typedef.into_py(py))
    }
}

#[pymethods]
impl EquivalentToClause {
    #[new]
    fn __init__(obj: &PyRawObject, id: Ident) {
        obj.init(Self::new(obj.py(), id));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the equivalent typedef.
    fn get_typedef(&self) -> &Ident {
        &self.typedef
    }
}

impl_raw_tag!(EquivalentToClause, "equivalent_to");
impl_raw_value!(EquivalentToClause, "{}", self.typedef);

#[pyproto]
impl PyObjectProtocol for EquivalentToClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, EquivalentToClause(self.typedef))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef)
    }
}

// --- DisjointFrom ----------------------------------------------------------

/// DisjointFromClause(typedef)
/// --
///
/// A clause stating is disjoint from another relationship.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct DisjointFromClause {
    #[pyo3(set)]
    typedef: Ident,
}

impl DisjointFromClause {
    pub fn new<I>(py: Python, typedef: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            typedef: typedef.into_py(py),
        }
    }
}

impl ClonePy for DisjointFromClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
        }
    }
}

impl Display for DisjointFromClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<DisjointFromClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: DisjointFromClause, py: Python) -> Self {
        ast::TypedefClause::DisjointFrom(clause.typedef.into_py(py))
    }
}

#[pymethods]
impl DisjointFromClause {
    #[new]
    fn __init__(obj: &PyRawObject, typedef: Ident) {
        obj.init(Self::new(obj.py(), typedef));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the disjoint typedef.
    fn get_typedef(&self) -> &Ident {
        &self.typedef
    }
}

impl_raw_tag!(DisjointFromClause, "disjoint_from");
impl_raw_value!(DisjointFromClause, "{}", self.typedef);

#[pyproto]
impl PyObjectProtocol for DisjointFromClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, DisjointFromClause(self.typedef))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef)
    }
}

// --- InverseOf -------------------------------------------------------------

/// InverseOfClause(typedef)
/// --
///
/// A clause declaring the inverse of this relationship type.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct InverseOfClause {
    #[pyo3(set)]
    typedef: Ident,
}

impl InverseOfClause {
    pub fn new<R>(py: Python, typedef: R) -> Self
    where
        R: IntoPy<Ident>,
    {
        Self {
            typedef: typedef.into_py(py),
        }
    }
}

impl ClonePy for InverseOfClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
        }
    }
}

impl Display for InverseOfClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<InverseOfClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: InverseOfClause, py: Python) -> Self {
        ast::TypedefClause::InverseOf(clause.typedef.into_py(py))
    }
}

#[pymethods]
impl InverseOfClause {
    #[new]
    fn __init__(obj: &PyRawObject, typedef: Ident) {
        obj.init(Self::new(obj.py(), typedef));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the inverse relationship.
    fn get_typedef(&self) -> &Ident {
        &self.typedef
    }
}

impl_raw_tag!(InverseOfClause, "inverse_of");
impl_raw_value!(InverseOfClause, "{}", self.typedef);

#[pyproto]
impl PyObjectProtocol for InverseOfClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, InverseOfClause(self.typedef))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef)
    }
}

// --- TransitiveOver --------------------------------------------------------

/// TransitiveOverClause(typedef)
/// --
///
/// A clause declaring another relation that this relation is transitive over.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct TransitiveOverClause {
    #[pyo3(set)]
    typedef: Ident,
}

impl TransitiveOverClause {
    pub fn new<R>(py: Python, typedef: R) -> Self
    where
        R: IntoPy<Ident>,
    {
        Self {
            typedef: typedef.into_py(py),
        }
    }
}

impl ClonePy for TransitiveOverClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
        }
    }
}

impl Display for TransitiveOverClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<TransitiveOverClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: TransitiveOverClause, py: Python) -> Self {
        ast::TypedefClause::TransitiveOver(clause.typedef.into_py(py))
    }
}

#[pymethods]
impl TransitiveOverClause {
    #[new]
    fn __init__(obj: &PyRawObject, typedef: Ident) {
        obj.init(Self::new(obj.py(), typedef));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the transitive relationship.
    fn get_typedef(&self) -> &Ident {
        &self.typedef
    }
}

impl_raw_tag!(TransitiveOverClause, "transitive_over");
impl_raw_value!(TransitiveOverClause, "{}", self.typedef);

#[pyproto]
impl PyObjectProtocol for TransitiveOverClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, TransitiveOverClause(self.typedef))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef)
    }
}

// --- EquivalentToChain -----------------------------------------------------

/// EquivalentToChainClause(first, last)
/// --
///
/// A clause declaring a property chain this relationship is equivalent to.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct EquivalentToChainClause {
    #[pyo3(set)]
    first: Ident,
    #[pyo3(set)]
    last: Ident,
}

impl EquivalentToChainClause {
    pub fn new<R1, R2>(py: Python, first: R1, last: R2) -> Self
    where
        R1: IntoPy<Ident>,
        R2: IntoPy<Ident>,
    {
        Self {
            first: first.into_py(py),
            last: last.into_py(py),
        }
    }
}

impl ClonePy for EquivalentToChainClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            first: self.first.clone_py(py),
            last: self.last.clone_py(py),
        }
    }
}

impl Display for EquivalentToChainClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<EquivalentToChainClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: EquivalentToChainClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::EquivalentToChain(
            clause.first.into_py(py),
            clause.last.into_py(py),
        )
    }
}

#[pymethods]
impl EquivalentToChainClause {
    #[new]
    fn __init__(obj: &PyRawObject, first: Ident, last: Ident) {
        obj.init(Self::new(obj.py(), first, last));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the first typedef of the chain.
    fn get_first(&self) -> &Ident {
        &self.first
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the last typedef of the chain.
    fn get_last(&self) -> &Ident {
        &self.last
    }
}

impl_raw_tag!(EquivalentToChainClause, "equivalent_to_chain");
impl_raw_value!(EquivalentToChainClause, "{} {}", self.first, self.last);

#[pyproto]
impl PyObjectProtocol for EquivalentToChainClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, EquivalentToChainClause(self.first, self.last))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.first && self.last)
    }
}

// --- DisjointOver ----------------------------------------------------------

/// DisjointOverClause(typedef)
/// --
///
/// A clause declaring a relationship this relationship is disjoint over.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct DisjointOverClause {
    #[pyo3(set)]
    typedef: Ident,
}

impl DisjointOverClause {
    pub fn new<R>(py: Python, typedef: R) -> Self
    where
        R: IntoPy<Ident>,
    {
        Self {
            typedef: typedef.into_py(py),
        }
    }
}

impl ClonePy for DisjointOverClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
        }
    }
}

impl Display for DisjointOverClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<DisjointOverClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: DisjointOverClause, py: Python) -> Self {
        ast::TypedefClause::DisjointOver(clause.typedef.into_py(py))
    }
}

#[pymethods]
impl DisjointOverClause {
    #[new]
    fn __init__(obj: &PyRawObject, typedef: Ident) {
        obj.init(Self::new(obj.py(), typedef));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the disjoint relationship.
    fn get_typedef(&self) -> &Ident {
        &self.typedef
    }
}

impl_raw_tag!(DisjointOverClause, "disjoint_over");
impl_raw_value!(DisjointOverClause, "{}", self.typedef);

#[pyproto]
impl PyObjectProtocol for DisjointOverClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, DisjointOverClause(self.typedef))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef)
    }
}

// --- Relationship ----------------------------------------------------------

/// RelationshipClause(typedef, target)
/// --
///
/// A clause declaring a relationship this relation has to another relation.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct RelationshipClause {
    #[pyo3(set)]
    typedef: Ident,
    #[pyo3(set)]
    target: Ident,
}

impl RelationshipClause {
    pub fn new<R, T>(py: Python, typedef: R, target: T) -> Self
    where
        R: IntoPy<Ident>,
        T: IntoPy<Ident>,
    {
        Self {
            typedef: typedef.into_py(py),
            target: target.into_py(py),
        }
    }
}

impl ClonePy for RelationshipClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
            target: self.target.clone_py(py),
        }
    }
}

impl Display for RelationshipClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<RelationshipClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: RelationshipClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::Relationship(clause.typedef.into_py(py), clause.target.into_py(py))
    }
}

impl_raw_tag!(RelationshipClause, "relationship");
impl_raw_value!(RelationshipClause, "{} {}", self.typedef, self.target);

#[pymethods]
impl RelationshipClause {
    #[new]
    fn __init__(obj: &PyRawObject, typedef: Ident, target: Ident) {
        obj.init(Self::new(obj.py(), typedef, target));
    }

    #[getter]
    fn get_typedef(&self) -> &Ident {
        &self.typedef
    }

    #[getter]
    fn get_target(&self) -> &Ident {
        &self.target
    }
}

#[pyproto]
impl PyObjectProtocol for RelationshipClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, RelationshipClause(self.typedef, self.target))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef && self.target)
    }
}

// --- IsObsolete ------------------------------------------------------------

/// IsObsoleteClause(obsolete)
/// --
///
/// A clause indicating whether or not this relationship is obsolete.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<IsObsoleteClause> for fastobo::ast::TypedefClause {
    fn from(clause: IsObsoleteClause) -> Self {
        fastobo::ast::TypedefClause::IsObsolete(clause.obsolete)
    }
}

impl FromPy<IsObsoleteClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsObsoleteClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::from(clause)
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

/// ReplacedByClause(typedef)
/// --
///
/// A clause giving a relation which replaces this obsolete relation.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct ReplacedByClause {
    #[pyo3(set)]
    typedef: Ident,
}

impl ReplacedByClause {
    pub fn new<I>(py: Python, typedef: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            typedef: typedef.into_py(py),
        }
    }
}

impl ClonePy for ReplacedByClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
        }
    }
}

impl Display for ReplacedByClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<ReplacedByClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: ReplacedByClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::ReplacedBy(clause.typedef.into_py(py))
    }
}

impl_raw_tag!(ReplacedByClause, "replaced_by");
impl_raw_value!(ReplacedByClause, "{}", self.typedef);

#[pymethods]
impl ReplacedByClause {
    #[new]
    fn __init__(obj: &PyRawObject, typedef: Ident) {
        obj.init(Self::new(obj.py(), typedef));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the replacing relationship.
    fn get_typedef(&self) -> &Ident {
        &self.typedef
    }
}

#[pyproto]
impl PyObjectProtocol for ReplacedByClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, ReplacedByClause(self.typedef))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef)
    }
}

// --- Consider --------------------------------------------------------------

/// ConsiderClause(typedef)
/// --
///
/// A clause giving a potential substitute for an obsolete typedef.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct ConsiderClause {
    #[pyo3(set)]
    typedef: Ident,
}

impl ConsiderClause {
    pub fn new<I>(py: Python, typedef: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self {
            typedef: typedef.into_py(py),
        }
    }
}

impl ClonePy for ConsiderClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            typedef: self.typedef.clone_py(py),
        }
    }
}

impl Display for ConsiderClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<ConsiderClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: ConsiderClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::Consider(clause.typedef.into_py(py))
    }
}

impl_raw_tag!(ConsiderClause, "consider");
impl_raw_value!(ConsiderClause, "{}", self.typedef);

#[pymethods]
impl ConsiderClause {
    #[new]
    fn __init__(obj: &PyRawObject, typedef: Ident) {
        obj.init(Self::new(obj.py(), typedef));
    }

    #[getter]
    /// `~fastobo.id.Ident`: the identifier of the relationship to consider.
    fn get_typedef(&self) -> &Ident {
        &self.typedef
    }
}


#[pyproto]
impl PyObjectProtocol for ConsiderClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, ConsiderClause(self.typedef))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.typedef)
    }
}

// --- CreatedBy -------------------------------------------------------------

/// CreatedByClause(creator)
/// --
///
/// A term clause stating the name of the creator of this relationship.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<CreatedByClause> for fastobo::ast::TypedefClause {
    fn from(clause: CreatedByClause) -> Self {
        fastobo::ast::TypedefClause::CreatedBy(clause.creator)
    }
}

impl FromPy<CreatedByClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: CreatedByClause, _py: Python) -> fastobo::ast::TypedefClause {
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
/// A clause declaring the date and time a typedef was created.
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
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<CreationDateClause> for fastobo::ast::TypedefClause {
    fn from(clause: CreationDateClause) -> Self {
        fastobo::ast::TypedefClause::CreationDate(clause.date)
    }
}

impl FromPy<CreationDateClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: CreationDateClause, _py: Python) -> Self {
        Self::from(clause)
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
    /// `datetime.datetime`: the date and time this typedef was created.
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

// --- ExpandAssertionTo -----------------------------------------------------

/// ExpandAssertionToClause(definition, xrefs)
/// --
///
/// An OWL macro that adds an `IAO:0000425` annotation to this relation.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct ExpandAssertionToClause {
    definition: fastobo::ast::QuotedString,
    xrefs: XrefList,
}

impl ExpandAssertionToClause {
    pub fn new<X>(py: Python, desc: fastobo::ast::QuotedString, xrefs: X) -> Self
    where
        X: IntoPy<XrefList>,
    {
        Self {
            definition: desc,
            xrefs: xrefs.into_py(py),
        }
    }
}

impl ClonePy for ExpandAssertionToClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            definition: self.definition.clone(),
            xrefs: self.xrefs.clone_py(py),
        }
    }
}

impl Display for ExpandAssertionToClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<ExpandAssertionToClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: ExpandAssertionToClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::ExpandAssertionTo(clause.definition, clause.xrefs.into_py(py))
    }
}

#[pymethods]
impl ExpandAssertionToClause {
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
    /// `str`: a textual definition of the assertion.
    fn get_definition(&self) -> PyResult<&str> {
        Ok(&self.definition.as_str())
    }

    #[setter]
    fn set_definition(&mut self, definition: String) {
        self.definition = fastobo::ast::QuotedString::new(definition);
    }

    #[getter]
    /// `~fastobo.xrefs.XrefList`: a list of xrefs supporting the assertion.
    fn get_xrefs<'py>(&self, py: Python<'py>) -> PyResult<XrefList> {
        Ok(self.xrefs.clone_py(py))
    }

    pub fn raw_value(&self) -> PyResult<String> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let xrefs = fastobo::ast::XrefList::from_py(self.xrefs.clone_py(py), py);
        Ok(format!("{} {}", self.definition, xrefs))
    }
}

impl_raw_tag!(ExpandAssertionToClause, "expand_assertion_to");

#[pyproto]
impl PyObjectProtocol for ExpandAssertionToClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, ExpandAssertionToClause(self.definition, self.xrefs))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.definition && self.xrefs)
    }
}

// --- ExpandExpressionTo ----------------------------------------------------

/// ExpandExpressionToClause(definition, xrefs)
/// --
///
/// An OWL macro that adds an `IAO:0000424` annotation to this relation.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct ExpandExpressionToClause {
    definition: fastobo::ast::QuotedString,
    xrefs: XrefList,
}

impl ExpandExpressionToClause {
    pub fn new<X>(py: Python, desc: fastobo::ast::QuotedString, xrefs: X) -> Self
    where
        X: IntoPy<XrefList>,
    {
        Self {
            definition: desc,
            xrefs: xrefs.into_py(py),
        }
    }
}

impl ClonePy for ExpandExpressionToClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            definition: self.definition.clone(),
            xrefs: self.xrefs.clone_py(py),
        }
    }
}

impl Display for ExpandExpressionToClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::TypedefClause::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<ExpandExpressionToClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: ExpandExpressionToClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::ExpandExpressionTo(
            clause.definition,
            clause.xrefs.into_py(py),
        )
    }
}

#[pymethods]
impl ExpandExpressionToClause {
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
    /// `str`: a textual definition of the expression.
    fn get_definition(&self) -> PyResult<&str> {
        Ok(&self.definition.as_str())
    }

    #[setter]
    fn set_definition(&mut self, definition: String) {
        self.definition = fastobo::ast::QuotedString::new(definition);
    }

    #[getter]
    /// `~fastobo.xrefs.XrefList`: a list of xrefs supporting the expression.
    fn get_xrefs<'py>(&self, py: Python<'py>) -> PyResult<XrefList> {
        Ok(self.xrefs.clone_py(py))
    }

    pub fn raw_value(&self) -> PyResult<String> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let xrefs = fastobo::ast::XrefList::from_py(self.xrefs.clone_py(py), py);
        Ok(format!("{} {}", self.definition, xrefs))
    }
}

impl_raw_tag!(ExpandExpressionToClause, "expand_expression_to");

#[pyproto]
impl PyObjectProtocol for ExpandExpressionToClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, ExpandExpressionToClause(self.definition, self.xrefs))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.definition && self.xrefs)
    }
}

// --- IsMetadataTag ---------------------------------------------------------

/// IsMetadataTagClause(metadata_tag)
/// --
///
/// A clause declaring whether this relationship is a metadata tag or not.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsMetadataTagClause {
    #[pyo3(get, set)]
    metadata_tag: bool,
}

impl IsMetadataTagClause {
    pub fn new(_py: Python, metadata_tag: bool) -> Self {
        Self { metadata_tag }
    }
}

impl Display for IsMetadataTagClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<IsMetadataTagClause> for fastobo::ast::TypedefClause {
    fn from(clause: IsMetadataTagClause) -> Self {
        fastobo::ast::TypedefClause::IsMetadataTag(clause.metadata_tag)
    }
}

impl FromPy<IsMetadataTagClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsMetadataTagClause, _py: Python) -> Self {
        Self::from(clause)
    }
}

impl_raw_tag!(IsMetadataTagClause, "is_metadata_tag");
impl_raw_value!(IsMetadataTagClause, "{}", self.metadata_tag);

#[pymethods]
impl IsMetadataTagClause {
    #[new]
    fn __init__(obj: &PyRawObject, metadata_tag: bool) {
        obj.init(Self::new(obj.py(), metadata_tag))
    }
}

#[pyproto]
impl PyObjectProtocol for IsMetadataTagClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsMetadataTagClause(self.metadata_tag))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.metadata_tag)
    }
}

// --- IsClassLevel ----------------------------------------------------------

/// IsClassLevelClause(class_level)
/// --
///
/// A clause declaring wether this relationship is class level or not.
#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsClassLevelClause {
    #[pyo3(get, set)]
    class_level: bool,
}

impl IsClassLevelClause {
    pub fn new(_py: Python, class_level: bool) -> Self {
        Self { class_level }
    }
}

impl Display for IsClassLevelClause {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        fastobo::ast::TypedefClause::from(self.clone()).fmt(f)
    }
}

impl From<IsClassLevelClause> for fastobo::ast::TypedefClause {
    fn from(clause: IsClassLevelClause) -> Self {
        fastobo::ast::TypedefClause::IsClassLevel(clause.class_level)
    }
}

impl FromPy<IsClassLevelClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsClassLevelClause, _py: Python) -> Self {
        Self::from(clause)
    }
}

impl_raw_tag!(IsClassLevelClause, "is_class_level");
impl_raw_value!(IsClassLevelClause, "{}", self.class_level);

#[pymethods]
impl IsClassLevelClause {
    #[new]
    fn __init__(obj: &PyRawObject, class_level: bool) {
        obj.init(Self::new(obj.py(), class_level))
    }
}

#[pyproto]
impl PyObjectProtocol for IsClassLevelClause {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, IsClassLevelClause(self.class_level))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(self, other, op, self.class_level)
    }
}
