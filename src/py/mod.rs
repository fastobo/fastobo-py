//! Definition of the Python classes exported in the `fastobo` module.

use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::BufReader;
use std::rc::Rc;
use std::str::FromStr;
use std::string::ToString;

use pyo3::class::gc::PyVisit;
use pyo3::exceptions::IndexError;
use pyo3::exceptions::RuntimeError;
use pyo3::exceptions::TypeError;
use pyo3::exceptions::ValueError;
use pyo3::gc::PyTraverseError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::PyGCProtocol;
use pyo3::PyNativeType;
use pyo3::PyObjectProtocol;
use pyo3::PySequenceProtocol;
use pyo3::PyTypeInfo;

use fastobo::ast as obo;

use crate::error::Error;
use crate::pyfile::PyFile;
use crate::utils::AsGILRef;
use crate::utils::ClonePy;

// ---------------------------------------------------------------------------

pub mod abc;
pub mod doc;
pub mod header;
pub mod id;
pub mod pv;
pub mod syn;
pub mod term;
pub mod typedef;
pub mod xref;

use self::doc::OboDoc;
use super::built;

// --- Module export ---------------------------------------------------------

/// The Faultless AST for Open Biomedical Ontologies.
///
///
#[pymodule]
fn fastobo(py: Python, m: &PyModule) -> PyResult<()> {
    use self::abc::PyInit_abc;
    use self::doc::PyInit_doc;
    use self::header::PyInit_header;
    use self::id::PyInit_id;
    use self::pv::PyInit_pv;
    use self::syn::PyInit_syn;
    use self::term::PyInit_term;
    use self::typedef::PyInit_typedef;
    use self::xref::PyInit_xref;

    m.add("__package__", "fastobo")?;
    // m.add("__built__", pyo3_built!(py, built))?;

    add_submodule!(py, m, abc);
    add_submodule!(py, m, doc);
    add_submodule!(py, m, header);
    add_submodule!(py, m, id);
    add_submodule!(py, m, pv);
    add_submodule!(py, m, syn);
    add_submodule!(py, m, term);
    add_submodule!(py, m, typedef);
    add_submodule!(py, m, xref);

    /// load(fh)
    /// --
    ///
    /// Load an OBO document from the given path or file handle.
    ///
    /// Arguments:
    ///     fh (str or file-handle): the path to an OBO file, or a **binary**
    ///         stream that contains a serialized OBO document. *A binary
    ///         stream needs a* ``read(x)`` *method returning* ``x`` *bytes*.
    ///
    /// Returns:
    ///     `~fastobo.doc.OboDoc`: the OBO document deserialized into an
    ///     Abstract Syntax Tree.
    ///
    /// Raises:
    ///     TypeError: when the argument is not a `str` or a binary stream.
    ///     SyntaxError: when the document is not in valid OBO syntax.
    ///     OSError: when an underlying OS error occurs.
    ///     *other*: any exception raised by ``fh.read``.
    ///
    /// Example:
    ///     Use ``requests`` and ``fastobo`` to parse an ontology downloaded
    ///     from the Berkeley BOP portal:
    ///
    ///     >>> import requests
    ///     >>> url = "http://purl.obolibrary.org/obo/cmo.obo"
    ///     >>> doc = fastobo.load(requests.get(url, stream=True).raw)
    ///     >>> doc.header[2]
    ///     OntologyClause('cmo.obo')
    #[pyfn(m, "load")]
    fn load(py: Python, fh: &PyAny) -> PyResult<OboDoc> {
        if let Ok(s) = fh.downcast_ref::<PyString>() {
            let path = s.to_string()?;
            match obo::OboDoc::from_file(path.as_ref()) {
                Ok(doc) => Ok(doc.into_py(py)),
                Err(e) => Error::from(e).into(),
            }
        } else {
            match PyFile::from_object(fh.py(), fh) {
                // Object is a binary file-handle: attempt to parse the
                // document and return an `OboDoc` object.
                Ok(f) => {
                    let mut bufreader = std::io::BufReader::new(f);
                    match obo::OboDoc::from_stream(&mut bufreader) {
                        Ok(doc) => Ok(doc.into_py(py)),
                        Err(e) => bufreader
                            .into_inner()
                            .into_err()
                            .unwrap_or_else(|| Error::from(e).into())
                            .into(),
                    }
                }
                // Object is not a binary file-handle: wrap the inner error
                // into a `TypeError` and raise that error.
                Err(inner) => {
                    let msg = "expected path or binary file handle";
                    let err = TypeError::py_err(msg).to_object(py);
                    err.call_method1(
                        py,
                        "__setattr__",
                        ("__cause__".to_object(py), inner.to_object(py)),
                    )?;
                    Err(PyErr::from_instance(err.as_ref(py).as_ref()))
                }
            }
        }
    }

    /// loads(document)
    /// --
    ///
    /// Load an OBO document from a string.
    ///
    /// Arguments:
    ///     document (str): a string containing an OBO document.
    ///
    /// Returns:
    ///     `~fastobo.doc.OboDoc`: the OBO document deserialized into an
    ///     Abstract Syntax Tree.
    ///
    /// Raises:
    ///     TypeError: when the argument is not a `str`.
    ///     SyntaxError: when the document is not in valid OBO syntax.
    ///
    /// Example:
    ///     Use ``fastobo.loads`` to deserialize a literal OBO frame into the
    ///     corresponding syntax tree:
    ///
    ///     >>> import textwrap
    ///     >>> doc = fastobo.loads(textwrap.dedent(
    ///     ...     """
    ///     ...     [Term]
    ///     ...     id: TST:001
    ///     ...     name: test item
    ///     ...     """
    ///     ... ))
    ///     >>> doc[0].id
    ///     PrefixedIdent('TST', '001')
    ///     >>> doc[0][0]
    ///     NameClause('test item')
    ///
    #[pyfn(m, "loads")]
    fn loads(py: Python, document: &str) -> PyResult<OboDoc> {
        match fastobo::ast::OboDoc::from_str(document) {
            Ok(doc) => Ok(doc.into_py(py)),
            Err(e) => Error::from(e).into(),
        }
    }

    Ok(())
}
