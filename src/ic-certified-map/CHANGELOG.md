# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Added
- `RbTree::value_range()` method to get a witness for a range of keys with values.
- `RbTree::iter()` method.
- impls of `Clone`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `FromIterator`, and `Debug` for `RbTree`.

### Changed
- RbTree::key_range() method returns tighter key bounds which reduces the size of witnesses.
- Updated the version of candid from `0.6.19` to `0.7.1` ([#72](https://github.com/dfinity/cdk-rs/pull/72)).
- Hash tree leaves can now hold both references and values ([#121](https://github.com/dfinity/cdk-rs/issues/121)).
  This is a BREAKING CHANGE, some clients might need to slightly change and recompile their code.

## [0.1.0] - 2021-05-04
### Added
* Initial release of the library.
