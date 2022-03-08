# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- Take slice rather than owned Vec as input arg (#217)
- Remove non-stable storage API (#215)
- Allow configuring export macros to not reply (#210)
- Add Clone and Copy to RejectionCode (#202)

### Fixed
- Do not call done() in stable_restore() (#216) 
- Remove out-of-bounds vulnerability (#208)

## [0.4.0] - 2022-01-26
### Changed
- `candid` is required to be included in `[dependencies]` to use the `#[import]` macro  (#190) 
- Deprecate block_on in favour of the new spawn function (#189) 
- Trap in setup panic hook (#172)

## [0.3.3] - 2021-11-17
### Added
- Update system API for 128 bit cycles (#167)

## [0.3.2] - 2021-09-16
### Added
- Add support for 64 bit stable memory (#137) 
- Add support for 'heartbeat' and 'inspect_message' (#129)
