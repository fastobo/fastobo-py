use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::ops::Deref;
use std::str::FromStr;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyString;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PyTypeInfo;

use fastobo::ast;

use super::id::Ident;
use crate::utils::AbstractClass;
use crate::utils::ClonePy;
use crate::utils::FinalClass;

// --- Module export ---------------------------------------------------------

#[pymodule]
#[pyo3(name = "pv")]
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

impl IntoPy<PropertyValue> for fastobo::ast::PropertyValue {
    fn into_py(self, py: Python) -> PropertyValue {
        match self {
            fastobo::ast::PropertyValue::Literal(lpv) => {
                Py::new(py, lpv.into_py(py)).map(PropertyValue::Literal)
            }
            fastobo::ast::PropertyValue::Resource(rpv) => {
                Py::new(py, rpv.into_py(py)).map(PropertyValue::Resource)
            }
        }
        .expect("could not allocate on Python heap")
    }
}

impl IntoPy<fastobo::ast::PropertyValue> for PropertyValue {
    fn into_py(self, py: Python) -> fastobo::ast::PropertyValue {
        match self {
            PropertyValue::Literal(t) => t.as_ref(py).borrow().deref().clone_py(py).into_py(py),
            PropertyValue::Resource(r) => r.as_ref(py).borrow().deref().clone_py(py).into_py(py),
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
#[base(AbstractPropertyValue)]
pub struct LiteralPropertyValue {
    relation: Ident,
    value: ast::QuotedString,
    datatype: Ident,
}

impl LiteralPropertyValue {
    pub fn new(relation: Ident, value: fastobo::ast::QuotedString, datatype: Ident) -> Self {
        LiteralPropertyValue {
            relation,
            value,
            datatype,
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

impl IntoPy<fastobo::ast::LiteralPropertyValue> for LiteralPropertyValue {
    fn into_py(self, py: Python) -> fastobo::ast::LiteralPropertyValue {
        fastobo::ast::LiteralPropertyValue::new(
            self.relation.into_py(py),
            self.value,
            self.datatype.into_py(py),
        )
    }
}

impl IntoPy<fastobo::ast::PropertyValue> for LiteralPropertyValue {
    fn into_py(self, py: Python) -> fastobo::ast::PropertyValue {
        fastobo::ast::PropertyValue::Literal(Box::new(self.into_py(py)))
    }
}

impl IntoPy<LiteralPropertyValue> for fastobo::ast::LiteralPropertyValue {
    fn into_py(mut self, py: Python) -> LiteralPropertyValue {
        let value = std::mem::take(self.literal_mut());
        let datatype = self.datatype().clone().into_py(py);
        let relation = self.property().clone().into_py(py);
        LiteralPropertyValue::new(relation, value, datatype)
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
            ast::QuotedString::new(s.to_str()?.to_string())
        } else {
            let n = value.get_type().name()?;
            let msg = format!("expected str for value, found {}", n);
            return Err(PyTypeError::new_err(msg));
        };
        let dt = datatype.extract::<Ident>()?;
        Ok(Self::new(r, v, dt).into())
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
#[base(AbstractPropertyValue)]
pub struct ResourcePropertyValue {
    relation: Ident,
    value: Ident,
}

impl ResourcePropertyValue {
    pub fn new(relation: Ident, value: Ident) -> Self {
        ResourcePropertyValue { relation, value }
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

impl IntoPy<fastobo::ast::ResourcePropertyValue> for ResourcePropertyValue {
    fn into_py(self, py: Python) -> fastobo::ast::ResourcePropertyValue {
        fastobo::ast::ResourcePropertyValue::new(self.relation.into_py(py), self.value.into_py(py))
    }
}

impl IntoPy<fastobo::ast::PropertyValue> for ResourcePropertyValue {
    fn into_py(self, py: Python) -> fastobo::ast::PropertyValue {
        fastobo::ast::PropertyValue::Resource(Box::new(self.into_py(py)))
    }
}

impl IntoPy<ResourcePropertyValue> for fastobo::ast::ResourcePropertyValue {
    fn into_py(self, py: Python) -> ResourcePropertyValue {
        let relation = self.property().clone().into_py(py);
        let value = self.target().clone().into_py(py);
        ResourcePropertyValue::new(relation, value)
    }
}

#[pymethods]
impl ResourcePropertyValue {
    #[new]
    fn __init__(relation: Ident, value: Ident) -> PyClassInitializer<Self> {
        let gil = Python::acquire_gil();
        Self::new(relation, value).into()
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
        impl_repr!(self, ResourcePropertyValue(self.relation, self.value))
    }

    fn __str__(&self) -> PyResult<String> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let pv: fastobo::ast::PropertyValue = self.clone_py(py).into_py(py);
        Ok(pv.to_string())
    }
}
