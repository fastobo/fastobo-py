# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## [Unreleased]

[Unreleased]: https://github.com/fastobo/fastobo-py/compare/v0.4.1...HEAD


## [v0.4.1] - 2019-09-15

[v0.4.1]: https://github.com/fastobo/fastobo/compare/v0.4.0...v0.4.1

### Added
- Precompiled wheels for CPython 3.5, 3.6 and 3.7 on Windows x86-64.


## [v0.4.0] - 2019-09-14

[v0.4.0]: https://github.com/fastobo/fastobo/compare/v0.3.3...v0.4.0

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

[v0.3.3]: https://github.com/fastobo/fastobo/compare/v0.3.2...v0.3.3

### Changed
- Use stable PyO3 release (`v0.8.0`).
- Add back `__build__` attribute to check build variables.


## [v0.3.2] - 2019-08-29

[v0.3.2]: https://github.com/fastobo/fastobo/compare/v0.3.1...v0.3.2

### Added
- Added BOSC 2019 poster reference to `README.md`.
- Added `__richcmp__` implementation to `fastobo.id.PrefixedIdent`.

### Changed
- Bumped Rust dependencies to latest PyO3 version.


## [v0.3.1] - 2019-08-08

[v0.3.1]: https://github.com/fastobo/fastobo/compare/v0.3.0...v0.3.1

### Fixed
- `PyFile.write` calling `write` with two arguments causing duck typing check
  to fail all the time in `dump_graph`.


## [v0.3.0] - 2019-08-08

[v0.3.0]: https://github.com/fastobo/fastobo/compare/v0.2.2...v0.3.0

### Added
- `load_graph` and `dump_graph` functions to read and write OBO JSON files.

### Changed
- Updated to `fastobo` [v0.7.1](https://github.com/fastobo/fastobo/blob/master/CHANGELOG.md#v071---2019-08-08).


## [v0.2.2] - 2019-07-24

[v0.2.2]: https://github.com/fastobo/fastobo/compare/v0.2.1...v0.2.2

### Changed
- Updated to `fastobo` [v0.6.1](https://github.com/fastobo/fastobo/blob/master/CHANGELOG.md#v061---2019-07-24).
- Updated to `url` v2.0.0.


## [v0.2.1] - 2019-07-23

[v0.2.1]: https://github.com/fastobo/fastobo/compare/v0.2.0...v0.2.1

### Changed
- Updated to `fastobo` [v0.6.0](https://github.com/fastobo/fastobo/blob/master/CHANGELOG.md#v060---2019-07-23).

### Fixed
- Parser now accepts ISO 8601 dates with fractional second.


## [v0.2.0] - 2019-07-22

[v0.2.0]: https://github.com/fastobo/fastobo/compare/v0.1.1...v0.2.0

### Changed
- Updated to `fastobo` [v0.5.0](https://github.com/fastobo/fastobo/blob/master/CHANGELOG.md#v050---2019-07-15).

### Added
- Support for OBO graph deserialization using `fastobo-graphs`.
- Limited support for instance frames in `fastobo.instance` module.
- Methods `OboDoc.compact_ids` and `OboDoc.decompact_ids` to create semantically
  equivalent OBO document with compacted/decompacted identifiers.


## [v0.1.1] - 2019-06-28

[v0.1.1]: https://github.com/fastobo/fastobo/compare/v0.1.0...v0.1.1

### Fixed
- PyPI release not being uploaded because of older development release.


## [v0.1.0] - 2019-06-28

[v0.1.0]: https://github.com/fastobo/fastobo/compare/77dd00c...v0.1.0

Initial release.
