use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::str::FromStr;

use pyo3::PyObjectProtocol;
use pyo3::PyTypeInfo;
use pyo3::PyNativeType;
use pyo3::prelude::*;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::types::PyAny;
use pyo3::types::PyString;

use fastobo::ast;
use fastobo::share::Cow;
use fastobo::share::Share;

use crate::utils::AsGILRef;
use crate::utils::ClonePy;
use super::id::Ident;

// --- Module export ---------------------------------------------------------

#[pymodule(pv)]
fn module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<self::BasePropertyValue>()?;
    m.add_class::<self::TypedPropertyValue>()?;
    m.add_class::<self::IdentifiedPropertyValue>()?;
    Ok(())
}

// --- Conversion Wrapper ----------------------------------------------------

#[derive(ClonePy, Debug, PartialEq, PyWrapper)]
#[wraps(BasePropertyValue)]
pub enum PropertyValue {
    Typed(Py<TypedPropertyValue>),
    Identified(Py<IdentifiedPropertyValue>),
}

impl<'p> AsGILRef<'p, fastobo::ast::PropVal<'p>> for PropertyValue {
    fn as_gil_ref(&'p self, py: Python<'p>) -> fastobo::ast::PropVal<'p> {
        match self {
            PropertyValue::Typed(pv) => pv.as_gil_ref(py).as_gil_ref(py),
            PropertyValue::Identified(pv) => pv.as_gil_ref(py).as_gil_ref(py),
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
            fastobo::ast::PropertyValue::Typed(r, d, ty) =>
                Py::new(py, TypedPropertyValue::new(py, r, d, ty))
                    .map(PropertyValue::Typed),
            fastobo::ast::PropertyValue::Identified(r, v) =>
                Py::new(py, IdentifiedPropertyValue::new(py, r, v))
                    .map(PropertyValue::Identified),
        }.expect("could not allocate on Python heap")
    }
}

impl FromPy<PropertyValue> for fastobo::ast::PropertyValue {
    fn from_py(pv: PropertyValue, py: Python) -> Self {
        match pv {
            PropertyValue::Typed(t) =>
                Self::from_py(t.as_ref(py).clone_py(py), py),
            PropertyValue::Identified(i) =>
                Self::from_py(i.as_ref(py).clone_py(py), py)
        }
    }
}

// --- Base ------------------------------------------------------------------

#[pyclass(subclass)]
#[derive(Debug)]
pub struct BasePropertyValue {}

// --- Typed -----------------------------------------------------------------

#[pyclass(extends=BasePropertyValue)]
#[derive(Debug)]
pub struct TypedPropertyValue {
    relation: Ident,
    value: ast::QuotedString,
    datatype: Ident,
}

impl TypedPropertyValue {
    pub fn new<R, V, D>(py: Python, relation: R, value: V, datatype: D) -> Self
    where
        R: IntoPy<Ident>,
        V: Into<fastobo::ast::QuotedString>,
        D: IntoPy<Ident>
    {
        TypedPropertyValue {
            relation: relation.into_py(py),
            value: value.into(),
            datatype: datatype.into_py(py),
        }
    }
}

impl<'p> AsGILRef<'p, fastobo::ast::PropVal<'p>> for TypedPropertyValue  {
    fn as_gil_ref(&'p self, py: Python<'p>) -> fastobo::ast::PropVal<'p> {
        fastobo::ast::PropVal::Typed(
            Cow::Borrowed(self.relation.as_gil_ref(py).into()),
            Cow::Borrowed(self.value.share()),
            Cow::Borrowed(self.datatype.as_gil_ref(py))
        )
    }
}

impl ClonePy for TypedPropertyValue {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            relation: self.relation.clone_py(py),
            value: self.value.clone(),
            datatype: self.datatype.clone_py(py)
        }
    }
}

impl Display for TypedPropertyValue {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        self.as_gil_ref(gil.python()).fmt(f)
    }
}

impl FromPy<TypedPropertyValue> for fastobo::ast::PropertyValue {
    fn from_py(pv: TypedPropertyValue, py: Python) -> Self {
        fastobo::ast::PropertyValue::Typed(
            pv.relation.into_py(py),
            pv.value,
            pv.datatype.into_py(py),
        )
    }
}

#[pymethods]
impl TypedPropertyValue {
    #[new]
    fn __init__(obj: &PyRawObject, relation: &PyAny , value: &PyAny, datatype: &PyAny) -> PyResult<()> {
        let r = relation.extract::<Ident>()?;
        let v = if let Ok(s) = value.extract::<&PyString>() {
            ast::QuotedString::new(s.to_string()?.to_string())
        } else {
            let n = value.get_type().name();
            return TypeError::into(format!("expected str for value, found {}", n))
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
impl PyObjectProtocol for TypedPropertyValue {

    fn __repr__(&self) -> PyResult<PyObject> {
        let py = unsafe { Python::assume_gil_acquired() };
        let fmt = PyString::new(py, "TypedPropertyValue({!r}, {!r}, {!r})");
        fmt.to_object(py).call_method1(py, "format", (
            self.relation.to_object(py),
            self.value.as_str(),
            self.datatype.to_object(py))
        )
    }

    fn __str__(&self) -> PyResult<String> {
        let py = unsafe { Python::assume_gil_acquired() };
        Ok(self.as_gil_ref(py).to_string())
    }
}

// --- Identified ------------------------------------------------------------

#[pyclass(extends=BasePropertyValue)]
#[derive(Debug)]
pub struct IdentifiedPropertyValue {
    relation: Ident,
    value: Ident,
}

impl IdentifiedPropertyValue {
    pub fn new<R, V>(py: Python, relation: R, value: V) -> Self
    where
        R: IntoPy<Ident>,
        V: IntoPy<Ident>,
    {
        IdentifiedPropertyValue {
            relation: relation.into_py(py),
            value: value.into_py(py)
        }
    }
}

impl<'p> AsGILRef<'p, fastobo::ast::PropVal<'p>> for IdentifiedPropertyValue {
    fn as_gil_ref(&'p self, py: Python<'p>) -> fastobo::ast::PropVal<'p> {
        fastobo::ast::PropVal::Identified(
            Cow::Borrowed(self.relation.as_gil_ref(py).into()),
            Cow::Borrowed(self.value.as_gil_ref(py).into())
        )
    }
}

impl ClonePy for IdentifiedPropertyValue {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            relation: self.relation.clone_py(py),
            value: self.value.clone_py(py),
        }
    }
}

impl Display for IdentifiedPropertyValue {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        self.as_gil_ref(gil.python()).fmt(f)
    }
}

impl FromPy<IdentifiedPropertyValue> for fastobo::ast::PropertyValue {
    fn from_py(pv: IdentifiedPropertyValue, py: Python) -> Self {
        fastobo::ast::PropertyValue::Identified(
            pv.relation.into_py(py),
            pv.value.into_py(py),
        )
    }
}

#[pymethods]
impl IdentifiedPropertyValue {
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
impl PyObjectProtocol for IdentifiedPropertyValue {
    fn __repr__(&self) -> PyResult<PyObject> {
        let py = unsafe { Python::assume_gil_acquired() };
        let fmt = PyString::new(py, "IdentifiedPropertyValue({!r}, {!r})");
        fmt.to_object(py).call_method1(py, "format", (
            self.relation.to_object(py),
            self.value.to_object(py),
        ))
    }

    fn __str__(&self) -> PyResult<String> {
        let py = unsafe { Python::assume_gil_acquired() };
        Ok(self.as_gil_ref(py).to_string())
    }
}
