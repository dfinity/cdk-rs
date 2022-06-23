# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

## [0.5.2] - 2022-06-23 
### Added
- `arg_data_raw_size` for checking the size of the arg-data-raw before copying to a vector or deserializing (#263)
- `performance_counter` for getting the value of specified performance counter (#277)

### Fixed
- Use explicitly type u8 in vector initialization (#264)
- Make `reply_raw` avoid writing empty replies
- Uses new format for candid environment variables in import macros. Requires DFX >=0.9.2 (#270)

## [0.5.1] - 2022-05-16
### Added
- `BufferedStableReader` for efficient reading from stable memory (#247)
- `BufferedStableWriter` for efficient writing to stable memory (#245)
- `reply_raw` and publish `arg_data_raw` for serialization-agnostic arguments fetching and replies (#256)
- Support for one-way calls (see `notify` and `notify_raw` functions) (#261)

### Fixed
- Panicking after `.await` does not leak resources anymore (#232, #250)

## [0.5.0] - 2022-03-29
### Added
- Update canister calling API for 128-bit cycles (#228)

### Changed
- Take slice rather than owned Vec as input arg (#217)
- Remove non-stable storage API (#215)
- Allow configuring export macros to not reply (#210)
- Add Clone and Copy to RejectionCode (#202)

### Fixed
- Do not call done() in stable_restore() (#216) 
- Remove out-of-bounds vulnerability (#208)
- Run inter-canister calls without awaiting (#233)

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
