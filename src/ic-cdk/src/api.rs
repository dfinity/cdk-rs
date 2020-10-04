//! System API and low level functions for it.
use ic_types::Principal;
use std::convert::TryFrom;

pub mod call;
pub mod stable;

mod ic0;

/// Prints the given message.
pub fn print<S: std::convert::AsRef<str>>(s: S) {
    let s = s.as_ref();
    unsafe {
        ic0::debug_print(s.as_ptr() as i32, s.len() as i32);
    }
}

/// Traps with the given message.
pub fn trap(message: &str) -> ! {
    unsafe {
        ic0::trap(message.as_ptr() as i32, message.len() as i32);
    }
    unreachable!()
}

pub fn time() -> i64 {
    unsafe { ic0::time() }
}

/// Returns the caller of the current call.
pub fn caller() -> Principal {
    let len: u32 = unsafe { ic0::msg_caller_size() as u32 };
    let mut bytes = vec![0; len as usize];
    unsafe {
        ic0::msg_caller_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    }
    Principal::try_from(&bytes).unwrap()
}

/// Returns the canister id as a blob.
pub fn id() -> Principal {
    let len: u32 = unsafe { ic0::canister_self_size() as u32 };
    let mut bytes = vec![0; len as usize];
    unsafe {
        ic0::canister_self_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    }
    Principal::try_from(&bytes).unwrap()
}

/// Get the amount of funds available in the canister.
pub fn canister_balance(unit: call::funds::Unit) -> i64 {
    let bytes = unit.to_bytes();
    unsafe { ic0::canister_balance(bytes.as_ptr() as i32, bytes.len() as i32) }
}
