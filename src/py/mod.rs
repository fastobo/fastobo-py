//! Definition of the Python classes exported in the `fastobo` module.

use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fmt::Result as FmtResult;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::rc::Rc;
use std::str::FromStr;
use std::string::ToString;

use pyo3::class::gc::PyVisit;
use pyo3::exceptions::PyRuntimeError;
use pyo3::exceptions::PyTypeError;
use pyo3::exceptions::PyValueError;
use pyo3::gc::PyTraverseError;
use pyo3::prelude::*;
use pyo3::types::PyAny;
use pyo3::types::PyDict;
use pyo3::types::PyList;
use pyo3::types::PyString;
use pyo3::PyTypeInfo;

use fastobo::ast as obo;
use fastobo::parser::Parser;
use fastobo::visit::VisitMut;
use fastobo_graphs::model::GraphDocument;
use fastobo_graphs::FromGraph;
use fastobo_graphs::IntoGraph;
use fastobo_owl::IntoOwl;
use fastobo_owl::IntoOwlPrefixes;
use horned_owl::error::HornedError;
use horned_owl::io::ofn::writer::AsFunctional;
use horned_owl::model::AnnotatedComponent;
use horned_owl::ontology::component_mapped::ComponentMappedOntology;

use crate::error::Error;
use crate::error::GraphError;
use crate::error::OwlError;
use crate::iter::FrameReader;
use crate::iter::InternalParser;
use crate::pyfile::PyFileRead;
use crate::pyfile::PyFileWrite;
use crate::raise;
use crate::utils::ClonePy;
use crate::utils::IntoPy;

// ---------------------------------------------------------------------------

pub mod abc;
pub mod doc;
pub mod exceptions;
pub mod header;
pub mod id;
pub mod instance;
pub mod pv;
pub mod syn;
pub mod term;
pub mod typedef;
pub mod xref;

use self::doc::EntityFrame;
use self::doc::OboDoc;
use super::built;

// --- Module export ---------------------------------------------------------

/// Iterate over the frames contained in an OBO document.
///
/// The header frame can be accessed with the ``header`` method of the
/// returned object. Entity frames are yielded one after like any Python
/// iterator. See the *Examples* section.
///
/// Arguments:
///     file (str, `os.PathLike` or file-handle): The path to an OBO file, or
///         a **binary** stream that contains a serialized OBO document.
///         *A binary stream needs a* ``read(x)`` *method returning* ``x``
///         `bytes`.
///     ordered (bool): Whether or not to yield the frames in the same
///         order they are declared in the source document.
///     threads (int): The number of threads to use for parsing. Set to
///         **0** to detect the number of logical cores, **1** to use the
///         single threadeded parser, or to any positive integer value.
///
/// Yields:
///     `~fastobo.abc.AbstractFrame`: The individual frames contained
///     in the OBO document.
///
/// Raises:
///     TypeError: When the argument is not a `str` or a binary stream.
///     SyntaxError: When the document is not in valid OBO syntax.
///     OSError: When an underlying OS error occurs.
///
/// Example:
///     Use ``fastobo.iter`` to load an ontology frame-by-frame, even
///     a larger one:
///
///     >>> reader = fastobo.iter('ms.obo')
///     >>> reader.header()
///     HeaderFrame([...])
///     >>> next(reader)
///     TermFrame(PrefixedIdent('MS', '0000000'))
///     >>> list(reader)
///     [TermFrame(PrefixedIdent('MS', '1000001')), ...]
///
#[pyfunction]
#[pyo3(name = "iter", text_signature = "(file, ordered=True, threads=0)", signature = (file, ordered=true, threads=0))]
fn iter<'py>(
    py: Python<'py>,
    file: &Bound<'py, PyAny>,
    ordered: bool,
    threads: i16,
) -> PyResult<FrameReader> {
    // get the path to the given file
    let pathlike = py
        .import_bound(pyo3::intern!(py, "os"))?
        .call_method1(pyo3::intern!(py, "fsdecode"), (&file,));
    if let Ok(path) = pathlike {
        let decoded = path.downcast::<PyString>()?;
        FrameReader::from_path(decoded.to_str()?, ordered, threads)
    } else {
        match FrameReader::from_handle(file, ordered, threads) {
            Ok(r) => Ok(r),
            Err(inner) if inner.is_instance_of::<pyo3::exceptions::PySyntaxError>(py) => Err(inner),
            Err(inner) => {
                raise!(py, PyTypeError("expected path or binary file handle") from inner);
            }
        }
    }
}

/// Load an OBO document from the given path or file handle.
///
/// Arguments:
///     fh (str or file-handle): The path to an OBO file, or a **binary**
///         stream that contains a serialized OBO document. *A binary
///         stream needs a* ``read(x)`` *method returning* ``x`` *bytes*.
///     ordered (bool): Whether or not to yield the frames in the same
///         order they are declared in the source document.
///     threads (int): The number of threads to use for parsing. Set to
///         **0** to detect the number of logical cores, **1** to use the
///         single threadeded parser, or to any positive integer value.
///
/// Returns:
///     `~fastobo.doc.OboDoc`: The OBO document deserialized into an
///     Abstract Syntax Tree.
///
/// Raises:
///     TypeError: When the argument is not a `str` or a binary stream.
///     SyntaxError: When the document is not in valid OBO syntax.
///     OSError: When an underlying OS error occurs.
///
/// Example:
///     Use `~urllib.request.urlopen` and `fastobo.load` to parse an
///     ontology downloaded from the OBO Library:
///
///     >>> from urllib.request import urlopen
///     >>> url = "http://purl.obolibrary.org/obo/po.obo"
///     >>> doc = fastobo.load(urlopen(url))
///     >>> doc.header[3]
///     SubsetdefClause(UnprefixedIdent('Angiosperm'), 'Term for angiosperms')
///
#[pyfunction]
#[pyo3(name = "load", text_signature = "(file, ordered=True, threads=0)", signature=(file, ordered=true, threads=0))]
fn load<'py>(
    py: Python<'py>,
    file: &Bound<'py, PyAny>,
    ordered: bool,
    threads: i16,
) -> PyResult<OboDoc> {
    // extract either a path or a file-handle from the arguments
    let path: Option<String>;
    let pathlike = py
        .import_bound(pyo3::intern!(py, "os"))?
        .call_method1(pyo3::intern!(py, "fsdecode"), (&file,));
    let b: Box<dyn BufRead> = if let Ok(p) = pathlike {
        let decoded = p.downcast::<PyString>()?;
        // get a buffered reader to the resources pointed by `path`
        let bf = match std::fs::File::open(decoded.to_str()?) {
            Ok(f) => std::io::BufReader::new(f),
            Err(e) => return Err(PyErr::from(Error::from(e))),
        };
        // store the path for later
        path = Some(decoded.to_str()?.to_string());
        // use a sequential or a threaded reader depending on `threads`.
        Box::new(bf)
    } else {
        // get a buffered reader by wrapping the given file handle
        let bf = match PyFileRead::from_ref(file) {
            // Object is a binary file-handle: attempt to parse the
            // document and return an `OboDoc` object.
            Ok(f) => std::io::BufReader::new(f),
            // Object is not a binary file-handle: wrap the inner error
            // into a `TypeError` and raise that error.
            Err(e) => {
                raise!(py, PyTypeError("expected path or binary file handle") from e)
            }
        };
        // extract the path from the `name` attribute
        path = file
            .getattr("name")
            .and_then(|n| match n.downcast::<PyString>() {
                Ok(x) => Ok(x.to_str()?.to_string()),
                Err(e) => Err(PyErr::from(e)),
            })
            .ok();
        // use a sequential or a threaded reader depending on `threads`.
        Box::new(bf)
    };

    // create the reader and set the `ordered` flag
    let mut reader = InternalParser::with_thread_count(b, threads)?;
    reader.ordered(ordered);

    // read the header and check it did not error
    let header = match reader.next().unwrap() {
        Ok(frame) => Ok(frame.into_header().unwrap().into_py(py)),
        Err(e) if PyErr::occurred(py) => Err(PyErr::fetch(py)),
        Err(e) => match &path {
            Some(p) => Err(Error::from(e).with_path(p).into()),
            None => Err(Error::from(e).into()),
        },
    }?;

    // read the rest while transforming it to Python
    let frames = reader
        .map(|res| res.map(|frame| frame.into_entity().unwrap()))
        .map(|res| res.map(|entity| entity.into_py(py)))
        .collect::<fastobo::error::Result<Vec<EntityFrame>>>();

    // propagate the Python error if any error occurred
    match frames {
        Ok(entities) => Ok(OboDoc::with_entities(Py::new(py, header)?, entities)),
        Err(e) if PyErr::occurred(py) => Err(PyErr::fetch(py)),
        Err(e) => match &path {
            Some(p) => Err(Error::from(e).with_path(p).into()),
            None => Err(Error::from(e).into()),
        },
    }
}

/// Load an OBO document from a string.
///
/// Arguments:
///     document (str): A string containing an OBO document.
///     ordered (bool): Whether or not to yield the frames in the same
///         order they are declared in the source document.
///     threads (int): The number of threads to use for parsing. Set to
///         **0** to detect the number of logical cores, **1** to use the
///         single threadeded parser, or to any positive integer value.
///
/// Returns:
///     `~fastobo.doc.OboDoc`: The OBO document deserialized into an
///     Abstract Syntax Tree.
///
/// Raises:
///     TypeError: When the argument is not a `str`.
///     SyntaxError: When the document is not in valid OBO syntax.
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
#[pyfunction]
#[pyo3(name = "loads", text_signature = "(document, ordered=True, threads=0)", signature=(document, ordered=true, threads=0))]
fn loads<'py>(
    py: Python<'py>,
    document: &Bound<'py, PyString>,
    ordered: bool,
    threads: i16,
) -> PyResult<OboDoc> {
    let cursor = std::io::Cursor::new(document.to_str()?);
    let mut reader = InternalParser::with_thread_count(cursor, threads)?;
    reader.ordered(ordered);
    // match py.allow_threads(|| reader.try_into_doc()) {
    match reader.try_into_doc() {
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
///     file (str, `os.PathLike` or file-handle): The path to an OBO graph
///         file, or a **binary** stream that contains a serialized OBO
///         document. *A binary stream needs a* ``read(x)`` *method returning*
///         ``x`` `bytes`.
///
/// Returns:
///     `~fastobo.doc.OboDoc`: The first graph of the OBO graph
///     converted to an OBO document. The schema allows for more than
///     one graph but this is only used when merging imports into the
///     same document.
///
/// Raises:
///     TypeError: When the argument is not a `str` or a binary stream.
///     ValueError: When the JSON is not a valid OBO Graph.
///     SyntaxError: When the document contains invalid OBO identifiers.
///     OSError: When an underlying OS error occurs.
///
/// Example:
///     Use ``fastobo`` to parse an ontology graph in JSON format:
///
///     >>> graph = fastobo.load_graph("pato.json")
///     >>> terms = [
///     ...     term for term in graph
///     ...     if isinstance(term.id, fastobo.id.PrefixedIdent)
///     ...     and term.id.prefix == "PATO"
///     ... ]
///     >>> min(terms, key=lambda term: str(term.id))
///     TermFrame(PrefixedIdent('PATO', '0000000'))
///
#[pyfunction]
#[pyo3(name = "load_graph", text_signature = "(file)")]
fn load_graph<'py>(py: Python<'py>, file: &Bound<'py, PyAny>) -> PyResult<OboDoc> {
    // load graph from a path or a file-handle
    let pathlike = py
        .import_bound(pyo3::intern!(py, "os"))?
        .call_method1(pyo3::intern!(py, "fsdecode"), (&file,));
    let doc: GraphDocument = if let Ok(path) = pathlike {
        let decoded = path.downcast::<PyString>()?;
        fastobo_graphs::from_file(decoded.to_str()?)
            .map_err(|e| PyErr::from(GraphError::from(e)))?
    } else {
        // Argument is not a string, check if it is a file-handle.
        let mut f = match PyFileRead::from_ref(file) {
            Ok(f) => f,
            Err(e) => raise!(py, PyTypeError("expected path or binary file handle") from e),
        };
        // Extract the graph
        match fastobo_graphs::from_reader(&mut f) {
            Ok(doc) => doc,
            Err(e) if PyErr::occurred(py) => return Err(PyErr::fetch(py)),
            Err(e) => return Err(GraphError::from(e).into()),
        }
    };

    // Convert the graph to an OBO document
    let graph = doc.graphs.into_iter().next().unwrap();
    // let doc = py.allow_threads(|| obo::OboDoc::from_graph(graph))
    let doc = obo::OboDoc::from_graph(graph).map_err(GraphError::from)?;

    // Convert the OBO document to a Python `OboDoc` class
    Ok(doc.into_py(py))
}

/// Dump an OBO graph into the given writer or file handle, serialized
/// into a compact JSON representation.
///
/// Arguments:
///     file (str, `os.PathLike` or file-handle): The path to a file, or
///         a writable **binary** stream to write the serialized graph into.
///         *A binary stream needs a* ``write(b)`` *method that accepts
///         binary strings*.
///     doc (`~fastobo.doc.OboDoc`): The OBO document to be converted
///         into an OBO Graph.
///
/// Raises:
///     TypeError: When the argument have invalid types.
///     ValueError: When the JSON serialization fails.
///     OSError: When an underlying OS error occurs.
///
/// Example:
///     Use ``fastobo`` to convert an OBO file into an OBO graph:
///
///     >>> doc = fastobo.load("plana.obo")
///     >>> fastobo.dump_graph(doc, "plana.json")
///
#[pyfunction]
#[pyo3(name = "dump_graph", text_signature = "(doc, file)")]
fn dump_graph<'py>(py: Python<'py>, obj: &OboDoc, file: &Bound<'py, PyAny>) -> PyResult<()> {
    // Convert OBO document to an OBO Graph document.
    let doc: obo::OboDoc = obj.clone_py(py).into_py(py);
    // FIXME: let graph = py.allow_threads(|| doc.into_graph())
    let graph = doc
        .into_graph()
        .map_err(|e| PyErr::from(GraphError::from(e)))?;

    // Write the document
    let pathlike = py
        .import_bound(pyo3::intern!(py, "os"))?
        .call_method1(pyo3::intern!(py, "fsdecode"), (&file,));
    if let Ok(path) = pathlike {
        let decoded = path.downcast::<PyString>()?;
        fastobo_graphs::to_file(decoded.to_str()?, &graph)
            .map_err(|e| PyErr::from(GraphError::from(e)))
    } else {
        // Write into the handle if given a writable file.
        let mut f = match PyFileWrite::from_ref(file) {
            Ok(f) => f,
            Err(e) => {
                raise!(py, PyTypeError("expected path or binary file handle") from e)
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

/// Convert an OBO ontology to OWL and write it to the given handle.
///
/// Arguments:
///     file (str, `os.PathLike` or file-handle): The path to a file,
///         or a writable **binary** stream to write the serialized
///         ontology into. *A binary stream needs a* ``write(b)`` *method
///         that accepts* `bytes`.
///     doc (`~fastobo.doc.OboDoc`): The OBO document to be converted
///         into an OWL Ontology.
///     format (`str`): The OWL format to serialize the converted OWL
///         document into. Supported values are: ``ofn`` for
///         `Functional-style syntax <https://w3.org/TR/owl2-syntax/>`_,
///         ``owx`` for OWL/XML syntax, ``rdf`` for RDX/XML syntax.
///
/// Raises:
///     TypeError: When the argument have invalid types.
///     ValueError: When the conversion to OWL fails.
///     OSError: When an underlying OS error occurs.
///
/// Example:
///     Use ``fastobo`` to convert an OBO file into an OWL file:
///
///     >>> doc = fastobo.load("ms.obo")
///     >>> fastobo.dump_owl(doc, "ms.ofn", format="ofn")
///
/// Caution:
///     This method is experimental. Conversion to OWL is provided on a
///     best-effort basis using a dedicated Rust implementation of the
///     OBO to OWL2-DL mapping. It should produce a document with correct
///     syntax, but might fail to preserve the semantics. In such cases,
///     consider opening an issue directly on the ``fastobo-owl``
///     `issue tracker <https://github.com/fastobo/fastobo-owl/issues>`_.
///
/// Hint:
///     To support serialization to OWL, an OBO document is required to
///     declare an ``ontology`` clause in the header. Furthermore, every
///     entity frame must have a ``namespace`` clause, otherwise a
///     ``default-namespace`` clause must be declared in the header.
///     Failure to do both will result in a `ValueError` being thrown.
///
#[pyfunction]
#[pyo3(name = "dump_owl", text_signature = r#"(doc, file, format="ofn")"#, signature=(obj, file, format="ofn"))]
fn dump_owl<'py>(
    py: Python<'py>,
    obj: &OboDoc,
    file: &Bound<'py, PyAny>,
    format: &str,
) -> PyResult<()> {
    // get the path to the given file
    let pathlike = py
        .import_bound(pyo3::intern!(py, "os"))?
        .call_method1(pyo3::intern!(py, "fsdecode"), (&file,));
    let mut b: Box<dyn Write> = if let Ok(path) = pathlike {
        let decoded = path.downcast::<PyString>()?;
        std::fs::File::create(decoded.to_str()?)
            .map(std::io::BufWriter::new)
            .map(Box::new)?
    } else {
        PyFileWrite::from_ref(&file)
            .map(std::io::BufWriter::new)
            .map(Box::new)?
    };

    // Convert OBO document to an OWL document.
    let doc: obo::OboDoc = obj.clone_py(py).into_py(py);
    let prefixes = doc.prefixes();
    let ont = doc
        .into_owl::<ComponentMappedOntology<Rc<str>, Rc<AnnotatedComponent<Rc<str>>>>>()
        .map_err(OwlError::from)?;

    // Write the OWL document to the given handle
    let result = match format {
        "ofn" => horned_owl::io::ofn::writer::write(&mut b, &ont, Some(&prefixes)),
        "owx" => horned_owl::io::owx::writer::write(&mut b, &ont, Some(&prefixes)),
        "rdf" => horned_owl::io::rdf::writer::write(&mut b, &ont),
        _ => return Err(PyValueError::new_err(format!("invalid format: {}", format))),
    };

    if let Err(e) = result {
        match e {
            HornedError::IOError(e) => return Err(PyErr::from(e)),
            HornedError::ValidityError(_, _) => return Err(PyRuntimeError::new_err(e.to_string())),
            HornedError::CommandError(_) | HornedError::ParserError(_, _) => unreachable!(),
        }
    }

    Ok(())
}

/// The Faultless AST for Open Biomedical Ontologies.
///
///
#[pymodule]
#[pyo3(name = "fastobo")]
pub fn init<'py>(py: Python<'py>, m: &Bound<'py, PyModule>) -> PyResult<()> {
    m.add("__package__", "fastobo")?;
    m.add("__build__", pyo3_built!(py, built))?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add("__author__", env!("CARGO_PKG_AUTHORS").replace(':', "\n"))?;

    add_submodule!(py, m, abc);
    add_submodule!(py, m, doc);
    add_submodule!(py, m, exceptions);
    add_submodule!(py, m, header);
    add_submodule!(py, m, id);
    add_submodule!(py, m, instance);
    add_submodule!(py, m, pv);
    add_submodule!(py, m, syn);
    add_submodule!(py, m, term);
    add_submodule!(py, m, typedef);
    add_submodule!(py, m, xref);

    m.add_function(wrap_pyfunction!(self::iter, m)?)?;
    m.add_function(wrap_pyfunction!(self::load, m)?)?;
    m.add_function(wrap_pyfunction!(self::loads, m)?)?;
    m.add_function(wrap_pyfunction!(self::load_graph, m)?)?;
    m.add_function(wrap_pyfunction!(self::dump_graph, m)?)?;
    m.add_function(wrap_pyfunction!(self::dump_owl, m)?)?;

    Ok(())
}
