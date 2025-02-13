use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::ops::Deref;
use std::str::FromStr;

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyString;
use pyo3::PyTypeInfo;

use fastobo::ast;

use super::id::Ident;
use crate::utils::AbstractClass;
use crate::utils::ClonePy;
use crate::utils::EqPy;
use crate::utils::FinalClass;
use crate::utils::IntoPy;

// --- Module export ---------------------------------------------------------

#[pymodule]
#[pyo3(name = "pv")]
pub fn init<'py>(_py: Python<'py>, m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add_class::<self::AbstractPropertyValue>()?;
    m.add_class::<self::LiteralPropertyValue>()?;
    m.add_class::<self::ResourcePropertyValue>()?;
    m.add("__name__", "fastobo.pv")?;
    Ok(())
}

// --- Conversion Wrapper ----------------------------------------------------

#[derive(ClonePy, Debug, PyWrapper, EqPy)]
#[wraps(AbstractPropertyValue)]
pub enum PropertyValue {
    Literal(Py<LiteralPropertyValue>),
    Resource(Py<ResourcePropertyValue>),
}

impl Display for PropertyValue {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Python::with_gil(|py| match self {
            PropertyValue::Literal(lpv) => lpv.bind(py).borrow().fmt(f),
            PropertyValue::Resource(rpv) => rpv.bind(py).borrow().fmt(f),
        })
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
            PropertyValue::Literal(t) => t.bind(py).borrow().deref().clone_py(py).into_py(py),
            PropertyValue::Resource(r) => r.bind(py).borrow().deref().clone_py(py).into_py(py),
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
#[derive(Debug, FinalClass, EqPy)]
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
        let pv: fastobo::ast::PropertyValue = Python::with_gil(|py| self.clone_py(py).into_py(py));
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
    fn __init__<'py>(
        relation: Bound<'py, PyAny>,
        value: Bound<'py, PyAny>,
        datatype: Bound<'py, PyAny>,
    ) -> PyResult<PyClassInitializer<Self>> {
        let r = relation.extract::<Ident>()?;
        let v = if let Ok(s) = value.downcast::<PyString>() {
            ast::QuotedString::new(s.to_str()?.to_string())
        } else {
            let n = value.get_type().name()?;
            let msg = format!("expected str for value, found {}", n);
            return Err(PyTypeError::new_err(msg));
        };
        let dt = datatype.extract::<Ident>()?;
        Ok(Self::new(r, v, dt).into())
    }

    fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let fmt = PyString::new(py, "LiteralPropertyValue({!r}, {!r}, {!r})");
        fmt.call_method1(
            "format",
            (&self.relation, self.value.as_str(), &self.datatype),
        )
    }

    fn __str__(&self) -> PyResult<String> {
        let pv: fastobo::ast::PropertyValue = Python::with_gil(|py| self.clone_py(py).into_py(py));
        Ok(pv.to_string())
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

// --- Resource ------------------------------------------------------------

#[pyclass(extends=AbstractPropertyValue, module="fastobo.pv")]
#[derive(Debug, FinalClass, EqPy)]
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
        let pv: fastobo::ast::PropertyValue = Python::with_gil(|py| self.clone_py(py).into_py(py));
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
        Self::new(relation, value).into()
    }

    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, ResourcePropertyValue(self.relation, self.value))
    }

    fn __str__(&self) -> PyResult<String> {
        let pv: fastobo::ast::PropertyValue = Python::with_gil(|py| self.clone_py(py).into_py(py));
        Ok(pv.to_string())
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
