use candid::{CandidType, Nat, Principal};
use ic_cdk::api::call::{call, CallResult};
use serde::Deserialize;

use super::{CanisterId, CanisterIdRecord, CanisterSettings};

#[derive(Default, Clone, CandidType, Deserialize, Debug)]
pub struct ProvisionalCreateCanisterWithCyclesArgument {
    pub amount: Option<Nat>,
    pub settings: Option<CanisterSettings>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct ProvisionalTopUpCanisterArgument {
    pub canister_id: CanisterId,
    pub amount: Nat,
}

/**
provisional_create_canister_with_cycles : (record {
  amount: opt nat;
  settings : opt canister_settings
}) -> (record {canister_id : canister_id});
*/
pub async fn provisional_create_canister_with_cycles(
    arg: ProvisionalCreateCanisterWithCyclesArgument,
) -> CallResult<(CanisterIdRecord,)> {
    call(
        Principal::management_canister(),
        "provisional_create_canister_with_cycles",
        (arg,),
    )
    .await
}

/**
provisional_top_up_canister :
  (record { canister_id: canister_id; amount: nat }) -> ();
*/
pub async fn provisional_top_up_canister(arg: ProvisionalTopUpCanisterArgument) -> CallResult<()> {
    call(
        Principal::management_canister(),
        "provisional_top_up_canister",
        (arg,),
    )
    .await
}
