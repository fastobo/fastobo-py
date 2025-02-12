pub mod clause;
pub mod frame;

use pyo3::prelude::*;

#[pymodule]
#[pyo3(name = "instance")]
pub fn init<'py>(py: Python<'py>, m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add_class::<self::frame::InstanceFrame>()?;

    register!(py, m, InstanceFrame, "collections.abc", MutableSequence);

    m.add("__name__", "fastobo.instance")?;

    Ok(())
}
