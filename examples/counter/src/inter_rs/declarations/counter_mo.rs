// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
use ic_cdk::api::call::CallResult;
use ic_cdk::export::candid::{self, CandidType, Deserialize, Principal};

pub struct SERVICE(pub Principal);
impl SERVICE {
    pub async fn inc(&self) -> CallResult<()> {
        ic_cdk::call(self.0, "inc", ()).await
    }
    pub async fn read(&self) -> CallResult<(candid::Nat,)> {
        ic_cdk::call(self.0, "read", ()).await
    }
    pub async fn write(&self, arg0: candid::Nat) -> CallResult<()> {
        ic_cdk::call(self.0, "write", (arg0,)).await
    }
}
pub fn counter_mo() -> SERVICE {
    SERVICE(Principal::from_text("by6od-j4aaa-aaaaa-qaadq-cai").unwrap())
}
