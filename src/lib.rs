#![recursion_limit="128"]
#![allow(unused_imports, unused_unsafe, unused_variables)]

extern crate fastobo;
extern crate pyo3;
extern crate pest;
extern crate libc;
extern crate url;

#[macro_use]
extern crate opaque_typedef_macros;
extern crate opaque_typedef;

#[macro_use]
extern crate fastobo_py_derive;

pub mod utils;
pub mod py;
pub mod pyfile;
pub mod error;
