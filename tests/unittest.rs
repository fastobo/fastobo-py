extern crate pyo3;
extern crate fastobo_py;

use std::path::Path;

use pyo3::prelude::PyResult;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use pyo3::types::PyModule;
use pyo3::Python;


pub fn main() -> PyResult<()> {
    // get the relative path to the project folder
    let folder = Path::new(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    // spawn a Python interpreter
    pyo3::prepare_freethreaded_python();
    Python::with_gil(|py| {
        // insert the project folder in `sys.modules` so that 
        // the main module can be imported by Python
        let sys = py.import("sys").unwrap();
        sys.getattr("path")
            .unwrap()
            .downcast::<PyList>()
            .unwrap()
            .insert(0, folder)
            .unwrap();

        // create a Python module from our rust code with debug symbols
        let module = PyModule::new(py, "fastobo").unwrap();
        fastobo_py::py::init(py, &module).unwrap();
        sys.getattr("modules")
            .unwrap()
            .downcast::<PyDict>()
            .unwrap()
            .set_item("fastobo", module)
            .unwrap();

        // run unittest on the tests
        let kwargs = PyDict::new(py);
        kwargs.set_item("exit", false).unwrap();
        kwargs.set_item("verbosity", 2u8).unwrap();
        py.import("unittest").unwrap().call_method(
            "TestProgram",
            ("tests",),
            Some(kwargs),
        ).map(|_| ())
    })
}
