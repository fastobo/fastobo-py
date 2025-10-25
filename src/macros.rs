macro_rules! impl_hash {
    ($($field:expr),*) => ({
        use std::hash::Hasher;
        use std::hash::Hash;
        let mut hasher = crate::utils::Hasher::default();
        $($field.hash(&mut hasher);)*
        hasher.finish()
    });
}

macro_rules! impl_richcmp {
    ($self:ident, $other:ident, $op:ident, $(self . $attr:ident)&&*) => ({
        use pyo3::types::PyBool;
        match $op {
            $crate::pyo3::class::basic::CompareOp::Eq => {
                let py = $other.py();
                let val = if let Ok(ref clause) = $other.extract::<Py<Self>>() {
                    let clause = clause.bind(py).borrow();
                    $($self.$attr == clause.$attr)&&*
                } else {
                    false
                };
                Ok(PyBool::new(py, val).to_owned().into_any().unbind())
            }
            _ => Ok($other.py().NotImplemented())
        }
    });
}

macro_rules! impl_richcmp_py {
    ($self:ident, $other:ident, $op:ident, $(self . $attr:ident)&&*) => ({
        use pyo3::types::PyBool;
        match $op {
            $crate::pyo3::class::basic::CompareOp::Eq => {
                let py = $other.py();
                let val = if let Ok(ref clause) = $other.extract::<Py<Self>>() {
                    let clause = clause.bind(py).borrow();
                    $($self.$attr.eq_py(&clause.$attr, py))&&*
                } else {
                    false
                };
                Ok(PyBool::new(py, val).to_owned().into_any().unbind())
            }
            _ => Ok($other.py().NotImplemented())
        }
    });
}

macro_rules! impl_repr_py {
    ($self:ident, $cls:ident($($field:expr),*)) => ({
        let py = $self.py();
        let args = &[
            $((&$field).into_pyobject(py)?.as_any().repr()?.to_str()?,)*
        ].join(", ");
        Ok(PyString::new(py, &format!("{}({})", stringify!($cls), args)).into_pyobject(py)?.into_any())
    })
}

macro_rules! register {
    ($py:ident, $m:ident, $cls:ident, $module:expr, $metacls:ident) => {
        $py.import($module)?
            .getattr(stringify!($metacls))?
            .call_method1("register", ($m.getattr(stringify!($cls))?,))?;
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
        $sup.add(stringify!($sub), &module)?;

        // add the submodule to the `sys.modules` index
        $py.import("sys")?
            .getattr("modules")?
            .downcast::<pyo3::types::PyDict>()?
            .set_item(concat!("fastobo.", stringify!($sub)), &module)?;
    }};
}
