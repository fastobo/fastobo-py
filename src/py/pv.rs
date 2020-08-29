use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::ops::Deref;
use std::str::FromStr;

use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyString;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PyTypeInfo;

use fastobo::ast;

use super::id::Ident;
use crate::utils::ClonePy;
use crate::utils::FinalClass;
use crate::utils::AbstractClass;

// --- Module export ---------------------------------------------------------

#[pymodule(pv)]
pub fn init(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<self::AbstractPropertyValue>()?;
    m.add_class::<self::LiteralPropertyValue>()?;
    m.add_class::<self::ResourcePropertyValue>()?;
    m.add("__name__", "fastobo.pv")?;
    Ok(())
}

// --- Conversion Wrapper ----------------------------------------------------

#[derive(ClonePy, Debug, PartialEq, PyWrapper)]
#[wraps(AbstractPropertyValue)]
pub enum PropertyValue {
    Literal(Py<LiteralPropertyValue>),
    Resource(Py<ResourcePropertyValue>),
}

impl Display for PropertyValue {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        match self {
            PropertyValue::Literal(lpv) => lpv.as_ref(py).borrow().fmt(f),
            PropertyValue::Resource(rpv) => rpv.as_ref(py).borrow().fmt(f),
        }
    }
}

impl FromPy<fastobo::ast::PropertyValue> for PropertyValue {
    fn from_py(pv: fastobo::ast::PropertyValue, py: Python) -> Self {
        match pv {
            fastobo::ast::PropertyValue::Literal(lpv) => {
                Py::new(py, LiteralPropertyValue::from_py(*lpv, py)).map(PropertyValue::Literal)
            }
            fastobo::ast::PropertyValue::Resource(rpv) => {
                Py::new(py, ResourcePropertyValue::from_py(*rpv, py)).map(PropertyValue::Resource)
            }
        }
        .expect("could not allocate on Python heap")
    }
}

impl FromPy<PropertyValue> for fastobo::ast::PropertyValue {
    fn from_py(pv: PropertyValue, py: Python) -> Self {
        match pv {
            PropertyValue::Literal(t) => {
                Self::from_py(t.as_ref(py).borrow().deref().clone_py(py), py)
            }
            PropertyValue::Resource(r) => {
                Self::from_py(r.as_ref(py).borrow().deref().clone_py(py), py)
            }
        }
    }
}

// --- Base ------------------------------------------------------------------

#[pyclass(subclass, module = "fastobo.pv")]
#[derive(Debug, Default)]
pub struct AbstractPropertyValue {}

impl AbstractClass for AbstractPropertyValue {
    fn initializer() -> PyClassInitializer<Self> {
        PyClassInitializer::from(Self {})
    }
}

// --- Literal -----------------------------------------------------------------

#[pyclass(extends=AbstractPropertyValue, module="fastobo.pv")]
#[derive(Debug, FinalClass)]
pub struct LiteralPropertyValue {
    relation: Ident,
    value: ast::QuotedString,
    datatype: Ident,
}

impl LiteralPropertyValue {
    pub fn new<R, V, D>(py: Python, relation: R, value: V, datatype: D) -> Self
    where
        R: IntoPy<Ident>,
        V: Into<fastobo::ast::QuotedString>,
        D: IntoPy<Ident>,
    {
        LiteralPropertyValue {
            relation: relation.into_py(py),
            value: value.into(),
            datatype: datatype.into_py(py),
        }
    }
}

impl ClonePy for LiteralPropertyValue {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            relation: self.relation.clone_py(py),
            value: self.value.clone(),
            datatype: self.datatype.clone_py(py),
        }
    }
}

impl Display for LiteralPropertyValue {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pv: fastobo::ast::PropertyValue = self.clone_py(py).into_py(py);
        pv.fmt(f)
    }
}

impl FromPy<LiteralPropertyValue> for fastobo::ast::LiteralPropertyValue {
    fn from_py(pv: LiteralPropertyValue, py: Python) -> Self {
        fastobo::ast::LiteralPropertyValue::new(
            pv.relation.into_py(py),
            pv.value,
            pv.datatype.into_py(py)
        )
    }
}

impl FromPy<LiteralPropertyValue> for fastobo::ast::PropertyValue {
    fn from_py(pv: LiteralPropertyValue, py: Python) -> Self {
        fastobo::ast::PropertyValue::from(
            fastobo::ast::LiteralPropertyValue::from_py(pv, py)
        )
    }
}

impl FromPy<fastobo::ast::LiteralPropertyValue> for LiteralPropertyValue {
    fn from_py(mut pv: fastobo::ast::LiteralPropertyValue, py: Python) -> Self {
        let value = std::mem::take(pv.literal_mut());
        let datatype = pv.datatype().clone();
        let relation = pv.property().clone();
        Self::new(py, relation, value, datatype)
    }
}

#[pymethods]
impl LiteralPropertyValue {
    #[new]
    fn __init__(
        relation: &PyAny,
        value: &PyAny,
        datatype: &PyAny,
    ) -> PyResult<PyClassInitializer<Self>> {
        let r = relation.extract::<Ident>()?;
        let v = if let Ok(s) = value.extract::<&PyString>() {
            ast::QuotedString::new(s.to_string()?.to_string())
        } else {
            let n = value.get_type().name();
            return TypeError::into(format!("expected str for value, found {}", n));
        };
        let dt = datatype.extract::<Ident>()?;
        Ok(Self::new(relation.py(), r, v, dt).into())
    }

    #[getter]
    fn get_relation(&self) -> PyResult<&Ident> {
        Ok(&self.relation)
    }

    #[setter]
    fn set_relation(&mut self, relation: Ident) -> PyResult<()> {
        self.relation = relation;
        Ok(())
    }

    #[getter]
    fn get_value(&self) -> PyResult<&str> {
        Ok(self.value.as_str())
    }

    #[setter]
    fn set_value(&mut self, value: String) -> PyResult<()> {
        self.value = fastobo::ast::QuotedString::new(value);
        Ok(())
    }

    #[getter]
    fn get_datatype(&self) -> PyResult<&Ident> {
        Ok(&self.datatype)
    }

    #[setter]
    fn set_datatype(&mut self, datatype: Ident) -> PyResult<()> {
        self.datatype = datatype;
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for LiteralPropertyValue {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let fmt = PyString::new(py, "LiteralPropertyValue({!r}, {!r}, {!r})");
        fmt.to_object(py).call_method1(
            py,
            "format",
            (
                self.relation.to_object(py),
                self.value.as_str(),
                self.datatype.to_object(py),
            ),
        )
    }

    fn __str__(&self) -> PyResult<String> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pv: fastobo::ast::PropertyValue = self.clone_py(py).into_py(py);
        Ok(pv.to_string())
    }
}

// --- Resource ------------------------------------------------------------

#[pyclass(extends=AbstractPropertyValue, module="fastobo.pv")]
#[derive(Debug, FinalClass)]
pub struct ResourcePropertyValue {
    relation: Ident,
    value: Ident,
}

impl ResourcePropertyValue {
    pub fn new<R, V>(py: Python, relation: R, value: V) -> Self
    where
        R: IntoPy<Ident>,
        V: IntoPy<Ident>,
    {
        ResourcePropertyValue {
            relation: relation.into_py(py),
            value: value.into_py(py),
        }
    }
}

impl ClonePy for ResourcePropertyValue {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            relation: self.relation.clone_py(py),
            value: self.value.clone_py(py),
        }
    }
}

impl Display for ResourcePropertyValue {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let pv: fastobo::ast::PropertyValue = self.clone_py(py).into_py(py);
        pv.fmt(f)
    }
}

impl FromPy<ResourcePropertyValue> for fastobo::ast::ResourcePropertyValue {
    fn from_py(pv: ResourcePropertyValue, py: Python) -> Self {
        Self::new(pv.relation.into_py(py), pv.value.into_py(py))
    }
}

impl FromPy<ResourcePropertyValue> for fastobo::ast::PropertyValue {
    fn from_py(pv: ResourcePropertyValue, py: Python) -> Self {
        fastobo::ast::PropertyValue::from(
            fastobo::ast::ResourcePropertyValue::from_py(pv, py)
        )
    }
}

impl FromPy<fastobo::ast::ResourcePropertyValue> for ResourcePropertyValue {
    fn from_py(pv: fastobo::ast::ResourcePropertyValue, py: Python) -> Self {
        let relation = pv.property().clone();
        let value = pv.target().clone();
        ResourcePropertyValue::new(py, relation, value)
    }
}

#[pymethods]
impl ResourcePropertyValue {
    #[new]
    fn __init__(relation: Ident, value: Ident) -> PyClassInitializer<Self> {
        let gil = Python::acquire_gil();
        Self::new(gil.python(), relation, value).into()
    }

    #[getter]
    fn get_relation(&self) -> PyResult<&Ident> {
        Ok(&self.relation)
    }

    #[setter]
    fn set_relation(&mut self, relation: Ident) -> PyResult<()> {
        self.relation = relation;
        Ok(())
    }

    #[getter]
    fn get_value(&self) -> PyResult<&Ident> {
        Ok(&self.value)
    }

    #[setter]
    fn set_value(&mut self, value: Ident) -> PyResult<()> {
        self.value = value;
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for ResourcePropertyValue {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let fmt = PyString::new(py, "ResourcePropertyValue({!r}, {!r})");
        fmt.to_object(py).call_method1(
            py,
            "format",
            (self.relation.to_object(py), self.value.to_object(py)),
        )
    }

    fn __str__(&self) -> PyResult<String> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pv: fastobo::ast::PropertyValue = self.clone_py(py).into_py(py);
        Ok(pv.to_string())
    }
}
