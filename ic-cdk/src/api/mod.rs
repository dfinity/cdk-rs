//! System API and low level functions for it.
use candid::Principal;
use std::convert::TryFrom;

pub mod call;
pub mod stable;

/// Prints the given message.
pub fn print<S: std::convert::AsRef<str>>(s: S) {
    let s = s.as_ref();
    // SAFETY: `s`, being &str, is a readable sequence of bytes and therefore can be passed to ic0.debug_print.
    unsafe {
        ic0::debug_print(s.as_ptr() as usize, s.len());
    }
}

/// Traps with the given message.
pub fn trap(message: &str) -> ! {
    // SAFETY: `message`, being &str, is a readable sequence of bytes and therefore can be passed to ic0.trap.
    unsafe {
        ic0::trap(message.as_ptr() as usize, message.len());
    }
    unreachable!()
}

/// Gets current timestamp, in nanoseconds since the epoch (1970-01-01)
pub fn time() -> u64 {
    // SAFETY: ic0.time is always safe to call.
    unsafe { ic0::time() }
}

/// Returns the caller of the current call.
pub fn caller() -> Principal {
    // SAFETY: ic0.msg_caller_size is always safe to call.
    let len = unsafe { ic0::msg_caller_size() };
    let mut bytes = vec![0u8; len];
    // SAFETY: Because `bytes` is mutable, and allocated to `len` bytes, it is safe to be passed to `ic0.msg_caller_copy` with a 0-offset.
    unsafe {
        ic0::msg_caller_copy(bytes.as_mut_ptr() as usize, 0, len);
    }
    Principal::try_from(&bytes).unwrap()
}

/// Returns the canister id as a blob.
pub fn id() -> Principal {
    // SAFETY: ic0.canister_self_size is always safe to call.
    let len = unsafe { ic0::canister_self_size() };
    let mut bytes = vec![0u8; len];
    // SAFETY: Because `bytes` is mutable, and allocated to `len` bytes, it is safe to be passed to `ic0.canister_self_copy` with a 0-offset.
    unsafe {
        ic0::canister_self_copy(bytes.as_mut_ptr() as usize, 0, len);
    }
    Principal::try_from(&bytes).unwrap()
}

/// Gets the amount of funds available in the canister.
pub fn canister_balance128() -> u128 {
    let mut recv = 0u128;
    // SAFETY: recv is writable and the size expected by ic0.canister_cycle_balance128.
    unsafe { ic0::canister_cycle_balance128(&mut recv as *mut u128 as usize) }
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
    unsafe { ic0::certified_data_set(data.as_ptr() as usize, data.len()) }
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
    let mut buf = vec![0u8; n];
    // SAFETY: Because `buf` is mutable and allocated to `n` bytes, it is valid to receive from ic0.data_certificate_bytes with no offset
    unsafe {
        ic0::data_certificate_copy(buf.as_mut_ptr() as usize, 0, n);
    }
    Some(buf)
}

/// Returns the number of instructions that the canister executed since the last [entry
/// point](https://internetcomputer.org/docs/current/references/ic-interface-spec/#entry-points).
#[inline]
pub fn instruction_counter() -> u64 {
    performance_counter(0)
}

/// Returns the number of WebAssembly instructions the canister has executed
/// within the call context of the current Message execution since
/// Call context creation.
///
/// The counter monotonically increases across all message executions
/// in the call context until the corresponding call context is removed.
#[inline]
pub fn call_context_instruction_counter() -> u64 {
    performance_counter(1)
}

/// Gets the value of specified performance counter.
///
/// Supported counter types:
/// * `0` : current execution instruction counter. The number of WebAssembly
///         instructions the canister has executed since the beginning of the
///         current Message execution.
/// * `1` : call context instruction counter. The number of WebAssembly
///         instructions the canister has executed within the call context
///         of the current Message execution since Call context creation.
///         The counter monotonically increases across all message executions
///         in the call context until the corresponding call context is removed.
#[inline]
pub fn performance_counter(counter_type: u32) -> u64 {
    // SAFETY: ic0.performance_counter is always safe to call.
    unsafe { ic0::performance_counter(counter_type) }
}

/// Gets the value of canister version.
pub fn canister_version() -> u64 {
    // SAFETY: ic0.canister_version is always safe to call.
    unsafe { ic0::canister_version() }
}

/// Determines if a Principal is a controller of the canister.
pub fn is_controller(principal: &Principal) -> bool {
    let slice = principal.as_slice();
    // SAFETY: `principal.as_bytes()`, being `&[u8]`, is a readable sequence of bytes and therefore safe to pass to `ic0.is_controller`.
    unsafe { ic0::is_controller(slice.as_ptr() as usize, slice.len()) != 0 }
}

/// Burns cycles from the canister.
///
/// Returns the amount of cycles that were actually burned.
pub fn cycles_burn(amount: u128) -> u128 {
    let amount_high = (amount >> 64) as u64;
    let amount_low = (amount & u64::MAX as u128) as u64;
    let mut dst = 0u128;
    // SAFETY: `dst` is writable and sixteen bytes wide, and therefore safe to pass to ic0.cycles_burn128
    unsafe { ic0::cycles_burn128(amount_high, amount_low, &mut dst as *mut u128 as usize) }
    dst
}

/// Sets global timer.
///
/// The canister can set a global timer to make the system
/// schedule a call to the exported canister_global_timer
/// Wasm method after the specified time.
/// The time must be provided as nanoseconds since 1970-01-01.
///
/// The function returns the previous value of the timer.
/// If no timer is set before invoking the function, then the function returns zero.
///
/// Passing zero as an argument to the function deactivates the timer and thus
/// prevents the system from scheduling calls to the canister's canister_global_timer Wasm method.
pub fn set_global_timer(timestamp: u64) -> u64 {
    // SAFETY: ic0.global_timer_set is always safe to call.
    unsafe { ic0::global_timer_set(timestamp) }
}

/// Checks if in replicated execution.
///
/// The canister can check whether it is currently running in replicated or non replicated execution.
pub fn in_replicated_execution() -> bool {
    // SAFETY: ic0.in_replicated_execution is always safe to call.
    match unsafe { ic0::in_replicated_execution() } {
        0 => false,
        1 => true,
        _ => unreachable!(),
    }
}