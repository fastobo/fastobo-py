use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fmt::Write;
use std::str::FromStr;

use pyo3::class::basic::CompareOp;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyString;
use pyo3::AsPyPointer;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PyTypeInfo;

use fastobo::ast;
use fastobo::share::Cow;
use fastobo::share::Redeem;
use fastobo::share::Share;

use crate::utils::AsGILRef;
use crate::utils::ClonePy;

// --- Module export ----------------------------------------------------------

#[pymodule(id)]
fn module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<self::BaseIdent>()?;
    m.add_class::<self::PrefixedIdent>()?;
    m.add_class::<self::UnprefixedIdent>()?;
    m.add_class::<self::IdentPrefix>()?;
    m.add_class::<self::IdentLocal>()?;
    m.add_class::<self::Url>()?;
    m.add("__name__", "fastobo.id")?;


    /// parse(s)
    /// --
    ///
    /// Parse a string into an OBO identifier.
    ///
    /// Arguments:
    ///     s (str): the string representation of an OBO identifier
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
    #[pyfn(m, "parse")]
    fn parse(py: Python, s: &str) -> PyResult<Ident> {
        fastobo::ast::Ident::from_str(s)
            .or(ValueError::into("could not parse identifier"))
            .map(|ident| Ident::from_py(ident, py))
    }


    Ok(())
}

// --- Conversion Wrapper -----------------------------------------------------

macro_rules! impl_convert {
    ($base:ident, $cls:ident) => {
        impl FromPy<$crate::fastobo::ast::$base> for $cls {
            fn from_py(id: $crate::fastobo::ast::$base, py: Python) -> $cls {
                let ident: ast::Ident = id.into();
                $cls::from_py(ident, py)
            }
        }

        impl FromPy<$cls> for $crate::fastobo::ast::$base {
            fn from_py(id: $cls, py: Python) -> $crate::fastobo::ast::$base {
                let ident: ast::Ident = id.into_py(py);
                $crate::fastobo::ast::$base::from(ident)
            }
        }
    };
}

#[derive(ClonePy, Debug, PartialEq, PyWrapper)]
#[wraps(BaseIdent)]
pub enum Ident {
    Unprefixed(Py<UnprefixedIdent>),
    Prefixed(Py<PrefixedIdent>),
    Url(Py<Url>),
}

impl<'p> AsGILRef<'p, fastobo::ast::Id<'p>> for Ident {
    fn as_gil_ref(&'p self, py: Python<'p>) -> fastobo::ast::Id<'p> {
        match self {
            Ident::Unprefixed(ref id) => {
                let x: &UnprefixedIdent = id.as_gil_ref(py);
                fastobo::ast::Id::Unprefixed(Cow::Borrowed(x.as_gil_ref(py)))
            }
            Ident::Prefixed(ref id) => {
                let x: &PrefixedIdent = id.as_gil_ref(py);
                fastobo::ast::Id::Prefixed(Cow::Borrowed(x.as_gil_ref(py)))
            }
            Ident::Url(ref url) => {
                let x: &Url = url.as_gil_ref(py);
                fastobo::ast::Id::Url(Cow::Borrowed(x.as_gil_ref(py)))
            }
        }
    }
}

impl Display for Ident {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        self.as_gil_ref(gil.python()).fmt(f)
    }
}

impl FromPy<fastobo::ast::Ident> for Ident {
    fn from_py(ident: fastobo::ast::Ident, py: Python) -> Self {
        match ident {
            ast::Ident::Unprefixed(id) => {
                Py::new(py, UnprefixedIdent::from_py(id, py)).map(Ident::Unprefixed)
            }
            ast::Ident::Prefixed(id) => {
                Py::new(py, PrefixedIdent::from_py(id, py)).map(Ident::Prefixed)
            }
            ast::Ident::Url(id) => Py::new(py, Url::from_py(id, py)).map(Ident::Url),
        }
        .expect("could not allocate on Python heap")
    }
}

impl FromPy<Ident> for fastobo::ast::Ident {
    fn from_py(ident: Ident, py: Python) -> Self {
        match ident {
            Ident::Unprefixed(id) => {
                let i: UnprefixedIdent = id.as_ref(py).clone_py(py);
                ast::Ident::Unprefixed(i.into_py(py))
            }
            Ident::Prefixed(id) => {
                let i: PrefixedIdent = id.as_ref(py).clone_py(py);
                ast::Ident::Prefixed(i.into_py(py))
            }
            Ident::Url(id) => {
                let url: Url = id.as_ref(py).clone();
                ast::Ident::Url(url.into_py(py))
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
pub struct BaseIdent {}

// --- PrefixedIdent ----------------------------------------------------------

/// An identifier with a prefix.
///
/// Example:
///     >>> ident = fastobo.id.PrefixedIdent('GO', '0009637')
///     >>> ident.prefix
///     IdentPrefix('GO')
///     >>> ident.local
///     IdentLocal('0009637')
///     >>> str(ident)
///     'GO:0009637'
///
#[pyclass(extends=BaseIdent, module="fastobo.id")]
#[derive(Debug)]
pub struct PrefixedIdent {
    prefix: Py<IdentPrefix>,
    local: Py<IdentLocal>,
}

impl PrefixedIdent {
    fn new(prefix: Py<IdentPrefix>, local: Py<IdentLocal>) -> Self {
        PrefixedIdent { prefix, local }
    }
}

impl<'p> AsGILRef<'p, fastobo::ast::PrefixedId<'p>> for PrefixedIdent {
    fn as_gil_ref(&'p self, py: Python<'p>) -> fastobo::ast::PrefixedId<'p> {
        // NB(@althonos): We can actually access the data as long as we hold
        //                the GIL ('p), so we're fine here.
        unsafe {
            let prefix: &IdentPrefix = self.prefix.as_gil_ref(py);
            let local: &IdentLocal = self.local.as_gil_ref(py);
            fastobo::ast::PrefixedId::new(prefix.as_gil_ref(py), local.as_gil_ref(py))
        }
    }
}

impl ClonePy for PrefixedIdent {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            prefix: self.prefix.clone_ref(py),
            local: self.local.clone_ref(py),
        }
    }
}

impl Display for PrefixedIdent {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        self.as_gil_ref(gil.python()).fmt(f)
    }
}

impl PartialEq for PrefixedIdent {
    fn eq(&self, other: &Self) -> bool {
        let gil = Python::acquire_gil();
        let py = gil.python();

        *self.prefix.as_ref(py) == *other.prefix.as_ref(py)
            && *self.local.as_ref(py) == *other.local.as_ref(py)
    }
}

impl Eq for PrefixedIdent {}

impl FromPy<PrefixedIdent> for ast::PrefixedIdent {
    fn from_py(ident: PrefixedIdent, py: Python) -> Self {
        ast::PrefixedIdent::new(
            ident.prefix.as_ref(py).clone(),
            ident.local.as_ref(py).clone(),
        )
    }
}

impl FromPy<PrefixedIdent> for ast::Ident {
    fn from_py(ident: PrefixedIdent, py: Python) -> Self {
        Self::from(ast::PrefixedIdent::from_py(ident, py))
    }
}

impl FromPy<ast::PrefixedIdent> for PrefixedIdent {
    fn from_py(id: ast::PrefixedIdent, py: Python) -> Self {
        let prefix = id.prefix().clone();
        let local = id.local().clone();

        Self::new(
            Py::new(py, prefix.into()).expect("could not allocate on Python heap"),
            Py::new(py, local.into()).expect("could not allocate on Python heap"),
        )
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
    ///     prefix (str or `IdentPrefix`): the idspace of the identifier.
    ///     local (str or `IdentLocal`): the local part of the identifier.
    ///
    #[new]
    fn __init__(obj: &PyRawObject, prefix: &PyAny, local: &PyAny) -> PyResult<()> {
        let py = prefix.py();

        let p = if prefix.downcast_ref::<IdentPrefix>().is_ok() {
            unsafe { Py::from_borrowed_ptr(prefix.as_ptr()) }
        } else if let Ok(ref s) = PyString::try_from(prefix) {
            let string = s.to_string();
            Py::new(py, IdentPrefix::new(ast::IdentPrefix::new(string)))?
        } else {
            let ty = prefix.get_type().name();
            let msg = format!("expected IdentPrefix or str, found {}", ty);
            return TypeError::into(msg);
        };

        let l = if local.downcast_ref::<IdentLocal>().is_ok() {
            unsafe { Py::from_borrowed_ptr(local.as_ptr()) }
        } else if let Ok(ref s) = PyString::try_from(local) {
            let string = s.to_string();
            Py::new(py, IdentLocal::new(ast::IdentLocal::new(string)))?
        } else {
            let ty = local.get_type().name();
            let msg = format!("expected IdentLocal or str, found {}", ty);
            return TypeError::into(msg);
        };

        Ok(obj.init(Self::new(p, l)))
    }

    /// `~fastobo.id.IdentPrefix`: the IDspace of the identifier.
    #[getter]
    fn get_prefix<'py>(&self, py: Python<'py>) -> PyResult<Py<IdentPrefix>> {
        Ok(self.prefix.clone_ref(py))
    }

    #[setter]
    fn set_prefix(&mut self, prefix: &PyAny) -> PyResult<()> {
        let py = prefix.py();
        self.prefix = if prefix.downcast_ref::<IdentPrefix>().is_ok() {
            unsafe { Py::from_borrowed_ptr(prefix.as_ptr()) }
        } else if let Ok(ref s) = PyString::try_from(prefix) {
            let string = s.to_string();
            Py::new(py, IdentPrefix::new(ast::IdentPrefix::new(string)))?
        } else {
            let ty = prefix.get_type().name();
            let msg = format!("expected IdentPrefix or str, found {}", ty);
            return TypeError::into(msg);
        };
        Ok(())
    }

    /// `~fastobo.id.IdentLocal`: the local part of the identifier.
    #[getter]
    fn get_local<'py>(&self, py: Python<'py>) -> PyResult<Py<IdentLocal>> {
        Ok(self.local.clone_ref(py))
    }

    #[setter]
    fn set_local(&mut self, local: &PyAny) -> PyResult<()> {
        let py = local.py();
        self.local = if local.downcast_ref::<IdentLocal>().is_ok() {
            unsafe { Py::from_borrowed_ptr(local.as_ptr()) }
        } else if let Ok(ref s) = PyString::try_from(local) {
            let string = s.to_string();
            Py::new(py, IdentLocal::new(ast::IdentLocal::new(string)))?
        } else {
            let ty = local.get_type().name();
            let msg = format!("expected IdentLocal or str, found {}", ty);
            return TypeError::into(msg);
        };
        Ok(())
    }
}

#[pyproto]
impl PyObjectProtocol for PrefixedIdent {
    fn __repr__(&self) -> PyResult<PyObject> {
        // acquire the GIL
        let gil = Python::acquire_gil();
        let py = gil.python();
        // extract inner references
        let pref = self.prefix.as_ref(py);
        let lref = self.local.as_ref(py);
        // extract string slices
        let p = pref.inner.as_str();
        let l = lref.inner.as_str();
        // return the formatted `repr` string
        let fmt = PyString::new(py, "PrefixedIdent({!r}, {!r})").to_object(py);
        fmt.call_method1(py, "format", (p, l))
    }

    fn __str__(&self) -> PyResult<String> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        Ok(self.as_gil_ref(py).to_string())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
        let py = other.py();
        if let Ok(r) = other.downcast_ref::<PrefixedIdent>() {
            let lp = &*self.prefix.as_ref(py);
            let ll = &*self.local.as_ref(py);
            let rp = &*r.prefix.as_ref(py);
            let rl = &*r.local.as_ref(py);
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
                    let n = other.get_type().name();
                    let msg = format!("expected PrefixedIdent, found {}", n);
                    TypeError::into(msg)
                }
            }
        }
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
#[derive(Clone, Debug, Eq, Hash, OpaqueTypedef, PartialEq)]
pub struct UnprefixedIdent {
    inner: ast::UnprefixedIdent,
}

impl UnprefixedIdent {
    fn new(id: ast::UnprefixedIdent) -> Self {
        UnprefixedIdent { inner: id }
    }
}

impl AsRef<ast::UnprefixedId> for UnprefixedIdent {
    fn as_ref(&self) -> &ast::UnprefixedId {
        self.inner.share()
    }
}

impl<'p> AsGILRef<'p, &'p fastobo::ast::UnprefixedId> for UnprefixedIdent {
    fn as_gil_ref(&'p self, _py: Python<'p>) -> &'p fastobo::ast::UnprefixedId {
        self.inner.share()
    }
}

impl ClonePy for UnprefixedIdent {
    fn clone_py(&self, _py: Python) -> Self {
        self.clone()
    }
}

impl Display for UnprefixedIdent {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        self.as_gil_ref(gil.python()).fmt(f)
    }
}

impl From<UnprefixedIdent> for ast::UnprefixedIdent {
    fn from(id: UnprefixedIdent) -> Self {
        id.inner
    }
}

impl FromPy<UnprefixedIdent> for ast::UnprefixedIdent {
    fn from_py(id: UnprefixedIdent, _py: Python) -> Self {
        Self::from(id)
    }
}

impl From<UnprefixedIdent> for ast::Ident {
    fn from(id: UnprefixedIdent) -> Self {
        ast::Ident::Unprefixed(ast::UnprefixedIdent::from(id))
    }
}

impl FromPy<UnprefixedIdent> for ast::Ident {
    fn from_py(id: UnprefixedIdent, _py: Python) -> Self {
        Self::from(id)
    }
}

impl From<ast::UnprefixedIdent> for UnprefixedIdent {
    fn from(id: ast::UnprefixedIdent) -> Self {
        Self::new(id)
    }
}

impl FromPy<ast::UnprefixedIdent> for UnprefixedIdent {
    fn from_py(id: ast::UnprefixedIdent, _py: Python) -> Self {
        Self::from(id)
    }
}

#[pymethods]
impl UnprefixedIdent {
    /// Create a new `UnprefixedIdent` instance.
    ///
    /// Arguments:
    ///     value (`str`): the unescaped representation of the identifier.
    #[new]
    fn __init__(obj: &PyRawObject, value: &str) -> PyResult<()> {
        let id = ast::UnprefixedIdent::new(value.to_string());
        Ok(obj.init(UnprefixedIdent::new(id)))
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

#[pyproto]
impl PyObjectProtocol for UnprefixedIdent {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let fmt = PyString::new(py, "UnprefixedIdent({!r})").to_object(py);
        fmt.call_method1(py, "format", (self.inner.as_str(),))
    }

    fn __str__(&'p self) -> PyResult<&'p str> {
        Ok(self.inner.as_str())
    }

    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
        if let Ok(u) = other.downcast_ref::<UnprefixedIdent>() {
            match op {
                CompareOp::Lt => Ok(self.inner < u.inner),
                CompareOp::Le => Ok(self.inner <= u.inner),
                CompareOp::Eq => Ok(self.inner == u.inner),
                CompareOp::Ne => Ok(self.inner != u.inner),
                CompareOp::Gt => Ok(self.inner > u.inner),
                CompareOp::Ge => Ok(self.inner >= u.inner),
            }
        } else {
            match op {
                CompareOp::Eq => Ok(false),
                CompareOp::Ne => Ok(true),
                _ => {
                    let n = other.get_type().name();
                    let msg = format!("expected UnprefixedIdent, found {}", n);
                    TypeError::into(msg)
                }
            }
        }
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
#[derive(Clone, ClonePy, Debug, Eq, Hash, OpaqueTypedef, Ord, PartialEq, PartialOrd)]
#[opaque_typedef(derive(FromInner, IntoInner))]
pub struct Url {
    inner: url::Url,
}

impl Url {
    pub fn new(url: url::Url) -> Self {
        Self { inner: url }
    }
}

impl<'p> AsGILRef<'p, &'p url::Url> for Url {
    fn as_gil_ref(&'p self, _py: Python<'p>) -> &'p url::Url {
        &self.inner
    }
}

impl Display for Url {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        self.as_gil_ref(gil.python()).fmt(f)
    }
}

impl FromPy<url::Url> for Url {
    fn from_py(url: url::Url, _py: Python) -> Self {
        Self::new(url)
    }
}

impl From<Url> for fastobo::ast::Ident {
    fn from(url: Url) -> Self {
        fastobo::ast::Ident::Url(url.inner)
    }
}

impl FromPy<Url> for fastobo::ast::Ident {
    fn from_py(url: Url, _py: Python) -> Self {
        Self::from(url)
    }
}

impl FromPy<Url> for url::Url {
    fn from_py(url: Url, _py: Python) -> Self {
        url.inner
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
    fn __new__(obj: &PyRawObject, value: &str) -> PyResult<()> {
        match url::Url::from_str(value) {
            Ok(url) => Ok(obj.init(Url::new(url))),
            Err(e) => ValueError::into(format!("invalid url: {}", e)),
        }
    }
}

#[pyproto]
impl PyObjectProtocol for Url {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let fmt = PyString::new(py, "Url({!r})").to_object(py);
        fmt.call_method1(py, "format", (self.inner.as_str(),))
    }

    /// Retrieve the URL in a serialized form.
    fn __str__(&'p self) -> PyResult<&'p str> {
        Ok(self.inner.as_str())
    }

    /// Compare to another `Url` or `str` instance.
    fn __richcmp__(&self, other: &PyAny, op: CompareOp) -> PyResult<bool> {
        if let Ok(url) = other.downcast_ref::<Url>() {
            match op {
                CompareOp::Lt => Ok(self < url),
                CompareOp::Le => Ok(self <= url),
                CompareOp::Eq => Ok(self == url),
                CompareOp::Ne => Ok(self != url),
                CompareOp::Gt => Ok(self > url),
                CompareOp::Ge => Ok(self >= url),
            }
        } else {
            match op {
                CompareOp::Eq => Ok(false),
                CompareOp::Ne => Ok(true),
                _ => {
                    let n = other.get_type().name();
                    let msg = format!("expected str or Url, found {}", n);
                    TypeError::into(msg)
                }
            }
        }
    }
}

// --- IdentPrefix -----------------------------------------------------------

/// The prefix of a prefixed identifier.
#[pyclass(module = "fastobo.id")]
#[derive(Clone, ClonePy, Debug, Eq, Hash, OpaqueTypedef, Ord, PartialEq, PartialOrd)]
#[opaque_typedef(derive(FromInner, IntoInner, AsRefInner))]
pub struct IdentPrefix {
    inner: ast::IdentPrefix,
}

impl IdentPrefix {
    pub fn new(prefix: ast::IdentPrefix) -> Self {
        Self { inner: prefix }
    }
}

impl<'p> AsGILRef<'p, fastobo::ast::IdPrefix<'p>> for IdentPrefix {
    fn as_gil_ref(&'p self, _py: Python<'p>) -> fastobo::ast::IdPrefix<'p> {
        self.inner.share()
    }
}

impl Display for IdentPrefix {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        self.as_gil_ref(gil.python()).fmt(f)
    }
}

impl ToPyObject for IdentPrefix {
    fn to_object(&self, py: pyo3::Python<'_>) -> PyObject {
        self.inner.as_str().to_object(py)
    }
}

#[pymethods]
impl IdentPrefix {
    /// Create a new `IdentPrefix` instance.
    ///
    /// Arguments:
    ///     value (`str`): the unescaped representation of the prefix.
    #[new]
    fn __init__(obj: &PyRawObject, value: String) -> PyResult<()> {
        Ok(obj.init(Self::new(ast::IdentPrefix::new(value))))
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

#[pyproto]
impl PyObjectProtocol for IdentPrefix {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let fmt = PyString::new(py, "IdentPrefix({!r})").to_object(py);
        fmt.call_method1(py, "format", (self.inner.as_str(),))
    }

    fn __str__(&'p self) -> PyResult<&'p str> {
        Ok(self.inner.as_str())
    }
}

// --- IdentLocal ------------------------------------------------------------

/// The local component of a prefixed identifier.
#[pyclass(module = "fastobo.id")]
#[derive(Clone, ClonePy, Debug, Eq, Hash, OpaqueTypedef, Ord, PartialEq, PartialOrd)]
#[opaque_typedef(derive(FromInner, IntoInner))]
pub struct IdentLocal {
    inner: ast::IdentLocal,
}

impl IdentLocal {
    pub fn new(local: ast::IdentLocal) -> Self {
        Self { inner: local }
    }
}

impl<'p> AsGILRef<'p, fastobo::ast::IdLocal<'p>> for IdentLocal {
    fn as_gil_ref(&'p self, _py: Python<'p>) -> fastobo::ast::IdLocal<'p> {
        self.inner.share()
    }
}

impl Display for IdentLocal {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        self.as_gil_ref(gil.python()).fmt(f)
    }
}

#[pymethods]
impl IdentLocal {
    /// Create a new `IdentLocal` instance.
    #[new]
    fn __init__(obj: &PyRawObject, value: String) -> PyResult<()> {
        Ok(obj.init(Self::new(ast::IdentLocal::new(value))))
    }

    /// `str`: the escaped representation of the identifier.
    #[getter]
    fn get_escaped(&self) -> PyResult<String> {
        Ok(self.inner.to_string())
    }

    /// `str`: the unescaped representation of the identifier.
    #[getter]
    fn get_unescaped(&self) -> PyResult<&str> {
        Ok(self.inner.as_str())
    }
}

#[pyproto]
impl PyObjectProtocol for IdentLocal {
    fn __repr__(&self) -> PyResult<PyObject> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let fmt = PyString::new(py, "IdentLocal({!r})").to_object(py);
        fmt.call_method1(py, "format", (self.inner.as_str(),))
    }

    fn __str__(&'p self) -> PyResult<&'p str> {
        Ok(self.inner.as_ref())
    }
}
