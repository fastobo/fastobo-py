macro_rules! impl_richmp {
    ($self:ident, $other:ident, $op:ident, $(self . $attr:ident)&&*) => ({
        match $op {
            $crate::pyo3::class::basic::CompareOp::Eq => {
                if let Ok(ref clause) = $other.downcast_ref::<Self>() {
                    Ok(($($self.$attr == clause.$attr)&&*).to_object($other.py()))
                } else {
                    Ok(false.to_object($other.py()))
                }
            }
            $crate::pyo3::class::basic::CompareOp::Ne => {
                if let Ok(ref clause) = $other.downcast_ref::<Self>() {
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
    ($fmt:literal, $($attr:ident),*) => (
        pub fn raw_value(&self) -> PyResult<String> {
           Ok(format!($fmt, $(self . $attr,)*))
        }
    )
}

macro_rules! impl_raw_tag {
    ($tag:literal) => (
        pub fn raw_tag(&self) -> PyResult<&str> {
           Ok($tag)
        }
    )
}
