use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::rc::Rc;
use std::str::FromStr;
use std::string::ToString;

use pyo3::class::gc::PyVisit;
use pyo3::exceptions::IndexError;
use pyo3::exceptions::RuntimeError;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::exceptions::NotImplementedError;
use pyo3::gc::PyTraverseError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::PyGCProtocol;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PySequenceProtocol;
use pyo3::PyTypeInfo;

use fastobo::ast as obo;

use crate::error::Error;
use crate::utils::AsGILRef;
use crate::utils::ClonePy;

use super::header::frame::HeaderFrame;
use super::id::Ident;
use super::term::frame::TermFrame;
use super::typedef::frame::TypedefFrame;

// --- Module export ---------------------------------------------------------

/// Base Classes defining common interfaces for classes in this library.
///
/// These base classes are here to define common methods and attributes shared
/// by numerous classes in the ``fastobo`` submodules. Since Rust is a
/// statically-typed language, all "subclasses" are known at compile-time, so
/// creating new subclasses hoping to use them with the current classes (and
/// in particular, collections such as `~fastobo.doc.OboDoc`) will not work,
/// and is likely to cause an undefined behaviour.
///
#[pymodule(abc)]
fn module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<self::AbstractFrame>()?;
    m.add_class::<self::AbstractEntityFrame>()?;
    m.add_class::<self::AbstractClause>()?;
    m.add_class::<self::AbstractEntityClause>()?;
    m.add("__name__", "fastobo.abc")?;
    Ok(())
}

// ---

/// An abstract OBO frame, storing a sequence of various clauses.
///
/// An OBO document contains a header frame (which may be empty, but should
/// at least contain a `~fastobo.header.FormatVersionClause` and a
/// `~fastobo.header.OntologyClause` for compatibility purposes), followed by
/// a various number of entity frames.
#[pyclass(subclass, module = "fastobo.abc")]
pub struct AbstractFrame {}

/// An abstract entity frame, which clauses define an entity.
///
/// Entity frames define OBO entities, which can be classes (terms),
/// relations (typedefs) and instances. All OBO entities have an identifier,
/// which is supposedly unique, that can be accessed through the ``id``
/// property in any concrete subclass.
#[pyclass(extends=AbstractFrame, module="fastobo.abc")]
pub struct AbstractEntityFrame {}

#[pymethods]
impl AbstractEntityFrame {
    /// `~fastobo.id.Ident`: the identifier of the described entity.
    #[getter]
    pub fn get_id(&self) -> PyResult<Ident> {
        NotImplementedError::into("AbstractEntityFrame.raw_tag")
    }
}

// ---

/// An abstract clause.
///
/// An OBO clause is a tag/value pair, with additional syntax requirements
/// depending on the tag. The raw tag and raw value of an OBO clause can be
/// accessed with the `raw_tag` and `raw_value` methods, for instance to
/// convert a frame into a Python `dict`.
///
/// Example:
///     >>> d = {}
///     >>> for clause in ms[1]:
///     ...     d.setdefault(clause.raw_tag(), []).append(clause.raw_value())
///     >>> pprint(d)
///     {'def': ['"A reference number relevant to the sample under study."'],
///      'is_a': ['MS:1000548'],
///      'name': ['sample number'],
///      'xref': ['value-type:xsd\\:string "The allowed value-type for this CV term."']}
///
#[pyclass(subclass, module = "fastobo.abc")]
pub struct AbstractClause {}

#[pymethods]
impl AbstractClause {
    /// Get the raw tag of the header clause.
    ///
    /// Returns:
    ///     `str`: the header clause value as it was extracted from the OBO
    ///     header, stripped from trailing qualifiers and comment.
    ///
    /// Example:
    ///     >>> clause = fastobo.header.OntologyClause("test")
    ///     >>> clause.raw_tag()
    ///     'ontology'
    ///     >>> str(clause)
    ///     'ontology: test'
    pub fn raw_tag(&self) -> PyResult<String> {
        NotImplementedError::into("BaseHeaderClause.raw_tag")
    }

    /// Get the raw value of the header clause.
    ///
    /// Returns:
    ///     `str`: the header clause value as it was extracted from the OBO
    ///     header, stripped from trailing qualifiers and comment.
    ///
    /// Example:
    ///     >>> dt = datetime.datetime(2019, 4, 29, 21, 52)
    ///     >>> clause = fastobo.header.DateClause(dt)
    ///     >>> clause.date
    ///     datetime.datetime(2019, 4, 29, 21, 52)
    ///     >>> clause.raw_value()
    ///     '29:04:2019 21:52'
    pub fn raw_value(&self) -> PyResult<String> {
        NotImplementedError::into("BaseHeaderClause.raw_value")
    }
}

/// An abstract entity clause.
#[pyclass(extends=AbstractClause, module="fastobo.abc")]
pub struct AbstractEntityClause {}
