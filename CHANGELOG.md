# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## [Unreleased]

[Unreleased]: https://github.com/fastobo/fastobo-py/compare/v0.7.1...HEAD


## [v0.7.1] - 2020-02-11

[v0.7.1]: https://github.com/fastobo/fastobo-py/compare/v0.7.0...v0.7.1

### Changed
- Bumped `fastobo` to `v0.8.2` to fix `Display` implementation of 
  `HeaderClause::Unreserved` variant.
- Bumped `built` build-dependency to `v0.4.0`.


## [v0.7.0] - 2020-01-24

[v0.7.0]: https://github.com/fastobo/fastobo-py/compare/v0.6.2...v0.7.0

### Changed
- Bumped `fastobo` to `v0.8.1` to use multi-threaded parser implementation.
- Added `ordered` keyword argument to top-level `fastobo` function to disable
  requirement to parse the document in order-preserving mode.


## [v0.6.2] - 2020-01-18

[v0.6.2]: https://github.com/fastobo/fastobo-py/compare/v0.6.1...v0.6.2

### Added
- Compilation of Python 3.8, PyPy 3.5 and PyPy 3.6 wheels for OSX.

### Fixed
- Bumped `fastobo` to `v0.7.5`, which should finally support Windows
  style line-endings.


## [v0.6.1] - 2019-11-19

[v0.6.1]: https://github.com/fastobo/fastobo-py/compare/v0.6.0...v0.6.1

### Added
- Compilation of Python 3.8 wheels for Linux and Windows
  ([#67](https://github.com/fastobo/fastobo-py/issues/67)).

### Fixed
- `BaseTypedefClause` not being declared in `fastobo.typdef` submodule.
- `fastobo.id` module performing unneeded string allocation.


## [v0.6.0] - 2019-10-08

[v0.6.0]: https://github.com/fastobo/fastobo-py/compare/v0.5.4...v0.6.0

### Added
- `__init__` to all classes of `fastobo.typedef`.
### Changed
- Renamed `term` field of `fastobo.typedef.Relationship` to `target`.


## [v0.5.4] - 2019-10-08

[v0.5.4]: https://github.com/fastobo/fastobo-py/compare/v0.5.3...v0.5.4

### Added
- `__init__` implementation for `fastobo.doc.OboDoc`.
- `__init__` implementation for `fastobo.term.ConsiderClause`.
- `__init__` implementation for `fastobo.term.IsObsoleteClause`.
- Add constructor signatures to classes in `fastobo.term` and `fastobo.header`.

### Changed
- Bumped `pyo3` dependency to `v0.8.1`.

### Fixed
- Automatic generation of `pyproject.toml` in `sdist`.


## [v0.5.3] - 2019-10-06

[v0.5.3]: https://github.com/fastobo/fastobo-py/compare/v0.5.2...v0.5.3

### Added
- `__init__` and `date` getter to `CreationDateClause` in both `fastobo.term`
  and `fastobo.typedef` with proper timezone support.


## [v0.5.2] - 2019-09-28

[v0.5.2]: https://github.com/fastobo/fastobo-py/compare/v0.5.1...v0.5.2

### Added
- Getters for `ReplacedByClause` in `fastobo.typedef`.

### Fixed
- `PyFileGILRead` (used in `fastobo.iter`) should now be thread safe.


## [v0.5.1] - 2019-09-19

[v0.5.1]: https://github.com/fastobo/fastobo-py/compare/v0.5.0...v0.5.1

### Added
- Getters for `ExpandExpressionToClause` and `ExpandAssertionToClause` in
 `fastobo.typedef`.

### Fixed
- Missing `NamespaceIdRuleClause` class is now a member of `fastobo.header`.


## [v0.5.0] - 2019-09-18

[v0.5.0]: https://github.com/fastobo/fastobo-py/compare/v0.4.2...v0.5.0

### Added
- `fastobo.iter` function to iterate over the entity frames of an OBO document.
- `From<std::io::Error>` impl for `Error`.

### Changed
- `OboDoc` cannot be subclassed anymore.


## [v0.4.2] - 2019-09-16

[v0.4.2]: https://github.com/fastobo/fastobo-py/compare/v0.4.1...v0.4.2

### Added
- Getters for `IsClassLevelClause` and `IsMetadataTag` in `fastobo.typedef`.
- Getters for `RelationshipClause` in `fastobo.term`.

### Changed
- Inconsistent naming for `PropertyValueClause` between `fastobo.header`
  and other submodules.

### Fixed
- Bug with `fastobo.header.ImportClause.reference` returning the whole clause
  serialization instead.


## [v0.4.1] - 2019-09-15

[v0.4.1]: https://github.com/fastobo/fastobo-py/compare/v0.4.0...v0.4.1

### Added
- Precompiled wheels for CPython 3.5, 3.6 and 3.7 on Windows x86-64.


## [v0.4.0] - 2019-09-14

[v0.4.0]: https://github.com/fastobo/fastobo-py/compare/v0.3.3...v0.4.0

### Added
- `__init__` implementation for `fastobo.term.DefClause`
- `__init__` implementation for `fastobo.syn.Synonym`
- Precompiled wheels for PyPy3 on OSX and Linux.

### Fixed
- `XrefList.__repr__` implementation entering infinite recursion.
- Derive macros generating weird error messages for some `TypeError`s.
- Enabled `extension-module` feature of `pyo3` to allow static linking to Python interpreter.
- Inconsistent error-chaining in Python causing issues with `try/except` blocks.


## [v0.3.3] - 2019-09-10

[v0.3.3]: https://github.com/fastobo/fastobo-py/compare/v0.3.2...v0.3.3

### Changed
- Use stable PyO3 release (`v0.8.0`).
- Add back `__build__` attribute to check build variables.


## [v0.3.2] - 2019-08-29

[v0.3.2]: https://github.com/fastobo/fastobo-py/compare/v0.3.1...v0.3.2

### Added
- Added BOSC 2019 poster reference to `README.md`.
- Added `__richcmp__` implementation to `fastobo.id.PrefixedIdent`.

### Changed
- Bumped Rust dependencies to latest PyO3 version.


## [v0.3.1] - 2019-08-08

[v0.3.1]: https://github.com/fastobo/fastobo-py/compare/v0.3.0...v0.3.1

### Fixed
- `PyFile.write` calling `write` with two arguments causing duck typing check
  to fail all the time in `dump_graph`.


## [v0.3.0] - 2019-08-08

[v0.3.0]: https://github.com/fastobo/fastobo-py/compare/v0.2.2...v0.3.0

### Added
- `load_graph` and `dump_graph` functions to read and write OBO JSON files.

### Changed
- Updated to `fastobo` [v0.7.1](https://github.com/fastobo/fastobo/blob/master/CHANGELOG.md#v071---2019-08-08).


## [v0.2.2] - 2019-07-24

[v0.2.2]: https://github.com/fastobo/fastobo-py/compare/v0.2.1...v0.2.2

### Changed
- Updated to `fastobo` [v0.6.1](https://github.com/fastobo/fastobo/blob/master/CHANGELOG.md#v061---2019-07-24).
- Updated to `url` v2.0.0.


## [v0.2.1] - 2019-07-23

[v0.2.1]: https://github.com/fastobo/fastobo-py/compare/v0.2.0...v0.2.1

### Changed
- Updated to `fastobo` [v0.6.0](https://github.com/fastobo/fastobo/blob/master/CHANGELOG.md#v060---2019-07-23).

### Fixed
- Parser now accepts ISO 8601 dates with fractional second.


## [v0.2.0] - 2019-07-22

[v0.2.0]: https://github.com/fastobo/fastobo-py/compare/v0.1.1...v0.2.0

### Changed
- Updated to `fastobo` [v0.5.0](https://github.com/fastobo/fastobo/blob/master/CHANGELOG.md#v050---2019-07-15).

### Added
- Support for OBO graph deserialization using `fastobo-graphs`.
- Limited support for instance frames in `fastobo.instance` module.
- Methods `OboDoc.compact_ids` and `OboDoc.decompact_ids` to create semantically
  equivalent OBO document with compacted/decompacted identifiers.


## [v0.1.1] - 2019-06-28

[v0.1.1]: https://github.com/fastobo/fastobo-py/compare/v0.1.0...v0.1.1

### Fixed
- PyPI release not being uploaded because of older development release.


## [v0.1.0] - 2019-06-28

[v0.1.0]: https://github.com/fastobo/fastobo-py/compare/77dd00c...v0.1.0

Initial release.
