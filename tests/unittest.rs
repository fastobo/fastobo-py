extern crate fastobo_py;
extern crate pyo3;

use std::path::Path;

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use pyo3::types::PyModule;
use pyo3::Python;

pub fn main() -> PyResult<()> {
    // prepare the Python interpreter
    Python::initialize();

    // spawn a Python interpreter
    Python::attach(|py| {
        // create a Python module from our rust code with debug symbols
        let module = PyModule::new(py, "fastobo")?;
        fastobo_py::py::init(py, &module)?;
        py.import("sys")?
            .getattr("modules")?
            .cast::<PyDict>()?
            .set_item("fastobo", &module)?;
        // patch `sys.path` to locate tests from the project folder
        py.import("sys")?
            .getattr("path")?
            .cast::<PyList>()?
            .insert(0, env!("CARGO_MANIFEST_DIR"))?;
        // run tests with the unittest runner
        let kwargs = PyDict::new(py);
        kwargs.set_item("verbosity", 2).unwrap();
        kwargs.set_item("exit", false).unwrap();
        let prog = py
            .import("unittest")
            .unwrap()
            .call_method("main", ("tests",), Some(&kwargs))
            .unwrap();
        // check run was was successful
        if !prog
            .getattr("result")?
            .call_method0("wasSuccessful")?
            .extract::<bool>()?
        {
            panic!("some tests failed");
        }

        Ok(())
    })
}
