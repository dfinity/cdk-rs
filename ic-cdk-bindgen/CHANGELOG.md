# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

- Support canister setting `log_visibility`.

### Changed

- Refactor!: move Rust code generation logic from candid_parser. (#480)

### Fixed

- Re-generate bindings if the canister ids changed (e.g. when switching networks) or when the path to the candid file of a dependency changed. (#479)

## [0.1.3] - 2024-02-27

### Added

- Resolve CANISTER_CANDID_PATH and CANISTER_ID from standardized environment variables (uppercase canister names). (#467)
  - The support for legacy (non-uppercase) env vars is kept.
  - It will be removed in next major release (v0.2).

## [0.1.2] - 2023-11-23

### Changed

- Change `candid` dependency to the new `candid_parser` library. (#448)
  More details here: https://github.com/dfinity/candid/blob/master/Changelog.md#2023-11-16-rust-0100

## [0.1.1] - 2023-09-18

### Changed

- Update `candid` dependency to 0.9.6 which change the Rust bindings. (#424)

## [0.1.0] - 2023-07-13

### Added

- First release. (#416)
