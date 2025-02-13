#![recursion_limit = "128"]
// #![allow(unused_imports, unused_variables)]
#![allow(unused, dead_code, deprecated)]

extern crate fastobo;
extern crate pyo3;
#[macro_use]
extern crate pyo3_built;
extern crate libc;
#[macro_use]
extern crate fastobo_py_derive_internal;
extern crate fastobo_graphs;
extern crate fastobo_owl;
extern crate horned_owl;

#[macro_use]
pub mod macros;
pub mod built;
pub mod date;
pub mod error;
pub mod iter;
pub mod py;
pub mod pyfile;
pub mod utils;
