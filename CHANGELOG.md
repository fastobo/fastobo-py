# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).


## [Unreleased]

[Unreleased]: https://github.com/fastobo/fastobo-py/compare/v0.2.1...HEAD


## [v0.2.1] - 2019-07-23

[v0.2.1]: https://github.com/fastobo/fastobo/compare/v0.2.0...v0.2.1

### Changed
- Updated to `fastobo` [v0.6.0](https://github.com/fastobo/fastobo/blob/master/CHANGELOG.md#v060-2019-07-23).

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
