//! The main functionalities in [the IC management canister][1].
//!
//! Most of the functions are for managing canister lifecycle.
//! [raw_rand] is also included in this module.
//!
//! [1]: https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-management-canister

use candid::Principal;
use ic_cdk::api::canister_version;
use ic_cdk::prelude::*;

mod types;
pub use types::*;

/// Register a new canister and get its canister id.
///
/// See [IC method `create_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister).
///
/// This call requires cycles payment. The required cycles varies according to the subnet size (number of nodes).
/// Check [Gas and cycles cost](https://internetcomputer.org/docs/current/developer-docs/gas-cost) for more details.
pub async fn create_canister(
    arg: CreateCanisterArgument,
    cycles: u128,
) -> CallResult<(CanisterIdRecord,)> {
    let extended_arg = CreateCanisterArgumentExtended {
        settings: arg.settings,
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "create_canister")
        .with_guaranteed_response()
        .with_args((extended_arg,))
        .with_cycles(cycles)
        .call()
        .await
}

/// Update the settings of a canister.
///
/// See [IC method `update_settings`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-update_settings).
pub async fn update_settings(arg: UpdateSettingsArgument) -> CallResult<()> {
    let extended_arg = UpdateSettingsArgumentExtended {
        canister_id: arg.canister_id,
        settings: arg.settings,
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "update_settings")
        .with_guaranteed_response()
        .with_args((extended_arg,))
        .call()
        .await
}

/// See [IC method `upload_chunk`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-upload_chunk).
pub async fn upload_chunk(arg: UploadChunkArgument) -> CallResult<(ChunkHash,)> {
    Call::new(Principal::management_canister(), "upload_chunk")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// See [IC method `clear_chunk_store`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-clear_chunk_store).
pub async fn clear_chunk_store(arg: ClearChunkStoreArgument) -> CallResult<()> {
    Call::new(Principal::management_canister(), "clear_chunk_store")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// See [IC method `stored_chunks`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-stored_chunks).
pub async fn stored_chunks(arg: StoredChunksArgument) -> CallResult<(Vec<ChunkHash>,)> {
    Call::new(Principal::management_canister(), "stored_chunks")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// Install code into a canister.
///
/// See [IC method `install_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-install_code).
pub async fn install_code(arg: InstallCodeArgument) -> CallResult<()> {
    let extended_arg = InstallCodeArgumentExtended {
        mode: arg.mode,
        canister_id: arg.canister_id,
        wasm_module: arg.wasm_module,
        arg: arg.arg,
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "install_code")
        .with_guaranteed_response()
        .with_args((extended_arg,))
        .call()
        .await
}

/// Install code into a canister where the code has previously been uploaded in chunks.
///
/// See [IC method `install_chunked_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-install_chunked_code).
pub async fn install_chunked_code(arg: InstallChunkedCodeArgument) -> CallResult<()> {
    let extended_arg = InstallChunkedCodeArgumentExtended {
        mode: arg.mode,
        target_canister: arg.target_canister,
        store_canister: arg.store_canister,
        chunk_hashes_list: arg.chunk_hashes_list,
        wasm_module_hash: arg.wasm_module_hash,
        arg: arg.arg,
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "install_chunked_code")
        .with_guaranteed_response()
        .with_args((extended_arg,))
        .call()
        .await
}

/// Remove a canister's code and state, making the canister empty again.
///
/// See [IC method `uninstall_code`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-uninstall_code).
pub async fn uninstall_code(arg: CanisterIdRecord) -> CallResult<()> {
    let extended_arg = CanisterIdRecordExtended {
        canister_id: arg.canister_id,
        sender_canister_version: Some(canister_version()),
    };
    Call::new(Principal::management_canister(), "uninstall_code")
        .with_guaranteed_response()
        .with_args((extended_arg,))
        .call()
        .await
}

/// Start a canister if the canister status was `stopped` or `stopping`.
///
/// See [IC method `start_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-start_canister).
pub async fn start_canister(arg: CanisterIdRecord) -> CallResult<()> {
    Call::new(Principal::management_canister(), "start_canister")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// Stop a canister.
///
/// See [IC method `stop_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-stop_canister).
pub async fn stop_canister(arg: CanisterIdRecord) -> CallResult<()> {
    Call::new(Principal::management_canister(), "stop_canister")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// Get status information about the canister.
///
/// See [IC method `canister_status`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_status).
pub async fn canister_status(arg: CanisterIdRecord) -> CallResult<(CanisterStatusResponse,)> {
    Call::new(Principal::management_canister(), "canister_status")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// Delete a canister from the IC.
///
/// See [IC method `delete_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-delete_canister).
pub async fn delete_canister(arg: CanisterIdRecord) -> CallResult<()> {
    Call::new(Principal::management_canister(), "delete_canister")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// Deposit cycles into the specified canister.
///
/// Note that, beyond the argument as specified in the interface description,
/// there is a second parameter `cycles` which is the amount of cycles to be deposited.
///
/// See [IC method `deposit_cycles`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-deposit_cycles).
pub async fn deposit_cycles(arg: CanisterIdRecord, cycles: u128) -> CallResult<()> {
    Call::new(Principal::management_canister(), "deposit_cycles")
        .with_guaranteed_response()
        .with_args((arg,))
        .with_cycles(cycles)
        .call()
        .await
}

/// Get 32 pseudo-random bytes.
///
/// See [IC method `raw_rand`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-raw_rand).
pub async fn raw_rand() -> CallResult<(Vec<u8>,)> {
    Call::new(Principal::management_canister(), "raw_rand")
        .with_guaranteed_response()
        .with_args(())
        .call()
        .await
}

/// Get public information about the canister.
///
/// See [IC method `canister_info`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_info).
pub async fn canister_info(arg: CanisterInfoRequest) -> CallResult<(CanisterInfoResponse,)> {
    Call::new(Principal::management_canister(), "canister_info")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// Take a snapshot of the specified canister.
///
/// A snapshot consists of the wasm memory, stable memory, certified variables, wasm chunk store and wasm binary.
///
/// See [IC method `take_canister_snapshot`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-take_canister_snapshot).
pub async fn take_canister_snapshot(arg: TakeCanisterSnapshotArgs) -> CallResult<(Snapshot,)> {
    Call::new(Principal::management_canister(), "take_canister_snapshot")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// Load a snapshot onto the canister.
///
/// It fails if no snapshot with the specified `snapshot_id` can be found.
///
/// See [IC method `load_canister_snapshot`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-load_canister_snapshot).
pub async fn load_canister_snapshot(arg: LoadCanisterSnapshotArgs) -> CallResult<()> {
    Call::new(Principal::management_canister(), "load_canister_snapshot")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// List the snapshots of the canister.
///
/// See [IC method `list_canister_snapshots`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-list_canister_snapshots).
pub async fn list_canister_snapshots(arg: CanisterIdRecord) -> CallResult<(Vec<Snapshot>,)> {
    Call::new(Principal::management_canister(), "list_canister_snapshots")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}

/// Delete a specified snapshot that belongs to an existing canister.
///
/// An error will be returned if the snapshot is not found.
///
/// See [IC method `delete_canister_snapshot`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-delete_canister_snapshot).
pub async fn delete_canister_snapshot(arg: DeleteCanisterSnapshotArgs) -> CallResult<()> {
    Call::new(Principal::management_canister(), "delete_canister_snapshot")
        .with_guaranteed_response()
        .with_args((arg,))
        .call()
        .await
}
