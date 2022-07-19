use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::call::{call, CallResult};

pub type CanisterId = Principal;

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

// create_canister : (record {
//   settings : opt canister_settings
// }) -> (record {canister_id : canister_id});
pub async fn create_canister(arg: CreateCanisterArgument) -> CallResult<(CreateCanisterReturn,)> {
    call(Principal::management_canister(), "create_canister", (arg,)).await
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct UpdateSettingsArgument {
    pub canister_id: CanisterId,
    pub settings: CanisterSettings,
}

// update_settings : (record {
//   canister_id : principal;
//   settings : canister_settings
// }) -> ();
pub async fn update_settings(arg: UpdateSettingsArgument) -> CallResult<()> {
    call(Principal::management_canister(), "update_settings", (arg,)).await
}

// raw_rand : () -> (blob);
pub async fn raw_rand() -> CallResult<(Vec<u8>,)> {
    call(Principal::management_canister(), "raw_rand", ()).await
}
