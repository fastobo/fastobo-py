#![recursion_limit="128"]
#![allow(unused_imports, unused_unsafe, unused_variables)]

extern crate fastobo;
extern crate pyo3;
#[macro_use]
extern crate pyo3_built;
extern crate pest;
extern crate libc;
extern crate url;
#[macro_use]
extern crate opaque_typedef_macros;
extern crate opaque_typedef;
#[macro_use]
extern crate fastobo_py_derive;

#[macro_use]
pub mod macros;
pub mod built;
pub mod utils;
pub mod py;
pub mod pyfile;
pub mod error;
