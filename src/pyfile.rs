use std::cell::RefCell;
use std::io::Error as IoError;
use std::io::Read;
use std::marker::PhantomData;
use std::sync::Arc;

use pyo3::exceptions::OSError;
use pyo3::exceptions::TypeError;
use pyo3::gc::PyGCProtocol;
use pyo3::gc::PyTraverseError;
use pyo3::gc::PyVisit;
use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::AsPyPointer;
use pyo3::PyDowncastError;
use pyo3::PyErrValue;
use pyo3::PyObject;

pub struct PyFile<'p> {
    file: pyo3::PyObject,
    py: Python<'p>,
    err: Option<PyErr>,
}

impl<'p> PyFile<'p> {
    pub fn from_object<T>(py: Python<'p>, obj: &T) -> PyResult<PyFile<'p>>
    where
        T: AsPyPointer,
    {
        unsafe {
            let file = PyObject::from_borrowed_ptr(py, obj.as_ptr());
            let res = file.call_method1(py, "read", (0,))?;
            if py.is_instance::<PyBytes, PyObject>(&res).unwrap_or(false) {
                Ok(PyFile {
                    file,
                    py,
                    err: None,
                })
            } else {
                let ty = res.as_ref(py).get_type().name().to_string();
                TypeError::into(format!("expected bytes, found {}", ty))
            }
        }
    }

    pub fn into_err(self) -> Option<PyErr> {
        self.err
    }
}

impl<'p> Read for PyFile<'p> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, IoError> {
        unsafe {
            match self.file.call_method1(self.py, "read", (buf.len(),)) {
                Ok(obj) => {
                    // Check `fh.read` returned bytes, else raise a `TypeError`.
                    if let Ok(bytes) = obj.extract::<&PyBytes>(self.py) {
                        let b = bytes.as_bytes();
                        (&mut buf[..b.len()]).copy_from_slice(b);
                        Ok(b.len())
                    } else {
                        let ty = obj.as_ref(self.py).get_type().name().to_string();
                        let msg = format!("expected bytes, found {}", ty);
                        self.err = Some(TypeError::py_err(msg));
                        Err(IoError::new(
                            std::io::ErrorKind::Other,
                            "fh.read did not return bytes",
                        ))
                    }
                }
                Err(e) => {
                    // Attempt to transmute the Python OSError to an actual
                    // Rust `std::io::Error` using `from_raw_os_error`.
                    if e.is_instance::<OSError>(self.py) {
                        if let PyErrValue::Value(obj) = &e.pvalue {
                            if let Ok(code) = obj.getattr(self.py, "errno") {
                                if let Ok(n) = code.extract::<i32>(self.py) {
                                    return Err(IoError::from_raw_os_error(n));
                                }
                            }
                        }
                    }

                    // if the conversion is not possible for any reason we fail
                    // silently, wrapping the Python error, and returning a
                    // generic Rust error instead.
                    self.err = Some(e);
                    Err(IoError::new(std::io::ErrorKind::Other, "fh.read failed"))
                }
            }
        }
    }
}
