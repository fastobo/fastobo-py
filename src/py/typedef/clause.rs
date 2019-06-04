use std::str::FromStr;

use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
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

#[pyclass(extends=AbstractEntityClause)]
pub struct BaseTypedefClause {}

// --- IsAnonymous -----------------------------------------------------------

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

impl FromPy<IsAnonymousClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsAnonymousClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::IsAnonymous(clause.anonymous)
    }
}

// --- Name ------------------------------------------------------------------

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

impl FromPy<NameClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: NameClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::Name(clause.name)
    }
}

// --- Namespace -------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct NamespaceClause {
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

impl FromPy<NamespaceClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: NamespaceClause, py: Python) -> Self {
        let ns = fastobo::ast::NamespaceIdent::from_py(clause.namespace, py);
        fastobo::ast::TypedefClause::Namespace(ns)
    }
}

// --- AltId -----------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct AltIdClause {
    id: Ident,
}

impl AltIdClause {
    pub fn new<I>(py: Python, id: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self { id: id.into_py(py) }
    }
}

impl ClonePy for AltIdClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            id: self.id.clone_py(py),
        }
    }
}

impl FromPy<AltIdClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: AltIdClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::AltId(clause.id.into_py(py))
    }
}

// --- Def -------------------------------------------------------------------

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

impl FromPy<DefClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: DefClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::Def(clause.definition, clause.xrefs.into_py(py))
    }
}

// --- Comment ---------------------------------------------------------------

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

impl FromPy<CommentClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: CommentClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::Comment(clause.comment)
    }
}

// --- Subset ----------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct SubsetClause {
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

impl FromPy<SubsetClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: SubsetClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::Subset(clause.subset.into_py(py))
    }
}

// --- Synonym ---------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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
            synonym: synonym.into_py(py),
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

impl FromPy<SynonymClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: SynonymClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::Synonym(clause.synonym.into_py(py))
    }
}

// --- Xref ------------------------------------------------------------------

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

// --- PropertyValue ---------------------------------------------------------

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

impl FromPy<PropertyValueClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: PropertyValueClause, py: Python) -> ast::TypedefClause {
        ast::TypedefClause::PropertyValue(clause.inner.into_py(py))
    }
}

// --- Domain ----------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct DomainClause {
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

impl FromPy<DomainClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: DomainClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::Domain(clause.domain.into_py(py))
    }
}

// --- Range -----------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct RangeClause {
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

impl FromPy<RangeClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: RangeClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::Range(clause.range.into_py(py))
    }
}

// --- Builtin ---------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct BuiltinClause {
    builtin: bool,
}

impl BuiltinClause {
    pub fn new(_py: Python, builtin: bool) -> Self {
        Self { builtin }
    }
}

impl FromPy<BuiltinClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: BuiltinClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::Builtin(clause.builtin)
    }
}

// --- HoldsOverChain --------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct HoldsOverChainClause {
    first: Ident,
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

impl FromPy<HoldsOverChainClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: HoldsOverChainClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::HoldsOverChain(
            clause.first.into_py(py),
            clause.last.into_py(py),
        )
    }
}

// --- IsAntiSymmetric -------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsAntiSymmetricClause {
    anti_symmetric: bool,
}

impl IsAntiSymmetricClause {
    pub fn new(_py: Python, anti_symmetric: bool) -> Self {
        Self { anti_symmetric }
    }
}

impl FromPy<IsAntiSymmetricClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsAntiSymmetricClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::IsAntiSymmetric(clause.anti_symmetric)
    }
}

// --- IsCyclic --------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsCyclicClause {
    cyclic: bool,
}

impl IsCyclicClause {
    pub fn new(_py: Python, cyclic: bool) -> Self {
        Self { cyclic }
    }
}

impl FromPy<IsCyclicClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsCyclicClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::IsCyclic(clause.cyclic)
    }
}

// --- IsReflexive -----------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsReflexiveClause {
    reflexive: bool,
}

impl IsReflexiveClause {
    pub fn new(_py: Python, reflexive: bool) -> Self {
        Self { reflexive }
    }
}

impl FromPy<IsReflexiveClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsReflexiveClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::IsReflexive(clause.reflexive)
    }
}

// --- IsSymmetric -----------------------------------------------------------

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

impl FromPy<IsSymmetricClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsSymmetricClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::IsSymmetric(clause.symmetric)
    }
}

// --- IsAsymmetric -----------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsAsymmetricClause {
    symmetric: bool,
}

impl IsAsymmetricClause {
    pub fn new(_py: Python, symmetric: bool) -> Self {
        Self { symmetric }
    }
}

impl FromPy<IsAsymmetricClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsAsymmetricClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::IsAsymmetric(clause.symmetric)
    }
}

// --- IsTransitive ----------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsTransitiveClause {
    transitive: bool,
}

impl IsTransitiveClause {
    pub fn new(_py: Python, transitive: bool) -> Self {
        Self { transitive }
    }
}

impl FromPy<IsTransitiveClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsTransitiveClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::IsTransitive(clause.transitive)
    }
}

// --- IsFunctional ----------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsFunctionalClause {
    functional: bool,
}

impl IsFunctionalClause {
    pub fn new(_py: Python, functional: bool) -> Self {
        Self { functional }
    }
}

impl FromPy<IsFunctionalClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsFunctionalClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::IsFunctional(clause.functional)
    }
}

// --- IsInverseFunctional ---------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsInverseFunctionalClause {
    inverse_functional: bool,
}

impl IsInverseFunctionalClause {
    pub fn new(_py: Python, inverse_functional: bool) -> Self {
        Self { inverse_functional }
    }
}

impl FromPy<IsInverseFunctionalClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsInverseFunctionalClause, _py: Python) -> Self {
        fastobo::ast::TypedefClause::IsInverseFunctional(clause.inverse_functional)
    }
}

// --- IsA -------------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct IsAClause {
    id: Ident,
}

impl IsAClause {
    pub fn new<I>(py: Python, id: I) -> Self
    where
        I: IntoPy<Ident>,
    {
        Self { id: id.into_py(py) }
    }
}

impl ClonePy for IsAClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            id: self.id.clone_py(py),
        }
    }
}

impl FromPy<IsAClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsAClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::IsA(clause.id.into_py(py))
    }
}

// --- IntersectionOf --------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct IntersectionOfClause {
    relation: Ident,
}

impl IntersectionOfClause {
    pub fn new<R>(py: Python, relation: R) -> Self
    where
        R: IntoPy<Ident>,
    {
        Self {
            relation: relation.into_py(py),
        }
    }
}

impl ClonePy for IntersectionOfClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            relation: self.relation.clone_py(py),
        }
    }
}

impl FromPy<IntersectionOfClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IntersectionOfClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::IntersectionOf(clause.relation.into_py(py))
    }
}

// --- UnionOf ---------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct UnionOfClause {
    term: Ident,
}

impl ClonePy for UnionOfClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            term: self.term.clone_py(py),
        }
    }
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

impl FromPy<UnionOfClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: UnionOfClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::UnionOf(clause.term.into_py(py))
    }
}

// --- EquivalentTo ----------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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

impl FromPy<EquivalentToClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: EquivalentToClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::EquivalentTo(clause.term.into_py(py))
    }
}

// --- DisjointFrom ----------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct DisjointFromClause {
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

impl FromPy<DisjointFromClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: DisjointFromClause, py: Python) -> Self {
        ast::TypedefClause::DisjointFrom(clause.term.into_py(py))
    }
}

// --- InverseOf -------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct InverseOfClause {
    relation: Ident,
}

impl InverseOfClause {
    pub fn new<R>(py: Python, relation: R) -> Self
    where
        R: IntoPy<Ident>,
    {
        Self {
            relation: relation.into_py(py),
        }
    }
}

impl ClonePy for InverseOfClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            relation: self.relation.clone_py(py),
        }
    }
}

impl FromPy<InverseOfClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: InverseOfClause, py: Python) -> Self {
        ast::TypedefClause::InverseOf(clause.relation.into_py(py))
    }
}

// --- TransitiveOver --------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct TransitiveOverClause {
    relation: Ident,
}

impl TransitiveOverClause {
    pub fn new<R>(py: Python, relation: R) -> Self
    where
        R: IntoPy<Ident>,
    {
        Self {
            relation: relation.into_py(py),
        }
    }
}

impl ClonePy for TransitiveOverClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            relation: self.relation.clone_py(py),
        }
    }
}

impl FromPy<TransitiveOverClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: TransitiveOverClause, py: Python) -> Self {
        ast::TypedefClause::TransitiveOver(clause.relation.into_py(py))
    }
}

// --- EquivalentToChain -----------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct EquivalentToChainClause {
    first: Ident,
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

impl FromPy<EquivalentToChainClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: EquivalentToChainClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::EquivalentToChain(
            clause.first.into_py(py),
            clause.last.into_py(py),
        )
    }
}

// --- DisjointOver ----------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct DisjointOverClause {
    relation: Ident,
}

impl DisjointOverClause {
    pub fn new<R>(py: Python, relation: R) -> Self
    where
        R: IntoPy<Ident>,
    {
        Self {
            relation: relation.into_py(py),
        }
    }
}

impl ClonePy for DisjointOverClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            relation: self.relation.clone_py(py),
        }
    }
}

impl FromPy<DisjointOverClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: DisjointOverClause, py: Python) -> Self {
        ast::TypedefClause::DisjointOver(clause.relation.into_py(py))
    }
}

// --- Relationship ----------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct RelationshipClause {
    relation: Ident,
    term: Ident,
}

impl RelationshipClause {
    pub fn new<R, T>(py: Python, relation: R, term: T) -> Self
    where
        R: IntoPy<Ident>,
        T: IntoPy<Ident>,
    {
        Self {
            relation: relation.into_py(py),
            term: term.into_py(py),
        }
    }
}

impl ClonePy for RelationshipClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            relation: self.relation.clone_py(py),
            term: self.term.clone_py(py),
        }
    }
}

impl FromPy<RelationshipClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: RelationshipClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::Relationship(clause.relation.into_py(py), clause.term.into_py(py))
    }
}

// --- IsObsolete ------------------------------------------------------------

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

impl FromPy<IsObsoleteClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsObsoleteClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::IsObsolete(clause.obsolete)
    }
}

// --- ReplacedBy ------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct ReplacedByClause {
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

impl FromPy<ReplacedByClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: ReplacedByClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::ReplacedBy(clause.term.into_py(py))
    }
}

// --- Consider --------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
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

impl FromPy<ConsiderClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: ConsiderClause, py: Python) -> fastobo::ast::TypedefClause {
        ast::TypedefClause::Consider(clause.term.into_py(py))
    }
}

// --- CreatedBy -------------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct CreatedByClause {
    name: fastobo::ast::UnquotedString,
}

impl CreatedByClause {
    pub fn new(_py: Python, name: fastobo::ast::UnquotedString) -> Self {
        Self { name }
    }
}

impl FromPy<CreatedByClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: CreatedByClause, py: Python) -> fastobo::ast::TypedefClause {
        fastobo::ast::TypedefClause::CreatedBy(clause.name)
    }
}

// --- CreationDate ----------------------------------------------------------

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

impl FromPy<CreationDateClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: CreationDateClause, py: Python) -> fastobo::ast::TypedefClause {
        fastobo::ast::TypedefClause::CreationDate(clause.date)
    }
}

// --- ExpandAssertionTo -----------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct ExpandAssertionToClause {
    description: fastobo::ast::QuotedString,
    xrefs: XrefList,
}

impl ExpandAssertionToClause {
    pub fn new<X>(py: Python, desc: fastobo::ast::QuotedString, xrefs: X) -> Self
    where
        X: IntoPy<XrefList>,
    {
        Self {
            description: desc,
            xrefs: xrefs.into_py(py),
        }
    }
}

impl ClonePy for ExpandAssertionToClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            description: self.description.clone(),
            xrefs: self.xrefs.clone_py(py),
        }
    }
}

impl FromPy<ExpandAssertionToClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: ExpandAssertionToClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::ExpandAssertionTo(clause.description, clause.xrefs.into_py(py))
    }
}

// --- ExpandExpressionTo ----------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Debug)]
pub struct ExpandExpressionToClause {
    description: fastobo::ast::QuotedString,
    xrefs: XrefList,
}

impl ExpandExpressionToClause {
    pub fn new<X>(py: Python, desc: fastobo::ast::QuotedString, xrefs: X) -> Self
    where
        X: IntoPy<XrefList>,
    {
        Self {
            description: desc,
            xrefs: xrefs.into_py(py),
        }
    }
}

impl ClonePy for ExpandExpressionToClause {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            description: self.description.clone(),
            xrefs: self.xrefs.clone_py(py),
        }
    }
}

impl FromPy<ExpandExpressionToClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: ExpandExpressionToClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::ExpandExpressionTo(
            clause.description,
            clause.xrefs.into_py(py),
        )
    }
}

// --- IsMetadataTag ---------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsMetadataTagClause {
    metadata_tag: bool,
}

impl IsMetadataTagClause {
    pub fn new(_py: Python, metadata_tag: bool) -> Self {
        Self { metadata_tag }
    }
}

impl FromPy<IsMetadataTagClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsMetadataTagClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::IsMetadataTag(clause.metadata_tag)
    }
}

// --- IsClassLevel ----------------------------------------------------------

#[pyclass(extends=BaseTypedefClause, module="fastobo.typedef")]
#[derive(Clone, ClonePy, Debug)]
pub struct IsClassLevelClause {
    class_level: bool,
}

impl IsClassLevelClause {
    pub fn new(_py: Python, class_level: bool) -> Self {
        Self { class_level }
    }
}

impl FromPy<IsClassLevelClause> for fastobo::ast::TypedefClause {
    fn from_py(clause: IsClassLevelClause, py: Python) -> Self {
        fastobo::ast::TypedefClause::IsClassLevel(clause.class_level)
    }
}
