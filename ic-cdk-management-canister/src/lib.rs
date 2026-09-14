#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

use candid::{CandidType, Nat, Principal, Reserved};
use ic_cdk::api::{
    SignCostError, canister_version, cost_create_canister,
    cost_sign_with_ecdsa as ic0_cost_sign_with_ecdsa,
    cost_sign_with_schnorr as ic0_cost_sign_with_schnorr,
    cost_vetkd_derive_key as ic0_cost_vetkd_derive_key, subnet_self_node_count,
};
use ic_cdk::call::{Call, CallFailed, CallResult, CandidDecodeFailed};
use serde::{Deserialize, Serialize};

// Re-export types from the `ic-management-canister-types` crate.
pub use ic_management_canister_types::{
    Bip341, CanisterId, CanisterIdRecord, CanisterInfoArgs, CanisterInfoResult,
    CanisterInstallMode, CanisterMetadataArgs, CanisterMetadataResult, CanisterSettings,
    CanisterStatusArgs, CanisterStatusResult, CanisterStatusType, CanisterTimer, Change,
    ChangeDetails, ChangeOrigin, ChunkHash, ClearChunkStoreArgs, CodeDeploymentMode,
    CodeDeploymentRecord, ControllersChangeRecord, CreateCanisterResult, CreationRecord,
    DefiniteCanisterSettings, DeleteCanisterArgs, DeleteCanisterSnapshotArgs, DepositCyclesArgs,
    EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgs, EcdsaPublicKeyResult, EnvironmentVariable,
    FlexibleHttpGlobalError, FlexibleHttpNodeDetail, FlexibleHttpNodeError,
    FlexibleHttpRequestArgs, FlexibleHttpRequestErr, FlexibleHttpRequestResult, FromCanisterRecord,
    FromUserRecord, HttpHeader, HttpMethod, HttpRequestArgs, HttpRequestResourceReport,
    HttpRequestResult, ListCanisterSnapshotsArgs, ListCanisterSnapshotsResult, LoadSnapshotRecord,
    LogVisibility, MemoryMetrics, NodeMetrics, NodeMetricsHistoryArgs, NodeMetricsHistoryRecord,
    NodeMetricsHistoryResult, OnLowWasmMemoryHookStatus, ProvisionalCreateCanisterWithCyclesResult,
    ProvisionalTopUpCanisterArgs, QueryStats, RawRandResult, ReadCanisterSnapshotDataArgs,
    ReadCanisterSnapshotDataResult, ReadCanisterSnapshotMetadataArgs,
    ReadCanisterSnapshotMetadataResult, RenameCanisterRecord, RenameToRecord, ReplicationCounts,
    ResourceUsage, SchnorrAlgorithm, SchnorrAux, SchnorrKeyId, SchnorrPublicKeyArgs,
    SchnorrPublicKeyResult, SignWithEcdsaArgs, SignWithEcdsaResult, SignWithSchnorrArgs,
    SignWithSchnorrResult, Snapshot, SnapshotDataKind, SnapshotDataOffset, SnapshotId,
    SnapshotMetadataGlobal, SnapshotSource, SnapshotVisibility, StartCanisterArgs,
    StatusVisibility, StopCanisterArgs, StoredChunksArgs, StoredChunksResult, SubnetInfoArgs,
    SubnetInfoResult, TakeCanisterSnapshotArgs, TakeCanisterSnapshotResult, TransformArgs,
    TransformContext, TransformFunc, UpgradeFlags, UploadCanisterSnapshotDataArgs,
    UploadCanisterSnapshotMetadataArgs, UploadCanisterSnapshotMetadataResult, UploadChunkArgs,
    UploadChunkResult, VetKDCurve, VetKDDeriveKeyArgs, VetKDDeriveKeyResult, VetKDKeyId,
    VetKDPublicKeyArgs, VetKDPublicKeyResult, WasmMemoryPersistence, WasmModule,
};

// Following Args types contain `sender_canister_version` field which is set automatically in the corresponding functions.
// We provide reduced versions of these types to avoid duplication of the field.
use ic_management_canister_types::{
    CreateCanisterArgs as CreateCanisterArgsComplete,
    InstallChunkedCodeArgs as InstallChunkedCodeArgsComplete,
    InstallCodeArgs as InstallCodeArgsComplete,
    LoadCanisterSnapshotArgs as LoadCanisterSnapshotArgsComplete,
    ProvisionalCreateCanisterWithCyclesArgs as ProvisionalCreateCanisterWithCyclesArgsComplete,
    UninstallCodeArgs as UninstallCodeArgsComplete,
    UpdateSettingsArgs as UpdateSettingsArgsComplete,
};

/// The error type for the [`sign_with_ecdsa`] and [`sign_with_schnorr`] functions.
#[derive(thiserror::Error, Debug, Clone)]
pub enum SignCallError {
    /// The signature cost calculation failed.
    #[error(transparent)]
    SignCostError(#[from] SignCostError),
    /// Failed to make the inter-canister call to the management canister.
    #[error(transparent)]
    CallFailed(#[from] CallFailed),
    /// Failed to decode the response from the management canister.
    #[error(transparent)]
    CandidDecodeFailed(#[from] CandidDecodeFailed),
}

/// Creates a new canister.
///
/// **Unbounded-wait call**
///
/// See [IC method `create_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister).
///
/// # Note
///
/// Canister creation costs cycles. That amount will be deducted from the newly created canister.
/// This method will only attach the required cycles for the canister creation (detemined by [`cost_create_canister`]).
/// The new canister will have a 0 cycle balance.
///
/// To ensure the new canister has extra cycles after creation, use [`create_canister_with_extra_cycles`] instead.
///
/// Cycles can also be deposited to the new canister using [`deposit_cycles`].
///
/// Check [Gas and cycles cost](https://internetcomputer.org/docs/current/developer-docs/gas-cost#canister-creation) for more details.
pub async fn create_canister(arg: &CreateCanisterArgs) -> CallResult<CreateCanisterResult> {
    let complete_arg = CreateCanisterArgsComplete {
        settings: arg.settings.clone(),
        sender_canister_version: Some(canister_version()),
    };
    let cycles = cost_create_canister();
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "create_canister")
            .with_arg(&complete_arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

/// Creates a new canister with extra cycles.
///
/// **Unbounded-wait call**
///
/// See [IC method `create_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister).
///
/// # Note
///
/// Canister creation costs cycles. That amount will be deducted from the newly created canister.
/// This method will attach the required cycles for the canister creation (detemined by [`cost_create_canister`]) plus the `extra_cycles` to the call.
/// The new cansiter will have a cycle balance of `extra_cycles`.
///
/// To simply create a canister with 0 cycle balance, use [`create_canister`] instead.
///
/// Check [Gas and cycles cost](https://internetcomputer.org/docs/current/developer-docs/gas-cost#canister-creation) for more details.
pub async fn create_canister_with_extra_cycles(
    arg: &CreateCanisterArgs,
    extra_cycles: u128,
) -> CallResult<CreateCanisterResult> {
    let complete_arg = CreateCanisterArgsComplete {
        settings: arg.settings.clone(),
        sender_canister_version: Some(canister_version()),
    };
    let cycles = cost_create_canister() + extra_cycles;
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "create_canister")
            .with_arg(&complete_arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

/// Argument type of [`create_canister`] and [`create_canister_with_extra_cycles`].
///
/// # Note
///
/// This type is a reduced version of [`ic_management_canister_types::CreateCanisterArgs`].
///
/// The `sender_canister_version` field is removed as it is set automatically in [`create_canister`] and [`create_canister_with_extra_cycles`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct CreateCanisterArgs {
    /// See [`CanisterSettings`].
    pub settings: Option<CanisterSettings>,
}

/// Updates the settings of a canister.
///
/// **Unbounded-wait call**
///
/// See [IC method `update_settings`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-update_settings).
pub async fn update_settings(arg: &UpdateSettingsArgs) -> CallResult<()> {
    let complete_arg = UpdateSettingsArgsComplete {
        canister_id: arg.canister_id,
        settings: arg.settings.clone(),
        sender_canister_version: Some(canister_version()),
    };
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "update_settings")
            .with_arg(&complete_arg)
            .await?
            .candid()?,
    )
}

/// Argument type of [`update_settings`]
///
/// # Note
///
/// This type is a reduced version of [`ic_management_canister_types::UpdateSettingsArgs`].
///
/// The `sender_canister_version` field is removed as it is set automatically in [`update_settings`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct UpdateSettingsArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// See [`CanisterSettings`].
    pub settings: CanisterSettings,
}

/// Uploads a chunk to the chunk store of a canister.
///
/// **Unbounded-wait call**
///
/// See [IC method `upload_chunk`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-upload_chunk).
pub async fn upload_chunk(arg: &UploadChunkArgs) -> CallResult<UploadChunkResult> {
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "upload_chunk")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Clears the chunk store of a canister.
///
/// **Unbounded-wait call**
///
/// See [IC method `clear_chunk_store`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-clear_chunk_store).
pub async fn clear_chunk_store(arg: &ClearChunkStoreArgs) -> CallResult<()> {
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "clear_chunk_store")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Gets the hashes of all chunks stored in the chunk store of a canister.
///
/// **Bounded-wait call**
///
/// See [IC method `stored_chunks`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-stored_chunks).
pub async fn stored_chunks(arg: &StoredChunksArgs) -> CallResult<StoredChunksResult> {
    Ok(
        Call::bounded_wait(Principal::management_canister(), "stored_chunks")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Installs code into a canister.
///
/// **Unbounded-wait call**
///
/// See [IC method `install_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-install_code).
pub async fn install_code(arg: &InstallCodeArgs) -> CallResult<()> {
    let complete_arg = InstallCodeArgsComplete {
        mode: arg.mode,
        canister_id: arg.canister_id,
        wasm_module: arg.wasm_module.clone(),
        arg: arg.arg.clone(),
        sender_canister_version: Some(canister_version()),
    };
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "install_code")
            .with_arg(&complete_arg)
            .await?
            .candid()?,
    )
}

/// Argument type of [`install_code`].
///
/// # Note
///
/// This type is a reduced version of [`ic_management_canister_types::InstallCodeArgs`].
///
/// The `sender_canister_version` field is removed as it is set automatically in [`install_code`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct InstallCodeArgs {
    /// See [`CanisterInstallMode`].
    pub mode: CanisterInstallMode,
    /// Canister ID.
    pub canister_id: CanisterId,
    /// Code to be installed.
    pub wasm_module: WasmModule,
    /// The argument to be passed to `canister_init` or `canister_post_upgrade`.
    #[serde(with = "serde_bytes")]
    pub arg: Vec<u8>,
}

/// Installs code into a canister where the code has previously been uploaded in chunks.
///
/// **Unbounded-wait call**
///
/// See [IC method `install_chunked_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-install_chunked_code).
pub async fn install_chunked_code(arg: &InstallChunkedCodeArgs) -> CallResult<()> {
    let complete_arg = InstallChunkedCodeArgsComplete {
        mode: arg.mode,
        target_canister: arg.target_canister,
        store_canister: arg.store_canister,
        chunk_hashes_list: arg.chunk_hashes_list.clone(),
        wasm_module_hash: arg.wasm_module_hash.clone(),
        arg: arg.arg.clone(),
        sender_canister_version: Some(canister_version()),
    };
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "install_chunked_code")
            .with_arg(&complete_arg)
            .await?
            .candid()?,
    )
}

/// Argument type of [`install_chunked_code`].
///
/// # Note
///
/// This type is a reduced version of [`ic_management_canister_types::InstallChunkedCodeArgs`].
///
/// The `sender_canister_version` field is removed as it is set automatically in [`install_chunked_code`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct InstallChunkedCodeArgs {
    /// See [`CanisterInstallMode`].
    pub mode: CanisterInstallMode,
    /// Principal of the canister being installed.
    pub target_canister: CanisterId,
    /// The canister in whose chunk storage the chunks are stored (defaults to `target_canister` if not specified).
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

/// Removes a canister's code and state, making the canister empty again.
///
/// **Unbounded-wait call**
///
/// See [IC method `uninstall_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-uninstall_code).
pub async fn uninstall_code(arg: &UninstallCodeArgs) -> CallResult<()> {
    let complete_arg = UninstallCodeArgsComplete {
        canister_id: arg.canister_id,
        sender_canister_version: Some(canister_version()),
    };
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "uninstall_code")
            .with_arg(&complete_arg)
            .await?
            .candid()?,
    )
}

/// Argument type of [`uninstall_code`].
///
/// # Note
///
/// This type is a reduced version of [`ic_management_canister_types::UninstallCodeArgs`].
///
/// The `sender_canister_version` field is removed as it is set automatically in [`uninstall_code`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct UninstallCodeArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
}

/// Starts a canister if the canister status was `stopped` or `stopping`.
///
/// **Unbounded-wait call**
///
/// See [IC method `start_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-start_canister).
pub async fn start_canister(arg: &StartCanisterArgs) -> CallResult<()> {
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "start_canister")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Stops a canister.
///
/// **Unbounded-wait call**
///
/// See [IC method `stop_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-stop_canister).
pub async fn stop_canister(arg: &StopCanisterArgs) -> CallResult<()> {
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "stop_canister")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Gets status information about the canister.
///
/// **Bounded-wait call**
///
/// See [IC method `canister_status`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_status).
pub async fn canister_status(arg: &CanisterStatusArgs) -> CallResult<CanisterStatusResult> {
    Ok(
        Call::bounded_wait(Principal::management_canister(), "canister_status")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Gets public information about the canister.
///
/// **Bounded-wait call**
///
/// See [IC method `canister_info`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_info).
pub async fn canister_info(arg: &CanisterInfoArgs) -> CallResult<CanisterInfoResult> {
    Ok(
        Call::bounded_wait(Principal::management_canister(), "canister_info")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}
/// Gets canister's metadata contained in custom sections whose names have the form `icp:public <name>` or `icp:private <name>`
///
/// **Bounded-wait call**
///
/// See [IC method `canister_metadata`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_metadata).
pub async fn canister_metadata(arg: &CanisterMetadataArgs) -> CallResult<CanisterMetadataResult> {
    Ok(
        Call::bounded_wait(Principal::management_canister(), "canister_metadata")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Deletes a canister.
///
/// **Unbounded-wait call**
///
/// See [IC method `delete_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-delete_canister).
pub async fn delete_canister(arg: &DeleteCanisterArgs) -> CallResult<()> {
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "delete_canister")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Deposits cycles to a canister.
///
/// **Unbounded-wait call**
///
/// See [IC method `deposit_cycles`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-deposit_cycles).
pub async fn deposit_cycles(arg: &DepositCyclesArgs, cycles: u128) -> CallResult<()> {
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "deposit_cycles")
            .with_arg(arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

/// Gets 32 pseudo-random bytes.
///
/// **Bounded-wait call**
///
/// See [IC method `raw_rand`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-raw_rand).
pub async fn raw_rand() -> CallResult<RawRandResult> {
    Ok(
        Call::bounded_wait(Principal::management_canister(), "raw_rand")
            .await?
            .candid()?,
    )
}

/// The maximum `max_response_bytes` an HTTP outcall may declare, and the value used when the
/// field is omitted.
const MAX_RESPONSE_BYTES_LIMIT: u64 = 2_000_000;
/// The longest the system waits for an HTTP response.
const MAX_ROUNDTRIP_TIME_MS: u64 = 60_000;
/// The instruction limit of a query call, which bounds a `transform` function.
const MAX_TRANSFORM_INSTRUCTIONS: u64 = 5_000_000_000;
/// Bytes reserved on top of `max_response_bytes` for the Candid encoding of a response.
const CANDID_OVERHEAD_RESERVE_BYTES: u64 = 1_024;
/// The block space the system has for the responses of one flexible outcall, which bounds the
/// combined size of the responses it can deliver.
const MAX_FLEXIBLE_RESULT_BYTES: u64 = 2 * 1024 * 1024;

/// # HTTP Outcall Type.
///
/// Which kind of HTTP outcall to price. See [`CostHttpRequestV2Args::outcall_type`].
#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum HttpOutcallType {
    /// An `http_request` with `is_replicated` set to `None` or `Some(true)`.
    #[serde(rename = "fully_replicated")]
    FullyReplicated(Reserved),
    /// An `http_request` with `is_replicated` set to `Some(false)`.
    #[serde(rename = "non_replicated")]
    NonReplicated(Reserved),
    /// A `flexible_http_request`, optionally with the replication counts it will use.
    ///
    /// If the counts are `None`, the endpoint's own defaults are priced.
    #[serde(rename = "flexible")]
    Flexible(Option<ReplicationCounts>),
}

/// # Cost HTTP Request V2 Args.
///
/// The resource usage to price. Argument type of [`cost_http_request_v2`].
#[derive(CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct CostHttpRequestV2Args {
    /// The byte length of the URL, the header names and values, the body, and the transform
    /// method name and context.
    pub request_bytes: u64,
    /// Milliseconds between sending the request and fully receiving the response.
    pub http_roundtrip_time_ms: u64,
    /// The byte length of the HTTP response, before transformation.
    pub raw_response_bytes: u64,
    /// The byte length of the response after transformation.
    pub transformed_response_bytes: u64,
    /// Instructions the transform function uses.
    pub transform_instructions: u64,
    /// The kind of outcall. If `None`, a fully replicated outcall is priced.
    pub outcall_type: Option<HttpOutcallType>,
}

/// Calculates the cost of an HTTP outcall priced with pricing version `2`.
///
/// This returns the amount to **attach** for an outcall that consumes exactly the resources in
/// `arg`, not a prediction of the charge. The surplus is refunded, so the eventual charge is at
/// most the amount attached.
///
/// [`HttpRequest`] and [`FlexibleHttpRequest`] invoke this internally and attach the result.
///
/// # Panics
///
/// Panics if `arg` cannot be Candid-encoded, which cannot happen for a well-formed value.
pub fn cost_http_request_v2(arg: &CostHttpRequestV2Args) -> u128 {
    let bytes = candid::encode_one(arg).expect("failed to Candid-encode the cost parameters");
    ic_cdk::api::cost_http_request_v2(&bytes)
}

/// The resource usage a caller expects, used to size the cycles reservation.
///
/// Any field left unset falls back to the maximum the outcall could consume, which yields a
/// reservation the outcall cannot exhaust.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Reservation {
    roundtrip_time_ms: Option<u64>,
    raw_response_bytes: Option<u64>,
    transformed_response_bytes: Option<u64>,
    transform_instructions: Option<u64>,
}

impl Reservation {
    /// Resolves the expected usage against `max_response_bytes`, filling unset fields with the
    /// maximum the outcall could consume.
    ///
    /// `transformed_default_cap` bounds the value an unset `transformed_response_bytes` falls
    /// back to. An expectation the caller set explicitly is always used as given.
    ///
    /// `has_transform` says whether the request sets a `transform` function. Without one the
    /// system never runs a transform, so an unset `transform_instructions` falls back to zero
    /// rather than to the query call instruction limit.
    fn resolve(
        &self,
        max_response_bytes: Option<u64>,
        transformed_default_cap: Option<u64>,
        has_transform: bool,
    ) -> (u64, u64, u64, u64) {
        let cap = max_response_bytes.unwrap_or(MAX_RESPONSE_BYTES_LIMIT);
        let transformed = self.transformed_response_bytes.unwrap_or_else(|| {
            let worst_case = cap.saturating_add(CANDID_OVERHEAD_RESERVE_BYTES);
            transformed_default_cap.map_or(worst_case, |c| worst_case.min(c))
        });
        (
            self.roundtrip_time_ms.unwrap_or(MAX_ROUNDTRIP_TIME_MS),
            self.raw_response_bytes.unwrap_or(cap),
            transformed,
            self.transform_instructions.unwrap_or(if has_transform {
                MAX_TRANSFORM_INSTRUCTIONS
            } else {
                0
            }),
        )
    }
}

/// The largest a flexible outcall's responses can average and still be deliverable.
///
/// `None` when no response is ever delivered, or when `min_responses` is zero.
fn flexible_transformed_default_cap(replication: Option<&ReplicationCounts>) -> Option<u64> {
    let min_responses = match replication {
        Some(counts) => {
            if counts.max_responses == 0 {
                // Fire-and-forget: no response is delivered, so nothing bounds its size.
                return None;
            }
            counts.min_responses
        }
        // The system's own default when `replication` is unset.
        None => 2 * subnet_self_node_count() / 3 + 1,
    };
    (min_responses > 0).then(|| MAX_FLEXIBLE_RESULT_BYTES.div_ceil(u64::from(min_responses)))
}

/// Computes the `request_bytes` an outcall is charged for.
fn request_bytes(
    url: &str,
    headers: &[HttpHeader],
    body: Option<&Vec<u8>>,
    transform: Option<&TransformContext>,
) -> u64 {
    (url.len()
        + headers
            .iter()
            .map(|h| h.name.len() + h.value.len())
            .sum::<usize>()
        + body.map_or(0, |b| b.len())
        + transform.map_or(0, |t| t.context.len() + t.function.0.method.len())) as u64
}

/// A builder for an HTTP outcall via the Management canister method
/// [`http_request`](https://internetcomputer.org/docs/references/ic-interface-spec#ic-http_request).
///
/// The outcall is always made with pricing version `2` ("pay-as-you-go"), which charges for the
/// resources the call actually consumes rather than for `max_response_bytes`.
///
/// Because the cycles attached to a version `2` outcall are also the budget each node may spend
/// on it, the amount to attach depends on how much the call is expected to consume. Every
/// `with_expected_*` method narrows that estimate; whatever is left unset falls back to the most
/// the outcall could consume, which yields a reservation the outcall cannot exhaust but which
/// holds far more cycles for the duration of the call.
///
/// Narrowing an expectation below what the call actually needs is not rejected up front. The
/// outcall runs with reduced limits, and potentially fails at a later point. A node that
/// exhausts its budget rejects instead of returning the response, possibly after the remote
/// server has already been contacted.
///
/// Use [`FlexibleHttpRequest`] for an outcall whose nodes return their individual responses.
///
/// # Examples
///
/// ```no_run
/// # use ic_cdk_management_canister::{HttpRequest, HttpMethod};
/// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
/// let response = HttpRequest::new("https://example.com/api")
///     .with_method(HttpMethod::POST)
///     .with_body(b"{}".to_vec())
///     .with_max_response_bytes(4_000)
///     // a 200 ms call with a cheap transform reserves far less than the worst case
///     .with_expected_roundtrip_time_ms(200)
///     .with_expected_transform_instructions(1_000_000)
///     .send()
///     .await?;
/// # Ok(()) }
/// ```
#[must_use = "an HttpRequest does nothing unless you call `send`"]
#[derive(Debug, Clone)]
pub struct HttpRequest {
    args: HttpRequestArgs,
    reservation: Reservation,
    #[cfg(feature = "transform-closure")]
    transform_guard: Option<std::sync::Arc<transform_closure::TransformGuard>>,
}

impl HttpRequest {
    /// Starts building an outcall to `url`.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            args: HttpRequestArgs {
                url: url.into(),
                pricing_version: Some(2),
                ..Default::default()
            },
            reservation: Reservation::default(),
            #[cfg(feature = "transform-closure")]
            transform_guard: None,
        }
    }

    /// Starts building an outcall from an existing [`HttpRequestArgs`].
    ///
    /// The `pricing_version` field is overwritten with `2`.
    pub fn from_args(args: HttpRequestArgs) -> Self {
        Self {
            args: HttpRequestArgs {
                pricing_version: Some(2),
                ..args
            },
            reservation: Reservation::default(),
            #[cfg(feature = "transform-closure")]
            transform_guard: None,
        }
    }

    /// Sets the HTTP method. Defaults to `GET`.
    ///
    /// `PUT`, `DELETE` and `PATCH` are accepted only in non-replicated mode, see
    /// [`Self::non_replicated`].
    pub fn with_method(mut self, method: HttpMethod) -> Self {
        self.args.method = method;
        self
    }

    /// Sets the request headers.
    pub fn with_headers(mut self, headers: Vec<HttpHeader>) -> Self {
        self.args.headers = headers;
        self
    }

    /// Appends one request header.
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.args.headers.push(HttpHeader {
            name: name.into(),
            value: value.into(),
        });
        self
    }

    /// Sets the request body.
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.args.body = Some(body);
        self
    }

    /// Sets the maximum size of the response in bytes, up to 2MB.
    ///
    /// Under pricing version `2` this does not set the price, but it still bounds the response
    /// and it affects how many cycles are held while the call runs. Setting it as low as the
    /// response allows keeps the reservation small.
    pub fn with_max_response_bytes(mut self, max_response_bytes: u64) -> Self {
        self.args.max_response_bytes = Some(max_response_bytes);
        self
    }

    /// Sets the transform function, which each node runs on its own response.
    pub fn with_transform(mut self, transform: TransformContext) -> Self {
        self.args.transform = Some(transform);
        self
    }

    /// Makes the request from a single node chosen by the system, rather than from every node.
    ///
    /// This gives weaker integrity guarantees: the single node could observe or modify the
    /// response. It avoids the rate-limit pressure of one request per node, and it is the only
    /// mode in which `PUT`, `DELETE` and `PATCH` are accepted.
    pub fn non_replicated(mut self) -> Self {
        self.args.is_replicated = Some(false);
        self
    }

    /// Sets the round-trip time the outcall is expected to take, in milliseconds.
    ///
    /// A lower expectation reserves fewer cycles; see [`Self`] for the risk of setting it
    /// below what the call needs.
    ///
    /// Defaults to the 60 second maximum the system allows.
    pub fn with_expected_roundtrip_time_ms(mut self, ms: u64) -> Self {
        self.reservation.roundtrip_time_ms = Some(ms);
        self
    }

    /// Sets the size the response is expected to have as it arrives from the server.
    ///
    /// A lower expectation reserves fewer cycles; see [`Self`] for the risk of setting it
    /// below what the call needs.
    ///
    /// Defaults to `max_response_bytes`, or 2MB if that is unset.
    pub fn with_expected_raw_response_bytes(mut self, bytes: u64) -> Self {
        self.reservation.raw_response_bytes = Some(bytes);
        self
    }

    /// Sets the size the response is expected to have after the transform function.
    ///
    /// A lower expectation reserves fewer cycles; see [`Self`] for the risk of setting it
    /// below what the call needs.
    ///
    /// Defaults to `max_response_bytes` plus the bytes reserved for the Candid encoding, or 2MB
    /// plus that reserve if `max_response_bytes` is unset.
    pub fn with_expected_transformed_response_bytes(mut self, bytes: u64) -> Self {
        self.reservation.transformed_response_bytes = Some(bytes);
        self
    }

    /// Sets the instructions the transform function is expected to use.
    ///
    /// A lower expectation reserves fewer cycles; see [`Self`] for the risk of setting it
    /// below what the call needs.
    ///
    /// Defaults to the query call instruction limit when a transform is set, and to zero when
    /// none is, since the system then never runs one.
    pub fn with_expected_transform_instructions(mut self, instructions: u64) -> Self {
        self.reservation.transform_instructions = Some(instructions);
        self
    }

    /// Sets a transform implemented as a closure, instead of an exported query method.
    ///
    /// Each node runs it on its own response. The closure is deregistered when this builder is
    /// dropped, so a builder that is never sent does not leak it.
    ///
    /// # Panics
    ///
    /// Panics if a transform has already been set, as the two would conflict.
    #[cfg(feature = "transform-closure")]
    #[cfg_attr(docsrs, doc(cfg(feature = "transform-closure")))]
    pub fn with_transform_closure(
        mut self,
        transform_func: impl FnOnce(HttpRequestResult) -> HttpRequestResult + 'static,
    ) -> Self {
        assert!(
            self.args.transform.is_none(),
            "a transform is already set on this outcall"
        );
        let (transform, guard) = transform_closure::register(transform_func);
        self.args.transform = Some(transform);
        self.transform_guard = Some(std::sync::Arc::new(guard));
        self
    }

    /// Returns the arguments the outcall will be made with.
    pub fn args(&self) -> &HttpRequestArgs {
        &self.args
    }

    /// Returns the cycles that [`Self::send`] will attach.
    pub fn get_cost(&self) -> u128 {
        let (roundtrip, raw, transformed, instructions) = self.reservation.resolve(
            self.args.max_response_bytes,
            None,
            self.args.transform.is_some(),
        );
        cost_http_request_v2(&CostHttpRequestV2Args {
            request_bytes: request_bytes(
                &self.args.url,
                &self.args.headers,
                self.args.body.as_ref(),
                self.args.transform.as_ref(),
            ),
            http_roundtrip_time_ms: roundtrip,
            raw_response_bytes: raw,
            transformed_response_bytes: transformed,
            transform_instructions: instructions,
            // Absent means fully replicated, which is the encoding the system documents as the
            // default. Note this still emits the field as `null`: Candid keeps an `opt` field in
            // the type table, so it is not the same as omitting it.
            outcall_type: if self.args.is_replicated == Some(false) {
                Some(HttpOutcallType::NonReplicated(Reserved))
            } else {
                None
            },
        })
    }

    /// Makes the outcall, attaching [`Self::get_cost`] cycles.
    ///
    /// **Unbounded-wait call**
    pub async fn send(self) -> CallResult<HttpRequestResult> {
        let cycles = self.get_cost();
        let result = Call::unbounded_wait(Principal::management_canister(), "http_request")
            .with_arg(&self.args)
            .with_cycles(cycles)
            .await?
            .candid();
        // The transform guard, if any, must outlive the call.
        #[cfg(feature = "transform-closure")]
        drop(self.transform_guard);
        Ok(result?)
    }
}

/// A builder for a flexible HTTP outcall via the Management canister method
/// [`flexible_http_request`](https://internetcomputer.org/docs/references/ic-interface-spec#ic-flexible_http_request).
///
/// A committee of nodes make the request and the canister receives their individual responses
/// rather than one the subnet agreed on, so reconciling them is the canister's job. Flexible
/// outcalls are always priced with pricing version `2`.
///
/// Because the cycles attached to a version `2` outcall are also the budget the nodes may spend
/// on it, the amount to attach depends on how much the outcall is expected to consume. Every
/// `with_expected_*` method narrows that estimate; whatever is left unset falls back to the most
/// the outcall could consume, which yields a reservation the outcall cannot exhaust but which
/// holds far more cycles for the duration of the call.
///
/// Here the budget is split between the `total_requests` nodes rather than across the subnet, and
/// too few cycles surface in one of two ways. A node that exhausts its own share rejects, counting
/// towards [`TooManyRejects`](FlexibleHttpGlobalError::TooManyRejects). Separately, what the
/// committee leaves unspent is pooled to pay for delivering the result, and once that pool no
/// longer covers any result the outcall could still produce, it fails with
/// [`OutOfCycles`](FlexibleHttpGlobalError::OutOfCycles) instead. Delivery is priced by the sizes
/// of the responses, so that verdict can come after the nodes have already made their HTTP
/// requests: the outcall can spend cycles and still deliver no responses.
///
/// # Examples
///
/// ```no_run
/// # use ic_cdk_management_canister::{FlexibleHttpRequest, FlexibleHttpRequestResult, ReplicationCounts};
/// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
/// // ask 3 nodes, accept any 2 or 3 answers
/// let result = FlexibleHttpRequest::new("https://example.com/price")
///     .with_replication(ReplicationCounts {
///         min_responses: 2,
///         max_responses: 3,
///         total_requests: 3,
///     })
///     .with_max_response_bytes(4_000)
///     .send()
///     .await?;
/// match result {
///     // fewer than `max_responses` is a normal success
///     FlexibleHttpRequestResult::Ok(responses) => { let _ = responses; }
///     FlexibleHttpRequestResult::Err(err) => { let _ = err.global_error; }
/// }
/// # Ok(()) }
/// ```
#[must_use = "a FlexibleHttpRequest does nothing unless you call `send`"]
#[derive(Debug, Clone)]
pub struct FlexibleHttpRequest {
    args: FlexibleHttpRequestArgs,
    reservation: Reservation,
    #[cfg(feature = "transform-closure")]
    transform_guard: Option<std::sync::Arc<transform_closure::TransformGuard>>,
}

impl FlexibleHttpRequest {
    /// Starts building a flexible outcall to `url`.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            args: FlexibleHttpRequestArgs {
                url: url.into(),
                ..Default::default()
            },
            reservation: Reservation::default(),
            #[cfg(feature = "transform-closure")]
            transform_guard: None,
        }
    }

    /// Starts building a flexible outcall from an existing [`FlexibleHttpRequestArgs`].
    pub fn from_args(args: FlexibleHttpRequestArgs) -> Self {
        Self {
            args,
            reservation: Reservation::default(),
            #[cfg(feature = "transform-closure")]
            transform_guard: None,
        }
    }

    /// Sets the HTTP method. Defaults to `GET`.
    ///
    /// `PUT`, `DELETE` and `PATCH` are accepted only when the replication counts are
    /// deterministic, that is when `min_responses`, `max_responses` and `total_requests` are all
    /// equal.
    pub fn with_method(mut self, method: HttpMethod) -> Self {
        self.args.method = method;
        self
    }

    /// Sets the request headers.
    pub fn with_headers(mut self, headers: Vec<HttpHeader>) -> Self {
        self.args.headers = headers;
        self
    }

    /// Appends one request header.
    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.args.headers.push(HttpHeader {
            name: name.into(),
            value: value.into(),
        });
        self
    }

    /// Sets the request body.
    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.args.body = Some(body);
        self
    }

    /// Sets the maximum size of any single node's response in bytes, up to 2MB.
    ///
    /// Note that at least `min_responses` must fit a 2MiB total, in order for a
    /// result to be delivered.
    pub fn with_max_response_bytes(mut self, max_response_bytes: u64) -> Self {
        self.args.max_response_bytes = Some(max_response_bytes);
        self
    }

    /// Sets the transform function, which each node runs on its own response.
    pub fn with_transform(mut self, transform: TransformContext) -> Self {
        self.args.transform = Some(transform);
        self
    }

    /// Sets how many nodes issue the request and how many responses to require and accept.
    ///
    /// Must satisfy `0 <= min_responses <= max_responses <= total_requests` and
    /// `1 <= total_requests <= N`, where `N` is
    /// [`subnet_self_node_count`]. Defaults to
    /// `floor(2 / 3 * N) + 1`, `N` and `N`.
    pub fn with_replication(mut self, replication: ReplicationCounts) -> Self {
        self.args.replication = Some(replication);
        self
    }

    /// Sets the round-trip time the outcall is expected to take, in milliseconds.
    ///
    /// A lower expectation reserves fewer cycles; see [`Self`] for the risk of setting it
    /// below what the call needs.
    ///
    /// Defaults to the 60 second maximum the system allows.
    pub fn with_expected_roundtrip_time_ms(mut self, ms: u64) -> Self {
        self.reservation.roundtrip_time_ms = Some(ms);
        self
    }

    /// Sets the size a response is expected to have as it arrives from the server.
    ///
    /// A lower expectation reserves fewer cycles; see [`Self`] for the risk of setting it
    /// below what the call needs.
    ///
    /// Defaults to `max_response_bytes`, or 2MB if that is unset.
    pub fn with_expected_raw_response_bytes(mut self, bytes: u64) -> Self {
        self.reservation.raw_response_bytes = Some(bytes);
        self
    }

    /// Sets the size a response is expected to have after the transform function.
    ///
    /// A lower expectation reserves fewer cycles; see [`Self`] for the risk of setting it
    /// below what the call needs.
    ///
    /// Defaults to `max_response_bytes` plus the bytes reserved for the Candid encoding, or 2MB
    /// plus that reserve if `max_response_bytes` is unset.
    pub fn with_expected_transformed_response_bytes(mut self, bytes: u64) -> Self {
        self.reservation.transformed_response_bytes = Some(bytes);
        self
    }

    /// Sets the instructions the transform function is expected to use.
    ///
    /// A lower expectation reserves fewer cycles; see [`Self`] for the risk of setting it
    /// below what the call needs.
    ///
    /// Defaults to the query call instruction limit when a transform is set, and to zero when
    /// none is, since the system then never runs one.
    pub fn with_expected_transform_instructions(mut self, instructions: u64) -> Self {
        self.reservation.transform_instructions = Some(instructions);
        self
    }

    /// Sets a transform implemented as a closure, instead of an exported query method.
    ///
    /// Each node runs it on its own response. The closure is deregistered when this builder is
    /// dropped, so a builder that is never sent does not leak it.
    ///
    /// # Panics
    ///
    /// Panics if a transform has already been set, as the two would conflict.
    #[cfg(feature = "transform-closure")]
    #[cfg_attr(docsrs, doc(cfg(feature = "transform-closure")))]
    pub fn with_transform_closure(
        mut self,
        transform_func: impl FnOnce(HttpRequestResult) -> HttpRequestResult + 'static,
    ) -> Self {
        assert!(
            self.args.transform.is_none(),
            "a transform is already set on this flexible outcall"
        );
        let (transform, guard) = transform_closure::register(transform_func);
        self.args.transform = Some(transform);
        self.transform_guard = Some(std::sync::Arc::new(guard));
        self
    }

    /// Returns the arguments the outcall will be made with.
    pub fn args(&self) -> &FlexibleHttpRequestArgs {
        &self.args
    }

    /// Returns the cycles that [`Self::send`] will attach.
    pub fn get_cost(&self) -> u128 {
        let (roundtrip, raw, transformed, instructions) = self.reservation.resolve(
            self.args.max_response_bytes,
            flexible_transformed_default_cap(self.args.replication.as_ref()),
            self.args.transform.is_some(),
        );
        cost_http_request_v2(&CostHttpRequestV2Args {
            request_bytes: request_bytes(
                &self.args.url,
                &self.args.headers,
                self.args.body.as_ref(),
                self.args.transform.as_ref(),
            ),
            http_roundtrip_time_ms: roundtrip,
            raw_response_bytes: raw,
            transformed_response_bytes: transformed,
            transform_instructions: instructions,
            outcall_type: Some(HttpOutcallType::Flexible(self.args.replication.clone())),
        })
    }

    /// Makes the outcall, attaching [`Self::get_cost`] cycles.
    ///
    /// **Unbounded-wait call**
    ///
    /// Both arms of [`FlexibleHttpRequestResult`] arrive as a reply. This returns `Err` only for
    /// failures detected before the requests are issued, such as invalid arguments, invalid
    /// replication counts, or too few attached cycles.
    pub async fn send(self) -> CallResult<FlexibleHttpRequestResult> {
        let cycles = self.get_cost();
        let result =
            Call::unbounded_wait(Principal::management_canister(), "flexible_http_request")
                .with_arg(&self.args)
                .with_cycles(cycles)
                .await?
                .candid();
        // The transform guard, if any, must outlive the call.
        #[cfg(feature = "transform-closure")]
        drop(self.transform_guard);
        Ok(result?)
    }
}

/// Constructs a [`TransformContext`] from a query method name and context.
pub fn transform_context_from_query(
    candid_function_name: String,
    context: Vec<u8>,
) -> TransformContext {
    TransformContext {
        context,
        function: TransformFunc(candid::Func {
            method: candid_function_name,
            principal: ic_cdk::api::canister_self(),
        }),
    }
}

#[cfg(feature = "transform-closure")]
mod transform_closure {
    use super::{HttpRequestResult, Principal, TransformArgs, TransformContext};
    use candid::{decode_one, encode_one};
    use slotmap::{DefaultKey, Key, KeyData, SlotMap};
    use std::cell::RefCell;

    thread_local! {
        #[allow(clippy::type_complexity)]
        static TRANSFORMS: RefCell<SlotMap<DefaultKey, Box<dyn FnOnce(HttpRequestResult) -> HttpRequestResult>>> = RefCell::default();
    }

    #[cfg_attr(
        target_family = "wasm",
        unsafe(export_name = "canister_query <ic-cdk internal> http_transform")
    )]
    #[cfg_attr(
        not(target_family = "wasm"),
        unsafe(export_name = "canister_query_ic_cdk_internal.http_transform")
    )]
    extern "C" fn http_transform() {
        ic_cdk_executor::in_tracking_query_executor_context(|| {
            use ic_cdk::api::{msg_arg_data, msg_caller, msg_reply};
            if msg_caller() != Principal::management_canister() {
                ic_cdk::trap(
                    "This function is internal to ic-cdk and should not be called externally.",
                );
            }
            let arg_bytes = msg_arg_data();
            let transform_args: TransformArgs = decode_one(&arg_bytes).unwrap();
            let int = u64::from_be_bytes(transform_args.context[..].try_into().unwrap());
            let key = DefaultKey::from(KeyData::from_ffi(int));
            let func = TRANSFORMS.with(|transforms| transforms.borrow_mut().remove(key));
            let Some(func) = func else {
                ic_cdk::trap(format!("Missing transform function for request {int}"));
            };
            let transformed = func(transform_args.response);
            let encoded = encode_one(transformed).unwrap();
            msg_reply(encoded);
        });
    }

    /// Deregisters a transform closure when dropped.
    ///
    /// A request builder holds this until its call completes, so that a builder which is dropped
    /// without being sent does not leak the closure.
    #[derive(Debug)]
    pub struct TransformGuard(DefaultKey);

    impl Drop for TransformGuard {
        fn drop(&mut self) {
            TRANSFORMS.with(|transforms| transforms.borrow_mut().remove(self.0));
        }
    }

    /// Registers `transform_func` and returns the [`TransformContext`] that routes to it, plus a
    /// guard that deregisters it when dropped.
    pub fn register(
        transform_func: impl FnOnce(HttpRequestResult) -> HttpRequestResult + 'static,
    ) -> (TransformContext, TransformGuard) {
        let transform_func = Box::new(transform_func) as _;
        let key = TRANSFORMS.with(|transforms| transforms.borrow_mut().insert(transform_func));
        let guard = TransformGuard(key);
        let context = guard.0.data().as_ffi().to_be_bytes().to_vec();
        let transform = super::transform_context_from_query(
            "<ic-cdk internal> http_transform".to_string(),
            context,
        );
        (transform, guard)
    }
}

/// Gets a SEC1 encoded ECDSA public key for the given canister using the given derivation path.
///
/// **Bounded-wait call**
///
/// See [IC method `ecdsa_public_key`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-ecdsa_public_key).
pub async fn ecdsa_public_key(arg: &EcdsaPublicKeyArgs) -> CallResult<EcdsaPublicKeyResult> {
    Ok(
        Call::bounded_wait(Principal::management_canister(), "ecdsa_public_key")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Calculates the cost of ECDSA signanature with the given [`SignWithEcdsaArgs`].
///
/// [`sign_with_ecdsa`] invokes this method internally and attaches the required cycles to the call.
///
/// # Note
///
/// Alternatively, [`api::cost_sign_with_ecdsa`][ic0_cost_sign_with_ecdsa] takes the numeric representation of the curve.
pub fn cost_sign_with_ecdsa(arg: &SignWithEcdsaArgs) -> Result<u128, SignCostError> {
    ic0_cost_sign_with_ecdsa(&arg.key_id.name, arg.key_id.curve.into())
}

/// Gets a new ECDSA signature of the given `message_hash` with a user-specified amount of cycles.
///
/// **Unbounded-wait call**
///
/// The signature can be separately verified against a derived ECDSA public key.
///
/// See [IC method `sign_with_ecdsa`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_ecdsa).
///
/// # Errors
///
/// This method returns an error of type [`SignCallError`].
///
/// The signature cost calculation may fail before the inter-canister call is made, resulting in a [`SignCallError::SignCostError`].
///
/// Since the call argument is constructed as [`SignWithEcdsaArgs`], the `ecdsa_curve` field is guaranteed to be valid.
/// Therefore, [`SignCostError::InvalidCurveOrAlgorithm`] should not occur. If it does, it is likely an issue with the IC. Please report it.
///
/// # Note
///
/// Signature costs cycles which varies for different curves and key names.
/// This method attaches the required cycles (detemined by [`cost_sign_with_ecdsa`]) to the call.
///
/// Check [Threshold signatures](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#api-fees) for more details.
pub async fn sign_with_ecdsa(
    arg: &SignWithEcdsaArgs,
) -> Result<SignWithEcdsaResult, SignCallError> {
    let cycles = cost_sign_with_ecdsa(arg)?;
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "sign_with_ecdsa")
            .with_arg(arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

/// Gets a SEC1 encoded Schnorr public key for the given canister using the given derivation path.
///
/// **Bounded-wait call**
///
/// See [IC method `schnorr_public_key`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-schnorr_public_key).
pub async fn schnorr_public_key(arg: &SchnorrPublicKeyArgs) -> CallResult<SchnorrPublicKeyResult> {
    Ok(
        Call::bounded_wait(Principal::management_canister(), "schnorr_public_key")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Calculates the cost of Schnorr signanature with the given [`SignWithSchnorrArgs`].
///
/// [`sign_with_schnorr`] invokes this method internally and attaches the required cycles to the call.
///
/// # Note
///
/// Alternatively, [`api::cost_sign_with_schnorr`][ic0_cost_sign_with_schnorr] takes the numeric representation of the algorithm.
pub fn cost_sign_with_schnorr(arg: &SignWithSchnorrArgs) -> Result<u128, SignCostError> {
    ic0_cost_sign_with_schnorr(&arg.key_id.name, arg.key_id.algorithm.into())
}

/// Gets a new Schnorr signature of the given message with a user-specified amount of cycles.
///
/// **Unbounded-wait call**
///
/// The signature can be separately verified against a derived Schnorr public key.
///
/// See [IC method `sign_with_schnorr`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_schnorr).
///
/// # Errors
///
/// This method returns an error of type [`SignCallError`].
///
/// The signature cost calculation may fail before the inter-canister call is made, resulting in a [`SignCallError::SignCostError`].
///
/// Since the call argument is constructed as [`SignWithSchnorrArgs`], the `algorithm` field is guaranteed to be valid.
/// Therefore, [`SignCostError::InvalidCurveOrAlgorithm`] should not occur. If it does, it is likely an issue with the IC. Please report it.
///
/// # Note
///
/// Signature costs cycles which varies for different algorithms and key names.
/// This method attaches the required cycles (detemined by [`cost_sign_with_schnorr`]) to the call.
///
/// Check [Threshold signatures](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#api-fees) for more details.
pub async fn sign_with_schnorr(
    arg: &SignWithSchnorrArgs,
) -> Result<SignWithSchnorrResult, SignCallError> {
    let cycles = cost_sign_with_schnorr(arg)?;
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "sign_with_schnorr")
            .with_arg(arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

/// Gets a VetKD public key.
///
/// **Bounded-wait call**
///
/// As of 2025-05-01, the vetKD feature is not yet available on the IC mainnet.
/// The lastest PocketIC with the `with_nonmainnet_features(true)` flag can be used to test it.
///
/// See [IC method `vetkd_public_key`](https://github.com/dfinity/portal/pull/3763).
///
/// Later, the description will be available in [the interface spec](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-vetkd_public_key).
pub async fn vetkd_public_key(arg: &VetKDPublicKeyArgs) -> CallResult<VetKDPublicKeyResult> {
    Ok(
        Call::bounded_wait(Principal::management_canister(), "vetkd_public_key")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Calculates the cost of VetKD key derivation with the given [`VetKDDeriveKeyArgs`].
///
/// [`vetkd_derive_key`] invokes this method internally and attaches the required cycles to the call.
///
/// # Note
///
/// Alternatively, [`api::cost_vetkd_derive_key`][ic0_cost_vetkd_derive_key] takes the numeric representation of the algorithm.
pub fn cost_vetkd_derive_key(arg: &VetKDDeriveKeyArgs) -> Result<u128, SignCostError> {
    ic0_cost_vetkd_derive_key(&arg.key_id.name, arg.key_id.curve.into())
}

/// Derives a key from the given input.
///
/// **Unbounded-wait call**
///
/// The returned encrypted key can be separately decrypted using the private secret key corresponding to the transport public key provided in the request, and the derivation correctness can be verified against the input and context provided in the request. See the [`ic_vetkeys` frontend library](https://github.com/dfinity/vetkd-devkit/tree/main/frontend/ic_vetkeys) for more details.
///
/// As of 2025-05-01, the vetKD feature is not yet available on the IC mainnet.
/// The lastest PocketIC with the `with_nonmainnet_features(true)` flag can be used to test it.
///
/// See [IC method `vetkd_derive_key`](https://github.com/dfinity/portal/pull/3763) for the API specification.
///
/// Later, the description will be available in [the interface spec](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-vetkd_derive_key).
///
/// # Errors
///
/// This method returns an error of type [`SignCallError`].
///
/// The signature cost calculation may fail before the inter-canister call is made, resulting in a [`SignCallError::SignCostError`].
///
/// Since the call argument is constructed as [`VetKDDeriveKeyArgs`], the `curve` field is guaranteed to be valid.
/// Therefore, [`SignCostError::InvalidCurveOrAlgorithm`] should not occur. If it does, it is likely an issue with the IC. Please report it.
///
/// # Note
///
/// VetKD key derivation costs cycles which varies for different algorithms and key names.
/// This method attaches the required cycles (detemined by [`cost_vetkd_derive_key`]) to the call.
///
/// Check [Threshold signatures](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#api-fees) for more details.
pub async fn vetkd_derive_key(
    arg: &VetKDDeriveKeyArgs,
) -> Result<VetKDDeriveKeyResult, SignCallError> {
    let cycles = cost_vetkd_derive_key(arg)?;
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "vetkd_derive_key")
            .with_arg(arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

/// Gets a time series of subnet's node metrics.
///
/// **Bounded-wait call**
///
/// See [IC method `node_metrics_history`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-node_metrics_history).
pub async fn node_metrics_history(
    arg: &NodeMetricsHistoryArgs,
) -> CallResult<NodeMetricsHistoryResult> {
    Ok(
        Call::bounded_wait(Principal::management_canister(), "node_metrics_history")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Gets the metadata about a subnet.
///
/// **Bounded-wait call**
///
/// See [IC method `subnet_info`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-subnet_info).
pub async fn subnet_info(arg: &SubnetInfoArgs) -> CallResult<SubnetInfoResult> {
    Ok(
        Call::bounded_wait(Principal::management_canister(), "subnet_info")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Creates a new canister with specified amount of cycles balance.
///
/// **Unbounded-wait call**
///
/// # Note
///
/// This method is only available in local development instances.
///
/// See [IC method `provisional_create_canister_with_cycles`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-provisional_create_canister_with_cycles).
pub async fn provisional_create_canister_with_cycles(
    arg: &ProvisionalCreateCanisterWithCyclesArgs,
) -> CallResult<ProvisionalCreateCanisterWithCyclesResult> {
    let complete_arg = ProvisionalCreateCanisterWithCyclesArgsComplete {
        amount: arg.amount.clone(),
        settings: arg.settings.clone(),
        specified_id: arg.specified_id,
        sender_canister_version: Some(canister_version()),
    };
    Ok(Call::unbounded_wait(
        Principal::management_canister(),
        "provisional_create_canister_with_cycles",
    )
    .with_arg(&complete_arg)
    .await?
    .candid()?)
}

/// Argument type of [`provisional_create_canister_with_cycles`].
///
/// # Note
///
/// This type is a reduced version of [`ic_management_canister_types::ProvisionalCreateCanisterWithCyclesArgs`].
///
/// The `sender_canister_version` field is removed as it is set automatically in [`provisional_create_canister_with_cycles`].
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
}

/// Adds cycles to a canister.
///
/// **Unbounded-wait call**
///
/// # Note
///
/// This method is only available in local development instances.
///
/// See [IC method `provisional_top_up_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-provisional_top_up_canister).
pub async fn provisional_top_up_canister(arg: &ProvisionalTopUpCanisterArgs) -> CallResult<()> {
    Ok(Call::unbounded_wait(
        Principal::management_canister(),
        "provisional_top_up_canister",
    )
    .with_arg(arg)
    .await?
    .candid()?)
}

/// Takes a snapshot of the specified canister.
///
/// **Unbounded-wait call**
///
/// A snapshot consists of the wasm memory, stable memory, certified variables, wasm chunk store and wasm binary.
///
/// See [IC method `take_canister_snapshot`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-take_canister_snapshot).
pub async fn take_canister_snapshot(
    arg: &TakeCanisterSnapshotArgs,
) -> CallResult<TakeCanisterSnapshotResult> {
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "take_canister_snapshot")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Loads a snapshot onto the canister.
///
/// **Unbounded-wait call**
///
/// It fails if no snapshot with the specified `snapshot_id` can be found.
///
/// See [IC method `load_canister_snapshot`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-load_canister_snapshot).
pub async fn load_canister_snapshot(arg: &LoadCanisterSnapshotArgs) -> CallResult<()> {
    let complete_arg = LoadCanisterSnapshotArgsComplete {
        canister_id: arg.canister_id,
        snapshot_id: arg.snapshot_id.clone(),
        sender_canister_version: Some(canister_version()),
    };
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "load_canister_snapshot")
            .with_arg(&complete_arg)
            .await?
            .candid()?,
    )
}

/// Argument type of [`load_canister_snapshot`].
///
/// # Note
///
/// This type is a reduced version of [`ic_management_canister_types::LoadCanisterSnapshotArgs`].
///
/// The `sender_canister_version` field is removed as it is set automatically in [`load_canister_snapshot`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct LoadCanisterSnapshotArgs {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// ID of the snapshot to be loaded.
    pub snapshot_id: SnapshotId,
}

/// Reads metadata of a snapshot of a canister.
///
/// **Bounded-wait call**
///
/// See [IC method `read_canister_snapshot_metadata`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-read_canister_snapshot_metadata).
pub async fn read_canister_snapshot_metadata(
    arg: &ReadCanisterSnapshotMetadataArgs,
) -> CallResult<ReadCanisterSnapshotMetadataResult> {
    Ok(Call::bounded_wait(
        Principal::management_canister(),
        "read_canister_snapshot_metadata",
    )
    .with_arg(arg)
    .await?
    .candid()?)
}

/// Reads data of a snapshot of a canister.
///
/// **Bounded-wait call**
///
/// See [IC method `read_canister_snapshot_data`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-read_canister_snapshot_data).
pub async fn read_canister_snapshot_data(
    arg: &ReadCanisterSnapshotDataArgs,
) -> CallResult<ReadCanisterSnapshotDataResult> {
    Ok(Call::bounded_wait(
        Principal::management_canister(),
        "read_canister_snapshot_data",
    )
    .with_arg(arg)
    .await?
    .candid()?)
}

/// Creates a snapshot of that canister by uploading the snapshot's metadata.
///
/// **Bounded-wait call**
///
/// See [IC method `upload_canister_snapshot_metadata`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-upload_canister_snapshot_metadata).
pub async fn upload_canister_snapshot_metadata(
    arg: &UploadCanisterSnapshotMetadataArgs,
) -> CallResult<UploadCanisterSnapshotMetadataResult> {
    Ok(Call::bounded_wait(
        Principal::management_canister(),
        "upload_canister_snapshot_metadata",
    )
    .with_arg(arg)
    .await?
    .candid()?)
}

/// Uploads data to a snapshot of that canister.
///
/// **Bounded-wait call**
///
/// See [IC method `upload_canister_snapshot_data`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-upload_canister_snapshot_data).
pub async fn upload_canister_snapshot_data(arg: &UploadCanisterSnapshotDataArgs) -> CallResult<()> {
    Ok(Call::bounded_wait(
        Principal::management_canister(),
        "upload_canister_snapshot_data",
    )
    .with_arg(arg)
    .await?
    .candid()?)
}

/// Lists the snapshots of the canister.
///
/// **Bounded-wait call**
///
/// See [IC method `list_canister_snapshots`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-list_canister_snapshots).
pub async fn list_canister_snapshots(
    arg: &ListCanisterSnapshotsArgs,
) -> CallResult<ListCanisterSnapshotsResult> {
    Ok(
        Call::bounded_wait(Principal::management_canister(), "list_canister_snapshots")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

/// Deletes a specified snapshot that belongs to an existing canister.
///
/// **Unbounded-wait call**
///
/// An error will be returned if the snapshot is not found.
///
/// See [IC method `delete_canister_snapshot`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-delete_canister_snapshot).
pub async fn delete_canister_snapshot(arg: &DeleteCanisterSnapshotArgs) -> CallResult<()> {
    Ok(
        Call::unbounded_wait(Principal::management_canister(), "delete_canister_snapshot")
            .with_arg(arg)
            .await?
            .candid()?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The cost parameter record must round-trip through Candid, since it is handed to the
    /// system API as an encoded blob.
    #[test]
    fn cost_args_candid_round_trip() {
        for outcall_type in [
            None,
            Some(HttpOutcallType::FullyReplicated(Reserved)),
            Some(HttpOutcallType::NonReplicated(Reserved)),
            Some(HttpOutcallType::Flexible(None)),
            Some(HttpOutcallType::Flexible(Some(ReplicationCounts {
                min_responses: 2,
                max_responses: 3,
                total_requests: 3,
            }))),
        ] {
            let args = CostHttpRequestV2Args {
                request_bytes: 1,
                http_roundtrip_time_ms: 2,
                raw_response_bytes: 3,
                transformed_response_bytes: 4,
                transform_instructions: 5,
                outcall_type,
            };
            let bytes = candid::encode_one(&args).unwrap();
            let decoded: CostHttpRequestV2Args = candid::decode_one(&bytes).unwrap();
            assert_eq!(args, decoded);
        }
    }

    /// An unset expectation must fall back to the maximum the outcall could consume.
    #[test]
    fn reservation_defaults_to_the_maxima() {
        let (roundtrip, raw, transformed, instructions) =
            Reservation::default().resolve(None, None, true);
        assert_eq!(roundtrip, MAX_ROUNDTRIP_TIME_MS);
        assert_eq!(raw, MAX_RESPONSE_BYTES_LIMIT);
        assert_eq!(
            transformed,
            MAX_RESPONSE_BYTES_LIMIT + CANDID_OVERHEAD_RESERVE_BYTES
        );
        assert_eq!(instructions, MAX_TRANSFORM_INSTRUCTIONS);
    }

    /// `max_response_bytes` bounds both response sizes when they are not given explicitly.
    #[test]
    fn reservation_defaults_follow_max_response_bytes() {
        let (_, raw, transformed, _) = Reservation::default().resolve(Some(4_000), None, true);
        assert_eq!(raw, 4_000);
        assert_eq!(transformed, 4_000 + CANDID_OVERHEAD_RESERVE_BYTES);
    }

    /// An explicit expectation must win over the default.
    #[test]
    fn reservation_uses_supplied_values() {
        let reservation = Reservation {
            roundtrip_time_ms: Some(300),
            raw_response_bytes: Some(1_000),
            transformed_response_bytes: Some(900),
            transform_instructions: Some(1_000_000),
        };
        assert_eq!(
            reservation.resolve(Some(4_000), None, true),
            (300, 1_000, 900, 1_000_000)
        );
    }

    /// The builder must always ask for pricing version 2, including via `from_args`.
    #[test]
    fn builder_always_selects_pricing_version_2() {
        assert_eq!(
            HttpRequest::new("https://example.com")
                .args()
                .pricing_version,
            Some(2)
        );
        let args = HttpRequestArgs {
            url: "https://example.com".to_string(),
            pricing_version: Some(1),
            ..Default::default()
        };
        assert_eq!(HttpRequest::from_args(args).args().pricing_version, Some(2));
    }

    /// `non_replicated` must be reflected in both the args and the priced outcall type.
    #[test]
    fn non_replicated_is_recorded() {
        let req = HttpRequest::new("https://example.com").non_replicated();
        assert_eq!(req.args().is_replicated, Some(false));
    }

    /// The flexible builder records the replication counts, and leaves them unset so the
    /// system picks its defaults when the caller does not.
    #[test]
    fn flexible_replication_is_recorded() {
        assert_eq!(
            FlexibleHttpRequest::new("https://example.com")
                .args()
                .replication,
            None
        );
        let counts = ReplicationCounts {
            min_responses: 2,
            max_responses: 3,
            total_requests: 3,
        };
        assert_eq!(
            FlexibleHttpRequest::new("https://example.com")
                .with_replication(counts.clone())
                .args()
                .replication,
            Some(counts)
        );
    }

    /// The flexible default for `transformed_response_bytes` is the block budget split across
    /// the responses that have to be delivered together.
    #[test]
    fn flexible_default_cap_divides_the_block_budget() {
        let counts = ReplicationCounts {
            min_responses: 9,
            max_responses: 13,
            total_requests: 13,
        };
        let cap = flexible_transformed_default_cap(Some(&counts)).unwrap();
        assert_eq!(cap, MAX_FLEXIBLE_RESULT_BYTES.div_ceil(9));
        // The rounding only matters when every node's response has to be delivered, since the
        // reserve prices `total_requests` of them. Rounding down is 8 bytes short here.
        let deterministic = ReplicationCounts {
            min_responses: 9,
            max_responses: 9,
            total_requests: 9,
        };
        let cap = flexible_transformed_default_cap(Some(&deterministic)).unwrap();
        assert!(9 * cap >= MAX_FLEXIBLE_RESULT_BYTES);
        // Rounding down instead would have left the reserve 8 bytes short of a full block.
        assert_eq!(9 * (cap - 1), MAX_FLEXIBLE_RESULT_BYTES - 8);
    }

    /// Nothing bounds the response size when no response is delivered, or when no minimum is
    /// required, so those must not divide by `min_responses`.
    #[test]
    fn flexible_default_cap_is_absent_when_nothing_must_be_delivered() {
        // Fire-and-forget.
        assert_eq!(
            flexible_transformed_default_cap(Some(&ReplicationCounts {
                min_responses: 0,
                max_responses: 0,
                total_requests: 3,
            })),
            None
        );
        // No minimum, but responses may still arrive.
        assert_eq!(
            flexible_transformed_default_cap(Some(&ReplicationCounts {
                min_responses: 0,
                max_responses: 3,
                total_requests: 3,
            })),
            None
        );
    }

    /// The cap applies to the fallback only, and never raises it.
    #[test]
    fn flexible_cap_bounds_the_default_but_not_an_explicit_expectation() {
        let cap = Some(233_016);
        // Unset: the worst case is cut down to the cap.
        let (_, _, transformed, _) = Reservation::default().resolve(None, cap, true);
        assert_eq!(transformed, 233_016);
        // A small `max_response_bytes` already sits below the cap, so nothing changes.
        let (_, _, transformed, _) = Reservation::default().resolve(Some(4_000), cap, true);
        assert_eq!(transformed, 4_000 + CANDID_OVERHEAD_RESERVE_BYTES);
        // An explicit expectation is used as given, above the cap or below it.
        let reservation = Reservation {
            transformed_response_bytes: Some(1_000_000),
            ..Default::default()
        };
        let (_, _, transformed, _) = reservation.resolve(None, cap, true);
        assert_eq!(transformed, 1_000_000);
    }

    /// Without a transform the system never runs one, so nothing is reserved for it.
    #[test]
    fn no_transform_reserves_no_instructions() {
        let (.., instructions) = Reservation::default().resolve(None, None, false);
        assert_eq!(instructions, 0);
        let (.., instructions) = Reservation::default().resolve(None, None, true);
        assert_eq!(instructions, MAX_TRANSFORM_INSTRUCTIONS);
        // An explicit expectation still wins, transform or not.
        let reservation = Reservation {
            transform_instructions: Some(7),
            ..Default::default()
        };
        let (.., instructions) = reservation.resolve(None, None, false);
        assert_eq!(instructions, 7);
    }

    /// `request_bytes` counts the URL, the headers, the body and the transform.
    #[test]
    fn request_bytes_counts_every_variable_part() {
        let headers = vec![HttpHeader {
            name: "ab".to_string(),
            value: "cde".to_string(),
        }];
        let body = vec![0u8; 7];
        assert_eq!(
            request_bytes("https://x", &headers, Some(&body), None),
            (9 + 2 + 3 + 7) as u64
        );
    }
}
