use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::call::{call, CallResult};
use serde::Serialize;

pub type CanisterId = Principal;

// TODO: what common traits should we derive for these structs?
#[derive(Default, Clone, CandidType, Deserialize, Debug)]
pub struct CanisterSettings {
    pub controllers: Option<Vec<Principal>>,
    pub compute_allocation: Option<Nat>,
    pub memory_allocation: Option<Nat>,
    pub freezing_threshold: Option<Nat>,
}

#[derive(Default, Clone, CandidType, Deserialize, Debug)]
pub struct CreateCanisterArgument {
    pub settings: Option<CanisterSettings>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct CreateCanisterReturn {
    pub canister_id: CanisterId,
}

/// create_canister : (record {
///   settings : opt canister_settings
/// }) -> (record {canister_id : canister_id});
pub async fn create_canister(arg: CreateCanisterArgument) -> CallResult<(CreateCanisterReturn,)> {
    call(Principal::management_canister(), "create_canister", (arg,)).await
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct UpdateSettingsArgument {
    pub canister_id: CanisterId,
    pub settings: CanisterSettings,
}

/// update_settings : (record {
///   canister_id : principal;
///   settings : canister_settings
/// }) -> ();
pub async fn update_settings(arg: UpdateSettingsArgument) -> CallResult<()> {
    call(Principal::management_canister(), "update_settings", (arg,)).await
}

/// The mode with which a canister is installed.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Eq, Hash, CandidType, Copy)]
pub enum CanisterInstallMode {
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

pub type WasmModule = Vec<u8>;

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct InstallCodeArgument {
    pub mode: CanisterInstallMode,
    pub canister_id: CanisterId,
    pub wasm_module: WasmModule,
    pub arg: Vec<u8>,
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

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct CanisterIdArg {
    pub canister_id: CanisterId,
}

/// uninstall_code : (record {canister_id : canister_id}) -> ();
pub async fn uninstall_code(arg: CanisterIdArg) -> CallResult<()> {
    call(Principal::management_canister(), "uninstall_code", (arg,)).await
}

/// start_canister : (record {canister_id : canister_id}) -> ();
pub async fn start_canister(arg: CanisterIdArg) -> CallResult<()> {
    call(Principal::management_canister(), "start_canister", (arg,)).await
}

/// stop_canister : (record {canister_id : canister_id}) -> ();
pub async fn stop_canister(arg: CanisterIdArg) -> CallResult<()> {
    call(Principal::management_canister(), "stop_canister", (arg,)).await
}

/// raw_rand : () -> (blob);
pub async fn raw_rand() -> CallResult<(Vec<u8>,)> {
    call(Principal::management_canister(), "raw_rand", ()).await
}
