use std::convert::Infallible;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::rc::Rc;
use std::str::FromStr;
use std::string::ToString;

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
use pyo3::AsPyPointer;
use pyo3::PyTypeInfo;

use super::id::Ident;
use crate::utils::ClonePy;
use crate::utils::EqPy;
use crate::utils::IntoPy;

// --- Module export ---------------------------------------------------------

#[pymodule]
#[pyo3(name = "xref")]
pub fn init<'py>(_py: Python<'py>, m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add_class::<self::Xref>()?;
    m.add_class::<self::XrefList>()?;
    m.add("__name__", "fastobo.xref")?;
    Ok(())
}

// --- Xref ------------------------------------------------------------------

/// A cross-reference to another entity or an external resource.
///
/// Xrefs can be used in a `~fastobo.term.DefClause` to indicate the provenance
/// of the definition, or in a `~fastobo.syn.Synonym` to add evidence from
/// literature supporting the origin of the synonym.
///
/// Example:
///     >>> xref = fastobo.xref.Xref(
///     ...     fastobo.id.PrefixedIdent('ISBN', '978-0-321-84268-8'),
///     ... )
#[pyclass(module = "fastobo.xref")]
#[derive(Debug, EqPy)]
pub struct Xref {
    #[pyo3(set)]
    id: Ident,
    desc: Option<fastobo::ast::QuotedString>,
}

impl Xref {
    pub fn new(id: Ident) -> Self {
        Self { id, desc: None }
    }

    pub fn with_desc(id: Ident, desc: Option<fastobo::ast::QuotedString>) -> Self {
        Self { id, desc }
    }
}

impl ClonePy for Xref {
    fn clone_py(&self, py: Python) -> Self {
        Xref {
            id: self.id.clone_py(py),
            desc: self.desc.clone(),
        }
    }
}

impl Display for Xref {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let xref: fastobo::ast::Xref = Python::with_gil(|py| self.clone_py(py).into_py(py));
        xref.fmt(f)
    }
}

impl IntoPy<Xref> for fastobo::ast::Xref {
    fn into_py(mut self, py: Python) -> Xref {
        // Take ownership over `xref.description` w/o reallocation or clone.
        let desc = self.description_mut().map(|d| std::mem::take(d));

        // Take ownership over `xref.id` w/o reallocation or clone.
        let empty = fastobo::ast::UnprefixedIdent::new(String::new());
        let id = std::mem::replace(self.id_mut(), empty.into());

        Xref::with_desc(id.into_py(py), desc)
    }
}

impl IntoPy<fastobo::ast::Xref> for Xref {
    fn into_py(self, py: Python) -> fastobo::ast::Xref {
        let id: fastobo::ast::Ident = self.id.into_py(py);
        fastobo::ast::Xref::with_desc(id, self.desc)
    }
}

#[pymethods]
impl Xref {
    /// Create a new `Xref` instance from an ID and an optional description.
    ///
    /// Arguments:
    ///     id (~fastobo.id.Ident): the identifier of the reference.
    ///     desc (str, optional): an optional description for the reference.
    #[new]
    fn __init__(id: Ident, desc: Option<String>) -> Self {
        if let Some(s) = desc {
            Self::with_desc(id, Some(fastobo::ast::QuotedString::new(s)))
        } else {
            Self::new(id)
        }
    }

    fn __repr__<'py>(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            if let Some(ref d) = self.desc {
                PyString::new(py, "Xref({!r}, {!r})")
                    .call_method1("format", (&self.id, d.as_str()))
                    .map(|x| x.unbind())
            } else {
                PyString::new(py, "Xref({!r})")
                    .call_method1("format", (&self.id,))
                    .map(|x| x.unbind())
            }
        })
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__<'py>(&self, other: &Bound<'py, PyAny>, op: CompareOp) -> PyResult<PyObject> {
        impl_richcmp_py!(self, other, op, self.id && self.desc)
    }

    /// `~fastobo.id.Ident`: the identifier of the reference.
    #[getter]
    fn get_id(&self) -> PyResult<&Ident> {
        Ok(&self.id)
    }

    /// `str` or `None`: the description of the reference, if any.
    #[getter]
    fn get_desc(&self) -> PyResult<Option<&str>> {
        match &self.desc {
            Some(d) => Ok(Some(d.as_str())),
            None => Ok(None),
        }
    }

    #[setter]
    fn set_desc(&mut self, desc: Option<String>) -> PyResult<()> {
        self.desc = desc.map(fastobo::ast::QuotedString::new);
        Ok(())
    }
}

// --- XrefList --------------------------------------------------------------

/// A list of cross-references.
///
/// Example:
///     >>> xrefs = ms[0][1].xrefs
///     >>> print(xrefs)
///     [PSI:MS]
///     >>> xrefs[0]
///     Xref(PrefixedIdent('PSI', 'MS'))
///
#[pyclass(module = "fastobo.xref")]
#[derive(Debug, Default, EqPy)]
pub struct XrefList {
    xrefs: Vec<Py<Xref>>,
}

impl XrefList {
    /// Create a new `XrefList` from a vector of Xrefs.
    pub fn new(xrefs: Vec<Py<Xref>>) -> Self {
        Self { xrefs }
    }

    /// Create a new `XrefList` from a `PyIterator`.
    pub fn collect<'py>(py: Python<'py>, xrefs: &Bound<'py, PyAny>) -> PyResult<Self> {
        let mut vec = Vec::new();
        for item in PyIterator::from_object(xrefs)? {
            let i = item?;
            if let Ok(xref) = i.extract::<Py<Xref>>() {
                vec.push(xref.clone_ref(py));
            } else {
                let ty = i.get_type().name()?;
                let msg = format!("expected Xref, found {}", ty);
                return Err(PyTypeError::new_err(msg));
            }
        }
        Ok(Self { xrefs: vec })
    }

    /// Check whether the `XrefList` is empty
    pub fn is_empty(&self) -> bool {
        self.xrefs.is_empty()
    }
}

impl ClonePy for XrefList {
    fn clone_py(&self, py: Python) -> Self {
        XrefList {
            xrefs: self.xrefs.clone_py(py),
        }
    }
}

impl IntoPy<XrefList> for fastobo::ast::XrefList {
    fn into_py(self, py: Python) -> XrefList {
        let mut xrefs = Vec::with_capacity((&self).len());
        for xref in self.into_iter() {
            xrefs.push(Py::new(py, xref.into_py(py)).unwrap())
        }
        XrefList::new(xrefs)
    }
}

impl IntoPy<fastobo::ast::XrefList> for XrefList {
    fn into_py(self, py: Python) -> fastobo::ast::XrefList {
        (&self).into_py(py)
    }
}

impl IntoPy<fastobo::ast::XrefList> for &XrefList {
    fn into_py<'py>(self, py: Python) -> fastobo::ast::XrefList {
        self.xrefs
            .iter()
            .map(|xref| xref.bind(py).borrow().clone_py(py).into_py(py))
            .collect()
    }
}

#[listlike(field = "xrefs", type = "Py<Xref>")]
#[pymethods]
impl XrefList {
    #[new]
    #[pyo3(signature = (xrefs = None))]
    fn __init__<'py>(xrefs: Option<&Bound<'py, PyAny>>) -> PyResult<Self> {
        if let Some(x) = xrefs {
            Python::with_gil(|py| Self::collect(py, x))
        } else {
            Ok(Self::new(Vec::new()))
        }
    }

    fn __repr__(&self) -> PyResult<PyObject> {
        Python::with_gil(|py| {
            if self.xrefs.is_empty() {
                Ok("XrefList()".to_object(py))
            } else {
                let fmt = PyString::new(py, "XrefList({!r})").to_object(py);
                fmt.call_method1(py, "format", (&self.xrefs.to_object(py),))
            }
        })
    }

    fn __str__(&self) -> PyResult<String> {
        let frame: fastobo::ast::XrefList = Python::with_gil(|py| self.clone_py(py).into_py(py));
        Ok(frame.to_string())
    }

    fn __len__(&self) -> PyResult<usize> {
        Ok(self.xrefs.len())
    }

    fn __getitem__(&self, index: isize) -> PyResult<Py<Xref>> {
        if index < self.xrefs.len() as isize {
            Python::with_gil(|py| Ok(self.xrefs[index as usize].clone_ref(py)))
        } else {
            Err(PyIndexError::new_err("list index out of range"))
        }
    }

    fn __contains__<'py>(&self, item: &Bound<'py, PyAny>) -> PyResult<bool> {
        if let Ok(xref) = item.extract::<Py<Xref>>() {
            let py = item.py();
            Ok(self
                .xrefs
                .iter()
                .any(|x| (*x.bind(py).borrow()).eq_py(&xref.bind(py).borrow(), py)))
        } else {
            let ty = item.get_type().name()?;
            let msg = format!("'in <XrefList>' requires Xref as left operand, not {}", ty);
            Err(PyTypeError::new_err(msg))
        }
    }
}
