pub mod clause;
pub mod frame;

use pyo3::prelude::*;

#[pymodule(typedef)]
pub fn module(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<self::frame::TypedefFrame>()?;
    m.add_class::<self::clause::BaseTypedefClause>()?;
    m.add_class::<self::clause::IsAnonymousClause>()?;
    m.add_class::<self::clause::NameClause>()?;
    m.add_class::<self::clause::NamespaceClause>()?;
    m.add_class::<self::clause::AltIdClause>()?;
    m.add_class::<self::clause::DefClause>()?;
    m.add_class::<self::clause::CommentClause>()?;
    m.add_class::<self::clause::SubsetClause>()?;
    m.add_class::<self::clause::SynonymClause>()?;
    m.add_class::<self::clause::XrefClause>()?;
    m.add_class::<self::clause::PropertyValueClause>()?;
    m.add_class::<self::clause::DomainClause>()?;
    m.add_class::<self::clause::RangeClause>()?;
    m.add_class::<self::clause::BuiltinClause>()?;
    m.add_class::<self::clause::HoldsOverChainClause>()?;
    m.add_class::<self::clause::IsAntiSymmetricClause>()?;
    m.add_class::<self::clause::IsCyclicClause>()?;
    m.add_class::<self::clause::IsReflexiveClause>()?;
    m.add_class::<self::clause::IsSymmetricClause>()?;
    m.add_class::<self::clause::IsAsymmetricClause>()?;
    m.add_class::<self::clause::IsTransitiveClause>()?;
    m.add_class::<self::clause::IsFunctionalClause>()?;
    m.add_class::<self::clause::IsInverseFunctionalClause>()?;
    m.add_class::<self::clause::IsAClause>()?;
    m.add_class::<self::clause::IntersectionOfClause>()?;
    m.add_class::<self::clause::UnionOfClause>()?;
    m.add_class::<self::clause::EquivalentToClause>()?;
    m.add_class::<self::clause::DisjointFromClause>()?;
    m.add_class::<self::clause::InverseOfClause>()?;
    m.add_class::<self::clause::TransitiveOverClause>()?;
    m.add_class::<self::clause::EquivalentToChainClause>()?;
    m.add_class::<self::clause::DisjointOverClause>()?;
    m.add_class::<self::clause::RelationshipClause>()?;
    m.add_class::<self::clause::IsObsoleteClause>()?;
    m.add_class::<self::clause::ReplacedByClause>()?;
    m.add_class::<self::clause::ConsiderClause>()?;
    m.add_class::<self::clause::CreatedByClause>()?;
    m.add_class::<self::clause::CreationDateClause>()?;
    m.add_class::<self::clause::ExpandAssertionToClause>()?;
    m.add_class::<self::clause::ExpandExpressionToClause>()?;
    m.add_class::<self::clause::IsMetadataTagClause>()?;
    m.add_class::<self::clause::IsClassLevelClause>()?;

    register!(py, m, TypedefFrame, "collections.abc", MutableSequence);

    m.add("__name__", "fastobo.typedef")?;

    Ok(())
}
