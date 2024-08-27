# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

* Add `AllowedViewers` to `LogVisibility` enum.

### Changed

- BREAKING: Add the `LoadSnapshot` variant to `CanisterChangeDetails`. (#504)

### Added

- Support Canister State Snapshots. (#504)
  - Add methods: `take_canister_snapshot`, `load_canister_snapshot`, `list_canister_snapshots`, `delete_canister_snapshot`
  - Add types: `LoadSnapshotRecord`, `SnapshotId`, `Snapshot`, `TakeCanisterSnapshotArgs`, `LoadCanisterSnapshotArgs`, `DeleteCanisterSnapshotArgs`

## [0.15.0] - 2024-07-01

### Changed

- BREAKING: Stable Memory always use 64-bit addresses and `stable64_*` system API. (#498)
- BREAKING: Add `log_visibility` to the management canister API types: (#497)
   - `CanisterSettings`
   - `DefiniteCanisterSettings`.

## [0.14.0] - 2024-05-17
## [0.13.3] - 2024-05-10 (yanked)

### Added

- Provide safe wrapper of `in_replicated_execution` in ic-cdk. (#489)

### Changed

- Upgrade `ic0` to v0.23.0. (#489)
- BREAKING: Add `wasm_memory_limit` to the management canister API types: (#483)
   - `CanisterSettings`
   - `DefiniteCanisterSettings`.

## [0.13.2] - 2024-04-08

### Added

- Management canister methods for interacting with the chunk store. (#461)
- Provide safe wrapper of `global_timer_set` in ic-cdk. (#475)

## [0.13.1] - 2024-03-01

### Changed

- Upgrade `ic-cdk-macros` to v0.9.0.

## [0.13.0] - 2024-03-01 (yanked)

### Added

- Add `is_recovering_from_trap` function for implementing trap cleanup logic. (#456)
- Allow setting decoding quota for canister endpoints and inter-canister calls. (#465)
  * When defining canister endpoints, we add the following attributes: `#[update(decoding_quota = 10000, skipping_quota = 100, debug = true)]`
    - `skipping_quota` limits the amount of work allowed for skipping unneeded data on the wire. If this attributes is not present, we set a default quota of `10_000`. This affects ALL existing canisters, and is mainly used to improve canister throughput. See [docs on the Candid library](https://docs.rs/candid/latest/candid/de/struct.DecoderConfig.html#method.set_skipping_quota) to understand the skipping cost.
    - `decoding_quota` limits the total amount of work the deserializer can perform. See [docs on the Candid library](https://docs.rs/candid/latest/candid/de/struct.DecoderConfig.html#method.set_decoding_quota) to understand the cost model.
    - `debug = true` prints the instruction count and the decoding/skipping cost to the replica log, after a successful deserialization. The decoding/skipping cost is logged only when you have already set a quota in the attributes. The debug mode is useful to determine the right quotas above. Developers can send a few large payloads to the debugging endpoint and know the actual decoding cost.
  * When making inter-canister calls, we have a new function `call_with_config` to config the same decoding quotas described above. It's strongly recommended to use `call_with_config` when calling third-party untrusted canisters.

### Changed

- `ic_cdk::api::call::arg_data` takes `ArgDecoderConfig` as argument. (#465)

## [0.12.1] - 2024-01-12

### Changed

- Add "reserved cycles" fields to the management canister API: (#449)
  - `reserved_cycles` to `CanisterStatusResponse`
  - `reserved_cycles_limit` to `CanisterSettings` and `DefiniteCanisterSettings`

### Fixed

- The README file is now more informative and used as the front page of the doc site.
- The `call*` methods are documented with examples and notes.

## [0.12.0] - 2023-11-23

### Changed

- Upgrade `candid` to `0.10`. (#448)

## [0.11.4] - 2023-11-20

### Added

- `query_stats` in `canister_status` response. (#432)
  
## [0.11.3] - 2023-10-12

### Added

- Another type of performance counter: "call context instruction counter".
  Can be fetched using either method below: (#435)
  - `ic_cdk::api::performance_counter(1)`;
  - `ic_cdk::api::call_context_instruction_counter()` as a shorthand;

### Changed

- Deprecate `ic_cdk::api::call::performance_counter()` in favor of `ic_cdk::api::performance_counter()`. (#435)

## [0.11.2] - 2023-10-11

### Added

- `cycles_burn` corresponding to system API `ic0.cycles_burn128`. (#434)

### Changed

- Upgrade `ic0` to `0.21.1`. (#434)

## [0.11.1] - 2023-10-11

### Changed

- Upgrade `ic0` to `0.21.0`. (#433)

## [0.11.0] - 2023-09-18

### Changed

- Candid Export workflow is changed. (#424)
  * No need to compile for WASI separately.
  * Canisters should still invoke `ic_cdk::export_candid!()` to export candid.
  * Then use [`candid-extractor`](../candid-extractor/) to extract candid from the canister WASM.

## [0.10.0] - 2023-07-13

### Changed

- Upgrade `candid` to `0.9`. (#411)
- Remove `export` module. Please use candid directly in your project instead of using `ic_cdk::export::candid`.
- Remove `ic_cdk_macro::import` module. See below for a new way to import canisters.

### Added

- Export Candid: (#386)
  * A wasi feature that builds the canister as a standalone WASI binary. Running the binary in wasmtime outputs the canister interface
  * Build step:
  ```
  cargo build --target wasm32-unknown-unknown \
      --release \
      --package "$package" --features "ic-cdk/wasi"

  wasmtime "target/wasm32-unknown-unknown/release/$package.wasm" > $did_file

  cargo build --target wasm32-unknown-unknown \
      --release \
      --package "$package"

  ic-wasm "target/wasm32-unknown-unknown/release/$package.wasm" \
      -o "target/wasm32-unknown-unknown/release/$package.wasm" \
      metadata candid:service -v public -f $did_file
  ```
  * In the canister code, users have to add `ic_cdk::export_candid!()` at the end of `lib.rs`. In the future we may lift this requirement to provide a better DX.

- Import Candid: (#390)

  * Canister project adds `ic_cdk_bindgen` as a build dependency to generate canister bindings
  * build.rs
  ```
  use ic_cdk_bindgen::{Builder, Config};
  fn main() {
      let counter = Config::new("counter");
      let mut builder = Builder::new();
      builder.add(counter);
      builder.build(None);  // default write to src/declarations
  }
  ```
  * In the canister code,
  ```
  mod declarations;
  use declarations::counter::counter;

  counter.inc().await?
  ```

## [0.9.2] - 2023-06-22

### Changed

- Hardcodes the fee for `sign_with_ecdsa`. (#407)

## [0.9.1] - 2023-06-21 (yanked)

### Changed

- Bitcoin API handles cycles cost under the hood. (#406)

## [0.9.0] - 2023-06-20 (yanked)

### Added

- Set caller's canister version in the field `sender_canister_version` of management canister call payloads. (#401)
- Add management canister types for `canister_info` management canister call (`CanisterInfoRequest` and `CanisterInfoResponse`). (#401)

### Changed

- No hard-coded fees for management canister calls. (#404)

## [0.8.0] - 2023-05-26

### Added

- `ic0.is_controller` as a public function. (#383)

### Changed

- `TransformContext::new` has been replaced with dedicated functions that accept closures. (#385)
- `CallFuture` only makes an inter-canister call if it is awaited. (#391)

## [0.7.4] - 2023-03-21

### Added

- `WASM_PAGE_SIZE_IN_BYTES` made `pub`. (#380)
- `http_request_with_cycles`. (#381)

## [0.7.3] - 2023-03-01

### Fixed

- Addressed a compatibility error in the signature of the `call` family of functions. (#379)

## [0.7.2] - 2023-03-01

### Fixed

- Fix type name in error message when a deserialization error occurs after making a canister-to-canister call. (#355)

## [0.7.1] - 2023-02-22

### Fixed

- Update document for http_request. (#372)

## [0.7.0] - 2023-02-03

### Changed

- The timers API is not a feature anymore, it moved into a separate library, `ic-cdk-timers`. (#368)

## [0.6.10] - 2023-01-20

### Added

- Added `ic0.canister_version` as a public function. (#350)

## [0.6.9] - 2023-01-18

### Fixed

- Allow timers to cancel themselves. (#360)

### Refactored

- Change from pleco to tanton for the chess library in the chess example. (#345)
- Refactor the executor to prevent a double-free on `join_all`. (#357)

## [0.6.8] - 2022-11-28

### Added

- Added composite queries via `#[query(composite = true)]`. (#344)

  Composite queries cannot be run as update calls, but can make inter-canister calls to other query functions.

- Implemented the canister timers API, located in module `ic_cdk::timer`. (#342)

## [0.6.7] - 2022-11-16

### Changed

- Improve error message on trap while decoding arguments. (#339)

## [0.6.6] - 2022-11-09

### Added

- Added `StableIO` to implement both `io::Write` and `io::Read` for stable memory. (#335)
- Added 64-bit support for `io::Write` and `io::Read` via `StableIO`.
- Implement `io::Seek` for stable storage.

### Changed

-  `StableWriter` and `StableReader` are now wrappers around `StableIO`.

## [0.6.5] - 2022-11-04

### Changed

BREAKING CHANGE of experimental API:
- `http_request` to support `context` field in callback function. (#326)

## [0.6.4] - 2022-10-28

### Added

- Expose `offset` of `StableReader` and `StableWriter`. (#330)

## [0.6.3] - 2022-10-26

### Fixed

- Doc can build on docs.rs. (#327)

## [0.6.2] - 2022-10-24

### Refactored

- Separate `ic0` crate for system API. (#324)

## [0.6.1] - 2022-10-14

### Added

- `create_canister_with_extra_cycles` to specify cycles when create canister (#322)

### Fixed

- `create_canister` should charge 0.1T cycles (#322)

## [0.6.0] - 2022-10-03

### Changed

- Upgrade `candid` to `0.8.0` (#321)

## [0.5.7] - 2022-09-27

### Fixed
- Overhaul management canister, especially `transform` type in `http_request`  (#312)

## [0.5.6] - 2022-08-10

### Added
- New `ic_cdk::api::management_canister` module for calling the IC management canister (#295)
- Derive common traits for `RejectionCode` (#294)
- `ManualReply::reject` function (#297)

### Fixed
- Failure to decode the reply in `ic_cdk::call` does not trap anymore (#301)

## [0.5.5] - 2022-07-22

### Added
- Derive `CandidType` and `Deserialize` for `RejectionCode` (#291, #293)

## [0.5.3] - 2022-07-19

### Added
- `instruction_counter` function as a shorthand for `performance_counter(0)` (#283)

### Changed
- Make `CanisterStableMemory` public (#281)
- BREAKING CHANGE: move performance_counter from the `ic_cdk::api::call` to `ic_cdk::api` module (#283)

### Fixed
- Outdated documentation for `ManualReply` (#286)

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
