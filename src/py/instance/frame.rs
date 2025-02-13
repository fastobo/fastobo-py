use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::fmt::Write;
use std::str::FromStr;

use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyIterator;
use pyo3::types::PyString;
use pyo3::AsPyPointer;
use pyo3::PyTypeInfo;

use fastobo::ast;

use super::super::abc::AbstractEntityFrame;
use super::super::id::Ident;
use crate::utils::AbstractClass;
use crate::utils::ClonePy;
use crate::utils::EqPy;
use crate::utils::FinalClass;
use crate::utils::IntoPy;

#[pyclass(extends=AbstractEntityFrame, module="fastobo.instance")]
#[derive(Debug, FinalClass, EqPy)]
#[base(AbstractEntityFrame)]
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
        let frame: fastobo::ast::InstanceFrame =
            Python::with_gil(|py| self.clone_py(py).into_py(py));
        frame.fmt(f)
    }
}

impl IntoPy<InstanceFrame> for fastobo::ast::InstanceFrame {
    fn into_py(self, py: Python) -> InstanceFrame {
        let id: Ident = self.id().as_ref().clone().into_py(py);
        InstanceFrame::new(id)
    }
}

impl IntoPy<fastobo::ast::InstanceFrame> for InstanceFrame {
    fn into_py(self, py: Python) -> fastobo::ast::InstanceFrame {
        fastobo::ast::InstanceFrame::new(fastobo::ast::InstanceIdent::new(self.id.into_py(py)))
    }
}

impl IntoPy<fastobo::ast::EntityFrame> for InstanceFrame {
    fn into_py(self, py: Python) -> fastobo::ast::EntityFrame {
        let frame: fastobo::ast::InstanceFrame = self.into_py(py);
        fastobo::ast::EntityFrame::from(frame)
    }
}
