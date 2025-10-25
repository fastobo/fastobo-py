use std::convert::Infallible;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::rc::Rc;
use std::str::FromStr;
use std::string::ToString;

use fastobo::ast::QuotedString;
use pyo3::class::basic::CompareOp;
use pyo3::class::gc::PyVisit;
use pyo3::exceptions::PyIndexError;
use pyo3::exceptions::PyTypeError;
use pyo3::gc::PyTraverseError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyIterator;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::PyTypeInfo;

use super::id::Ident;
use crate::utils::ClonePy;
use crate::utils::EqPy;
use crate::utils::IntoPy;

// --- Module export ---------------------------------------------------------

#[pymodule]
#[pyo3(name = "qual")]
pub fn init<'py>(_py: Python<'py>, m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add_class::<self::Qualifier>()?;
    m.add_class::<self::QualifierList>()?;
    m.add("__name__", "fastobo.qual")?;
    Ok(())
}

// --- Qualifier ------------------------------------------------------------------

/// A qualifier used as a modifier for a clause.
///
#[pyclass(module = "fastobo.qual")]
#[derive(Debug, EqPy)]
pub struct Qualifier {
    #[pyo3(set)]
    key: Ident,
    value: fastobo::ast::QuotedString,
}

impl Qualifier {
    pub fn new(key: Ident, value: fastobo::ast::QuotedString) -> Self {
        Self { key, value }
    }
}

impl ClonePy for Qualifier {
    fn clone_py(&self, py: Python) -> Self {
        Qualifier {
            key: self.key.clone_py(py),
            value: self.value.clone(),
        }
    }
}

impl Display for Qualifier {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let xref: fastobo::ast::Qualifier = Python::with_gil(|py| self.clone_py(py).into_py(py));
        xref.fmt(f)
    }
}

impl IntoPy<Qualifier> for fastobo::ast::Qualifier {
    fn into_py(mut self, py: Python) -> Qualifier {
        // Take ownership over `qualifier.description` w/o reallocation or clone.
        let value = std::mem::take(self.value_mut());

        // Take ownership over `qualifier.id` w/o reallocation or clone.
        let empty = fastobo::ast::UnprefixedIdent::new(String::new());
        let key = std::mem::replace(self.key_mut(), empty.into());

        Qualifier::new(key.into_py(py), value)
    }
}

impl IntoPy<fastobo::ast::Qualifier> for Qualifier {
    fn into_py(self, py: Python) -> fastobo::ast::Qualifier {
        let key: fastobo::ast::Ident = self.key.into_py(py);
        fastobo::ast::Qualifier::new(key.into(), self.value)
    }
}

#[pymethods]
impl Qualifier {
    /// Create a new `Qualifier` instance from a key and value.
    ///
    /// Arguments:
    ///     key (~fastobo.id.Ident): the key of the qualifier.
    ///     value (str, optional): the value of the qualifier.
    #[new]
    #[pyo3(signature = (key, value))]
    fn __init__(key: Ident, value: &str) -> Self {
        Self::new(key, fastobo::ast::QuotedString::from(value))
    }

    fn __repr__<'py>(slf: PyRef<'py, Self>) -> PyResult<Bound<'py, PyAny>> {
        let py = slf.py();
        PyString::new(py, "Qualifier({!r}, {!r})")
            .call_method1("format", (&slf.key, slf.value.as_str()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__<'py>(&self, other: &Bound<'py, PyAny>, op: CompareOp) -> PyResult<PyObject> {
        impl_richcmp_py!(self, other, op, self.key && self.value)
    }

    /// `~fastobo.id.Ident`: the key of the qualifier.
    #[getter]
    fn get_key(&self) -> &Ident {
        &self.key
    }

    /// `str`: the value of the qualifier.
    #[getter]
    fn get_value(&self) -> &str {
        self.value.as_str()
    }

    #[setter]
    fn set_value(&mut self, value: &str) -> PyResult<()> {
        self.value = fastobo::ast::QuotedString::new(value);
        Ok(())
    }
}

// --- XrefList --------------------------------------------------------------

/// A list of qualifiers.
///
#[pyclass(module = "fastobo.xref")]
#[derive(Debug, Default, EqPy)]
pub struct QualifierList {
    qualifiers: Vec<Py<Qualifier>>,
}

impl QualifierList {
    /// Create a new `XrefList` from a vector of Xrefs.
    pub fn new(qualifiers: Vec<Py<Qualifier>>) -> Self {
        Self { qualifiers }
    }

    /// Create a new `QualifierList` from a `PyIterator`.
    pub fn collect<'py>(py: Python<'py>, qualifiers: &Bound<'py, PyAny>) -> PyResult<Self> {
        let mut vec = Vec::new();
        for item in PyIterator::from_object(qualifiers)? {
            let i = item?;
            if let Ok(qual) = i.extract::<Py<Qualifier>>() {
                vec.push(qual.clone_ref(py));
            } else {
                let ty = i.get_type().name()?;
                let msg = format!("expected Qualifier, found {}", ty);
                return Err(PyTypeError::new_err(msg));
            }
        }
        Ok(Self { qualifiers: vec })
    }

    /// Check whether the `XrefList` is empty
    pub fn is_empty(&self) -> bool {
        self.qualifiers.is_empty()
    }
}

impl ClonePy for QualifierList {
    fn clone_py(&self, py: Python) -> Self {
        QualifierList {
            qualifiers: self.qualifiers.clone_py(py),
        }
    }
}

impl IntoPy<QualifierList> for fastobo::ast::QualifierList {
    fn into_py(self, py: Python) -> QualifierList {
        let mut qualifiers = Vec::with_capacity((&self).len());
        for qual in self.into_iter() {
            qualifiers.push(Py::new(py, qual.into_py(py)).unwrap())
        }
        QualifierList::new(qualifiers)
    }
}

impl IntoPy<fastobo::ast::QualifierList> for QualifierList {
    fn into_py(self, py: Python) -> fastobo::ast::QualifierList {
        (&self).into_py(py)
    }
}

impl IntoPy<fastobo::ast::QualifierList> for &QualifierList {
    fn into_py<'py>(self, py: Python) -> fastobo::ast::QualifierList {
        self.qualifiers
            .iter()
            .map(|qual| qual.bind(py).borrow().clone_py(py).into_py(py))
            .collect()
    }
}

#[listlike(field = "qualifiers", type = "Py<Qualifier>")]
#[pymethods]
impl QualifierList {
    #[new]
    #[pyo3(signature = (qualifiers = None))]
    fn __init__<'py>(qualifiers: Option<&Bound<'py, PyAny>>) -> PyResult<Self> {
        if let Some(x) = qualifiers {
            Python::with_gil(|py| Self::collect(py, x))
        } else {
            Ok(Self::new(Vec::new()))
        }
    }

    fn __repr__(slf: PyRef<Self>) -> PyResult<Bound<PyAny>> {
        if slf.qualifiers.is_empty() {
            Ok(PyString::intern(slf.py(), "QualifierList()").into_any())
        } else {
            let fmt = PyString::intern(slf.py(), "QualifierList({!r})");
            fmt.call_method1("format", (&slf.qualifiers,))
        }
    }

    fn __str__(&self) -> PyResult<String> {
        let frame: fastobo::ast::QualifierList =
            Python::with_gil(|py| self.clone_py(py).into_py(py));
        Ok(frame.to_string())
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.qualifiers.len())
    }

    fn __getitem__(&self, index: isize) -> PyResult<Py<Qualifier>> {
        if index < self.qualifiers.len() as isize {
            Python::with_gil(|py| Ok(self.qualifiers[index as usize].clone_ref(py)))
        } else {
            Err(PyIndexError::new_err("list index out of range"))
        }
    }

    fn __contains__<'py>(&self, item: &Bound<'py, PyAny>) -> PyResult<bool> {
        if let Ok(qual) = item.extract::<Py<Qualifier>>() {
            let py = item.py();
            Ok(self
                .qualifiers
                .iter()
                .any(|x| (*x.bind(py).borrow()).eq_py(&qual.bind(py).borrow(), py)))
        } else {
            let ty = item.get_type().name()?;
            let msg = format!(
                "'in <QualifierList>' requires Qualifier as left operand, not {}",
                ty
            );
            Err(PyTypeError::new_err(msg))
        }
    }
}
