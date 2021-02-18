#![recursion_limit = "128"]
#![cfg_attr(feature = "extension-module", crate_type = "cdylib")]
#![allow(unused_imports, unused_variables)]

extern crate fastobo;
extern crate pyo3;
#[macro_use]
extern crate pyo3_built;
extern crate libc;
extern crate pest;
#[macro_use]
extern crate fastobo_py_derive_internal;
extern crate fastobo_graphs;

#[macro_use]
pub mod macros;
pub mod built;
pub mod date;
pub mod error;
pub mod py;
pub mod pyfile;
pub mod utils;
pub mod iter;
