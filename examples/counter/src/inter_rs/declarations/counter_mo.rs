// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
use ic_cdk::export::candid::{self, CandidType, Deserialize, Principal};
use ic_cdk::api::call::CallResult as Result;

pub struct SERVICE(pub Principal);
impl SERVICE {
  pub async fn inc(&self) -> Result<()> {
    ic_cdk::call(self.0, "inc", ()).await
  }
  pub async fn read(&self) -> Result<(candid::Nat,)> {
    ic_cdk::call(self.0, "read", ()).await
  }
  pub async fn write(&self, arg0: candid::Nat) -> Result<()> {
    ic_cdk::call(self.0, "write", (arg0,)).await
  }
}
pub const counter_mo: SERVICE = SERVICE(Principal::from_slice(&[128, 0, 0, 0, 0, 16, 0, 7, 1, 1])); // by6od-j4aaa-aaaaa-qaadq-cai