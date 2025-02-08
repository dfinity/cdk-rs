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
use crate::call::{Call, CallResult};
use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

// Re-export types from `ic_management_canister_types` crate.
pub use ic_management_canister_types::{
    CanisterId, CanisterInfoArgs, CanisterInfoResult, CanisterInstallMode, CanisterSettings,
    CanisterStatusArgs, CanisterStatusResult, CanisterStatusType, Change, ChangeDetails,
    ChangeOrigin, ChunkHash, ClearChunkStoreArgs, CodeDeploymentMode, CodeDeploymentRecord,
    ControllersChangeRecord, CreateCanisterResult, CreationRecord, DefiniteCanisterSettings,
    DeleteCanisterArgs, DeleteCanisterSnapshotArgs, DepositCyclesArgs, EcdsaCurve, EcdsaKeyId,
    EcdsaPublicKeyArgs, EcdsaPublicKeyResult, FromCanisterRecord, FromUserRecord, HttpHeader,
    HttpMethod, HttpRequestArgs, HttpRequestResult, ListCanisterSnapshotsArgs,
    ListCanisterSnapshotsReturn, LoadSnapshotRecord, LogVisibility, NodeMetrics,
    NodeMetricsHistoryArgs, NodeMetricsHistoryRecord, NodeMetricsHistoryResult,
    ProvisionalCreateCanisterWithCyclesResult, ProvisionalTopUpCanisterArgs, QueryStats,
    SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgs, SchnorrPublicKeyResult,
    SignWithEcdsaArgs, SignWithEcdsaResult, SignWithSchnorrArgs, SignWithSchnorrResult, Snapshot,
    SnapshotId, StartCanisterArgs, StopCanisterArgs, StoredChunksArgs, StoredChunksResult,
    SubnetInfoArgs, SubnetInfoResult, TakeCanisterSnapshotArgs, TakeCanisterSnapshotReturn,
    TransformArgs, TransformContext, TransformFunc, UpgradeFlags, UploadChunkArgs,
    UploadChunkResult, WasmMemoryPersistence, WasmModule,
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

/// Registers a new canister and get its canister id.
///
/// See [IC method `create_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister).
///
/// This call requires cycles payment. The required cycles varies according to the subnet size (number of nodes).
/// Check [Gas and cycles cost](https://internetcomputer.org/docs/current/developer-docs/gas-cost) for more details.
pub async fn create_canister(
    arg: &CreateCanisterArgs,
    cycles: u128,
) -> CallResult<CreateCanisterResult> {
    let complete_arg = CreateCanisterArgsComplete {
        settings: arg.settings.clone(),
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "create_canister")
        .with_arg(&complete_arg)
        .with_guaranteed_response()
        .with_cycles(cycles)
        .call()
        .await
}

/// Argument type of [`create_canister`].
///
/// # Note
///
/// This type is a reduced version of [`ic_management_canister_types::CreateCanisterArgs`].
///
/// The `sender_canister_version` field is removed as it is set automatically in [`create_canister`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct CreateCanisterArgs {
    /// See [`CanisterSettings`].
    pub settings: Option<CanisterSettings>,
}

/// Updates the settings of a canister.
///
/// See [IC method `update_settings`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-update_settings).
pub async fn update_settings(arg: &UpdateSettingsArgs) -> CallResult<()> {
    let complete_arg = UpdateSettingsArgsComplete {
        canister_id: arg.canister_id,
        settings: arg.settings.clone(),
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "update_settings")
        .with_arg(&complete_arg)
        .call()
        .await
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
    /// See [CanisterSettings].
    pub settings: CanisterSettings,
}

/// Uploads a chunk to the chunk store of a canister.
///
/// See [IC method `upload_chunk`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-upload_chunk).
pub async fn upload_chunk(arg: &UploadChunkArgs) -> CallResult<UploadChunkResult> {
    Call::new(Principal::management_canister(), "upload_chunk")
        .with_arg(arg)
        .call()
        .await
}

/// Clears the chunk store of a canister.
///
/// See [IC method `clear_chunk_store`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-clear_chunk_store).
pub async fn clear_chunk_store(arg: &ClearChunkStoreArgs) -> CallResult<()> {
    Call::new(Principal::management_canister(), "clear_chunk_store")
        .with_arg(arg)
        .call()
        .await
}

/// Gets the hashes of all chunks stored in the chunk store of a canister.
///
/// See [IC method `stored_chunks`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-stored_chunks).
pub async fn stored_chunks(arg: &StoredChunksArgs) -> CallResult<StoredChunksResult> {
    Call::new(Principal::management_canister(), "stored_chunks")
        .with_arg(arg)
        .call()
        .await
}

/// Installs code into a canister.
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
    Call::new(Principal::management_canister(), "install_code")
        .with_arg(&complete_arg)
        .call()
        .await
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
    /// See [CanisterInstallMode].
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
    Call::new(Principal::management_canister(), "install_chunked_code")
        .with_arg(&complete_arg)
        .call()
        .await
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

/// Removes a canister's code and state, making the canister empty again.
///
/// See [IC method `uninstall_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-uninstall_code).
pub async fn uninstall_code(arg: &UninstallCodeArgs) -> CallResult<()> {
    let complete_arg = UninstallCodeArgsComplete {
        canister_id: arg.canister_id,
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "uninstall_code")
        .with_arg(&complete_arg)
        .call()
        .await
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
/// See [IC method `start_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-start_canister).
pub async fn start_canister(arg: &StartCanisterArgs) -> CallResult<()> {
    Call::new(Principal::management_canister(), "start_canister")
        .with_arg(arg)
        .call()
        .await
}

/// Stops a canister.
///
/// See [IC method `stop_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-stop_canister).
pub async fn stop_canister(arg: &StopCanisterArgs) -> CallResult<()> {
    Call::new(Principal::management_canister(), "stop_canister")
        .with_arg(arg)
        .call()
        .await
}

/// Gets status information about the canister.
///
/// See [IC method `canister_status`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_status).
pub async fn canister_status(arg: &CanisterStatusArgs) -> CallResult<CanisterStatusResult> {
    Call::new(Principal::management_canister(), "canister_status")
        .with_arg(arg)
        .call()
        .await
}

/// Gets public information about the canister.
///
/// See [IC method `canister_info`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_info).
pub async fn canister_info(arg: &CanisterInfoArgs) -> CallResult<CanisterInfoResult> {
    Call::new(Principal::management_canister(), "canister_info")
        .with_arg(arg)
        .call()
        .await
}

/// Deletes a canister.
///
/// See [IC method `delete_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-delete_canister).
pub async fn delete_canister(arg: &DeleteCanisterArgs) -> CallResult<()> {
    Call::new(Principal::management_canister(), "delete_canister")
        .with_arg(arg)
        .call()
        .await
}

/// Deposits cycles to a canister.
///
/// See [IC method `deposit_cycles`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-deposit_cycles).
pub async fn deposit_cycles(arg: &DepositCyclesArgs, cycles: u128) -> CallResult<()> {
    Call::new(Principal::management_canister(), "deposit_cycles")
        .with_arg(arg)
        .with_guaranteed_response()
        .with_cycles(cycles)
        .call()
        .await
}

// Gets 32 pseudo-random bytes.
///
/// See [IC method `raw_rand`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-raw_rand).
pub async fn raw_rand() -> CallResult<Vec<u8>> {
    Call::new(Principal::management_canister(), "raw_rand")
        .call()
        .await
}

/// Makes an HTTP request to a given URL and return the HTTP response, possibly after a transformation.
///
/// See [IC method `http_request`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-http_request).
///
/// This call requires cycles payment. The required cycles is a function of the request size and max_response_bytes.
/// Check [HTTPS outcalls cycles cost](https://internetcomputer.org/docs/current/developer-docs/gas-cost#https-outcalls) for more details.
pub async fn http_request(arg: &HttpRequestArgs, cycles: u128) -> CallResult<HttpRequestResult> {
    Call::new(Principal::management_canister(), "http_request")
        .with_arg(arg)
        .with_guaranteed_response()
        .with_cycles(cycles)
        .call()
        .await
}

/// Constructs a [`TransformContext`] from a query method name and context.
/// The principal is assumed to be the ID of current canister.
pub fn transform_context_from_query(
    candid_function_name: String,
    context: Vec<u8>,
) -> TransformContext {
    TransformContext {
        context,
        function: TransformFunc(candid::Func {
            method: candid_function_name,
            principal: crate::api::canister_self(),
        }),
    }
}

#[cfg(feature = "transform-closure")]
mod transform_closure {
    use super::{
        http_request, transform_context_from_query, CallResult, HttpRequestArgs, HttpRequestResult,
        Principal, TransformArgs,
    };
    use candid::{decode_one, encode_one};
    use slotmap::{DefaultKey, Key, KeyData, SlotMap};
    use std::cell::RefCell;

    thread_local! {
        #[allow(clippy::type_complexity)]
        static TRANSFORMS: RefCell<SlotMap<DefaultKey, Box<dyn FnOnce(HttpRequestResult) -> HttpRequestResult>>> = RefCell::default();
    }

    #[export_name = "canister_query <ic-cdk internal> http_transform"]
    extern "C" fn http_transform() {
        use crate::api::{msg_arg_data, msg_caller, msg_reply};
        if msg_caller() != Principal::management_canister() {
            crate::trap("This function is internal to ic-cdk and should not be called externally.");
        }
        crate::setup();
        let arg_bytes = msg_arg_data();
        let transform_args: TransformArgs = decode_one(&arg_bytes).unwrap();
        let int = u64::from_be_bytes(transform_args.context[..].try_into().unwrap());
        let key = DefaultKey::from(KeyData::from_ffi(int));
        let func = TRANSFORMS.with(|transforms| transforms.borrow_mut().remove(key));
        let Some(func) = func else {
            crate::trap(format!("Missing transform function for request {int}"));
        };
        let transformed = func(transform_args.response);
        let encoded = encode_one(transformed).unwrap();
        msg_reply(encoded);
    }

    /// Make an HTTP request to a given URL and return the HTTP response, after a transformation.
    ///
    /// Do not set the `transform` field of `arg`. To use a Candid function, call [`http_request`] instead.
    ///
    /// See [IC method `http_request`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-http_request).
    ///
    /// This call requires cycles payment. The required cycles is a function of the request size and max_response_bytes.
    /// Check [Gas and cycles cost](https://internetcomputer.org/docs/current/developer-docs/gas-cost) for more details.
    #[cfg_attr(docsrs, doc(cfg(feature = "transform-closure")))]
    pub async fn http_request_with_closure(
        arg: &HttpRequestArgs,
        cycles: u128,
        transform_func: impl FnOnce(HttpRequestResult) -> HttpRequestResult + 'static,
    ) -> CallResult<HttpRequestResult> {
        assert!(
            arg.transform.is_none(),
            "The `transform` field in `HttpRequestArgs` must be `None` when using a closure"
        );
        let transform_func = Box::new(transform_func) as _;
        let key = TRANSFORMS.with(|transforms| transforms.borrow_mut().insert(transform_func));
        struct DropGuard(DefaultKey);
        impl Drop for DropGuard {
            fn drop(&mut self) {
                TRANSFORMS.with(|transforms| transforms.borrow_mut().remove(self.0));
            }
        }
        let key = DropGuard(key);
        let context = key.0.data().as_ffi().to_be_bytes().to_vec();
        let arg = HttpRequestArgs {
            transform: Some(transform_context_from_query(
                "<ic-cdk internal> http_transform".to_string(),
                context,
            )),
            ..arg.clone()
        };
        http_request(&arg, cycles).await
    }
}

#[cfg(feature = "transform-closure")]
pub use transform_closure::http_request_with_closure;

/// Gets a SEC1 encoded ECDSA public key for the given canister using the given derivation path.
///
/// See [IC method `ecdsa_public_key`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-ecdsa_public_key).
pub async fn ecdsa_public_key(arg: &EcdsaPublicKeyArgs) -> CallResult<EcdsaPublicKeyResult> {
    Call::new(Principal::management_canister(), "ecdsa_public_key")
        .with_arg(arg)
        .call()
        .await
}

/// Gets a new ECDSA signature of the given message_hash that can be separately verified against a derived ECDSA public key.
///
/// See [IC method `sign_with_ecdsa`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_ecdsa).
///
/// This call requires cycles payment.
/// This method handles the cycles cost under the hood.
/// Check [Threshold signatures](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works) for more details.
pub async fn sign_with_ecdsa(arg: &SignWithEcdsaArgs) -> CallResult<SignWithEcdsaResult> {
    Call::new(Principal::management_canister(), "sign_with_ecdsa")
        .with_arg(arg)
        .with_guaranteed_response()
        .with_cycles(SIGN_WITH_ECDSA_FEE)
        .call()
        .await
}

/// https://internetcomputer.org/docs/current/references/t-sigs-how-it-works#fees-for-the-t-ecdsa-production-key
const SIGN_WITH_ECDSA_FEE: u128 = 26_153_846_153;

/// Gets a SEC1 encoded Schnorr public key for the given canister using the given derivation path.
///
/// See [IC method `schnorr_public_key`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-schnorr_public_key).
pub async fn schnorr_public_key(arg: &SchnorrPublicKeyArgs) -> CallResult<SchnorrPublicKeyResult> {
    Call::new(Principal::management_canister(), "schnorr_public_key")
        .with_arg(arg)
        .call()
        .await
}

/// Gets a new Schnorr signature of the given message that can be separately verified against a derived Schnorr public key.
///
/// See [IC method `sign_with_schnorr`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_schnorr).
///
/// This call requires cycles payment.
/// This method handles the cycles cost under the hood.
/// Check [Threshold signatures](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works) for more details.
pub async fn sign_with_schnorr(arg: &SignWithSchnorrArgs) -> CallResult<SignWithSchnorrResult> {
    Call::new(Principal::management_canister(), "sign_with_schnorr")
        .with_arg(arg)
        .with_guaranteed_response()
        .with_cycles(SIGN_WITH_SCHNORR_FEE)
        .call()
        .await
}

/// https://internetcomputer.org/docs/current/references/t-sigs-how-it-works/#fees-for-the-t-schnorr-production-key
const SIGN_WITH_SCHNORR_FEE: u128 = 26_153_846_153;

/// Gets a time series of subnet's node metrics.
///
/// See [IC method `node_metrics_history`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-node_metrics_history).
// ! The actual url ends with `ic-node-metrics-history` instead of `ic-node_metrics_history`.
// ! It will likely be changed to be consistent with the other methods soon.
pub async fn node_metrics_history(
    arg: &NodeMetricsHistoryArgs,
) -> CallResult<NodeMetricsHistoryResult> {
    Call::new(Principal::management_canister(), "node_metrics_history")
        .with_arg(arg)
        .call()
        .await
}

/// Gets the metadata about a subnet.
///
/// See [IC method `subnet_info`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-subnet_info).
// ! The actual url ends with `ic-subnet-info` instead of `ic-subnet_info`.
// ! It will likely be changed to be consistent with the other methods soon.
pub async fn subnet_info(arg: &SubnetInfoArgs) -> CallResult<SubnetInfoResult> {
    Call::new(Principal::management_canister(), "subnet_info")
        .with_arg(arg)
        .call()
        .await
}

/// Creates a new canister with specified amount of cycles balance.
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
    Call::new(
        Principal::management_canister(),
        "provisional_create_canister_with_cycles",
    )
    .with_arg(&complete_arg)
    .with_guaranteed_response()
    .call()
    .await
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

/// Add cycles to a canister.
///
/// This method is only available in local development instances.
///
/// See [IC method `provisional_top_up_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-provisional_top_up_canister).
pub async fn provisional_top_up_canister(arg: &ProvisionalTopUpCanisterArgs) -> CallResult<()> {
    Call::new(
        Principal::management_canister(),
        "provisional_top_up_canister",
    )
    .with_arg(arg)
    .with_guaranteed_response()
    .call()
    .await
}

/// Take a snapshot of the specified canister.
///
/// A snapshot consists of the wasm memory, stable memory, certified variables, wasm chunk store and wasm binary.
///
/// See [IC method `take_canister_snapshot`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-take_canister_snapshot).
pub async fn take_canister_snapshot(
    arg: &TakeCanisterSnapshotArgs,
) -> CallResult<TakeCanisterSnapshotReturn> {
    Call::new(Principal::management_canister(), "take_canister_snapshot")
        .with_arg(arg)
        .with_guaranteed_response()
        .call()
        .await
}

/// Load a snapshot onto the canister.
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
    Call::new(Principal::management_canister(), "load_canister_snapshot")
        .with_arg(&complete_arg)
        .with_guaranteed_response()
        .call()
        .await
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

/// List the snapshots of the canister.
///
/// See [IC method `list_canister_snapshots`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-list_canister_snapshots).
pub async fn list_canister_snapshots(
    arg: &ListCanisterSnapshotsArgs,
) -> CallResult<ListCanisterSnapshotsReturn> {
    Call::new(Principal::management_canister(), "list_canister_snapshots")
        .with_arg(arg)
        .call()
        .await
}

/// Delete a specified snapshot that belongs to an existing canister.
///
/// An error will be returned if the snapshot is not found.
///
/// See [IC method `delete_canister_snapshot`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-delete_canister_snapshot).
pub async fn delete_canister_snapshot(arg: &DeleteCanisterSnapshotArgs) -> CallResult<()> {
    Call::new(Principal::management_canister(), "delete_canister_snapshot")
        .with_arg(arg)
        .call()
        .await
}
