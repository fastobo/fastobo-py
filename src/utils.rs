use std::ops::Deref;

use pyo3::ffi::PyObject;
use pyo3::AsPyPointer;
use pyo3::Py;
use pyo3::PyClass;
use pyo3::PyClassInitializer;
use pyo3::PyRef;
use pyo3::PyTypeInfo;
use pyo3::Python;

use fastobo::ast;

// ---

pub trait IntoPy<T> {
    fn into_py(self, py: Python) -> T;
}

// ---

/// A trait for objects that can be cloned while the GIL is held.
pub trait ClonePy {
    fn clone_py(&self, py: Python) -> Self;
}

impl<T> ClonePy for Py<T> {
    fn clone_py(&self, py: Python) -> Self {
        self.clone_ref(py)
    }
}

impl<T> ClonePy for Vec<T>
where
    T: ClonePy,
{
    fn clone_py(&self, py: Python) -> Self {
        self.iter().map(|x| x.clone_py(py)).collect()
    }
}

impl<T> ClonePy for Option<T>
where
    T: ClonePy,
{
    fn clone_py(&self, py: Python) -> Self {
        self.as_ref().map(|x| x.clone_py(py))
    }
}

// ---

macro_rules! derive_eqpy {
    ($type:ty) => {
        impl EqPy for $type {
            fn eq_py(&self, other: &Self, _py: Python) -> bool {
                self == other
            }
        }
    };
}

/// A trait for objects that can be compared for equality while the GIL is held.
pub trait EqPy {
    fn eq_py(&self, other: &Self, py: Python) -> bool;
    fn neq_py(&self, other: &Self, py: Python) -> bool {
        !self.eq_py(other, py)
    }
}

impl<T> EqPy for Option<T>
where
    T: EqPy,
{
    fn eq_py(&self, other: &Self, py: Python) -> bool {
        match (self, other) {
            (Some(l), Some(r)) => l.eq_py(r, py),
            (None, None) => true,
            _ => false,
        }
    }
}

impl<T> EqPy for Vec<T>
where
    T: EqPy,
{
    fn eq_py(&self, other: &Self, py: Python) -> bool {
        if self.len() == other.len() {
            for (x, y) in self.iter().zip(other.iter()) {
                if !x.eq_py(y, py) {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

impl<T> EqPy for Py<T>
where
    T: EqPy + PyClass,
{
    fn eq_py(&self, other: &Self, py: Python) -> bool {
        let l = self.borrow(py);
        let r = other.borrow(py);
        (*l).eq_py(&*r, py)
    }
}

derive_eqpy!(bool);
derive_eqpy!(fastobo::ast::CreationDate);
derive_eqpy!(fastobo::ast::IdentPrefix);
derive_eqpy!(fastobo::ast::Import);
derive_eqpy!(fastobo::ast::NaiveDateTime);
derive_eqpy!(fastobo::ast::PrefixedIdent);
derive_eqpy!(fastobo::ast::QuotedString);
derive_eqpy!(fastobo::ast::SynonymScope);
derive_eqpy!(fastobo::ast::UnprefixedIdent);
derive_eqpy!(fastobo::ast::UnquotedString);
derive_eqpy!(fastobo::ast::Url);

// ---

/// A trait for Python classes that are purely abstract.
pub trait AbstractClass: PyClass {
    fn initializer() -> PyClassInitializer<Self>;
}

/// A trait for Python classes that are final.
pub trait FinalClass: PyClass {}

// ---

pub type Hasher = std::collections::hash_map::DefaultHasher;
