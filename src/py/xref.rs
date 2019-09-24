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
use crate::utils::AsGILRef;
use crate::utils::ClonePy;

// --- Module export ---------------------------------------------------------

#[pymodule(xref)]
fn module(_py: Python, m: &PyModule) -> PyResult<()> {
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
#[derive(Debug, PartialEq)]
pub struct Xref {
    #[pyo3(set)]
    id: Ident,
    desc: Option<fastobo::ast::QuotedString>,
}

impl Xref {
    pub fn new(_py: Python, id: Ident) -> Self {
        Self { id, desc: None }
    }

    pub fn with_desc<D>(py: Python, id: Ident, desc: Option<D>) -> Self
    where
        D: IntoPy<fastobo::ast::QuotedString>,
    {
        Self {
            id,
            desc: desc.map(|d| d.into_py(py)),
        }
    }

    pub fn from_object(py: Python, obj: &PyAny) -> PyResult<Py<Self>> {
        if Xref::is_instance(&obj) {
            unsafe {
                let ptr = obj.as_ptr();
                Ok(Py::from_borrowed_ptr(ptr))
            }
        } else {
            let ty = obj.get_type().name();
            TypeError::into(format!("expected Xref, found {}", ty))
        }
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
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::Xref::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<fastobo::ast::Xref> for Xref {
    fn from_py(mut xref: fastobo::ast::Xref, py: Python) -> Self {
        // Take ownership over `xref.description` w/o reallocation or clone.
        let empty = fastobo::ast::QuotedString::new(String::new());
        let desc = xref.description_mut().map(|d| std::mem::replace(d, empty));

        // Take ownership over `xref.id` w/o reallocation or clone.
        let empty = fastobo::ast::UnprefixedIdent::new(String::new());
        let id = std::mem::replace(xref.id_mut(), empty.into());

        Self::with_desc(py, id.into_py(py), desc)
    }
}

impl FromPy<Xref> for fastobo::ast::Xref {
    fn from_py(xref: Xref, py: Python) -> Self {
        let id: fastobo::ast::Ident = xref.id.into_py(py);
        Self::with_desc(id, xref.desc)
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
    fn __init__(obj: &PyRawObject, id: Ident, desc: Option<String>) -> PyResult<()> {
        Ok(obj.init(Self::with_desc(
            obj.py(),
            id,
            desc.map(fastobo::ast::QuotedString::new),
        )))
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

#[pyproto]
impl PyObjectProtocol for Xref {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        if let Some(ref d) = self.desc {
            PyString::new(py, "Xref({!r}, {!r})")
                .to_object(py)
                .call_method1(py, "format", (&self.id, d.as_str()))
        } else {
            PyString::new(py, "Xref({!r})")
                .to_object(py)
                .call_method1(py, "format", (&self.id,))
        }
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
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
#[derive(Debug, PartialEq)]
pub struct XrefList {
    xrefs: Vec<Py<Xref>>,
}

impl XrefList {
    /// Create a new `XrefList` from a vector of Xrefs.
    pub fn new(_py: Python, xrefs: Vec<Py<Xref>>) -> Self {
        Self { xrefs }
    }

    /// Create a new `XrefList` from a `PyIterator`.
    pub fn collect(py: Python, xrefs: &PyAny) -> PyResult<Self> {
        let mut vec = Vec::new();
        for item in PyIterator::from_object(py, xrefs)? {
            let i = item?;
            if Xref::is_exact_instance(i) {
                unsafe {
                    vec.push(Py::from_borrowed_ptr(i.as_ptr()));
                }
            } else {
                let ty = i.get_type().name();
                return TypeError::into(format!("expected Xref, found {}", ty));
            }
        }
        Ok(Self { xrefs: vec })
    }
}

impl ClonePy for XrefList {
    fn clone_py(&self, py: Python) -> Self {
        XrefList {
            xrefs: self.xrefs.clone_py(py),
        }
    }
}

impl FromPy<fastobo::ast::XrefList> for XrefList {
    fn from_py(list: fastobo::ast::XrefList, py: Python) -> Self {
        let mut xrefs = Vec::with_capacity((&list).len());
        for xref in list.into_iter() {
            xrefs.push(Py::new(py, xref.into_py(py)).unwrap())
        }
        Self::new(py, xrefs)
    }
}

impl FromPy<XrefList> for fastobo::ast::XrefList {
    fn from_py(list: XrefList, py: Python) -> Self {
        list.xrefs
            .into_iter()
            .map(|xref| xref.as_ref(py).clone_py(py).into_py(py))
            .collect()
    }
}

impl ToPyObject for XrefList {
    fn to_object(&self, py: Python) -> PyObject {
        let list = self.xrefs
            .iter()
            .map(|xref| xref.clone_py(py))
            .collect();
        IntoPy::into_py(XrefList::new(py, list), py)
    }
}

#[pymethods]
impl XrefList {
    #[new]
    fn __init__(obj: &PyRawObject, xrefs: Option<&PyAny>) -> PyResult<()> {
        if let Some(x) = xrefs {
            Ok(obj.init(Self::collect(obj.py(), x)?))
        } else {
            Ok(obj.init(Self::new(obj.py(), Vec::new())))
        }
    }
}

#[pyproto]
impl PyObjectProtocol for XrefList {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let fmt = PyString::new(py, "XrefList({!r})").to_object(py);
        fmt.call_method1(py, "format", (&self.xrefs.to_object(py),))
    }

    fn __str__(&self) -> PyResult<String> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let frame: fastobo::ast::XrefList = self.clone_py(py).into_py(py);
        Ok(frame.to_string())
    }
}

#[pyproto]
impl PySequenceProtocol for XrefList {
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.xrefs.len())
    }

    fn __getitem__(&self, index: isize) -> PyResult<Py<Xref>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        if index < self.xrefs.len() as isize {
            Ok(self.xrefs[index as usize].clone_ref(py))
        } else {
            IndexError::into("list index out of range")
        }
    }

    fn __contains__(&self, item: &PyAny) -> PyResult<bool> {
        if let Ok(xref) = item.downcast_ref::<Xref>() {
            let py = item.py();
            Ok(self.xrefs.iter().any(|x| x.as_gil_ref(py) == xref))
        } else {
            let ty = item.get_type().name();
            let msg = format!("'in <XrefList>' requires Xref as left operand, not {}", ty);
            TypeError::into(msg)
        }
    }
}
