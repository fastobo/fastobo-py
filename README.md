# `fastobo-py` [![Star me](https://img.shields.io/github/stars/althonos/fastobo.svg?style=social&label=Star&maxAge=3600)](https://github.com/althonos/fastobo/stargazers)

*Faultless AST for Open Biomedical Ontologies in Python.*

[![TravisCI](https://img.shields.io/travis/althonos/fastobo/master.svg?maxAge=600&style=flat-square)](https://travis-ci.org/althonos/fastobo/branches)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square&maxAge=2678400)](https://choosealicense.com/licenses/mit/)
[![Source](https://img.shields.io/badge/source-GitHub-303030.svg?maxAge=2678400&style=flat-square)](https://github.com/althonos/fastobo/)
[![PyPI](https://img.shields.io/pypi/v/fastobo.svg?style=flat-square&maxAge=600)](https://pypi.org/project/fastobo)
[![Wheel](https://img.shields.io/pypi/wheel/fastobo.svg?style=flat-square&maxAge=2678400)](https://pypi.org/project/fastobo/#files)
[![Python Versions](https://img.shields.io/pypi/pyversions/fastobo.svg?style=flat-square&maxAge=600)](https://pypi.org/project/fastobo/#files)
[![PyPI - Implementation](https://img.shields.io/pypi/implementation/fastobo.svg?style=flat-square&maxAge=600)](https://pypi.org/project/fastobo/#files)
[![Changelog](https://img.shields.io/badge/keep%20a-changelog-8A0707.svg?maxAge=2678400&style=flat-square)](https://github.com/althonos/fastobo/blob/master/fastobo-py/CHANGELOG.md)
[![Documentation](https://img.shields.io/readthedocs/fastobo.svg?maxAge=3600&style=flat-square)](https://fastobo.readthedocs.io/)
[![GitHub issues](https://img.shields.io/github/issues/althonos/fastobo.svg?style=flat-square&maxAge=600)](https://github.com/althonos/fastobo/issues)

## Overview

`fastobo` is a Rust library implementing a reliable parser for the OBO file format 1.4.
This crate exports idiomatic Python bindings that can be used to load, edit and serialize
ontologies in the OBO format.


## Installation

If your platform has no pre-built binaries available, you will need to have the Rust
compiler installed. See the [documentation on `rust-lang.org`](https://forge.rust-lang.org/other-installation-methods.html) to learn how to install Rust on your machine.

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

*Comments are not supported currently, because of a limitation with `pyo3` (the library used to
generate the Python bindings). They are partially supported in the Rust version of `fastobo`.*


## Feedback

Found a bug ? Have an enhancement request ? Head over to the
[GitHub issue tracker](https://github.com/althonos/fastobo/issues) of the project if
you need to report or ask something. If you are filling in on a bug, please include as much
information as you can about the issue, and try to recreate the same bug in a simple, easily
reproducible situation.


## About

This project is currently being developed by [Martin Larralde](https://github.com/althonos)
as part of a Master's Degree internship in the [BBOP team](http://berkeleybop.org/) of the
[Lawrence Berkeley National Laboratory](https://www.lbl.gov/), under the supervision of
[Chris Mungall](http://biosciences.lbl.gov/profiles/chris-mungall/).
