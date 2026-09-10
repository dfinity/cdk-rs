# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

### Added

- `HttpRequest`, a builder for `http_request`. It always selects pricing version `2` ("pay-as-you-go"), which charges for the resources the outcall consumes rather than for `max_response_bytes`.
- `FlexibleHttpRequest`, a builder for the new `flexible_http_request` method, in which a committee of nodes return their individual HTTP responses instead of the subnet reaching consensus on one.
- `with_expected_roundtrip_time_ms`, `with_expected_raw_response_bytes`, `with_expected_transformed_response_bytes` and `with_expected_transform_instructions` on both builders. Under pricing version `2` the attached cycles are also the budget each node may spend, so these narrow the reservation from "the most the outcall could consume" to what the caller expects. Anything left unset falls back to the maximum, which yields a reservation the outcall cannot exhaust but which holds far more cycles for the duration of the call.
- `with_transform_closure` on both builders, replacing the free function `http_request_with_closure` and extending closure transforms to flexible outcalls.
- `cost_http_request_v2` and its argument types `CostHttpRequestV2Args` and `HttpOutcallType`.
- Re-exports of `SnapshotVisibility` and `StatusVisibility`, the types of the `CanisterSettings` and `DefiniteCanisterSettings` fields of the same name, and of `RenameCanisterRecord` and `RenameToRecord`, the payload of `ChangeDetails::RenameCanister`. All four were reachable only by depending on `ic-management-canister-types` directly, which left those fields impossible to construct or match on.
- Re-exports of the new `ic-management-canister-types` items: `FlexibleHttpRequestArgs`, `FlexibleHttpRequestResult`, `FlexibleHttpRequestErr`, `FlexibleHttpGlobalError`, `FlexibleHttpNodeDetail`, `FlexibleHttpNodeError`, `HttpRequestResourceReport`, `ReplicationCounts` and `ResourceUsage`.

### Removed

- The free functions `http_request`, `cost_http_request` and `http_request_with_closure`. Use `HttpRequest` instead, which prices the outcall with version `2`. Migrating deliberately rather than switching the pricing version underneath an unchanged call is the reason this is a breaking change rather than a silent one.
  - `HttpRequest::from_args` accepts an existing `HttpRequestArgs`, so an existing call site can be migrated without rewriting how it builds its arguments.
  - `ic_cdk::api::cost_http_request` still exposes version `1` pricing for callers that need it.

### Changed

- `ic-management-canister-types` bumped from `0.7.1` to `0.10`, which adds the `pricing_version` field to `HttpRequestArgs`, the `flexible_http_request` types, and three fields to `CanisterSettings`.

## [0.1.1] - 2026-03-10

### Fixed

- Fixed docs.rs build by adding `#![cfg_attr(docsrs, feature(doc_cfg))]`.

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
