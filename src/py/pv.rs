use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
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
use fastobo::share::Cow;
use fastobo::share::Share;

use super::id::Ident;
use crate::utils::AsGILRef;
use crate::utils::ClonePy;

// --- Module export ---------------------------------------------------------

#[pymodule(pv)]
fn module(_py: Python, m: &PyModule) -> PyResult<()> {
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

impl<'p> AsGILRef<'p, fastobo::ast::PropVal<'p>> for PropertyValue {
    fn as_gil_ref(&'p self, py: Python<'p>) -> fastobo::ast::PropVal<'p> {
        match self {
            PropertyValue::Literal(pv) => pv.as_gil_ref(py).as_gil_ref(py),
            PropertyValue::Resource(pv) => pv.as_gil_ref(py).as_gil_ref(py),
        }
    }
}

impl Display for PropertyValue {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        self.as_gil_ref(gil.python()).fmt(f)
    }
}

impl FromPy<fastobo::ast::PropertyValue> for PropertyValue {
    fn from_py(pv: fastobo::ast::PropertyValue, py: Python) -> Self {
        match pv {
            fastobo::ast::PropertyValue::Literal(r, d, ty) => {
                Py::new(py, LiteralPropertyValue::new(py, r, d, ty)).map(PropertyValue::Literal)
            }
            fastobo::ast::PropertyValue::Resource(r, v) => {
                Py::new(py, ResourcePropertyValue::new(py, r, v)).map(PropertyValue::Resource)
            }
        }
        .expect("could not allocate on Python heap")
    }
}

impl FromPy<PropertyValue> for fastobo::ast::PropertyValue {
    fn from_py(pv: PropertyValue, py: Python) -> Self {
        match pv {
            PropertyValue::Literal(t) => Self::from_py(t.as_ref(py).clone_py(py), py),
            PropertyValue::Resource(i) => Self::from_py(i.as_ref(py).clone_py(py), py),
        }
    }
}

// --- Base ------------------------------------------------------------------

#[pyclass(subclass, module = "fastobo.pv")]
#[derive(Debug)]
pub struct AbstractPropertyValue {}

// --- Literal -----------------------------------------------------------------

#[pyclass(extends=AbstractPropertyValue, module="fastobo.pv")]
#[derive(Debug)]
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

impl<'p> AsGILRef<'p, fastobo::ast::PropVal<'p>> for LiteralPropertyValue {
    fn as_gil_ref(&'p self, py: Python<'p>) -> fastobo::ast::PropVal<'p> {
        fastobo::ast::PropVal::Literal(
            Cow::Borrowed(self.relation.as_gil_ref(py).into()),
            Cow::Borrowed(self.value.share()),
            Cow::Borrowed(self.datatype.as_gil_ref(py)),
        )
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
        self.as_gil_ref(gil.python()).fmt(f)
    }
}

impl FromPy<LiteralPropertyValue> for fastobo::ast::PropertyValue {
    fn from_py(pv: LiteralPropertyValue, py: Python) -> Self {
        fastobo::ast::PropertyValue::Literal(
            pv.relation.into_py(py),
            pv.value,
            pv.datatype.into_py(py),
        )
    }
}

#[pymethods]
impl LiteralPropertyValue {
    #[new]
    fn __init__(
        obj: &PyRawObject,
        relation: &PyAny,
        value: &PyAny,
        datatype: &PyAny,
    ) -> PyResult<()> {
        let r = relation.extract::<Ident>()?;
        let v = if let Ok(s) = value.extract::<&PyString>() {
            ast::QuotedString::new(s.to_string()?.to_string())
        } else {
            let n = value.get_type().name();
            return TypeError::into(format!("expected str for value, found {}", n));
        };
        let dt = datatype.extract::<Ident>()?;
        Ok(obj.init(Self::new(obj.py(), r, v, dt)))
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
        Ok(self.as_gil_ref(py).to_string())
    }
}

// --- Resource ------------------------------------------------------------

#[pyclass(extends=AbstractPropertyValue, module="fastobo.pv")]
#[derive(Debug)]
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

impl<'p> AsGILRef<'p, fastobo::ast::PropVal<'p>> for ResourcePropertyValue {
    fn as_gil_ref(&'p self, py: Python<'p>) -> fastobo::ast::PropVal<'p> {
        fastobo::ast::PropVal::Resource(
            Cow::Borrowed(self.relation.as_gil_ref(py).into()),
            Cow::Borrowed(self.value.as_gil_ref(py).into()),
        )
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
        self.as_gil_ref(gil.python()).fmt(f)
    }
}

impl FromPy<ResourcePropertyValue> for fastobo::ast::PropertyValue {
    fn from_py(pv: ResourcePropertyValue, py: Python) -> Self {
        fastobo::ast::PropertyValue::Resource(pv.relation.into_py(py), pv.value.into_py(py))
    }
}

#[pymethods]
impl ResourcePropertyValue {
    #[new]
    fn __init__(obj: &PyRawObject, relation: Ident, value: Ident) -> PyResult<()> {
        Ok(obj.init(Self::new(obj.py(), relation, value)))
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
        Ok(self.as_gil_ref(py).to_string())
    }
}
