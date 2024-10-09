//! Provisional functions only available in local development instances.

use ic_cdk::prelude::*;

use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

pub use super::core::{CanisterIdRecord, CanisterSettings};
use crate::CanisterId;

/// Argument type of [provisional_create_canister_with_cycles].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct ProvisionalCreateCanisterWithCyclesArgument {
    /// The created canister will have this amount of cycles.
    pub amount: Option<Nat>,
    /// See [CanisterSettings].
    pub settings: Option<CanisterSettings>,
}

/// Argument type of [provisional_top_up_canister].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub struct ProvisionalTopUpCanisterArgument {
    /// Canister ID.
    pub canister_id: CanisterId,
    /// Amount of cycles to be added.
    pub amount: Nat,
}

/// Create a new canister with specified amount of cycles balance.
///
/// This method is only available in local development instances.
///
/// See [IC method `provisional_create_canister_with_cycles`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-provisional_create_canister_with_cycles).
pub async fn provisional_create_canister_with_cycles(
    arg: ProvisionalCreateCanisterWithCyclesArgument,
) -> CallResult<(CanisterIdRecord,)> {
    Call::new(
        Principal::management_canister(),
        "provisional_create_canister_with_cycles",
    )
    .with_guaranteed_response()
    .with_args((arg,))
    .call()
    .await
}

/// Add cycles to a canister.
///
/// This method is only available in local development instances.
///
/// See [IC method `provisional_top_up_canister`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-provisional_top_up_canister).
pub async fn provisional_top_up_canister(arg: ProvisionalTopUpCanisterArgument) -> CallResult<()> {
    Call::new(
        Principal::management_canister(),
        "provisional_top_up_canister",
    )
    .with_guaranteed_response()
    .with_args((arg,))
    .call()
    .await
}
