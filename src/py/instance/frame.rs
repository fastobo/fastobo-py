use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fmt::Write;
use std::str::FromStr;

use pyo3::exceptions::IndexError;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyIterator;
use pyo3::types::PyString;
use pyo3::AsPyPointer;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PySequenceProtocol;
use pyo3::PyTypeInfo;

use fastobo::ast;
use fastobo::share::Cow;
use fastobo::share::Redeem;
use fastobo::share::Share;

use super::super::abc::AbstractEntityFrame;
use super::super::id::Ident;
use crate::utils::ClonePy;

#[pyclass(extends=AbstractEntityFrame, module="fastobo.instance")]
// #[derive(Debug, PyList)]
#[derive(Debug)]
pub struct InstanceFrame {
    id: Ident,
    //clauses: Vec<InstanceClause>,
}

impl InstanceFrame {
    pub fn new(id: Ident) -> Self {
        // Self::with_clauses(id, Vec::new())
        Self { id }
    }

    // pub fn with_clauses(id: Ident, clauses: Vec<InstanceClause>) -> Self {
    //     Self { id, clauses }
    // }
}

impl ClonePy for InstanceFrame {
    fn clone_py(&self, py: Python) -> Self {
        Self {
            id: self.id.clone_py(py),
            // clauses: self.clauses.clone_py(py),
        }
    }
}

impl Display for InstanceFrame {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        let gil = Python::acquire_gil();
        let py = gil.python();
        fastobo::ast::InstanceFrame::from_py(self.clone_py(py), py).fmt(f)
    }
}

impl FromPy<fastobo::ast::InstanceFrame> for InstanceFrame {
    fn from_py(frame: fastobo::ast::InstanceFrame, py: Python) -> Self {
        // Self::with_clauses(
        //     Ident::from_py(frame.id().as_ref().clone(), py),
        //     frame
        //         .into_iter()
        //         .map(|line| TypedefClause::from_py(line.into_inner(), py))
        //         .collect(),
        // )
        Self::new(Ident::from_py(frame.id().as_ref().clone(), py))
    }
}

impl FromPy<InstanceFrame> for fastobo::ast::InstanceFrame {
    fn from_py(frame: InstanceFrame, py: Python) -> Self {
        // fastobo::ast::InstanceFrame::with_clauses(
        //     fastobo::ast::InstanceIdent::new(frame.id.into_py(py)),
        //     frame
        //         .clauses
        //         .iter()
        //         .map(|f| fastobo::ast::InstanceClause::from_py(f, py))
        //         .map(fastobo::ast::Line::from)
        //         .collect(),
        // )
        fastobo::ast::InstanceFrame::new(
            fastobo::ast::InstanceIdent::new(frame.id.into_py(py))
        )
    }
}

impl FromPy<InstanceFrame> for fastobo::ast::EntityFrame {
    fn from_py(frame: InstanceFrame, py: Python) -> Self {
        Self::from(fastobo::ast::InstanceFrame::from_py(frame, py))
    }
}
