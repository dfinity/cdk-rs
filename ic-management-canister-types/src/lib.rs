#![doc = include_str!("../README.md")]

use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

/// # Canister ID.
pub type CanisterId = Principal;

/// # Chunk hash.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct ChunkHash {
    /// The hash of an uploaded chunk
    #[serde(with = "serde_bytes")]
    pub hash: Vec<u8>,
}

/// # Log Visibility.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub enum LogVisibility {
    /// Controllers.
    #[default]
    #[serde(rename = "controllers")]
    Controllers,
    /// Public.
    #[serde(rename = "public")]
    Public,
    /// Allowed viewers.
    #[serde(rename = "allowed_viewers")]
    AllowedViewers(Vec<Principal>),
}

/// # Canister Settings
///
/// For arguments of [`create_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister),
/// [`update_settings`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-update_settings) and
/// [`provisional_create_canister_with_cycles`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-provisional_create_canister_with_cycles).
///
/// All fields are `Option` types, allowing selective settings/updates.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct CanisterSettings {
    /// A list of at most 10 principals.
    ///
    /// The principals in this list become the *controllers* of the canister.
    ///
    /// Default value: A list containing only the caller of the `create_canister` call.
    pub controllers: Option<Vec<Principal>>,
    /// Indicates how much compute power should be guaranteed to this canister,
    /// expressed as a percentage of the maximum compute power that a single canister can allocate.
    ///
    /// If the IC cannot provide the requested allocation,
    /// for example because it is oversubscribed, the call will be **rejected**.
    ///
    /// Must be a number between 0 and 100, inclusively.
    ///
    /// Default value: 0
    pub compute_allocation: Option<Nat>,
    /// Indicates how much memory (bytes) the canister is allowed to use in total.
    ///
    /// If the IC cannot provide the requested allocation,
    /// for example because it is oversubscribed, the call will be **rejected**.
    ///
    /// If set to 0, then memory growth of the canister will be best-effort and subject to the available memory on the IC.
    ///
    /// Must be a number between 0 and 2<sup>48</sup> (i.e 256TB), inclusively.
    ///
    /// Default value: 0
    pub memory_allocation: Option<Nat>,
    /// Indicates a length of time in seconds.
    /// A canister is considered frozen whenever the IC estimates that the canister would be depleted of cycles
    /// before freezing_threshold seconds pass, given the canister's current size and the IC's current cost for storage.
    ///
    /// Must be a number between 0 and 2<sup>64</sup>-1, inclusively.
    ///
    /// Default value: 2592000 (approximately 30 days).
    pub freezing_threshold: Option<Nat>,
    /// Indicates the upper limit on [`CanisterStatusResult::reserved_cycles`] of the canister.
    ///
    /// Must be a number between 0 and 2<sup>128</sup>-1, inclusively.
    ///
    /// Default value: 5_000_000_000_000 (5 trillion cycles).
    pub reserved_cycles_limit: Option<Nat>,
    /// Defines who is allowed to read the canister's logs.
    ///
    /// Default value: [`LogVisibility::Controllers`].
    pub log_visibility: Option<LogVisibility>,
    /// Indicates the upper limit on the WASM heap memory (bytes) consumption of the canister.
    ///
    /// Must be a number between 0 and 2<sup>48</sup>-1 (i.e 256TB), inclusively.
    ///
    /// Default value: 3_221_225_472 (3 GiB).
    pub wasm_memory_limit: Option<Nat>,
}

/// # Definite Canister Settings
///
/// Represents the actual settings in effect.
///
/// For return of [`canister_status`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_status).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct DefiniteCanisterSettings {
    /// Controllers of the canister.
    pub controllers: Vec<Principal>,
    /// Guaranteed compute allocation as a percentage of the maximum compute power that a single canister can allocate.
    pub compute_allocation: Nat,
    /// Total memory (bytes) the canister is allowed to use.
    pub memory_allocation: Nat,
    /// Time in seconds after which the canister is considered frozen.
    pub freezing_threshold: Nat,
    /// Upper limit on [`CanisterStatusResult::reserved_cycles`] of the canister.
    pub reserved_cycles_limit: Nat,
    /// Visibility of canister logs.
    pub log_visibility: LogVisibility,
    /// Upper limit on the WASM heap memory (bytes) consumption of the canister.
    pub wasm_memory_limit: Nat,
}

/// # Create Canister Args
///
/// Argument type of [`create_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct CreateCanisterArgs {
    /// Canister settings.
    pub settings: Option<CanisterSettings>,
    /// Must match the canister's [`canister_version`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-canister-version) value when specified.
    pub sender_canister_version: Option<u64>,
}

/// # Create Canister Result
///
/// Result type of [`create_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy,
)]
pub struct CreateCanisterResult {
    /// Canister ID of the created canister.
    pub canister_id: CanisterId,
}

/// # Update Settings Args
///
/// Argument type of [`update_settings`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-update_settings).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct UpdateSettingsArgs {
    /// Canister ID of the canister whose settings are to be updated.
    pub canister_id: CanisterId,
    ///Canister settings.
    pub settings: CanisterSettings,
    /// Must match the canister's [`canister_version`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-canister-version) value when specified.
    pub sender_canister_version: Option<u64>,
}

/// # Upload Chunk Args
///
/// Argument type of [`upload_chunk`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-upload_chunk).
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

/// # Upload Chunk Result
///
/// Result type of [`upload_chunk`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-upload_chunk).
pub type UploadChunkResult = ChunkHash;

/// # Clear Chunk Store Args
///
/// Argument type of [`clear_chunk_store`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-clear_chunk_store).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct ClearChunkStoreArgs {
    /// The canister whose chunk store will be cleared.
    pub canister_id: CanisterId,
}

/// # Stored Chunks Args
///
/// Argument type of [`stored_chunks`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-stored_chunks).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct StoredChunksArgs {
    /// The canister whose chunk store will be queried.
    pub canister_id: CanisterId,
}

/// # Stored Chunks Result
///
/// Result type of [`stored_chunks`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-stored_chunks).
pub type StoredChunksResult = Vec<ChunkHash>;

/// # Canister Install Mode
///
/// See [`InstallCodeArgs`].
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
pub enum CanisterInstallMode {
    /// A fresh install of a new canister.
    #[default]
    #[serde(rename = "install")]
    Install,
    /// Reinstalling a canister that was already installed.
    #[serde(rename = "reinstall")]
    Reinstall,
    /// Upgrade an existing canister.
    #[serde(rename = "upgrade")]
    Upgrade(Option<UpgradeFlags>),
}

/// # Upgrade Flags
///
/// See [`CanisterInstallMode::Upgrade`].
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
    /// If set to [`WasmMemoryPersistence::Keep`], the WASM heap memory will be preserved instead of cleared.
    pub wasm_memory_persistence: Option<WasmMemoryPersistence>,
}

/// # Wasm Memory Persistence
///
/// See [`UpgradeFlags::wasm_memory_persistence`].
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
pub enum WasmMemoryPersistence {
    /// Preserve heap memory.
    #[serde(rename = "keep")]
    Keep,
    /// Clear heap memory.
    #[default]
    #[serde(rename = "replace")]
    Replace,
}

/// # WASM Module
pub type WasmModule = Vec<u8>;

/// # Install Code Args
///
/// Argument type of [`install_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-install_code).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct InstallCodeArgs {
    /// Canister install mode.
    pub mode: CanisterInstallMode,
    /// Canister ID.
    pub canister_id: CanisterId,
    /// Code to be installed.
    pub wasm_module: WasmModule,
    /// The argument to be passed to `canister_init` or `canister_post_upgrade`.
    #[serde(with = "serde_bytes")]
    pub arg: Vec<u8>,
    /// Must match the canister's [`canister_version`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-canister-version) value when specified.
    pub sender_canister_version: Option<u64>,
}

/// # Install Chunked Code Args
///
/// Argument type of [`install_chunked_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-install_chunked_code).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct InstallChunkedCodeArgs {
    /// Canister install mode.
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
    /// Must match the canister's [`canister_version`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-canister-version) value when specified.
    pub sender_canister_version: Option<u64>,
}

/// # Uninstall Code Args
///
/// Argument type of [`uninstall_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-uninstall_code).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct UninstallCodeArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// Must match the canister's [`canister_version`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-canister-version) value when specified.
    pub sender_canister_version: Option<u64>,
}

/// # Start Canister Args
///
/// Argument type of [`start_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-start_canister).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct StartCanisterArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
}

/// # Stop Canister Args
///
/// Argument type of [`stop_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-stop_canister).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct StopCanisterArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
}

/// # Canister Status Args
///
/// Argument type of [`canister_status`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_status).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct CanisterStatusArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
}

/// # Canister Status Result
///
/// Result type of [`canister_status`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_status).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct CanisterStatusResult {
    /// Status of the canister.
    pub status: CanisterStatusType,
    /// Canister settings in effect.
    pub settings: DefiniteCanisterSettings,
    /// A SHA256 hash of the module installed on the canister. This is null if the canister is empty.
    pub module_hash: Option<Vec<u8>>,
    /// The memory size taken by the canister.
    pub memory_size: Nat,
    /// The cycle balance of the canister.
    pub cycles: Nat,
    /// The reserved cycles balance of the canister.
    ///
    /// These are cycles that are reserved by the resource reservation mechanism on storage allocation.
    /// See also the [`CanisterSettings::reserved_cycles_limit`] parameter in canister settings.
    pub reserved_cycles: Nat,
    /// Amount of cycles burned per day.
    pub idle_cycles_burned_per_day: Nat,
    /// Query statistics.
    pub query_stats: QueryStats,
}

/// # Canister Status Type
///
/// Status of a canister.
///
/// See [`CanisterStatusResult::status`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy,
)]
pub enum CanisterStatusType {
    /// The canister is running.
    #[serde(rename = "running")]
    Running,
    /// The canister is stopping.
    #[serde(rename = "stopping")]
    Stopping,
    /// The canister is stopped.
    #[serde(rename = "stopped")]
    Stopped,
}

/// # Query Stats
///
/// Query statistics.
///
/// See [`CanisterStatusResult::query_stats`].
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

/// # Canister Info Args
///
/// Argument type of [`canister_info`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_info).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct CanisterInfoArgs {
    /// Canister ID.
    pub canister_id: Principal,
    /// Number of most recent changes requested to be retrieved from canister history.
    /// No changes are retrieved if this field is null.
    pub num_requested_changes: Option<u64>,
}

/// # Canister Info Result
///
/// Result type of [`canister_info`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_info).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct CanisterInfoResult {
    /// Total number of changes ever recorded in canister history.
    ///
    /// This might be higher than the number of canister changes in `recent_changes`
    /// because the IC might drop old canister changes from its history
    /// (with `20` most recent canister changes to always remain in the list).
    pub total_num_changes: u64,
    /// The canister changes stored in the order from the oldest to the most recent.
    pub recent_changes: Vec<Change>,
    /// A SHA256 hash of the module installed on the canister. This is null if the canister is empty.
    pub module_hash: Option<Vec<u8>>,
    /// Controllers of the canister.
    pub controllers: Vec<Principal>,
}

/// # From User Record
///
/// Details about a canister change initiated by a user.
///
/// See [`ChangeOrigin::FromUser`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct FromUserRecord {
    /// Principal of the user.
    pub user_id: Principal,
}

/// # From Canister Record
///
/// Details about a canister change initiated by a canister (called _originator_).
///
/// See [`ChangeOrigin::FromCanister`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct FromCanisterRecord {
    /// Canister ID of the _originator_.
    pub canister_id: Principal,
    /// Canister version of the _originator_ when the _originator_ initiated the change.
    ///
    /// This is null if the original does not include its canister version
    /// in the field `sender_canister_version` of the management canister payload.
    pub canister_version: Option<u64>,
}

/// # Change Origin
///
/// Provides details on who initiated a canister change.
///
/// See [`Change::origin`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub enum ChangeOrigin {
    /// The change was initiated by a user.
    #[serde(rename = "from_user")]
    FromUser(FromUserRecord),
    /// The change was initiated by a canister.
    #[serde(rename = "from_canister")]
    FromCanister(FromCanisterRecord),
}

/// # Creation Record
///
/// Details about a canister creation.
///
/// See [`ChangeDetails::Creation`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct CreationRecord {
    /// Initial set of canister controllers.
    pub controllers: Vec<Principal>,
}

/// # Code Deployment Mode
///
/// The mode with which a canister is installed.
///
/// See [`CodeDeploymentRecord::mode`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy,
)]
pub enum CodeDeploymentMode {
    /// A fresh install of a new canister.
    #[serde(rename = "install")]
    Install,
    /// Reinstalling a canister that was already installed.
    #[serde(rename = "reinstall")]
    Reinstall,
    /// Upgrade an existing canister.
    #[serde(rename = "upgrade")]
    Upgrade,
}

/// # Code Deployment Record
///
/// Details about a canister code deployment.
///
/// See [`ChangeDetails::CodeDeployment`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct CodeDeploymentRecord {
    /// Deployment mode.
    pub mode: CodeDeploymentMode,
    /// A SHA256 hash of the new module installed on the canister.
    #[serde(with = "serde_bytes")]
    pub module_hash: Vec<u8>,
}

/// # Load Snapshot Record
///
/// Details about loading canister snapshot.
///
/// See [`ChangeDetails::LoadSnapshot`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct LoadSnapshotRecord {
    /// The version of the canister at the time that the snapshot was taken
    pub canister_version: u64,
    /// The ID of the snapshot that was loaded.
    pub snapshot_id: SnapshotId,
    /// The timestamp at which the snapshot was taken.
    pub taken_at_timestamp: u64,
}

/// # Snapshot ID
///
/// Details about updating canister controllers.
///
/// See [`ChangeDetails::ControllersChange`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct ControllersChangeRecord {
    /// The complete new set of canister controllers.
    pub controllers: Vec<Principal>,
}

/// # Change Details
///
/// Provides details on the respective canister change.
///
/// See [`Change::details`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub enum ChangeDetails {
    /// The change was canister creation.
    #[serde(rename = "creation")]
    Creation(CreationRecord),
    /// The change was canister uninstallation.
    #[serde(rename = "code_uninstall")]
    CodeUninstall,
    /// The change was canister code deployment.
    #[serde(rename = "code_deployment")]
    CodeDeployment(CodeDeploymentRecord),
    /// The change was loading a canister snapshot.
    #[serde(rename = "load_snapshot")]
    LoadSnapshot(LoadSnapshotRecord),
    /// The change was updating canister controllers.
    #[serde(rename = "controllers_change")]
    ControllersChange(ControllersChangeRecord),
}

/// # Change
///
/// Represents a canister change as stored in the canister history.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct Change {
    /// The system timestamp (in nanoseconds since Unix Epoch) at which the change was performed.
    pub timestamp_nanos: u64,
    /// The canister version after performing the change.
    pub canister_version: u64,
    /// The change's origin (a user or a canister).
    pub origin: ChangeOrigin,
    /// The change's details.
    pub details: ChangeDetails,
}

/// # Delete Canister Args
///
/// Argument type of [`delete_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-delete_canister).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct DeleteCanisterArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
}

/// # Deposit Cycles Args
///
/// Argument type of [`deposit_cycles`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-deposit_cycles).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct DepositCyclesArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
}

/// # HTTP Request Args
///
/// Argument type of [`http_request`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-http_request).
#[derive(CandidType, Deserialize, Debug, PartialEq, Eq, Clone, Default)]
pub struct HttpRequestArgs {
    /// The requested URL.
    pub url: String,
    /// The maximal size of the response in bytes.
    ///
    /// If None, 2MiB will be the limit.
    /// This value affects the cost of the http request and it is highly recommended
    /// to set it as low as possible to avoid unnecessary extra costs.
    ///
    /// See also the [pricing section of HTTP outcalls documentation](https://internetcomputer.org/docs/current/developer-docs/integrations/http_requests/http_requests-how-it-works#pricing).
    pub max_response_bytes: Option<u64>,
    /// The method of HTTP request.
    pub method: HttpMethod,
    /// List of HTTP request headers and their corresponding values.
    pub headers: Vec<HttpHeader>,
    /// Optionally provide request body.
    pub body: Option<Vec<u8>>,
    /// Name of the transform function which is `func (transform_args) -> (http_response) query`.
    pub transform: Option<TransformContext>,
}

/// # HTTP Request Result
///
/// Result type of [`http_request`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-http_request).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct HttpRequestResult {
    /// The response status (e.g. 200, 404).
    pub status: candid::Nat,
    /// List of HTTP response headers and their corresponding values.
    pub headers: Vec<HttpHeader>,
    /// The response’s body.
    #[serde(with = "serde_bytes")]
    pub body: Vec<u8>,
}

/// # HTTP Method.
///
/// Represents a HTTP method.
///
/// See [`HttpRequestArgs::method`].
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
pub enum HttpMethod {
    /// GET
    #[default]
    #[serde(rename = "get")]
    GET,
    /// POST
    #[serde(rename = "post")]
    POST,
    /// HEAD
    #[serde(rename = "head")]
    HEAD,
}

/// # HTTP Header.
///
/// Represents a HTTP header.
///
/// See [`HttpRequestArgs::headers`] and [`HttpRequestResult::headers`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct HttpHeader {
    /// Name of the header.
    pub name: String,
    /// Value of the header.
    pub value: String,
}

/// # Transform Context.
///
/// ```text
/// record {
///     function : func(record { response : http_request_result; context : blob }) -> (http_request_result) query;
///     context : blob;
/// };
/// ```
///
/// See [`HttpRequestArgs::transform`].
#[derive(CandidType, Clone, Debug, Deserialize, PartialEq, Eq)]
pub struct TransformContext {
    /// `func(record { response : http_request_result; context : blob }) -> (http_request_result) query;`.
    pub function: TransformFunc,

    /// Context to be passed to `transform` function to transform HTTP response for consensus
    #[serde(with = "serde_bytes")]
    pub context: Vec<u8>,
}

mod transform_func {
    #![allow(missing_docs)]
    use super::{HttpRequestResult, TransformArgs};
    candid::define_function!(pub TransformFunc : (TransformArgs) -> (HttpRequestResult) query);
}

/// # Transform Function.
///
/// The "transform" function of type:
/// ```text
/// func(record { response : http_request_result; context : blob }) -> (http_request_result) query
/// ```
#[doc(inline)]
pub use transform_func::TransformFunc;

/// # Transform Args.
///
/// ```text
/// record {
///     response : http_response;
///     context : blob;
/// }
/// ```
///
/// See [`TransformContext`].
#[derive(CandidType, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransformArgs {
    /// Raw response from remote service, to be transformed
    pub response: HttpRequestResult,

    /// Context for response transformation
    #[serde(with = "serde_bytes")]
    pub context: Vec<u8>,
}

/// # ECDSA Key ID.
///
/// See [`EcdsaPublicKeyArgs::key_id`] and [`SignWithEcdsaArgs::key_id`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct EcdsaKeyId {
    /// Curve of the key.
    pub curve: EcdsaCurve,
    /// Name of the key.
    pub name: String,
}

/// # ECDSA Curve.
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
pub enum EcdsaCurve {
    /// secp256k1
    #[default]
    #[serde(rename = "secp256k1")]
    Secp256k1,
}

/// # ECDSA Public Key Args.
///
/// Argument type of [`ecdsa_public_key`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-ecdsa_public_key).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct EcdsaPublicKeyArgs {
    /// Canister id, default to the canister id of the caller if `None`.
    pub canister_id: Option<CanisterId>,
    /// A vector of variable length byte strings.
    pub derivation_path: Vec<Vec<u8>>,
    /// The key ID.
    pub key_id: EcdsaKeyId,
}

/// # ECDSA Public Key Result.
///
/// Result type of [`ecdsa_public_key`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-ecdsa_public_key).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct EcdsaPublicKeyResult {
    /// An ECDSA public key encoded in SEC1 compressed form.
    #[serde(with = "serde_bytes")]
    pub public_key: Vec<u8>,
    /// Can be used to deterministically derive child keys of the [`public_key`](Self::public_key).
    #[serde(with = "serde_bytes")]
    pub chain_code: Vec<u8>,
}

/// # Sign With ECDSA Args.
///
/// Argument type of [`sign_with_ecdsa`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_ecdsa).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SignWithEcdsaArgs {
    /// Hash of the message with length of 32 bytes.
    #[serde(with = "serde_bytes")]
    pub message_hash: Vec<u8>,
    /// A vector of variable length byte strings.
    pub derivation_path: Vec<Vec<u8>>,
    /// The key ID.
    pub key_id: EcdsaKeyId,
}

/// # Sign With ECDSA Result.
///
/// Result type of [`sign_with_ecdsa`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_ecdsa).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SignWithEcdsaResult {
    /// Encoded as the concatenation of the SEC1 encodings of the two values `r` and `s`.
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
}

/// # Schnorr Key ID.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SchnorrKeyId {
    /// Algorithm of the key.
    pub algorithm: SchnorrAlgorithm,
    /// Name of the key.
    pub name: String,
}

/// # Schnorr Algorithm.
///
/// See [`SchnorrKeyId::algorithm`].
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
pub enum SchnorrAlgorithm {
    /// BIP-340 secp256k1.
    #[serde(rename = "bip340secp256k1")]
    #[default]
    Bip340secp256k1,
    /// ed25519.
    #[serde(rename = "ed25519")]
    Ed25519,
}

/// # Schnorr Public Key Args.
///
/// Argument type of [`schnorr_public_key`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-schnorr_public_key).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SchnorrPublicKeyArgs {
    /// Canister id, default to the canister id of the caller if `None`.
    pub canister_id: Option<CanisterId>,
    /// A vector of variable length byte strings.
    pub derivation_path: Vec<Vec<u8>>,
    /// The key ID.
    pub key_id: SchnorrKeyId,
}

/// # Schnorr Public Key Result.
///
/// Result type of [`schnorr_public_key`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-schnorr_public_key).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SchnorrPublicKeyResult {
    /// An Schnorr public key encoded in SEC1 compressed form.
    #[serde(with = "serde_bytes")]
    pub public_key: Vec<u8>,
    /// Can be used to deterministically derive child keys of the public_key.
    #[serde(with = "serde_bytes")]
    pub chain_code: Vec<u8>,
}

/// # Sign With Schnorr Args.
///
/// Argument type of [`sign_with_schnorr`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_schnorr).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SignWithSchnorrArgs {
    /// Message to be signed.
    #[serde(with = "serde_bytes")]
    pub message: Vec<u8>,
    /// A vector of variable length byte strings.
    pub derivation_path: Vec<Vec<u8>>,
    /// The key ID.
    pub key_id: SchnorrKeyId,
}

/// # Sign With Schnorr Result.
///
/// Result type of [`sign_with_schnorr`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_schnorr).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SignWithSchnorrResult {
    /// The signature.
    ///
    /// The encoding of the signature depends on the key ID's algorithm.
    /// See [`sign_with_schnorr`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_schnorr) for more details.
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
}

/// # Node Metrics History Args.
///
/// Argument type of [`node_metrics_history`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-node_metrics_history).
// ! The actual url ends with `ic-node-metrics-history` instead of `ic-node_metrics_history`.
// ! It will likely be changed to be consistent with the other methods soon.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct NodeMetricsHistoryArgs {
    /// Subnet ID.
    pub subnet_id: Principal,
    /// The returned time series will start at this timestamp.
    pub start_at_timestamp_nanos: u64,
}

/// # Node Metrics History Result.
///
/// Result type of [`node_metrics_history`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-node_metrics_history).
// ! The actual url ends with `ic-node-metrics-history` instead of `ic-node_metrics_history`.
// ! It will likely be changed to be consistent with the other methods soon.
pub type NodeMetricsHistoryResult = Vec<NodeMetricsHistoryRecord>;

/// # Node Metrics History Record.
///
/// A record in [`NodeMetricsHistoryResult`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct NodeMetricsHistoryRecord {
    /// Timestamp of the record.
    pub timestamp_nanos: u64,
    /// Metrics of the nodes.
    pub node_metrics: Vec<NodeMetrics>,
}

/// # Node Metrics.
///
/// See [`NodeMetricsHistoryRecord::node_metrics`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct NodeMetrics {
    /// Node ID.
    pub node_id: Principal,
    /// Number of blocks proposed by this node.
    pub num_blocks_proposed_total: u64,
    /// Number of failed block proposals by this node.
    pub num_block_failures_total: u64,
}

/// # Subnet Info Args.
///
/// Argument type of [`subnet_info`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-subnet_info).
// ! The actual url ends with `ic-subnet-info` instead of `ic-subnet_info`.
// ! It will likely be changed to be consistent with the other methods soon.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct SubnetInfoArgs {
    /// Subnet ID.
    pub subnet_id: Principal,
}

/// # Subnet Info Result.
///
/// Result type of [`subnet_info`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-subnet_info).
// ! The actual url ends with `ic-subnet-info` instead of `ic-subnet_info`.
// ! It will likely be changed to be consistent with the other methods soon.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct SubnetInfoResult {
    /// Replica version of the subnet.
    pub replica_version: String,
}

/// # Provisional Create Canister With Cycles Args.
///
/// Argument type of [`provisional_create_canister_with_cycles`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-provisional_create_canister_with_cycles).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct ProvisionalCreateCanisterWithCyclesArgs {
    /// The created canister will have this amount of cycles.
    pub amount: Option<Nat>,
    /// Canister settings.
    pub settings: Option<CanisterSettings>,
    /// If set, the canister will be created under this id.
    pub specified_id: Option<CanisterId>,
    /// Must match the canister's [`canister_version`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-canister-version) value when specified.
    pub sender_canister_version: Option<u64>,
}

/// # Provisional Create Canister With Cycles Result.
///
/// Result type of [`provisional_create_canister_with_cycles`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-provisional_create_canister_with_cycles).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct ProvisionalCreateCanisterWithCyclesResult {
    /// Canister ID of the created canister.
    pub canister_id: CanisterId,
}

/// # Provisional Top Up Canister Args.
///
/// Argument type of [`provisional_top_up_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-provisional_top_up_canister).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct ProvisionalTopUpCanisterArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// Amount of cycles to be added.
    pub amount: Nat,
}

/// # Snapshot ID.
///
/// See [`Snapshot::id`].
pub type SnapshotId = Vec<u8>;

/// # Snapshot.
///
/// A snapshot of the canister's state at a given point in time.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct Snapshot {
    /// Snapshot ID.
    pub id: SnapshotId,
    /// Timestamp at which the snapshot was taken.
    pub taken_at_timestamp: u64,
    /// Total size of the snapshot in bytes.
    pub total_size: u64,
}

/// # Take Canister Snapshot Args.
///
/// Argument type of [`take_canister_snapshot`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-take_canister_snapshot).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct TakeCanisterSnapshotArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// An optional snapshot ID to be replaced by the new snapshot.
    ///
    /// The snapshot identified by the specified ID will be deleted once a new snapshot has been successfully created.
    pub replace_snapshot: Option<SnapshotId>,
}

/// # Take Canister Snapshot Return.
///
/// Return type of [`take_canister_snapshot`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-take_canister_snapshot).
pub type TakeCanisterSnapshotReturn = Snapshot;

/// # Load Canister Snapshot Args.
///
/// Argument type of [`load_canister_snapshot`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-load_canister_snapshot).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct LoadCanisterSnapshotArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// ID of the snapshot to be loaded.
    pub snapshot_id: SnapshotId,
    /// Must match the canister's [`canister_version`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-canister-version) value when specified.
    pub sender_canister_version: Option<u64>,
}

/// # List Canister Snapshots Args.
///
/// Argument type of [`list_canister_snapshots`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-list_canister_snapshots).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct ListCanisterSnapshotsArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
}

/// # List Canister Snapshots Return.
///
/// Return type of [`list_canister_snapshots`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-list_canister_snapshots).
pub type ListCanisterSnapshotsReturn = Vec<Snapshot>;

/// # Delete Canister Snapshot Args.
///
/// Argument type of [`delete_canister_snapshot`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-delete_canister_snapshot).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct DeleteCanisterSnapshotArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// ID of the snapshot to be deleted.
    pub snapshot_id: SnapshotId,
}
