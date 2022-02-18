use ic_cdk_macros::*;
use ic_cdk::{export::{candid, Principal}, api::call::{self, Empty}};

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

#[query(reply = true)]
fn read() -> Empty<candid::Nat> {
    unsafe { call::reply(&COUNTER.as_mut().unwrap()) };
    Empty::empty()
}

#[update]
fn write(input: candid::Nat) -> () {
    unsafe {
        COUNTER.as_mut().unwrap().0 = input.0;
    }
}
