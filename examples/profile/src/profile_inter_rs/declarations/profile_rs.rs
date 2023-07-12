// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
use candid::{self, CandidType, Deserialize, Principal};
use ic_cdk::api::call::CallResult as Result;

#[derive(CandidType, Deserialize)]
pub struct Profile { name: String, description: String, keywords: Vec<String> }

pub struct SERVICE(pub Principal);
impl SERVICE {
  pub async fn get(&self, arg0: String) -> Result<(Profile,)> {
    ic_cdk::call(self.0, "get", (arg0,)).await
  }
  pub async fn getSelf(&self) -> Result<(Profile,)> {
    ic_cdk::call(self.0, "getSelf", ()).await
  }
  pub async fn search(&self, arg0: String) -> Result<(Option<Profile>,)> {
    ic_cdk::call(self.0, "search", (arg0,)).await
  }
  pub async fn update(&self, arg0: Profile) -> Result<()> {
    ic_cdk::call(self.0, "update", (arg0,)).await
  }
}
pub const profile_rs: SERVICE = SERVICE(Principal::from_slice(&[95, 197, 249, 201, 198, 242, 95, 248])); // zk5gd-bk7yx-44trx-sl74a