# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

### Added

- Types for IC management canister method `fetch_canister_logs`.

### Fixed

- Doc: `HttpRequestArgs::max_response_bytes` is capped at 2MB, not 2MiB.

## [0.2.0] - 2025-02-18

### Changed

- Added `aux` field in `SignWithSchnorrArgs`, introducing `SchnorrAux` and `Bip341` types.
- Fixed `NodeMetrics` which should have a field `num_block_failures_total`, not `num_blocks_failures_total`.

## [0.1.0] - 2023-01-22

### Added

- Initial release of the `ic-management-canister-types` library.
