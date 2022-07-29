use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

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
pub struct UpdateSettingsArgument {
    pub canister_id: CanisterId,
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

pub type WasmModule = Vec<u8>;

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct InstallCodeArgument {
    pub mode: CanisterInstallMode,
    pub canister_id: CanisterId,
    pub wasm_module: WasmModule,
    pub arg: Vec<u8>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct CanisterIdRecord {
    pub canister_id: CanisterId,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, CandidType)]
pub enum CanisterStatusType {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopping")]
    Stopping,
    #[serde(rename = "stopped")]
    Stopped,
}

#[derive(Default, Clone, CandidType, Deserialize, Debug)]
pub struct DefiniteCanisterSettings {
    pub controllers: Vec<Principal>,
    pub compute_allocation: Nat,
    pub memory_allocation: Nat,
    pub freezing_threshold: Nat,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct CanisterStatusReply {
    pub status: CanisterStatusType,
    pub settings: DefiniteCanisterSettings,
    pub module_hash: Option<Vec<u8>>,
    pub memory_size: Nat,
    pub cycles: Nat,
    pub idle_cycles_burned_per_day: Nat,
}
