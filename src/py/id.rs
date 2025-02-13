use std::convert::Infallible;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fmt::Write;
use std::str::FromStr;

use pyo3::class::basic::CompareOp;
use pyo3::exceptions::PyTypeError;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyString;
use pyo3::AsPyPointer;
use pyo3::PyTypeInfo;

use fastobo::ast;
use fastobo::parser::FromSlice;

use crate::error::Error;
use crate::raise;
use crate::utils::AbstractClass;
use crate::utils::ClonePy;
use crate::utils::EqPy;
use crate::utils::FinalClass;
use crate::utils::IntoPy;

// --- Module export ----------------------------------------------------------

/// parse(s)
/// --
///
/// Parse a string into an OBO identifier.
///
/// Arguments:
///     s (`str`): the string representation of an OBO identifier
///
/// Returns:
///     `~fastobo.id.BaseIdent`: the appropriate concrete subclass that
///     can store the given identifier.
///
/// Raises:
///     ValueError: when the string could not be parsed as a valid OBO
///         identifier.
///
/// Example:
///     >>> fastobo.id.parse("MS:1000031")
///     PrefixedIdent('MS', '1000031')
///     >>> fastobo.id.parse("part_of")
///     UnprefixedIdent('part_of')
///     >>> fastobo.id.parse("http://purl.obolibrary.org/obo/IAO_0000231")
///     Url('http://purl.obolibrary.org/obo/IAO_0000231')
///
#[pyfunction]
#[pyo3(name = "parse")]
fn parse(py: Python, s: &str) -> PyResult<Ident> {
    match fastobo::ast::Ident::from_str(s) {
        Ok(id) => Ok(id.into_py(py)),
        Err(e) => {
            let err = PyErr::from(Error::from(e));
            raise!(py, PyValueError("could not parse identifier") from err)
        }
    }
}

/// is_valid(s)
/// --
///
/// Check whether or not a string is a valid OBO identifier.
///
/// Arguments
///     s (`str`): the identifier to validate.
///
/// Returns:
///     `bool`: whether or not the string is valid as an OBO identifier.
///
/// Example
///     >>> fastobo.id.is_valid("MS:1000031")
///     True
///     >>> fastobo.id.is_valid("https://purl.obolibrary.org/obo/MS_1000031")
///     True
///     >>> fastobo.id.is_valid("related_to")
///     True
///     >>> fastobo.id.is_valid("definitely not an identifier")
///     False
#[pyfunction]
#[pyo3(name = "is_valid")]
fn is_valid(_py: Python, s: &str) -> bool {
    let rule = fastobo::syntax::Rule::Id;
    match fastobo::syntax::Lexer::tokenize(rule, s) {
        Err(e) => false,
        Ok(pairs) => pairs.as_str().len() == s.len(),
    }
}

#[pymodule]
#[pyo3(name = "id")]
pub fn init<'py>(_py: Python<'py>, m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add_class::<self::BaseIdent>()?;
    m.add_class::<self::PrefixedIdent>()?;
    m.add_class::<self::UnprefixedIdent>()?;
    m.add_class::<self::Url>()?;
    m.add("__name__", "fastobo.id")?;

    m.add_function(wrap_pyfunction!(self::parse, m)?)?;
    m.add_function(wrap_pyfunction!(self::is_valid, m)?)?;

    Ok(())
}

// --- Conversion Wrapper -----------------------------------------------------

macro_rules! impl_convert {
    ($base:ident, $cls:ident) => {
        impl IntoPy<$cls> for $crate::fastobo::ast::$base {
            fn into_py(self, py: Python) -> $cls {
                let ident: $crate::fastobo::ast::Ident = self.into();
                ident.into_py(py)
            }
        }

        impl IntoPy<$crate::fastobo::ast::$base> for $cls {
            fn into_py(self, py: Python) -> $crate::fastobo::ast::$base {
                let ident: $crate::fastobo::ast::Ident = self.into_py(py);
                $crate::fastobo::ast::$base::from(ident)
            }
        }
    };
}

#[derive(Debug, ClonePy, PyWrapper, EqPy)]
#[wraps(BaseIdent)]
pub enum Ident {
    Unprefixed(Py<UnprefixedIdent>),
    Prefixed(Py<PrefixedIdent>),
    Url(Py<Url>),
}

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        Python::with_gil(|py| match self {
            Ident::Unprefixed(id) => id.bind(py).borrow().fmt(f),
            Ident::Prefixed(id) => id.bind(py).borrow().fmt(f),
            Ident::Url(id) => id.bind(py).borrow().fmt(f),
        })
    }
}

// impl EqPy for Ident {
//     fn eq_py(&self, other: &Self, py: Python) -> bool {
//         use self::Ident::*;
//         match (&self, &other) {
//             (Unprefixed(x), Unprefixed(y)) => x.bind(py).eq(y.bind(py)).unwrap(),
//             (Prefixed(x), Prefixed(y)) => x.bind(py).eq(y.bind(py)).unwrap(),
//             (Url(x), Url(y)) => x.bind(py).eq(y.bind(py)).unwrap(),
//             _ => false,
//         }
//     }
// }

impl IntoPy<Ident> for fastobo::ast::Ident {
    fn into_py(self, py: Python) -> Ident {
        match self {
            ast::Ident::Unprefixed(id) => Py::new(py, id.into_py(py)).map(Ident::Unprefixed),
            ast::Ident::Prefixed(id) => Py::new(py, id.into_py(py)).map(Ident::Prefixed),
            ast::Ident::Url(id) => Py::new(py, id.into_py(py)).map(Ident::Url),
        }
        .expect("could not allocate on Python heap")
    }
}

impl IntoPy<fastobo::ast::Ident> for Ident {
    fn into_py(self, py: Python) -> fastobo::ast::Ident {
        match self {
            Ident::Unprefixed(id) => {
                let i = id.bind(py).borrow();
                ast::Ident::from((*i).inner.clone())
            }
            Ident::Prefixed(id) => {
                let i = id.bind(py).borrow();
                ast::Ident::from((*i).inner.clone())
            }
            Ident::Url(id) => {
                let i = id.bind(py).borrow();
                ast::Ident::from((*i).inner.clone())
            }
        }
    }
}

impl_convert!(ClassIdent, Ident);
impl_convert!(RelationIdent, Ident);
impl_convert!(InstanceIdent, Ident);
impl_convert!(SubsetIdent, Ident);
impl_convert!(SynonymTypeIdent, Ident);
impl_convert!(NamespaceIdent, Ident);

// --- Base -------------------------------------------------------------------

/// A sequence of character used to refer to an OBO entity.
#[pyclass(subclass, module = "fastobo.id")]
#[derive(Default)]
pub struct BaseIdent {}

impl AbstractClass for BaseIdent {
    fn initializer() -> PyClassInitializer<Self> {
        PyClassInitializer::from(BaseIdent {})
    }
}

// --- PrefixedIdent ----------------------------------------------------------

/// An identifier with a prefix.
///
/// Example:
///     >>> ident = fastobo.id.PrefixedIdent('GO', '0009637')
///     >>> ident.prefix
///     'GO'
///     >>> ident.local
///     '0009637'
///     >>> str(ident)
///     'GO:0009637'
///
#[pyclass(extends=BaseIdent, module="fastobo.id")]
#[derive(Debug, FinalClass, Clone, PartialEq, Eq, EqPy)]
#[base(BaseIdent)]
pub struct PrefixedIdent {
    inner: ast::PrefixedIdent,
}

impl PrefixedIdent {
    fn new(prefix: &str, local: &str) -> Self {
        PrefixedIdent {
            inner: ast::PrefixedIdent::new(prefix, local),
        }
    }
}

impl ClonePy for PrefixedIdent {
    fn clone_py(&self, _py: Python) -> Self {
        self.clone()
    }
}

impl Display for PrefixedIdent {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.inner.fmt(f)
    }
}

impl From<ast::PrefixedIdent> for PrefixedIdent {
    fn from(id: ast::PrefixedIdent) -> Self {
        Self { inner: id }
    }
}

impl From<PrefixedIdent> for ast::PrefixedIdent {
    fn from(id: PrefixedIdent) -> Self {
        id.inner
    }
}

impl IntoPy<ast::PrefixedIdent> for PrefixedIdent {
    fn into_py(self, py: Python) -> ast::PrefixedIdent {
        self.inner
    }
}

impl IntoPy<ast::Ident> for PrefixedIdent {
    fn into_py(self, _py: Python) -> ast::Ident {
        ast::Ident::from(self.inner)
    }
}

impl IntoPy<PrefixedIdent> for ast::PrefixedIdent {
    fn into_py(self, _py: Python) -> PrefixedIdent {
        PrefixedIdent { inner: self }
    }
}

#[pymethods]
impl PrefixedIdent {
    /// Create a new `PrefixedIdent` instance.
    ///
    /// Arguments passed as `str` must be in unescaped form, otherwise double
    /// escaping will occur when serializing this identifier.
    ///
    /// Arguments:
    ///     prefix (str): the idspace of the identifier.
    ///     local (str): the local part of the identifier.
    ///
    #[new]
    fn __init__(prefix: &str, local: &str) -> PyResult<PyClassInitializer<Self>> {
        Ok(PyClassInitializer::from(BaseIdent {}).add_subclass(Self::new(prefix, local)))
    }

    fn __hash__(&self) -> u64 {
        impl_hash!(self.inner.prefix(), ":", self.inner.local())
    }

    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, PrefixedIdent(self.inner.prefix(), self.inner.local()))
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(self.to_string())
    }

    fn __richcmp__<'py>(&self, other: &Bound<'py, PyAny>, op: CompareOp) -> PyResult<bool> {
        let py = other.py();
        if let Ok(r) = other.downcast::<PrefixedIdent>() {
            let r = r.as_borrowed().borrow();
            let lp = self.inner.prefix();
            let ll = self.inner.local();
            let rp = r.inner.prefix();
            let rl = r.inner.local();
            match op {
                CompareOp::Eq => Ok((lp, ll) == (rp, rl)),
                CompareOp::Ne => Ok((lp, ll) != (rp, rl)),
                CompareOp::Lt => Ok((lp, ll) < (rp, rl)),
                CompareOp::Le => Ok((lp, ll) <= (rp, rl)),
                CompareOp::Gt => Ok((lp, ll) > (rp, rl)),
                CompareOp::Ge => Ok((lp, ll) >= (rp, rl)),
            }
        } else {
            match op {
                CompareOp::Eq => Ok(false),
                CompareOp::Ne => Ok(true),
                _ => {
                    let n = other.get_type().name()?;
                    let msg = format!("expected PrefixedIdent, found {}", n);
                    Err(PyTypeError::new_err(msg))
                }
            }
        }
    }

    /// `str`: the IDspace of the identifier.
    #[getter]
    fn get_prefix<'py>(&self) -> &str {
        self.inner.prefix()
    }

    #[setter]
    fn set_prefix(&mut self, prefix: &str) {
        self.inner = ast::PrefixedIdent::new(prefix, self.inner.local());
    }

    /// `str`: the local part of the identifier.
    #[getter]
    fn get_local<'py>(&self) -> &str {
        self.inner.local()
    }

    #[setter]
    fn set_local(&mut self, local: &str) {
        self.inner = ast::PrefixedIdent::new(self.inner.prefix(), local);
    }
}

// --- UnprefixedIdent --------------------------------------------------------

/// An identifier without a prefix.
///
/// Example:
///     >>> import fastobo
///     >>> ident = fastobo.id.UnprefixedIdent("hello world")
///     >>> print(ident.escaped)
///     hello\ world
///     >>> print(ident.unescaped)
///     hello world
///
#[pyclass(extends=BaseIdent, module="fastobo.id")]
#[derive(Clone, Debug, Eq, Hash, PartialEq, EqPy, FinalClass)]
#[base(BaseIdent)]
pub struct UnprefixedIdent {
    inner: ast::UnprefixedIdent,
}

impl UnprefixedIdent {
    fn new(id: ast::UnprefixedIdent) -> Self {
        UnprefixedIdent { inner: id }
    }
}

impl AsRef<ast::UnprefixedIdent> for UnprefixedIdent {
    fn as_ref(&self) -> &ast::UnprefixedIdent {
        &self.inner
    }
}

impl ClonePy for UnprefixedIdent {
    fn clone_py(&self, _py: Python) -> Self {
        self.clone()
    }
}

impl Display for UnprefixedIdent {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.inner.fmt(f)
    }
}

impl From<UnprefixedIdent> for ast::UnprefixedIdent {
    fn from(id: UnprefixedIdent) -> Self {
        id.inner
    }
}

impl IntoPy<ast::UnprefixedIdent> for UnprefixedIdent {
    fn into_py(self, py: Python) -> ast::UnprefixedIdent {
        ast::UnprefixedIdent::from(self)
    }
}

impl From<UnprefixedIdent> for ast::Ident {
    fn from(id: UnprefixedIdent) -> Self {
        ast::Ident::from(ast::UnprefixedIdent::from(id))
    }
}

impl IntoPy<ast::Ident> for UnprefixedIdent {
    fn into_py(self, py: Python) -> ast::Ident {
        ast::Ident::from(self)
    }
}

impl From<ast::UnprefixedIdent> for UnprefixedIdent {
    fn from(id: ast::UnprefixedIdent) -> Self {
        Self::new(id)
    }
}

impl IntoPy<UnprefixedIdent> for ast::UnprefixedIdent {
    fn into_py(self, py: Python) -> UnprefixedIdent {
        UnprefixedIdent::from(self)
    }
}

#[pymethods]
impl UnprefixedIdent {
    /// Create a new `UnprefixedIdent` instance.
    ///
    /// Arguments:
    ///     value (`str`): the unescaped representation of the identifier.
    #[new]
    fn __init__(value: &str) -> PyClassInitializer<Self> {
        let id = ast::UnprefixedIdent::new(value.to_string());
        PyClassInitializer::from(BaseIdent {}).add_subclass(UnprefixedIdent::new(id))
    }

    fn __hash__(&self) -> u64 {
        impl_hash!(self.inner.as_str())
    }

    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, UnprefixedIdent(self.inner.as_str()))
    }

    fn __str__(&self) -> PyResult<&str> {
        Ok(self.inner.as_str())
    }

    fn __richcmp__<'py>(&self, other: &Bound<'py, PyAny>, op: CompareOp) -> PyResult<bool> {
        if let Ok(u) = other.downcast::<UnprefixedIdent>() {
            let x = u.as_borrowed().borrow();
            match op {
                CompareOp::Lt => Ok(self.inner < x.inner),
                CompareOp::Le => Ok(self.inner <= x.inner),
                CompareOp::Eq => Ok(self.inner == x.inner),
                CompareOp::Ne => Ok(self.inner != x.inner),
                CompareOp::Gt => Ok(self.inner > x.inner),
                CompareOp::Ge => Ok(self.inner >= x.inner),
            }
        } else {
            match op {
                CompareOp::Eq => Ok(false),
                CompareOp::Ne => Ok(true),
                _ => {
                    let n = other.get_type().name()?;
                    let msg = format!("expected UnprefixedIdent, found {}", n);
                    Err(PyTypeError::new_err(msg))
                }
            }
        }
    }

    /// `str`: the escaped representation of the identifier.
    #[getter]
    fn escaped(&self) -> PyResult<String> {
        Ok(self.inner.to_string())
    }

    /// `str`: the unescaped representation of the identifier.
    #[getter]
    fn unescaped(&self) -> PyResult<&str> {
        Ok(self.inner.as_str())
    }
}

// --- UrlIdent ---------------------------------------------------------------

/// A URL used as an identifier.
///
/// Use `str` to retrieve a serialized string of the inner URL.
///
/// Example:
///     >>> import fastobo
///     >>> id = fastobo.id.Url('http://purl.obolibrary.org/obo/GO_0070412')
///     >>> str(id)
///     'http://purl.obolibrary.org/obo/GO_0070412'
///     >>> fastobo.id.Url('created_by')
///     Traceback (most recent call last):
///         ...
///     ValueError: invalid url: ...
///
#[pyclass(extends=BaseIdent, module="fastobo.id")]
#[derive(Clone, ClonePy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, EqPy, FinalClass)]
#[base(BaseIdent)]
pub struct Url {
    inner: ast::Url,
}

impl Url {
    pub fn new(url: ast::Url) -> Self {
        Self { inner: url }
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        self.inner.fmt(f)
    }
}

impl From<ast::Url> for Url {
    fn from(url: ast::Url) -> Self {
        Self::new(url)
    }
}

impl From<Url> for fastobo::ast::Url {
    fn from(url: Url) -> Self {
        url.inner
    }
}

impl From<Url> for fastobo::ast::Ident {
    fn from(url: Url) -> Self {
        fastobo::ast::Ident::from(url.inner)
    }
}

impl IntoPy<Url> for ast::Url {
    fn into_py(self, _py: Python) -> Url {
        Url::new(self)
    }
}

impl IntoPy<fastobo::ast::Ident> for Url {
    fn into_py(self, _py: Python) -> fastobo::ast::Ident {
        fastobo::ast::Ident::from(self)
    }
}

impl IntoPy<ast::Url> for Url {
    fn into_py(self, _py: Python) -> ast::Url {
        self.inner
    }
}

#[pymethods]
impl Url {
    /// Create a new URL identifier.
    ///
    /// Arguments:
    ///     value (str): the string containing the URL to use as an
    ///         identifier.
    ///
    /// Raises:
    ///     ValueError: when the given string is not a valid URL.
    #[new]
    fn __new__(value: &str) -> PyResult<PyClassInitializer<Self>> {
        let init = PyClassInitializer::from(BaseIdent {});
        match ast::Url::from_str(value) {
            Ok(url) => Ok(init.add_subclass(Url::new(url))),
            Err(e) => Err(PyValueError::new_err(format!("invalid url: {}", e))),
        }
    }

    fn __hash__(&self) -> u64 {
        impl_hash!(self.inner.as_str())
    }

    fn __repr__(&self) -> PyResult<PyObject> {
        impl_repr!(self, Url(self.inner.as_str()))
    }

    /// Retrieve the URL in a serialized form.
    fn __str__(&self) -> PyResult<&str> {
        Ok(self.inner.as_str())
    }

    /// Compare to another `Url` instance.
    fn __richcmp__<'py>(&self, other: Bound<'py, PyAny>, op: CompareOp) -> PyResult<bool> {
        if let Ok(url) = other.downcast::<Url>() {
            let x = url.as_borrowed().borrow();
            match op {
                CompareOp::Lt => Ok(self < &*x),
                CompareOp::Le => Ok(self <= &*x),
                CompareOp::Eq => Ok(self == &*x),
                CompareOp::Ne => Ok(self != &*x),
                CompareOp::Gt => Ok(self > &*x),
                CompareOp::Ge => Ok(self >= &*x),
            }
        } else {
            match op {
                CompareOp::Eq => Ok(false),
                CompareOp::Ne => Ok(true),
                _ => {
                    let n = other.get_type().name()?;
                    let msg = format!("expected str or Url, found {}", n);
                    Err(PyTypeError::new_err(msg))
                }
            }
        }
    }
}
