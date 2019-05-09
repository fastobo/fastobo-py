use std::io::Read;
use std::marker::PhantomData;

use pyo3::prelude::*;
use pyo3::PyObject;
use pyo3::AsPyPointer;
use pyo3::PyDowncastError;
use pyo3::types::PyBytes;

#[derive(Clone, Debug)]
pub struct PyFile<'p> {
    file: *mut pyo3::ffi::PyObject,
    __data: PhantomData<&'p PyObject>
}

impl<'p> PyFile<'p> {
    pub fn from_object<T>(py: Python<'p>, obj: &T) -> Result<PyFile<'p>, PyDowncastError>
    where
        T: AsPyPointer,
    {
        unsafe {
            let file = PyObject::from_borrowed_ptr(py, obj.as_ptr());
            if let Ok(res) = file.call_method1(py, "read", (0, )) {
                if py.is_instance::<PyBytes, PyObject>(&res).unwrap_or(false) {
                    Ok(PyFile {
                        file: obj.as_ptr(),
                        __data: PhantomData
                    })
                } else {
                    Err(PyDowncastError)
                }
            } else {
                Err(PyDowncastError)
            }
        }
    }
}

impl<'p> Read for PyFile<'p> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        unsafe {
            let py = Python::assume_gil_acquired();
            let file = PyObject::from_borrowed_ptr(py, self.file);
            let res = file.call_method1(py, "read", (buf.len(), ))
                .unwrap(); // FIXME -> map as OS Error
            let bytes = res
                .extract::<&PyBytes>(py)
                .unwrap();
            let b = bytes.as_bytes();
            (&mut buf[..b.len()]).copy_from_slice(b);
            Ok(b.len())
        }

    }
}
