//! System API and low level functions for it.
use candid::Principal;
use std::convert::TryFrom;

pub mod call;
pub mod management_canister;
pub mod stable;

/// Prints the given message.
pub fn print<S: std::convert::AsRef<str>>(s: S) {
    let s = s.as_ref();
    // SAFETY: `s`, being &str, is a readable sequence of bytes and therefore can be passed to ic0.debug_print.
    unsafe {
        ic0::debug_print(s.as_ptr() as i32, s.len() as i32);
    }
}

/// Traps with the given message.
pub fn trap(message: &str) -> ! {
    // SAFETY: `message`, being &str, is a readable sequence of bytes and therefore can be passed to ic0.trap.
    unsafe {
        ic0::trap(message.as_ptr() as i32, message.len() as i32);
    }
    unreachable!()
}

/// Get current timestamp, in nanoseconds since the epoch (1970-01-01)
pub fn time() -> u64 {
    // SAFETY: ic0.time is always safe to call.
    unsafe { ic0::time() as u64 }
}

/// Returns the caller of the current call.
pub fn caller() -> Principal {
    // SAFETY: ic0.msg_caller_size is always safe to call.
    let len: u32 = unsafe { ic0::msg_caller_size() as u32 };
    let mut bytes = vec![0u8; len as usize];
    // SAFETY: Because `bytes` is mutable, and allocated to `len` bytes, it is safe to be passed to `ic0.msg_caller_copy` with a 0-offset.
    unsafe {
        ic0::msg_caller_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    }
    Principal::try_from(&bytes).unwrap()
}

/// Returns the canister id as a blob.
pub fn id() -> Principal {
    // SAFETY: ic0.canister_self_size is always safe to call.
    let len: u32 = unsafe { ic0::canister_self_size() as u32 };
    let mut bytes = vec![0u8; len as usize];
    // SAFETY: Because `bytes` is mutable, and allocated to `len` bytes, it is safe to be passed to `ic0.canister_self_copy` with a 0-offset.
    unsafe {
        ic0::canister_self_copy(bytes.as_mut_ptr() as i32, 0, len as i32);
    }
    Principal::try_from(&bytes).unwrap()
}

/// Get the amount of funds available in the canister.
pub fn canister_balance() -> u64 {
    // SAFETY: ic0.canister_cycle_balance is always safe to call.
    unsafe { ic0::canister_cycle_balance() as u64 }
}

/// Get the amount of funds available in the canister.
pub fn canister_balance128() -> u128 {
    let mut recv = 0u128;
    // SAFETY: recv is writable and the size expected by ic0.canister_cycle_balance128.
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
    // SAFETY: because data is a slice ref, its pointer and length are valid to pass to ic0.certified_data_set.
    unsafe { ic0::certified_data_set(data.as_ptr() as i32, data.len() as i32) }
}

/// When called from a query call, returns the data certificate authenticating
/// certified_data set by this canister.
///
/// Returns None if called not from a query call.
pub fn data_certificate() -> Option<Vec<u8>> {
    // SAFETY: ic0.data_certificate_present is always safe to call.
    if unsafe { ic0::data_certificate_present() } == 0 {
        return None;
    }

    // SAFETY: ic0.data_certificate_size is always safe to call.
    let n = unsafe { ic0::data_certificate_size() };
    let mut buf = vec![0u8; n as usize];
    // SAFETY: Because `buf` is mutable and allocated to `n` bytes, it is valid to receive from ic0.data_certificate_bytes with no offset
    unsafe {
        ic0::data_certificate_copy(buf.as_mut_ptr() as i32, 0i32, n);
    }
    Some(buf)
}

/// Returns the number of instructions that the canister executed since the last [entry
/// point](https://internetcomputer.org/docs/current/references/ic-interface-spec/#entry-points).
#[inline]
pub fn instruction_counter() -> u64 {
    performance_counter(0)
}

/// Get the value of specified performance counter.
///
/// Supported counter type:
/// 0 : instruction counter. The number of WebAssembly instructions the system has determined that the canister has executed.
#[inline]
pub fn performance_counter(counter_type: u32) -> u64 {
    // SAFETY: ic0.performance_counter is always safe to call.
    unsafe { ic0::performance_counter(counter_type as i32) as u64 }
}

/// Get the value of canister version.
pub fn canister_version() -> u64 {
    // SAFETY: ic0.canister_version is always safe to call.
    unsafe { ic0::canister_version() as u64 }
}

/// Determine if a Principal is a controller of the canister.
pub fn is_controller(principal: &Principal) -> bool {
    let slice = principal.as_slice();
    // SAFETY: ic0.is_controller is always safe to call.
    let c: i32 = unsafe { ic0::is_controller(slice.as_ptr() as i32, slice.len() as i32) };
    c != 0
}
