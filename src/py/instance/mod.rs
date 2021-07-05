pub mod clause;
pub mod frame;

use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "instance")]
pub fn init(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<self::frame::InstanceFrame>()?;

    register!(py, m, InstanceFrame, "collections.abc", MutableSequence);

    m.add("__name__", "fastobo.instance")?;

    Ok(())
}
