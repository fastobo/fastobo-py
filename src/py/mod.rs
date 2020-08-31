//! Definition of the Python classes exported in the `fastobo` module.

use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::BufRead;
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
use fastobo::parser::Parser;
use fastobo::visit::VisitMut;
use fastobo_graphs::FromGraph;
use fastobo_graphs::IntoGraph;
use fastobo_graphs::model::GraphDocument;

use crate::raise;
use crate::error::Error;
use crate::error::GraphError;
use crate::iter::FrameReader;
use crate::iter::InternalParser;
use crate::pyfile::PyFileRead;
use crate::pyfile::PyFileWrite;
use crate::utils::ClonePy;

// ---------------------------------------------------------------------------

pub mod abc;
pub mod doc;
pub mod header;
pub mod id;
pub mod instance;
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
#[pymodule(fastobo)]
pub fn init(py: Python, m: &PyModule) -> PyResult<()> {
    m.add("__package__", "fastobo")?;
    m.add("__build__", pyo3_built!(py, built))?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", env!("CARGO_PKG_AUTHORS").replace(':', "\n"))?;

    add_submodule!(py, m, abc);
    add_submodule!(py, m, doc);
    add_submodule!(py, m, header);
    add_submodule!(py, m, id);
    add_submodule!(py, m, instance);
    add_submodule!(py, m, pv);
    add_submodule!(py, m, syn);
    add_submodule!(py, m, term);
    add_submodule!(py, m, typedef);
    add_submodule!(py, m, xref);

    /// Iterate over the frames contained in an OBO document.
    ///
    /// The header frame can be accessed with the ``header`` method of the
    /// returned object. Entity frames are yielded one after like any Python
    /// iterator. See the *Examples* section.
    ///
    /// Arguments:
    ///     fh (str or file-handle): the path to an OBO file, or a **binary**
    ///         stream that contains a serialized OBO document. *A binary
    ///         stream needs a* ``read(x)`` *method returning* ``x`` *bytes*.
    ///     ordered (bool): whether or not to yield the frames in the same
    ///         order they are declared in the source document.
    ///     threads (int): the number of threads to use for parsing. Set to
    ///         **0** to detect the number of logical cores, **1** to use the
    ///         single threadeded parser, or to any positive integer value.
    ///
    /// Raises:
    ///     TypeError: when the argument is not a `str` or a binary stream.
    ///     SyntaxError: when the document is not in valid OBO syntax.
    ///     OSError: when an underlying OS error occurs.
    ///     *other*: any exception raised by ``fh.read``.
    ///
    /// Example:
    ///     Use ``fastobo.iter`` to load an ontology frame-by-frame, even
    ///     a larger one:
    ///
    ///     >>> reader = fastobo.iter('tests/data/ms.obo')
    ///     >>> reader.header()
    ///     HeaderFrame([...])
    ///     >>> next(reader)
    ///     TermFrame(PrefixedIdent('MS', '0000000'))
    ///     >>> list(reader)
    ///     [TermFrame(PrefixedIdent('MS', '1000001')), ...]
    ///
    #[pyfn(m, "iter", ordered="true", threads="0")]
    #[text_signature = "(fh, ordered=True, threads=0)"]
    fn iter(py: Python, fh: &PyAny, ordered: bool, threads: i16) -> PyResult<FrameReader> {
        if let Ok(s) = fh.cast_as::<PyString>() {
            let path = s.to_string()?;
            FrameReader::from_path(path.as_ref(), ordered, threads)
        } else {
            match FrameReader::from_handle(fh, ordered, threads) {
                Ok(r) => Ok(r),
                Err(inner) => {
                    raise!(py, TypeError("expected path or binary file handle") from inner);
                }
            }
        }
    }

    /// Load an OBO document from the given path or file handle.
    ///
    /// Arguments:
    ///     fh (str or file-handle): the path to an OBO file, or a **binary**
    ///         stream that contains a serialized OBO document. *A binary
    ///         stream needs a* ``read(x)`` *method returning* ``x`` *bytes*.
    ///     ordered (bool): whether or not to yield the frames in the same
    ///         order they are declared in the source document.
    ///     threads (int): the number of threads to use for parsing. Set to
    ///         **0** to detect the number of logical cores, **1** to use the
    ///         single threadeded parser, or to any positive integer value.
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
    ///     Use ``urllib`` and ``fastobo`` to parse an ontology downloaded
    ///     from the Berkeley BOP portal:
    ///
    ///     >>> from urllib.request import urlopen
    ///     >>> url = "http://purl.obolibrary.org/obo/eco.obo"
    ///     >>> doc = fastobo.load(urlopen(url))
    ///     >>> doc.header[3]
    ///     SavedByClause('rctauber')
    ///
    #[pyfn(m, "load", ordered="true", threads="0")]
    #[text_signature = "(fh, threads=0)"]
    fn load(py: Python, fh: &PyAny, ordered: bool, threads: i16) -> PyResult<OboDoc> {
        if let Ok(s) = fh.cast_as::<PyString>() {
            // get a buffered reader to the resources pointed by `path`
            let path = s.to_string()?;
            let bf = match std::fs::File::open(&*path) {
                Ok(f) => std::io::BufReader::new(f),
                Err(e) => return Err(PyErr::from(Error::from(e))),
            };
            // use a sequential or a threaded reader depending on `threads`.
            let mut reader = InternalParser::with_thread_count(bf, threads)?;
            // set the `ordered` flag and parse the document using the reader
            reader.ordered(ordered);
            match py.allow_threads(|| reader.try_into_doc()) {
                Ok(doc) => Ok(doc.into_py(py)),
                Err(e) => Error::from(e).with_path(path).into(),
            }
        } else {
            // get a buffered reader by wrapping the given file handle
            let bf = match PyFileRead::from_ref(fh) {
                // Object is a binary file-handle: attempt to parse the
                // document and return an `OboDoc` object.
                Ok(f) => std::io::BufReader::new(f),
                // Object is not a binary file-handle: wrap the inner error
                // into a `TypeError` and raise that error.
                Err(e) => {
                    raise!(py, TypeError("expected path or binary file handle") from e)
                }
            };
            // use a sequential or a threaded reader depending on `threads`.
            let mut reader = InternalParser::with_thread_count(bf, threads)?;
            // set the `ordered` flag
            reader.ordered(ordered);
            //  and parse the document, check the result and extract the
            // internal Python error if the parser failed,
            match reader.try_into_doc() {
                Ok(doc) => Ok(doc.into_py(py)),
                Err(e) if PyErr::occurred(py) => Err(PyErr::fetch(py)),
                Err(e) => Err(Error::from(e).into())
            }
        }
    }

    /// Load an OBO document from a string.
    ///
    /// Arguments:
    ///     document (str): a string containing an OBO document.
    ///     ordered (bool): whether or not to yield the frames in the same
    ///         order they are declared in the source document.
    ///     threads (int): the number of threads to use for parsing. Set to
    ///         **0** to detect the number of logical cores, **1** to use the
    ///         single threadeded parser, or to any positive integer value.
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
    #[pyfn(m, "loads", ordered="true", threads="0")]
    #[text_signature = "(document)"]
    fn loads(py: Python, document: &str, ordered: bool, threads: i16) -> PyResult<OboDoc> {
        let cursor = std::io::Cursor::new(document);
        let mut reader = InternalParser::with_thread_count(cursor, threads)?;
        reader.ordered(ordered);
        match py.allow_threads(|| reader.try_into_doc()) {
            Ok(doc) => Ok(doc.into_py(py)),
            Err(e) => Error::from(e).into(),
        }
    }

    /// Load an OBO graph from the given path or file handle.
    ///
    /// Both JSON and YAML formats are supported. *Actually, since YAML is a
    /// superset of JSON, all graphs are in YAML format.*
    ///
    /// Arguments:
    ///     fh (str or file-handle): the path to an OBO graph file, or a
    ///         **binary** stream that contains a serialized OBO document.
    ///         *A binary stream needs a* ``read(x)`` *method returning*
    ///         ``x`` *bytes*.
    ///
    /// Returns:
    ///     `~fastobo.doc.OboDoc`: the first graph of the OBO graph
    ///     converted to an OBO document. The schema allows for more than
    ///     one graph but this is only used when merging imports into the
    ///     same document.
    ///
    /// Raises:
    ///     TypeError: when the argument is not a `str` or a binary stream.
    ///     ValueError: when the JSON is not a valid OBO Graph.
    ///     SyntaxError: when the document contains invalid OBO identifiers.
    ///     OSError: when an underlying OS error occurs.
    ///     *other*: any exception raised by ``fh.read``.
    ///
    /// Example:
    ///     Use ``urllib`` and ``fastobo`` to parse an ontology downloaded
    ///     from the Berkeley BOP portal:
    ///
    ///     >>> from urllib.request import urlopen
    ///     >>> url = "http://purl.obolibrary.org/obo/pato.json"
    ///     >>> doc = fastobo.load_graph(urlopen(url))
    ///     >>> doc[0]
    ///     TermFrame(PrefixedIdent('BFO', '0000002'))
    ///
    #[pyfn(m, "load_graph")]
    #[text_signature = "(fh)"]
    fn load_graph(py: Python, fh: &PyAny) -> PyResult<OboDoc> {
        let doc: GraphDocument = if let Ok(s) = fh.cast_as::<PyString>() {
            // Argument is a string, assumed to be a path: open the file.
            // and extract the graph
            let path = s.to_string()?;
            py.allow_threads(|| fastobo_graphs::from_file(path.as_ref()))
                .map_err(|e| PyErr::from(GraphError::from(e)))?
        } else {
            // Argument is not a string, check if it is a file-handle.
            let mut f = match PyFileRead::from_ref(fh) {
                Ok(f) => f,
                Err(e) => raise!(py, TypeError("expected path or binary file handle") from e)
            };
            // Extract the graph
            match fastobo_graphs::from_reader(&mut f) {
                Ok(doc) => doc,
                Err(e) if PyErr::occurred(py) => return Err(PyErr::fetch(py)),
                Err(e) => return Err(GraphError::from(e).into())
            }
        };

        // Convert the graph to an OBO document
        let graph = doc.graphs.into_iter().next().unwrap();
        let doc = py.allow_threads(|| obo::OboDoc::from_graph(graph))
            .map_err(GraphError::from)?;

        // Convert the OBO document to a Python `OboDoc` class
        Ok(OboDoc::from_py(doc, py))
    }

    /// Dump an OBO graph into the given writer or file handle, serialized
    /// into a compact JSON representation.
    ///
    /// Arguments:
    ///     fh (str or file-handle): the path to a file, or a writable
    ///         **binary** stream to write the serialized graph into.
    ///         *A binary stream needs a* ``write(b)`` *method that accepts
    ///         binary strings*.
    ///     doc (`~fastobo.doc.OboDoc`): the OBO document to be converted
    ///         into an OBO Graph.
    ///
    /// Raises:
    ///     TypeError: when the argument have invalid types.
    ///     ValueError: when the JSON serialization fails.
    ///     OSError: when an underlying OS error occurs.
    ///     *other*: any exception raised by ``fh.read``.
    ///
    /// Example:
    ///     Use ``fastobo`` to convert an OBO file into an OBO graph:
    ///
    ///     >>> doc = fastobo.load("tests/data/plana.obo")
    ///     >>> fastobo.dump_graph(doc, "tests/data/plana.json")
    ///
    #[pyfn(m, "dump_graph")]
    #[text_signature = "(doc, fh)"]
    fn dump_graph(py: Python, obj: &OboDoc, fh: &PyAny) -> PyResult<()> {
        // Convert OBO document to an OBO Graph document.
        let doc = obo::OboDoc::from_py(obj.clone_py(py), py);
        let graph = py.allow_threads(|| doc.into_graph())
            .map_err(|e| PyErr::from(GraphError::from(e)))?;

        // Write the document
        if let Ok(s) = fh.cast_as::<PyString>() {
            // Write into a file if given a path as a string.
            let path = s.to_string()?;
            py.allow_threads(|| fastobo_graphs::to_file(path.as_ref(), &graph))
                .map_err(|e| PyErr::from(GraphError::from(e)))
        } else {
            // Write into the handle if given a writable file.
            let mut f = match PyFileWrite::from_ref(fh) {
                Ok(f) => f,
                Err(e) => {
                    raise!(py, TypeError("expected path or binary file handle") from e)
                }
            };
            // Write the graph
            match fastobo_graphs::to_writer(&mut f, &graph) {
                Ok(()) => Ok(()),
                Err(_) if PyErr::occurred(py) => Err(PyErr::fetch(py)),
                Err(e) => Err(PyErr::from(GraphError::from(e))),
            }
        }
    }

    Ok(())
}
