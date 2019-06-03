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
use crate::pyfile::PyFile;
use crate::utils::AsGILRef;
use crate::utils::ClonePy;

use super::header::frame::HeaderFrame;
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

/// A base
#[pyclass(subclass, module = "fastobo.abc")]
pub struct AbstractFrame {}

#[pyclass(extends=AbstractFrame, module="fastobo.abc")]
pub struct AbstractEntityFrame {}

// ---

#[pyclass(subclass, module = "fastobo.abc")]
pub struct AbstractClause {}

#[pyclass(extends=AbstractClause, module="fastobo.abc")]
pub struct AbstractEntityClause {}
