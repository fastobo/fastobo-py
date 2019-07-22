# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## [Unreleased]

[Unreleased]: https://github.com/fastobo/fastobo-py/compare/v0.2.0...HEAD


## [0.2.0] - 2019-07-22

[0.2.0]: https://github.com/fastobo/fastobo/compare/v0.1.1...v0.2.0

### Changed
- Updated to `fastobo` [v0.5.0](https://github.com/fastobo/fastobo/blob/master/CHANGELOG.md).

### Added
- Support for OBO graph deserialization using `fastobo-graphs`.
- Limited support for instance frames in `fastobo.instance` module.
- Methods `OboDoc.compact_ids` and `OboDoc.decompact_ids` to create semantically
  equivalent OBO document with compacted/decompacted identifiers.


## [0.1.1] - 2019-06-28

[0.1.1]: https://github.com/fastobo/fastobo/compare/v0.1.0...v0.1.1

### Fixed
- PyPI release not being uploaded because of older development release.


## [0.1.0] - 2019-06-28

[0.1.0]: https://github.com/fastobo/fastobo/compare/77dd00c...v0.1.0

Initial release.
