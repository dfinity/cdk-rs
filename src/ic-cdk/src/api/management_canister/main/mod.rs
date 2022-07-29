use candid::Principal;
use ic_cdk::api::call::{call, CallResult};

mod types;
pub use types::*;

/// create_canister : (record {
///   settings : opt canister_settings
/// }) -> (record {canister_id : canister_id});
pub async fn create_canister(arg: CreateCanisterArgument) -> CallResult<(CanisterIdRecord,)> {
    call(Principal::management_canister(), "create_canister", (arg,)).await
}

/// update_settings : (record {
///   canister_id : principal;
///   settings : canister_settings
/// }) -> ();
pub async fn update_settings(arg: UpdateSettingsArgument) -> CallResult<()> {
    call(Principal::management_canister(), "update_settings", (arg,)).await
}

/// install_code : (record {
///   mode : variant {install; reinstall; upgrade};
///   canister_id : canister_id;
///   wasm_module : wasm_module;
///   arg : blob;
/// }) -> ();
pub async fn install_code(arg: InstallCodeArgument) -> CallResult<()> {
    call(Principal::management_canister(), "install_code", (arg,)).await
}

/// uninstall_code : (record {canister_id : canister_id}) -> ();
pub async fn uninstall_code(arg: CanisterIdRecord) -> CallResult<()> {
    call(Principal::management_canister(), "uninstall_code", (arg,)).await
}

/// start_canister : (record {canister_id : canister_id}) -> ();
pub async fn start_canister(arg: CanisterIdRecord) -> CallResult<()> {
    call(Principal::management_canister(), "start_canister", (arg,)).await
}

/// stop_canister : (record {canister_id : canister_id}) -> ();
pub async fn stop_canister(arg: CanisterIdRecord) -> CallResult<()> {
    call(Principal::management_canister(), "stop_canister", (arg,)).await
}

/// canister_status : (record {canister_id : canister_id}) -> (record {
///   status : variant { running; stopping; stopped };
///   settings: definite_canister_settings;
///   module_hash: opt blob;
///   memory_size: nat;
///   cycles: nat;
///   idle_cycles_burned_per_day: nat;
/// });
pub async fn canister_status(arg: CanisterIdRecord) -> CallResult<(CanisterStatusReply,)> {
    call(Principal::management_canister(), "canister_status", (arg,)).await
}

/// delete_canister : (record {canister_id : canister_id}) -> ();
pub async fn delete_canister(arg: CanisterIdRecord) -> CallResult<()> {
    call(Principal::management_canister(), "delete_canister", (arg,)).await
}

/// deposit_cycles : (record {canister_id : canister_id}) -> ();
pub async fn deposit_cycles(arg: CanisterIdRecord) -> CallResult<()> {
    call(Principal::management_canister(), "deposit_cycles", (arg,)).await
}

/// raw_rand : () -> (blob);
pub async fn raw_rand() -> CallResult<(Vec<u8>,)> {
    call(Principal::management_canister(), "raw_rand", ()).await
}
