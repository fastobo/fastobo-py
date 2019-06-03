use std::io::Read;
use std::marker::PhantomData;
use std::sync::Arc;

use pyo3::exceptions::OSError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::AsPyPointer;
use pyo3::PyDowncastError;
use pyo3::PyErrValue;
use pyo3::PyObject;

#[derive(Clone, Debug)]
pub struct PyFile<'p> {
    file: *mut pyo3::ffi::PyObject,
    __data: PhantomData<&'p PyObject>,
}

impl<'p> PyFile<'p> {
    pub fn from_object<T>(py: Python<'p>, obj: &T) -> Result<PyFile<'p>, PyDowncastError>
    where
        T: AsPyPointer,
    {
        unsafe {
            let file = PyObject::from_borrowed_ptr(py, obj.as_ptr());
            if let Ok(res) = file.call_method1(py, "read", (0,)) {
                if py.is_instance::<PyBytes, PyObject>(&res).unwrap_or(false) {
                    Ok(PyFile {
                        file: obj.as_ptr(),
                        __data: PhantomData,
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
            match file.call_method1(py, "read", (buf.len(),)) {
                Ok(obj) => {
                    let bytes = obj
                        .extract::<&PyBytes>(py)
                        .expect("read() did not return bytes");
                    let b = bytes.as_bytes();
                    (&mut buf[..b.len()]).copy_from_slice(b);
                    Ok(b.len())
                }
                Err(e) => {
                    if e.is_instance::<OSError>(py) {
                        if let PyErrValue::Value(obj) = e.pvalue {
                            let code = obj.getattr(py, "errno").expect("no errno found");
                            Err(std::io::Error::from_raw_os_error(
                                code.extract::<i32>(py).expect("errno is not an integer"),
                            ))
                        } else {
                            unreachable!()
                        }
                    } else {
                        if let PyErrValue::Value(obj) = e.pvalue {
                            Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "an error occurred",
                            ))
                        } else {
                            unreachable!()
                        }
                    }
                }
            }
        }
    }
}
