//! Functions and types for calling [the IC management canister][1].
//!
//! This module is a direct translation from its Candid interface description.
//!
//! The functions and types defined in this module serves these purposes:
//! * Make it easy to construct correct request data.
//! * Handle the response ergonomically.
//! * For those calls require cycles payments, the cycles amount is an explicit argument.
//!
//! [1]: https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-management-canister

use crate::api::canister_version;
use crate::call::{Call, CallResult, ConfigurableCall, SendableCall};
use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

/// Canister ID.
pub type CanisterId = Principal;

/// WASM module.
pub type WasmModule = Vec<u8>;

/// Snapshot ID.
pub type SnapshotId = Vec<u8>;

/// Chunk hash.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct ChunkHash {
    /// The hash of an uploaded chunk
    #[serde(with = "serde_bytes")]
    pub hash: Vec<u8>,
}

/// Log Visibility.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum LogVisibility {
    /// Controllers.
    #[default]
    Controllers,
    /// Public.
    Public,
    /// Allowed viewers.
    AllowedViewers(Vec<Principal>),
}

/// Canister settings.
///
/// The settings are optional. If they are not explicitly set, the default values will be applied automatically.
///
/// See [`settings`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct CanisterSettings {
    /// A list of at most 10 principals.
    ///
    /// The principals in this list become the *controllers* of the canister.
    ///
    /// Default value: A list containing only the caller of the create_canister call.
    pub controllers: Option<Vec<Principal>>,
    /// Must be a number between 0 and 100, inclusively.
    ///
    /// It indicates how much compute power should be guaranteed to this canister,
    /// expressed as a percentage of the maximum compute power that a single canister can allocate.
    ///
    /// If the IC cannot provide the requested allocation,
    /// for example because it is oversubscribed, the call will be **rejected**.
    ///
    /// Default value: 0
    pub compute_allocation: Option<Nat>,
    /// Must be a number between 0 and 2<sup>48</sup> (i.e 256TB), inclusively.
    ///
    /// It indicates how much memory the canister is allowed to use in total.
    ///
    /// If the IC cannot provide the requested allocation,
    /// for example because it is oversubscribed, the call will be **rejected**.
    ///
    /// If set to 0, then memory growth of the canister will be best-effort and subject to the available memory on the IC.
    ///
    /// Default value: 0
    pub memory_allocation: Option<Nat>,
    /// Must be a number between 0 and 2<sup>64</sup>-1, inclusively.
    ///
    /// It indicates a length of time in seconds.
    ///
    /// Default value: 2592000 (approximately 30 days).
    pub freezing_threshold: Option<Nat>,
    /// Must be a number between 0 and 2<sup>128</sup>-1, inclusively.
    ///
    /// It indicates the upper limit on `reserved_cycles` of the canister.
    ///
    /// Default value: 5_000_000_000_000 (5 trillion cycles).
    pub reserved_cycles_limit: Option<Nat>,
    /// Defines who is allowed to read the canister's logs.
    ///
    /// Default value: Controllers
    pub log_visibility: Option<LogVisibility>,
    /// Must be a number between 0 and 2<sup>48</sup>-1 (i.e 256TB), inclusively.
    ///
    /// It indicates the upper limit on the WASM heap memory consumption of the canister.
    ///
    /// Default value: 3_221_225_472 (3 GiB).
    pub wasm_memory_limit: Option<Nat>,
}

/// Like [CanisterSettings].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct DefiniteCanisterSettings {
    /// Controllers of the canister.
    pub controllers: Vec<Principal>,
    /// Compute allocation.
    pub compute_allocation: Nat,
    /// Memory allocation.
    pub memory_allocation: Nat,
    /// Freezing threshold.
    pub freezing_threshold: Nat,
    /// Reserved cycles limit.
    pub reserved_cycles_limit: Nat,
    /// Visibility of canister logs.
    pub log_visibility: LogVisibility,
    /// The Wasm memory limit.
    pub wasm_memory_limit: Nat,
}

// create_canister ------------------------------------------------------------

/// Register a new canister and get its canister id.
///
/// See [IC method `create_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister).
///
/// This call requires cycles payment. The required cycles varies according to the subnet size (number of nodes).
/// Check [Gas and cycles cost](https://internetcomputer.org/docs/current/developer-docs/gas-cost) for more details.
pub async fn create_canister(
    arg: CreateCanisterArgsReduced,
    cycles: u128,
) -> CallResult<CreateCanisterResult> {
    let complete_arg = CreateCanisterArgs {
        settings: arg.settings,
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "create_canister")
        .with_args((complete_arg,))
        .with_guaranteed_response()
        .with_cycles(cycles)
        .call::<(CreateCanisterResult,)>()
        .await
        .map(|result| result.0)
}

/// Argument type of [create_canister].
///
/// Please note that this type is a reduced version of [CreateCanisterArgs].
/// The `sender_canister_version` field is removed as it is set automatically in [create_canister].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct CreateCanisterArgsReduced {
    /// See [CanisterSettings].
    pub settings: Option<CanisterSettings>,
}

/// Complete argument type of `create_canister`.
///
/// Please note that this type is not used directly as the argument of [create_canister].
/// The function [create_canister] takes [CreateCanisterArgsReduced] instead.
///
/// If you want to manually call `create_canister` (construct and invoke a [Call]), you should use this complete type.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct CreateCanisterArgs {
    /// See [CanisterSettings].
    pub settings: Option<CanisterSettings>,
    /// sender_canister_version must be set to ic_cdk::api::canister_version()
    pub sender_canister_version: Option<u64>,
}

/// Result type of [create_canister].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy,
)]
pub struct CreateCanisterResult {
    /// Canister ID.
    pub canister_id: CanisterId,
}

// create_canister END --------------------------------------------------------

// update_settings ------------------------------------------------------------

/// Update the settings of a canister.
///
/// See [IC method `update_settings`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-update_settings).
pub async fn update_settings(arg: UpdateSettingsArgsReduced) -> CallResult<()> {
    let complete_arg = UpdateSettingsArgs {
        canister_id: arg.canister_id,
        settings: arg.settings,
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "update_settings")
        .with_args((complete_arg,))
        .call()
        .await
}

/// Argument type of [update_settings]
///
/// Please note that this type is a reduced version of [UpdateSettingsArgs].
///
/// The `sender_canister_version` field is removed as it is set automatically in [update_settings].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct UpdateSettingsArgsReduced {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// See [CanisterSettings].
    pub settings: CanisterSettings,
}

/// Complete argument type of `update_settings`.
///
/// Please note that this type is not used directly as the argument of [update_settings].
/// The function [update_settings] takes [UpdateSettingsArgsReduced] instead.
///
/// If you want to manually call `update_settings` (construct and invoke a [Call]), you should use this complete type.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct UpdateSettingsArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// See [CanisterSettings].
    pub settings: CanisterSettings,
    /// sender_canister_version must be set to ic_cdk::api::canister_version()
    pub sender_canister_version: Option<u64>,
}

// update_settings END --------------------------------------------------------

// upload_chunk ---------------------------------------------------------------

/// Upload a chunk to the chunk store of a canister.
///
/// See [IC method `upload_chunk`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-upload_chunk).
pub async fn upload_chunk(arg: UploadChunkArgs) -> CallResult<UploadChunkResult> {
    Call::new(Principal::management_canister(), "upload_chunk")
        .with_args((arg,))
        .call::<(UploadChunkResult,)>()
        .await
        .map(|result| result.0)
}

/// Argument type of [upload_chunk].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct UploadChunkArgs {
    /// The canister whose chunk store the chunk will be uploaded to.
    pub canister_id: CanisterId,
    /// The chunk bytes (max size 1MB).
    #[serde(with = "serde_bytes")]
    pub chunk: Vec<u8>,
}

/// Result type of [upload_chunk].
pub type UploadChunkResult = ChunkHash;

// upload_chunk END -----------------------------------------------------------

// clear_chunk_store ----------------------------------------------------------

/// Clear the chunk store of a canister.
///
/// See [IC method `clear_chunk_store`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-clear_chunk_store).
pub async fn clear_chunk_store(arg: ClearChunkStoreArgs) -> CallResult<()> {
    Call::new(Principal::management_canister(), "clear_chunk_store")
        .with_args((arg,))
        .call()
        .await
}

/// Argument type of [clear_chunk_store].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct ClearChunkStoreArgs {
    /// The canister whose chunk store will be cleared.
    pub canister_id: CanisterId,
}

// clear_chunk_store END ------------------------------------------------------

// stored_chunks --------------------------------------------------------------

/// Get the hashes of all chunks stored in the chunk store of a canister.
///
/// See [IC method `stored_chunks`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-stored_chunks).
pub async fn stored_chunks(arg: StoredChunksArgs) -> CallResult<StoredChunksResult> {
    Call::new(Principal::management_canister(), "stored_chunks")
        .with_args((arg,))
        .call::<(StoredChunksResult,)>()
        .await
        .map(|result| result.0)
}

/// Argument type of [stored_chunks].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct StoredChunksArgs {
    /// The canister whose chunk store will be queried.
    pub canister_id: CanisterId,
}

/// Result type of [stored_chunks].
pub type StoredChunksResult = Vec<ChunkHash>;

// stored_chunks END ----------------------------------------------------------

// install_code ---------------------------------------------------------------

/// Install code into a canister.
///
/// See [IC method `install_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-install_code).
pub async fn install_code(arg: InstallCodeArgsReduced) -> CallResult<()> {
    let complete_arg = InstallCodeArgs {
        mode: arg.mode,
        canister_id: arg.canister_id,
        wasm_module: arg.wasm_module,
        arg: arg.arg,
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "install_code")
        .with_args((complete_arg,))
        .call()
        .await
}

/// The mode with which a canister is installed.
///
/// This second version of the mode allows someone to specify the
/// optional `SkipPreUpgrade` parameter in case of an upgrade
#[derive(
    CandidType,
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    Copy,
    Default,
)]
#[serde(rename_all = "snake_case")]
pub enum CanisterInstallMode {
    /// A fresh install of a new canister.
    #[default]
    Install,
    /// Reinstalling a canister that was already installed.
    Reinstall,
    /// Upgrade an existing canister.
    Upgrade(Option<UpgradeFlags>),
}

/// Flags for canister installation with [`CanisterInstallMode::Upgrade`].
#[derive(
    CandidType,
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    Copy,
    Default,
)]
pub struct UpgradeFlags {
    /// If set to `true`, the `pre_upgrade` step will be skipped during the canister upgrade.
    pub skip_pre_upgrade: Option<bool>,
    /// If set to `Keep`, the WASM heap memory will be preserved instead of cleared.
    pub wasm_memory_persistence: Option<WasmMemoryPersistence>,
}

/// Wasm memory persistence setting for [UpgradeFlags].
#[derive(
    CandidType,
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    Copy,
    Default,
)]
#[serde(rename_all = "snake_case")]
pub enum WasmMemoryPersistence {
    /// Preserve heap memory.
    Keep,
    /// Clear heap memory.
    #[default]
    Replace,
}

/// Argument type of [install_code].
///
/// Please note that this type is a reduced version of [InstallCodeArgs].
/// The `sender_canister_version` field is removed as it is set automatically in [install_code].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct InstallCodeArgsReduced {
    /// See [CanisterInstallMode].
    pub mode: CanisterInstallMode,
    /// Canister ID.
    pub canister_id: CanisterId,
    /// Code to be installed.
    pub wasm_module: WasmModule,
    /// The argument to be passed to `canister_init` or `canister_post_upgrade`.
    pub arg: Vec<u8>,
}

/// Complete argument type of `install_code`.
///
/// Please note that this type is not used directly as the argument of [install_code].
/// The function [install_code] takes [InstallCodeArgsReduced] instead.
///
/// If you want to manually call `install_code` (construct and invoke a [Call]), you should use this complete type.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct InstallCodeArgs {
    /// See [CanisterInstallMode].
    pub mode: CanisterInstallMode,
    /// Canister ID.
    pub canister_id: CanisterId,
    /// Code to be installed.
    pub wasm_module: WasmModule,
    /// The argument to be passed to `canister_init` or `canister_post_upgrade`.
    pub arg: Vec<u8>,
    /// sender_canister_version must be set to ic_cdk::api::canister_version()
    pub sender_canister_version: Option<u64>,
}

// install_code END -----------------------------------------------------------

// install_chunked_code -------------------------------------------------------

/// Install code into a canister where the code has previously been uploaded in chunks.
///
/// See [IC method `install_chunked_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-install_chunked_code).
pub async fn install_chunked_code(arg: InstallChunkedCodeArgsReduced) -> CallResult<()> {
    let complete_arg = InstallChunkedCodeArgs {
        mode: arg.mode,
        target_canister: arg.target_canister,
        store_canister: arg.store_canister,
        chunk_hashes_list: arg.chunk_hashes_list,
        wasm_module_hash: arg.wasm_module_hash,
        arg: arg.arg,
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "install_chunked_code")
        .with_args((complete_arg,))
        .call()
        .await
}

/// Argument type of [install_chunked_code].
///
/// Please note that this type is a reduced version of [InstallChunkedCodeArgs].
/// The `sender_canister_version` field is removed as it is set automatically in [install_chunked_code].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct InstallChunkedCodeArgsReduced {
    /// See [CanisterInstallMode].
    pub mode: CanisterInstallMode,
    /// Principal of the canister being installed.
    pub target_canister: CanisterId,
    /// The canister in whose chunk storage the chunks are stored (defaults to target_canister if not specified).
    pub store_canister: Option<CanisterId>,
    /// The list of chunks that make up the canister wasm.
    pub chunk_hashes_list: Vec<ChunkHash>,
    /// The sha256 hash of the wasm.
    #[serde(with = "serde_bytes")]
    pub wasm_module_hash: Vec<u8>,
    /// The argument to be passed to `canister_init` or `canister_post_upgrade`.
    #[serde(with = "serde_bytes")]
    pub arg: Vec<u8>,
}

/// Complete argument type of `install_chunked_code`.
///
/// Please note that this type is not used directly as the argument of [install_chunked_code].
/// The function [install_chunked_code] takes [InstallChunkedCodeArgsReduced] instead.
///
/// If you want to manually call `install_chunked_code` (construct and invoke a [Call]), you should use this complete type.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct InstallChunkedCodeArgs {
    /// See [CanisterInstallMode].
    pub mode: CanisterInstallMode,
    /// Principal of the canister being installed.
    pub target_canister: CanisterId,
    /// The canister in whose chunk storage the chunks are stored (defaults to target_canister if not specified).
    pub store_canister: Option<CanisterId>,
    /// The list of chunks that make up the canister wasm.
    pub chunk_hashes_list: Vec<ChunkHash>,
    /// The sha256 hash of the wasm.
    #[serde(with = "serde_bytes")]
    pub wasm_module_hash: Vec<u8>,
    /// The argument to be passed to `canister_init` or `canister_post_upgrade`.
    #[serde(with = "serde_bytes")]
    pub arg: Vec<u8>,
    /// sender_canister_version must be set to ic_cdk::api::canister_version()
    pub sender_canister_version: Option<u64>,
}

// install_chunked_code END ---------------------------------------------------

// uninstall_code -------------------------------------------------------------

/// Remove a canister's code and state, making the canister empty again.
///
/// See [IC method `uninstall_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-uninstall_code).
pub async fn uninstall_code(arg: UninstallCodeArgsReduced) -> CallResult<()> {
    let complete_arg = UninstallCodeArgs {
        canister_id: arg.canister_id,
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "uninstall_code")
        .with_args((complete_arg,))
        .call()
        .await
}

/// Argument type of [uninstall_code].
///
/// Please note that this type is a reduced version of [UninstallCodeArgs].
/// The `sender_canister_version` field is removed as it is set automatically in [uninstall_code].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct UninstallCodeArgsReduced {
    /// Canister ID.
    pub canister_id: CanisterId,
}

/// Complete argument type of `uninstall_code`.
///
/// Please note that this type is not used directly as the argument of [uninstall_code].
/// The function [uninstall_code] takes [UninstallCodeArgsReduced] instead.
///
/// If you want to manually call `uninstall_code` (construct and invoke a [Call]), you should use this complete type.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct UninstallCodeArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// sender_canister_version must be set to ic_cdk::api::canister_version()
    pub sender_canister_version: Option<u64>,
}

// uninstall_code END ---------------------------------------------------------

// start_canister -------------------------------------------------------------

/// Start a canister if the canister status was `stopped` or `stopping`.
///
/// See [IC method `start_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-start_canister).
pub async fn start_canister(arg: StartCanisterArgs) -> CallResult<()> {
    Call::new(Principal::management_canister(), "start_canister")
        .with_args((arg,))
        .call()
        .await
}

/// Argument type of [start_canister].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct StartCanisterArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
}

// start_canister END ---------------------------------------------------------

// stop_canister --------------------------------------------------------------

/// Stop a canister.
///
/// See [IC method `stop_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-stop_canister).
pub async fn stop_canister(arg: StopCanisterArgs) -> CallResult<()> {
    Call::new(Principal::management_canister(), "stop_canister")
        .with_args((arg,))
        .call()
        .await
}

/// Argument type of [stop_canister].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct StopCanisterArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
}

// stop_canister END ----------------------------------------------------------

// canister_status ------------------------------------------------------------

/// Get status information about the canister.
///
/// See [IC method `canister_status`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_status).
pub async fn canister_status(arg: CanisterStatusArgs) -> CallResult<CanisterStatusResult> {
    Call::new(Principal::management_canister(), "canister_status")
        .with_args((arg,))
        .call::<(CanisterStatusResult,)>()
        .await
        .map(|result| result.0)
}

/// Argument type of [canister_status].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct CanisterStatusArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
}

/// Return type of [canister_status].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct CanisterStatusResult {
    /// See [CanisterStatusType].
    pub status: CanisterStatusType,
    /// See [DefiniteCanisterSettings].
    pub settings: DefiniteCanisterSettings,
    /// A SHA256 hash of the module installed on the canister. This is null if the canister is empty.
    pub module_hash: Option<Vec<u8>>,
    /// The memory size taken by the canister.
    pub memory_size: Nat,
    /// The cycle balance of the canister.
    pub cycles: Nat,
    /// Amount of cycles burned per day.
    pub idle_cycles_burned_per_day: Nat,
    /// The reserved cycles balance of the canister.
    /// These are cycles that are reserved by the resource reservation mechanism
    /// on storage allocation. See also the `reserved_cycles_limit` parameter in
    /// canister settings.
    pub reserved_cycles: Nat,
    /// Query statistics.
    pub query_stats: QueryStats,
}

/// Status of a canister.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy,
)]
#[serde(rename_all = "snake_case")]
pub enum CanisterStatusType {
    /// The canister is running.
    Running,
    /// The canister is stopping.
    Stopping,
    /// The canister is stopped.
    Stopped,
}

/// Query statistics, returned by [canister_status](super::canister_status).
#[derive(
    CandidType, Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct QueryStats {
    /// Total number of query calls.
    pub num_calls_total: Nat,
    /// Total number of instructions executed by query calls.
    pub num_instructions_total: Nat,
    /// Total number of payload bytes use for query call requests.
    pub request_payload_bytes_total: Nat,
    /// Total number of payload bytes use for query call responses.
    pub response_payload_bytes_total: Nat,
}

// canister_status END --------------------------------------------------------
