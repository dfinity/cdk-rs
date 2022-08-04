use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

/// Canister ID is Principal.
pub type CanisterId = Principal;

// TODO: what common traits should we derive for these structs?
/// Canister settings.
///
/// See [`settings`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-create_canister).
#[derive(Default, Clone, CandidType, Deserialize, Debug)]
pub struct CanisterSettings {
    /// A list of principals. Must be between 0 and 10 in size.
    pub controllers: Option<Vec<Principal>>,
    /// Must be a number between 0 and 100, inclusively.
    pub compute_allocation: Option<Nat>,
    /// Must be a number between 0 and 2^48^ (i.e 256TB), inclusively.
    pub memory_allocation: Option<Nat>,
    /// Must be a number between 0 and 2^64^-1, inclusively, and indicates a length of time in seconds.
    pub freezing_threshold: Option<Nat>,
}

/// See [`super::create_canister`].
#[derive(Default, Clone, CandidType, Deserialize, Debug)]
pub struct CreateCanisterArgument {
    /// See [`CanisterSettings`].
    pub settings: Option<CanisterSettings>,
}

/// See [`super::update_settings`].
#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct UpdateSettingsArgument {
    /// Principle of the canister.
    pub canister_id: CanisterId,
    /// See [`CanisterSettings`].
    pub settings: CanisterSettings,
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

/// WASM module.
pub type WasmModule = Vec<u8>;

/// See [`super::install_code`].
#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct InstallCodeArgument {
    /// See [`CanisterInstallMode`].
    pub mode: CanisterInstallMode,
    /// Principle of the canister.
    pub canister_id: CanisterId,
    /// Code to be installed.
    pub wasm_module: WasmModule,
    /// The argument to be passed to `canister_init` or `canister_post_upgrade`.
    pub arg: Vec<u8>,
}

/// A wrapper of canister id.
#[derive(Clone, Copy, CandidType, Deserialize, Debug)]
pub struct CanisterIdRecord {
    /// Principle of the canister.
    pub canister_id: CanisterId,
}

/// Status of a canister.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, CandidType)]
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

/// Like [`CanisterSettings`].
#[derive(Default, Clone, CandidType, Deserialize, Debug)]
pub struct DefiniteCanisterSettings {
    /// Controllers of the canister.
    pub controllers: Vec<Principal>,
    /// Compute allocation.
    pub compute_allocation: Nat,
    /// Memory allocation.
    pub memory_allocation: Nat,
    /// Freezing threshold.
    pub freezing_threshold: Nat,
}

/// See [`super::canister_status].
#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct CanisterStatusReply {
    /// See [`CanisterStatusType`].
    pub status: CanisterStatusType,
    /// See [`DefiniteCanisterSettings`].
    pub settings: DefiniteCanisterSettings,
    /// A SHA256 hash of the module installed on the canister. This is null if the canister is empty.
    pub module_hash: Option<Vec<u8>>,
    /// The memory size taken by the canister.
    pub memory_size: Nat,
    /// The cycle balance of the canister.
    pub cycles: Nat,
    /// Amount of cycles burned per day.
    pub idle_cycles_burned_per_day: Nat,
}
