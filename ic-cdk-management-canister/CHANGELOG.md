# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

## [0.1.0] - 2026-03-05

Initial release. The functionality was previously part of the `management_canister` module in `ic-cdk`.
Users of `ic-cdk 0.19.x` should migrate to this crate together with `ic-cdk 0.20.0`.

### Changed

- [BREAKING] Types are now provided by `ic-management-canister-types 0.7.1`.
  Compared to `ic-cdk 0.19.0`, the following additions and breaking changes apply:

  - Added `log_memory_limit` field to `CanisterSettings` and `DefiniteCanisterSettings`.
  - Added `filter` field to `FetchCanisterLogsArgs`.
    - `FetchCanisterLogsArgs` is now a struct instead of a type alias for `CanisterIdRecord`.
    - Added the type `CanisterLogFilter`.
  - Added `log_memory_store_size` field to `MemoryMetrics`.
  - Added `uninstall_code` and `sender_canister_version` fields to `TakeCanisterSnapshotArgs`.
  - Added `rename_canister` variant to `ChangeDetails` with types `RenameCanisterRecord` and `RenameToRecord`.

- The `transform-closure` feature has been moved from `ic-cdk` to this crate.
