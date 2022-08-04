//! Provisional functions only available in local development instances.

use crate::api::call::{call, CallResult};
use candid::{CandidType, Nat, Principal};
use serde::Deserialize;

pub use super::main::{CanisterId, CanisterIdRecord, CanisterSettings};

/// See [`provisional_create_canister_with_cycles`].
#[derive(Default, Clone, CandidType, Deserialize, Debug)]
pub struct ProvisionalCreateCanisterWithCyclesArgument {
    /// The created canister will have this amount of cycles.
    pub amount: Option<Nat>,
    /// See [`CanisterSettings`].
    pub settings: Option<CanisterSettings>,
}

/// See [`provisional_top_up_canister`]
#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct ProvisionalTopUpCanisterArgument {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// Amount of cycles to be added.
    pub amount: Nat,
}

/// Create a new canister with specified amout of cycles balance.
///
/// This method is only available in local development instances.
///
/// See [IC method `provisional_create_canister_with_cycles`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-provisional_create_canister_with_cycles).
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

/// Add cycles to a canister.
///
/// This method is only available in local development instances.
///
/// See [IC method `provisional_top_up_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-provisional_top_up_canister).
pub async fn provisional_top_up_canister(arg: ProvisionalTopUpCanisterArgument) -> CallResult<()> {
    call(
        Principal::management_canister(),
        "provisional_top_up_canister",
        (arg,),
    )
    .await
}
