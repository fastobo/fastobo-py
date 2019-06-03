pub mod clause;
pub mod frame;

use pyo3::prelude::*;

#[pymodule(term)]
pub fn module(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<self::frame::TermFrame>()?;
    m.add_class::<self::clause::BaseTermClause>()?;
    m.add_class::<self::clause::IsAnonymousClause>()?;
    m.add_class::<self::clause::NameClause>()?;
    m.add_class::<self::clause::NamespaceClause>()?;
    m.add_class::<self::clause::AltIdClause>()?;
    m.add_class::<self::clause::DefClause>()?;
    m.add_class::<self::clause::CommentClause>()?;
    m.add_class::<self::clause::SubsetClause>()?;
    m.add_class::<self::clause::SynonymClause>()?;
    m.add_class::<self::clause::XrefClause>()?;
    m.add_class::<self::clause::BuiltinClause>()?;
    m.add_class::<self::clause::PropertyValueClause>()?;
    m.add_class::<self::clause::IsAClause>()?;
    m.add_class::<self::clause::IntersectionOfClause>()?;
    m.add_class::<self::clause::UnionOfClause>()?;
    m.add_class::<self::clause::EquivalentToClause>()?;
    m.add_class::<self::clause::DisjointFromClause>()?;
    m.add_class::<self::clause::RelationshipClause>()?;
    m.add_class::<self::clause::IsObsoleteClause>()?;
    m.add_class::<self::clause::ReplacedByClause>()?;
    m.add_class::<self::clause::ConsiderClause>()?;
    m.add_class::<self::clause::CreatedByClause>()?;
    m.add_class::<self::clause::CreationDateClause>()?;

    register!(py, m, TermFrame, "collections.abc", MutableSequence);

    m.add("__name__", "fastobo.term")?;

    Ok(())
}
