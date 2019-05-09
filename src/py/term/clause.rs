use std::str::FromStr;

use pyo3::AsPyPointer;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PyTypeInfo;
use pyo3::prelude::*;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::types::PyAny;
use pyo3::types::PyString;

use fastobo::ast;
use fastobo::share::Share;
use fastobo::share::Cow;
use fastobo::share::Redeem;

use crate::utils::AsGILRef;
use crate::utils::ClonePy;
use super::super::id::Ident;
use super::super::pv::PropertyValue;
use super::super::xref::Xref;
use super::super::xref::XrefList;
use super::super::syn::Synonym;

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
    CreationDate(Py<CreationDateClause>)
}

impl FromPy<fastobo::ast::TermClause> for TermClause {
    fn from_py(clause: fastobo::ast::TermClause, py: Python) -> Self {
        use fastobo::ast::TermClause::*;
        match clause {
            IsAnonymous(b) =>
                Py::new(py, IsAnonymousClause::new(py, b))
                    .map(TermClause::IsAnonymous),
            Name(n) =>
                Py::new(py, NameClause::new(py, n))
                    .map(TermClause::Name),
            Namespace(ns) =>
                Py::new(py, NamespaceClause::new(py, ns))
                    .map(TermClause::Namespace),
            AltId(id) =>
                Py::new(py, AltIdClause::new(py, id))
                    .map(TermClause::AltId),
            Def(desc, xrefs) =>
                Py::new(py, DefClause::new(py, desc, xrefs))
                    .map(TermClause::Def),
            Comment(c) =>
                Py::new(py, CommentClause::new(py, c))
                    .map(TermClause::Comment),
            Subset(s) =>
                Py::new(py, SubsetClause::new(py, s))
                    .map(TermClause::Subset),
            Synonym(s) =>
                Py::new(py, SynonymClause::new(py, s))
                    .map(TermClause::Synonym),
            Xref(x) =>
                Py::new(py, XrefClause::new(py, x))
                    .map(TermClause::Xref),
            Builtin(b) =>
                Py::new(py, BuiltinClause::new(py, b))
                    .map(TermClause::Builtin),
            PropertyValue(pv) =>
                Py::new(py, PropertyValueClause::new(py, pv))
                    .map(TermClause::PropertyValue),
            IsA(id) =>
                Py::new(py, IsAClause::new(py, id))
                    .map(TermClause::IsA),
            IntersectionOf(r, cls) =>
                Py::new(py, IntersectionOfClause::new(py, r, cls))
                    .map(TermClause::IntersectionOf),
            UnionOf(cls) =>
                Py::new(py, UnionOfClause::new(py, cls))
                    .map(TermClause::UnionOf),
            EquivalentTo(cls) =>
                Py::new(py, EquivalentToClause::new(py, cls))
                    .map(TermClause::EquivalentTo),
            DisjointFrom(cls) =>
                Py::new(py, DisjointFromClause::new(py, cls))
                    .map(TermClause::DisjointFrom),
            Relationship(r, id) =>
                Py::new(py, RelationshipClause::new(py, r, id))
                    .map(TermClause::Relationship),
            IsObsolete(b) =>
                Py::new(py, IsObsoleteClause::new(py, b))
                    .map(TermClause::IsObsolete),
            ReplacedBy(id) =>
                Py::new(py, ReplacedByClause::new(py, id))
                    .map(TermClause::ReplacedBy),
            Consider(id) =>
                Py::new(py, ConsiderClause::new(py, id))
                    .map(TermClause::Consider),
            CreatedBy(name) =>
                Py::new(py, CreatedByClause::new(py, name))
                    .map(TermClause::CreatedBy),
            CreationDate(dt) =>
                Py::new(py, CreationDateClause::new(py, dt))
                    .map(TermClause::CreationDate),
        }.expect("could not allocate memory for `TermClause` in Python heap")
    }
}

// --- Base ------------------------------------------------------------------

/// A header clause, appearing in the OBO header frame.
#[pyclass(subclass)]
pub struct BaseTermClause {}

// --- IsAnonymous -----------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Clone, ClonePy, Debug)]
pub struct IsAnonymousClause {
    #[pyo3(get, set)]
    anonymous: bool
}

impl IsAnonymousClause {
    pub fn new(_py: Python, anonymous: bool) -> Self {
        Self { anonymous }
    }
}

impl FromPy<IsAnonymousClause> for fastobo::ast::TermClause {
    fn from_py(clause: IsAnonymousClause, py: Python) -> Self {
        fastobo::ast::TermClause::IsAnonymous(clause.anonymous)
    }
}


// --- Name ------------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Clone, ClonePy, Debug)]
pub struct NameClause {
    name: fastobo::ast::UnquotedString,
}

impl NameClause {
    pub fn new(_py: Python, name: fastobo::ast::UnquotedString) -> Self {
        Self { name }
    }
}

impl FromPy<NameClause> for fastobo::ast::TermClause {
    fn from_py(clause: NameClause, py: Python) -> Self {
        fastobo::ast::TermClause::Name(clause.name)
    }
}

#[pymethods]
impl NameClause {
    #[getter]
    /// `str`: the name of the current term.
    fn get_name(&self) -> PyResult<&str> {
        Ok(self.name.as_str())
    }

    #[setter]
    fn set_name(&mut self, name: String) -> PyResult<()> {
        self.name = fastobo::ast::UnquotedString::new(name);
        Ok(())
    }
}

// --- Namespace -------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct NamespaceClause {
    namespace: Ident
}

impl NamespaceClause {
    pub fn new<I>(py: Python, ns: I) -> Self
    where
        I: IntoPy<Ident>
    {
        Self { namespace: ns.into_py(py) }
    }
}

impl ClonePy for NamespaceClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            namespace: self.namespace.clone_py(py)
        }
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
    #[getter]
    /// `~fastobo.id.Ident`: the ID of the namespace this term is part of.
    fn get_namespace(&self) -> PyResult<&Ident> {
        Ok(&self.namespace)
    }
}


// --- AltId -----------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct AltIdClause {
    alt_id: Ident,
}

impl AltIdClause {
    pub fn new<I>(py: Python, id: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self { alt_id: id.into_py(py) }
    }
}

impl ClonePy for AltIdClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            alt_id: self.alt_id.clone_py(py)
        }
    }
}

impl FromPy<AltIdClause> for fastobo::ast::TermClause {
    fn from_py(clause: AltIdClause, py: Python) -> Self {
        fastobo::ast::TermClause::AltId(clause.alt_id.into_py(py))
    }
}

#[pymethods]
impl AltIdClause {
    #[getter]
    /// `~fastobo.id.Ident`: an alternative ID used to refer to this term.
    fn get_alt_id(&self) -> PyResult<&Ident> {
        Ok(&self.alt_id)
    }
}


// --- Def -------------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
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
        Self { definition, xrefs: xrefs.into_py(py) }
    }
}

impl ClonePy for DefClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            definition: self.definition.clone(),
            xrefs: self.xrefs.clone_py(py)
        }
    }
}

impl FromPy<DefClause> for fastobo::ast::TermClause {
    fn from_py(clause: DefClause, py: Python) -> Self {
        fastobo::ast::TermClause::Def(
            clause.definition,
            clause.xrefs.into_py(py)
        )
    }
}

#[pymethods]
impl DefClause {

    #[getter]
    /// `str`: a textual definition for this term.
    fn get_definition(&self) -> PyResult<&str> {
        Ok(&self.definition.as_str())
    }

    #[getter]
    /// `~fastobo.xrefs.XrefList`: a list of xrefs supporting the definition.
    fn get_xrefs(&self) -> PyResult<XrefList> {
        let py = unsafe { Python::assume_gil_acquired() };
        Ok(self.xrefs.clone_py(py))
    }

}

// --- Comment ---------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Clone, ClonePy, Debug)]
pub struct CommentClause {
    comment: fastobo::ast::UnquotedString
}

impl CommentClause {
    pub fn new(_py: Python, comment: fastobo::ast::UnquotedString) -> Self {
        Self { comment }
    }
}

impl FromPy<CommentClause> for fastobo::ast::TermClause {
    fn from_py(clause: CommentClause, _py: Python) -> Self {
        fastobo::ast::TermClause::Comment(clause.comment)
    }
}

#[pymethods]
impl CommentClause {
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

// --- Subset ----------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct SubsetClause {
    subset: Ident
}

impl SubsetClause {
    pub fn new<I>(py: Python, subset: I) -> Self
    where
        I: IntoPy<Ident>
    {
        Self { subset: subset.into_py(py) }
    }
}

impl ClonePy for SubsetClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            subset: self.subset.clone_py(py)
        }
    }
}

impl FromPy<SubsetClause> for fastobo::ast::TermClause {
    fn from_py(clause: SubsetClause, py: Python) -> Self {
        fastobo::ast::TermClause::Subset(clause.subset.into_py(py))
    }
}

#[pymethods]
impl SubsetClause {
    #[getter]
    /// `~fastobo.id.Ident`: the ID of the subset this term is part of.
    fn get_subset(&self) -> PyResult<&Ident> {
        Ok(&self.subset)
    }

    #[setter]
    fn set_subset(&mut self, subset: Ident) -> PyResult<()> {
        self.subset = subset;
        Ok(())
    }
}

// --- Synonym ---------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct SynonymClause {
    synonym: Synonym,
}

impl SynonymClause {
    pub fn new<S>(py: Python, synonym: S) -> Self
    where
        S: IntoPy<Synonym>,
    {
        Self {
            synonym: synonym.into_py(py)
        }
    }
}

impl ClonePy for SynonymClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            synonym: self.synonym.clone_py(py)
        }
    }
}

impl FromPy<SynonymClause> for fastobo::ast::TermClause {
    fn from_py(clause: SynonymClause, py: Python) -> Self {
        fastobo::ast::TermClause::Synonym(clause.synonym.into_py(py))
    }
}

#[pymethods]
impl SynonymClause {
    #[getter]
    /// `~fastobo.syn.Synonym`: a possible synonym for this term.
    fn get_synonym(&self) -> PyResult<Synonym> {
        let py = unsafe { Python::assume_gil_acquired() };
        Ok(self.synonym.clone_py(py))
    }
}

// --- Xref ------------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct XrefClause {
    xref: Py<Xref>
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
            xref: self.xref.clone_py(py)
        }
    }
}

impl FromPy<XrefClause> for fastobo::ast::TermClause {
    fn from_py(clause: XrefClause, py: Python) -> Self {
        fastobo::ast::TermClause::Xref(
            clause.xref.as_ref(py).clone_py(py).into_py(py)
        )
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
                .expect("could not allocate memory on Python heap for XrefClause")
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
    fn get_xref(&self) -> PyResult<Py<Xref>> {
        let py = unsafe { Python::assume_gil_acquired() };
        Ok(self.xref.clone_ref(py))
    }

    #[setter]
    fn set_ref(&mut self, xref: &PyAny) -> PyResult<()> {
        self.xref = Xref::from_object(xref.py(), xref)?;
        Ok(())
    }
}

// --- Builtin ---------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Clone, ClonePy, Debug)]
pub struct BuiltinClause {
    #[pyo3(get, set)]
    builtin: bool
}

impl BuiltinClause {
    pub fn new(_py: Python, builtin: bool) -> Self {
        Self { builtin }
    }
}

impl FromPy<BuiltinClause> for fastobo::ast::TermClause {
    fn from_py(clause: BuiltinClause, py: Python) -> Self {
        fastobo::ast::TermClause::Builtin(clause.builtin)
    }
}

// --- PropertyValue ---------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct PropertyValueClause {
    inner: PropertyValue,
}

impl PropertyValueClause {
    pub fn new<P>(py: Python, property_value: P) -> Self
    where
        P: IntoPy<PropertyValue>
    {
        Self { inner: property_value.into_py(py) }
    }
}

impl ClonePy for PropertyValueClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            inner: self.inner.clone_py(py)
        }
    }
}

impl FromPy<PropertyValueClause> for fastobo::ast::TermClause {
    fn from_py(clause: PropertyValueClause, py: Python) -> ast::TermClause {
        ast::TermClause::PropertyValue(clause.inner.into_py(py))
    }
}

// --- IsA -------------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct IsAClause {
    id: Ident
}

impl IsAClause {
    pub fn new<I>(py: Python, id: I) -> Self
    where
        I: IntoPy<Ident>
    {
        Self { id: id.into_py(py) }
    }
}

impl ClonePy for IsAClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            id: self.id.clone_py(py)
        }
    }
}

impl FromPy<IsAClause> for fastobo::ast::TermClause {
    fn from_py(clause: IsAClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::IsA(clause.id.into_py(py))
    }
}

// --- IntersectionOf --------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct IntersectionOfClause {
    relation: Option<Ident>,
    term: Ident,
}

impl IntersectionOfClause {
    pub fn new<R, C>(py: Python, relation: Option<R>, class: C) -> Self
    where
        R: IntoPy<Ident>,
        C: IntoPy<Ident>,
    {
        Self {
            relation: relation.map(|id| id.into_py(py)),
            term: class.into_py(py)
        }
    }
}

impl ClonePy for IntersectionOfClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            relation: self.relation.clone_py(py),
            term: self.term.clone_py(py),
        }
    }
}

impl FromPy<IntersectionOfClause> for fastobo::ast::TermClause {
    fn from_py(clause: IntersectionOfClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::IntersectionOf(
            clause.relation.map(|id| id.into_py(py)),
            clause.term.into_py(py)
        )
    }
}

// --- UnionOf ---------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct UnionOfClause {
    term: Ident,
}

impl ClonePy for UnionOfClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            term: self.term.clone_py(py)
        }
    }
}

impl UnionOfClause {
    pub fn new<I>(py: Python, term: I) -> Self
    where
        I: IntoPy<Ident>
    {
        Self { term: term.into_py(py) }
    }
}

impl FromPy<UnionOfClause> for fastobo::ast::TermClause {
    fn from_py(clause: UnionOfClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::UnionOf(clause.term.into_py(py))
    }
}

// --- EquivalentTo ----------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct EquivalentToClause {
    term: Ident,
}

impl EquivalentToClause {
    pub fn new<I>(py: Python, term: I) -> Self
    where
        I: IntoPy<Ident>
    {
        Self { term: term.into_py(py) }
    }
}

impl ClonePy for EquivalentToClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            term: self.term.clone_py(py)
        }
    }
}

impl FromPy<EquivalentToClause> for fastobo::ast::TermClause {
    fn from_py(clause: EquivalentToClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::EquivalentTo(clause.term.into_py(py))
    }
}

// --- DisjointFrom ----------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct DisjointFromClause {
    term: Ident,
}

impl DisjointFromClause {
    pub fn new<I>(py: Python, term: I) -> Self
    where
        I: IntoPy<Ident>
    {
        Self { term: term.into_py(py) }
    }
}

impl ClonePy for DisjointFromClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            term: self.term.clone_py(py),
        }
    }
}

impl FromPy<DisjointFromClause> for fastobo::ast::TermClause {
    fn from_py(clause: DisjointFromClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::DisjointFrom(clause.term.into_py(py))
    }
}

// --- Relationship ----------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct RelationshipClause {
    relation: Ident,
    term: Ident
}

impl RelationshipClause {
    pub fn new<R, T>(py: Python, relation: R, term: T) -> Self
    where
        R: IntoPy<Ident>,
        T: IntoPy<Ident>,
    {
        Self { relation: relation.into_py(py), term: term.into_py(py) }
    }
}

impl ClonePy for RelationshipClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            relation: self.relation.clone_py(py),
            term: self.term.clone_py(py)
        }
    }
}

impl FromPy<RelationshipClause> for fastobo::ast::TermClause {
    fn from_py(clause: RelationshipClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::Relationship(
            clause.relation.into_py(py),
            clause.term.into_py(py)
        )
    }
}

// --- IsObsolete ------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Clone, ClonePy, Debug)]
pub struct IsObsoleteClause {
    #[pyo3(get, set)]
    obsolete: bool
}

impl IsObsoleteClause {
    pub fn new(_py: Python, obsolete: bool) -> Self {
        Self { obsolete }
    }
}

impl FromPy<IsObsoleteClause> for fastobo::ast::TermClause {
    fn from_py(clause: IsObsoleteClause, py: Python) -> Self {
        fastobo::ast::TermClause::IsObsolete(clause.obsolete)
    }
}

// --- ReplacedBy ------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct ReplacedByClause {
    term: Ident,
}

impl ReplacedByClause {
    pub fn new<I>(py: Python, term: I) -> Self
    where
        I: IntoPy<Ident>
    {
        Self { term: term.into_py(py) }
    }
}

impl ClonePy for ReplacedByClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            term: self.term.clone_py(py)
        }
    }
}

impl FromPy<ReplacedByClause> for fastobo::ast::TermClause {
    fn from_py(clause: ReplacedByClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::ReplacedBy(clause.term.into_py(py))
    }
}

// --- Consider --------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Debug)]
pub struct ConsiderClause {
    term: Ident,
}

impl ConsiderClause {
    pub fn new<I>(py: Python, term: I) -> Self
    where
        I: IntoPy<Ident>
    {
        Self { term: term.into_py(py) }
    }
}

impl ClonePy for ConsiderClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            term: self.term.clone_py(py)
        }
    }
}

impl FromPy<ConsiderClause> for fastobo::ast::TermClause {
    fn from_py(clause: ConsiderClause, py: Python) -> fastobo::ast::TermClause {
        ast::TermClause::Consider(clause.term.into_py(py))
    }
}

// --- CreatedBy -------------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Clone, ClonePy, Debug)]
pub struct CreatedByClause {
    name: fastobo::ast::UnquotedString
}

impl CreatedByClause {
    pub fn new(_py: Python, name: fastobo::ast::UnquotedString) -> Self {
        Self { name }
    }
}

impl FromPy<CreatedByClause> for fastobo::ast::TermClause {
    fn from_py(clause: CreatedByClause, py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::CreatedBy(clause.name)
    }
}


// --- CreationDate ----------------------------------------------------------

#[pyclass(extends=BaseTermClause)]
#[derive(Clone, ClonePy, Debug)]
pub struct CreationDateClause {
    date: fastobo::ast::IsoDateTime,
}

impl CreationDateClause {
    pub fn new(_py: Python, date: fastobo::ast::IsoDateTime) -> Self {
        Self { date }
    }
}

impl FromPy<CreationDateClause> for fastobo::ast::TermClause {
    fn from_py(clause: CreationDateClause, py: Python) -> fastobo::ast::TermClause {
        fastobo::ast::TermClause::CreationDate(clause.date)
    }
}
