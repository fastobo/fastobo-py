macro_rules! impl_richmp {
    ($self:ident, $other:ident, $op:ident, $(self . $attr:ident)&&*) => ({
        match $op {
            $crate::pyo3::class::basic::CompareOp::Eq => {
                if let Ok(ref clause) = $other.extract::<Py<Self>>() {
                    let clause = clause.as_ref($other.py()).borrow();
                    Ok(($($self.$attr == clause.$attr)&&*).to_object($other.py()))
                } else {
                    Ok(false.to_object($other.py()))
                }
            }
            $crate::pyo3::class::basic::CompareOp::Ne => {
                if let Ok(ref clause) = $other.extract::<Py<Self>>() {
                    let clause = clause.as_ref($other.py()).borrow();
                    Ok(($($self.$attr != clause.$attr)||*).to_object($other.py()))
                } else {
                    Ok(true.to_object($other.py()))
                }
            }
            _ => Ok($other.py().NotImplemented())
        }
    });
}

macro_rules! impl_repr {
    ($self:ident, $cls:ident($(self . $attr:ident),*)) => ({
        let gil = Python::acquire_gil();
        let py = gil.python();

        let fmt = PyString::new(
            py,
            concat!(stringify!($cls), "({!r})")
        ).to_object(py);

        fmt.call_method1(
            py, "format",
            ($($self . $attr . to_object(py) ,)*)
        )
    })
}

macro_rules! impl_raw_value {
    ($cls:ty, $fmt:literal, $(self . $attr:ident),*) => (
        #[pymethods]
        impl $cls {
            pub fn raw_value(&self) -> PyResult<String> {
               Ok(format!($fmt, $(self . $attr,)*))
            }
        }
    )
}

macro_rules! impl_raw_tag {
    ($cls:ty, $tag:literal) => {
        #[pymethods]
        impl $cls {
            pub fn raw_tag(&self) -> PyResult<&str> {
                Ok($tag)
            }
        }
    };
}

macro_rules! register {
    ($py:ident, $m:ident, $cls:ident, $module:expr, $metacls:ident) => {
        $py.import($module)?
            .get(stringify!($metacls))?
            .to_object($py)
            .call_method1($py, "register", ($m.get(stringify!($cls))?,))?;
    };
}

macro_rules! add_submodule {
    ($py:ident, $sup:ident, $sub:ident) => {{
        use super::*;

        // create new module object and initialize it
        let module = PyModule::new($py, stringify!($ub))?;
        self::$sub::init($py, &module)?;
        module.add("__package__", $sup.get("__package__")?)?;

        // add the submodule to the supermodule
        $sup.add(stringify!($sub), &module)?;

        // add the submodule to the `sys.modules` index
        $py.import("sys")?
            .get("modules")?
            .cast_as::<pyo3::types::PyDict>()?
            .set_item(concat!("fastobo.", stringify!($sub)), module)?;
    }};
}
