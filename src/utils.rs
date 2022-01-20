use std::ops::Deref;

use pyo3::ffi::PyObject;
use pyo3::AsPyPointer;
use pyo3::Py;
use pyo3::PyClass;
use pyo3::PyClassInitializer;
use pyo3::PyRef;
use pyo3::PyTypeInfo;
use pyo3::Python;

// ---

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

pub trait AbstractClass: PyClass {
    fn initializer() -> PyClassInitializer<Self>;
}

pub trait FinalClass: PyClass {}

// ---

pub type Hasher = std::collections::hash_map::DefaultHasher;
