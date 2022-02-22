use ic_cdk::{
    api::call::{self, ManualReply},
    export::{candid, Principal},
};
use ic_cdk_macros::*;

static mut COUNTER: Option<candid::Nat> = None;
static mut OWNER: Option<Principal> = None;

#[init]
fn init() {
    unsafe {
        OWNER = Some(ic_cdk::api::caller());
        COUNTER = Some(candid::Nat::from(0));
    }
}

#[update]
fn inc() -> () {
    unsafe {
        ic_cdk::println!("{:?}", OWNER);
        COUNTER.as_mut().unwrap().0 += 1u64;
    }
}

#[query(manual_reply = true)]
fn read() -> ManualReply<candid::Nat> {
    unsafe { ManualReply::one(COUNTER.as_mut().unwrap()) }
}

#[update]
fn write(input: candid::Nat) -> () {
    unsafe {
        COUNTER.as_mut().unwrap().0 = input.0;
    }
}
