use std::ops::Deref;

use pyo3::ffi::PyObject;
use pyo3::AsPyPointer;
use pyo3::AsPyRef;
use pyo3::Py;
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

pub trait AsGILRef<'p, T>: 'p {
    fn as_gil_ref(&'p self, py: Python<'p>) -> T;
}

impl<'p, T> AsGILRef<'p, &'p T> for Py<T>
where
    T: PyTypeInfo,
{
    fn as_gil_ref(&'p self, py: Python<'p>) -> &'p T {
        unsafe { ptr_to_ref(py, self.as_ref(py).as_ptr()) }
    }
}

unsafe fn ptr_to_ref<'p, T>(_py: Python<'p>, t: *mut PyObject) -> &'p T
where
    T: PyTypeInfo,
{
    &*((t as *mut u8).offset(T::OFFSET) as *mut _ as *const _)
}
