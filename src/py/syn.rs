use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::rc::Rc;
use std::str::FromStr;
use std::string::ToString;

use pyo3::class::basic::CompareOp;
use pyo3::class::gc::PyVisit;
use pyo3::exceptions::IndexError;
use pyo3::exceptions::RuntimeError;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::gc::PyTraverseError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyIterator;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::AsPyPointer;
use pyo3::PyGCProtocol;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PySequenceProtocol;
use pyo3::PyTypeInfo;

use super::id::Ident;
use super::xref::XrefList;
use crate::utils::ClonePy;
use crate::utils::AsGILRef;

// --- Module export ---------------------------------------------------------

#[pymodule(syn)]
fn module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<self::Synonym>()?;
    m.add_class::<self::SynonymScope>()?;
    m.add("__name__", "fastobo.syn")?;
    Ok(())
}

// --- SynonymScope ----------------------------------------------------------

#[pyclass(module = "fastobo.syn")] // FIXME(@althonos): probably not needed since it is not exposed.
#[derive(Clone, ClonePy, Debug, Eq, PartialEq)]
pub struct SynonymScope {
    inner: fastobo::ast::SynonymScope,
}

impl SynonymScope {
    pub fn new(scope: fastobo::ast::SynonymScope) -> Self {
        Self { inner: scope }
    }
}

impl Display for SynonymScope {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.inner.fmt(f)
    }
}

impl From<fastobo::ast::SynonymScope> for SynonymScope {
    fn from(scope: fastobo::ast::SynonymScope) -> Self {
        Self::new(scope)
    }
}

impl FromPy<fastobo::ast::SynonymScope> for SynonymScope {
    fn from_py(scope: fastobo::ast::SynonymScope, _py: Python) -> Self {
        Self::from(scope)
    }
}

impl From<SynonymScope> for fastobo::ast::SynonymScope {
    fn from(scope: SynonymScope) -> Self {
        scope.inner
    }
}

impl FromPy<SynonymScope> for fastobo::ast::SynonymScope {
    fn from_py(scope: SynonymScope, _py: Python) -> Self {
        Self::from(scope)
    }
}

impl FromStr for SynonymScope {
    type Err = PyErr;
    fn from_str(s: &str) -> PyResult<Self> {
        match s {
            "EXACT" => Ok(Self::new(fastobo::ast::SynonymScope::Exact)),
            "BROAD" => Ok(Self::new(fastobo::ast::SynonymScope::Broad)),
            "NARROW" => Ok(Self::new(fastobo::ast::SynonymScope::Narrow)),
            "RELATED" => Ok(Self::new(fastobo::ast::SynonymScope::Related)),
            _ => ValueError::into(format!(
                "expected 'EXACT', 'BROAD', 'NARROW' or 'RELATED', found {:?}",
                s
            )),
        }
    }
}

impl ToPyObject for SynonymScope {
    fn to_object(&self, py: Python) -> PyObject {
        self.to_string().to_object(py)
    }
}

// --- Synonym ---------------------------------------------------------------

#[pyclass(module = "fastobo.syn")]
#[derive(Debug, PartialEq)]
pub struct Synonym {
    desc: fastobo::ast::QuotedString,
    scope: SynonymScope,
    ty: Option<Ident>,
    xrefs: XrefList,
}

impl ClonePy for Synonym {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            desc: self.desc.clone(),
            scope: self.scope.clone_py(py),
            ty: self.ty.clone_py(py),
            xrefs: self.xrefs.clone_py(py),
        }
    }
}

impl Display for Synonym {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::Synonym::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<fastobo::ast::Synonym> for Synonym {
    fn from_py(mut syn: fastobo::ast::Synonym, py: Python) -> Self {
        Self {
            desc: std::mem::replace(
                syn.description_mut(),
                fastobo::ast::QuotedString::new(String::new()),
            ),
            scope: SynonymScope::new(syn.scope().clone()),
            ty: syn.ty().map(|id| id.clone().into_py(py)),
            xrefs: std::mem::replace(
                syn.xrefs_mut(), fastobo::ast::XrefList::new(Vec::new())
            ).into_py(py)
        }
    }
}

impl FromPy<Synonym> for fastobo::ast::Synonym {
    fn from_py(syn: Synonym, py: Python) -> Self {
        Self::with_type_and_xrefs(
            syn.desc,
            syn.scope.inner,
            syn.ty.map(|ty| ty.into_py(py)),
            fastobo::ast::XrefList::from_py(syn.xrefs, py)
        )
    }
}

#[pymethods]
impl Synonym {

    #[new]
    pub fn __init__(
        obj: &PyRawObject,
        desc: String,
        scope: &str,
        ty: Option<Ident>,
        xrefs: Option<&PyAny>
    ) -> PyResult<()> {
        let list = match xrefs {
            Some(x) => XrefList::collect(obj.py(), x)?,
            None => XrefList::new(obj.py(), Vec::new()),
        };

        let synonym = Self {
            desc: fastobo::ast::QuotedString::new(desc),
            scope: SynonymScope::from_str(scope)?,
            xrefs: list,
            ty,
        };

        Ok(obj.init(synonym))
    }

    #[getter]
    pub fn get_desc(&self) -> PyResult<String> {
        Ok(self.desc.as_str().to_owned())
    }

    #[setter]
    pub fn set_desc(&mut self, desc: String) -> PyResult<()> {
        self.desc = fastobo::ast::QuotedString::new(desc);
        Ok(())
    }

    #[getter]
    pub fn get_scope(&self) -> PyResult<String> {
        Ok(self.scope.to_string())
    }

    #[setter]
    pub fn set_scope(&mut self, scope: &str) -> PyResult<()> {
        self.scope = scope.parse()?;
        Ok(())
    }

    #[getter]
    pub fn get_type(&self) -> PyResult<Option<&Ident>> {
        Ok(self.ty.as_ref())
    }

    #[setter]
    pub fn set_type(&mut self, ty: Option<Ident>) -> PyResult<()> {
        self.ty = ty;
        Ok(())
    }

    #[getter]
    pub fn get_xrefs<'py>(&self, py: Python<'py>) -> PyResult<XrefList> {
        Ok(self.xrefs.clone_py(py))
    }

    #[setter]
    pub fn set_xrefs(&mut self, xrefs: &XrefList) -> PyResult<()> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        self.xrefs = xrefs.clone_py(py);
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for Synonym {
    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, Synonym(self.desc, self.scope))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<PyObject> {
        impl_richmp!(
            self,
            other,
            op,
            self.desc && self.scope && self.ty && self.xrefs
        )
    }
}
