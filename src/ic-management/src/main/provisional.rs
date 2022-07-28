use candid::{CandidType, Nat, Principal};
use serde::{Deserialize, Serialize};

use super::CanisterSettings;

#[derive(Default, Clone, CandidType, Deserialize, Debug)]
pub struct ProvisionalCreateCanisterWithCyclesArgument {
    pub amount: Option<Nat>,
    pub settings: Option<CanisterSettings>,
}