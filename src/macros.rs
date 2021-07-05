macro_rules! impl_hash {
    ($($field:expr),*) => ({
        use std::hash::Hasher;
        use std::hash::Hash;
        let mut hasher = crate::utils::Hasher::default();
        $($field.hash(&mut hasher);)*
        hasher.finish()
    });
}

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
    ($self:ident, $cls:ident($($field:expr),*)) => ({
        let gil = Python::acquire_gil();
        let py = gil.python();

        let args = &[
            $($field.to_object(py).as_ref(py).repr()?.to_str()?,)*
        ].join(", ");

        Ok(PyString::new(py, &format!("{}({})", stringify!($cls), args)).to_object(py))
    })
}

macro_rules! register {
    ($py:ident, $m:ident, $cls:ident, $module:expr, $metacls:ident) => {
        $py.import($module)?
            .getattr(stringify!($metacls))?
            .to_object($py)
            .call_method1($py, "register", ($m.getattr(stringify!($cls))?,))?;
    };
}

macro_rules! add_submodule {
    ($py:ident, $sup:ident, $sub:ident) => {{
        use super::*;

        // create new module object and initialize it
        let module = PyModule::new($py, stringify!($ub))?;
        self::$sub::init($py, &module)?;
        module.add("__package__", $sup.getattr("__package__")?)?;

        // add the submodule to the supermodule
        $sup.add(stringify!($sub), module)?;

        // add the submodule to the `sys.modules` index
        $py.import("sys")?
            .getattr("modules")?
            .cast_as::<pyo3::types::PyDict>()?
            .set_item(concat!("fastobo.", stringify!($sub)), module)?;
    }};
}
