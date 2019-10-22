# `fastobo-py` [![Star me](https://img.shields.io/github/stars/fastobo/fastobo-py.svg?style=social&label=Star&maxAge=3600)](https://github.com/fastobo/fastobo-py/stargazers)

*Faultless AST for Open Biomedical Ontologies in Python.*

[![TravisCI](https://img.shields.io/travis/fastobo/fastobo-py/master.svg?logo=travis&maxAge=600&style=flat-square)](https://travis-ci.org/fastobo/fastobo-py/branches)
[![AppVeyor](https://img.shields.io/appveyor/ci/althonos/fastobo-py/master?logo=appveyor&style=flat-square&maxAge=600)](https://ci.appveyor.com/project/althonos/fastobo-py)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square&maxAge=2678400)](https://choosealicense.com/licenses/mit/)
[![Source](https://img.shields.io/badge/source-GitHub-303030.svg?maxAge=2678400&style=flat-square)](https://github.com/fastobo/fastobo-py/)
[![PyPI](https://img.shields.io/pypi/v/fastobo.svg?style=flat-square&maxAge=600)](https://pypi.org/project/fastobo)
[![Wheel](https://img.shields.io/pypi/wheel/fastobo.svg?style=flat-square&maxAge=2678400)](https://pypi.org/project/fastobo/#files)
[![Bioconda](https://img.shields.io/conda/vn/bioconda/fastobo?style=flat-square&maxAge=3600)](https://anaconda.org/bioconda/fastobo)
[![Python Versions](https://img.shields.io/pypi/pyversions/fastobo.svg?style=flat-square&maxAge=600)](https://pypi.org/project/fastobo/#files)
[![PyPI - Implementation](https://img.shields.io/pypi/implementation/fastobo.svg?style=flat-square&maxAge=600)](https://pypi.org/project/fastobo/#files)
[![Changelog](https://img.shields.io/badge/keep%20a-changelog-8A0707.svg?maxAge=2678400&style=flat-square)](https://github.com/fastobo/fastobo-py/blob/master/CHANGELOG.md)
[![Documentation](https://img.shields.io/readthedocs/fastobo.svg?maxAge=3600&style=flat-square)](https://fastobo.readthedocs.io/)
[![GitHub issues](https://img.shields.io/github/issues/fastobo/fastobo-py.svg?style=flat-square&maxAge=600)](https://github.com/fastobo/fastobo-py/issues)
[![DOI](https://img.shields.io/badge/doi-10.7490%2Ff1000research.1117405.1-brightgreen?style=flat-square&maxAge=31536000)](https://f1000research.com/posters/8-1500)
[![Downloads](https://img.shields.io/badge/dynamic/json?style=flat-square&color=303f9f&maxAge=86400&label=downloads&query=%24.total_downloads&url=https%3A%2F%2Fapi.pepy.tech%2Fapi%2Fprojects%2Ffastobo)](https://pepy.tech/project/fastobo)


## Overview

[`fastobo`](https://crates.io/crates/fastobo) is a Rust library implementing a
reliable parser for the OBO file format 1.4. This extension module exports
idiomatic Python bindings that can be used to load, edit and serialize ontologies
in the OBO format.


## Installation

If your platform has no pre-built binaries available, you will need to have the Rust
compiler installed. See the [documentation on `rust-lang.org`](https://forge.rust-lang.org/other-installation-methods.html)
to learn how to install Rust on your machine.

Installation is then supported through `pip`:
```console
$ pip install fastobo --user
```


## Usage

An `OboDoc` instance can be instantiated from a file-handle or from a binary file handle
using the `fastobo.load` function, or from a string using the `fastobo.loads` function.

```python
import fastobo
obodoc = fastobo.load("../data/ms.obo")
```

Loading from a `gzip` file is supported:
```python
import fastobo
import gzip
gzdoc = fastobo.load(gzip.open("../data/cl.obo.gz"))
```

*Comments can be parsed but neither edited nor serialized, because of a limitation
with `pyo3` (the library used to generate the Python bindings). They are supported
in the Rust version of `fastobo`.*

## Feedback

Found a bug ? Have an enhancement request ? Head over to the
[GitHub issue tracker](https://github.com/fastobo/fastobo-py/issues) of the project if
you need to report or ask something. If you are filling in on a bug, please include as much
information as you can about the issue, and try to recreate the same bug in a simple, easily
reproducible situation.


## About

This project was developed by [Martin Larralde](https://github.com/althonos)
as part of a Master's Degree internship in the [BBOP team](http://berkeleybop.org/) of the
[Lawrence Berkeley National Laboratory](https://www.lbl.gov/), under the supervision of
[Chris Mungall](http://biosciences.lbl.gov/profiles/chris-mungall/). Cite this project as:

*Larralde M.* **Developing Python and Rust libraries to improve the ontology ecosystem**
*\[version 1; not peer reviewed\].* F1000Research 2019, 8(ISCB Comm J):1500 (poster)
([https://doi.org/10.7490/f1000research.1117405.1](https://doi.org/10.7490/f1000research.1117405.1))
