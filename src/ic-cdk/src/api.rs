//! System API and low level functions for it.
use crate::export::Principal;
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

/// Get current timestamp
pub fn time() -> u64 {
    unsafe { ic0::time() as u64 }
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
pub fn canister_balance() -> u64 {
    unsafe { ic0::canister_cycle_balance() as u64 }
}

/// Get the amount of funds available in the canister.
pub fn canister_balance128() -> u128 {
    let mut recv = 0u128;
    unsafe { ic0::canister_cycle_balance128(&mut recv as *mut u128 as i32) }
    recv
}

/// Sets the certified data of this canister.
///
/// Canisters can store up to 32 bytes of data that is certified by
/// the system on a regular basis.  One can call [data_certificate]
/// function from a query call to get a certificate authenticating the
/// value set by calling this function.
///
/// This function can only be called from the following contexts:
///  * "canister_init", "canister_pre_upgrade" and "canister_post_upgrade"
///    hooks.
///  * "canister_update" calls.
///  * reply or reject callbacks.
///
/// # Panics
///
/// * This function traps if data.len() > 32.
/// * This function traps if it's called from an illegal context
///   (e.g., from a query call).
pub fn set_certified_data(data: &[u8]) {
    unsafe { ic0::certified_data_set(data.as_ptr() as i32, data.len() as i32) }
}

/// When called from a query call, returns the data certificate authenticating
/// certified_data set by this canister.
///
/// Returns None if called not from a query call.
pub fn data_certificate() -> Option<Vec<u8>> {
    if unsafe { ic0::data_certificate_present() } == 0 {
        return None;
    }

    let n = unsafe { ic0::data_certificate_size() };
    let mut buf = vec![0u8; n as usize];
    unsafe {
        ic0::data_certificate_copy(buf.as_mut_ptr() as i32, 0i32, n);
    }
    Some(buf)
}
