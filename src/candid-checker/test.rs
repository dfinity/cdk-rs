// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Deserialize, Principal};

pub type MyType = Principal;
#[derive(CandidType, Deserialize)]
pub struct Profile { pub age: u8, pub country: String, pub name: String }
#[derive(CandidType, Deserialize)]
pub struct ListInner { pub head: serde_bytes::ByteBuf, pub tail: Box<List> }
#[derive(CandidType, Deserialize)]
pub struct List(Option<ListInner>);

#[ic_cdk::update]
#[candid_method(update)]
pub async fn f(test: MyType, argument: Vec/* whatever */<Profile>) -> (List, u8) {
}

fn not_candid() {}

#[init]
fn take_init() {}

mod A {
  #[query(composite = true)]
  async fn inner(a: List) -> Result<List> {}
}

#[::ic_cdk::query(name="test")]
fn g() -> () {
}
impl T {
  #[query]
  fn h(&mut self) -> (u8,u16) {}
}
