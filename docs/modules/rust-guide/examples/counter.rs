use ic_cdk_macros::*;
use ic_types::Principal;

static mut COUNTER: Option<candid::Nat> = None;
static mut OWNER: Option<Principal> = None;

#[init]
fn init() {
    unsafe {
        OWNER = Some(ic_cdk::reflection::caller());
        COUNTER = Some(candid::Nat::from(0));
    }
}

#[update]
fn increment() -> () {
    unsafe {
        ic_cdk::println!("{:?}", OWNER);
        COUNTER.as_mut().unwrap().0 += 1u64;
    }
}

#[query]
fn get() -> candid::Nat {
    unsafe { COUNTER.as_mut().unwrap().clone() }
}

#[update]
fn set(input: candid::Nat) -> () {
    unsafe {
        COUNTER.as_mut().unwrap().0 = input.0;
    }
}
