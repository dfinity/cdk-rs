//! System API bindings.
//!
//! This module provides Rust ergonomic bindings to the system APIs.
//!
//! Some APIs require more advanced handling and are organized into separate modules:
//! * For the inter-canister calls API, see the [`call`](mod@crate::call) module.
//! * For the stable memory management API, see the .
//!   * The basic bindings are provided in this module including [`stable_size`], [`stable_grow`], [`stable_read`] and [`stable_write`].
//!   * The [`stable`](crate::stable) module provides more advanced functionalities, e.g. support for `std::io` traits.
//!
//! APIs that are only available for `wasm32` are not included.
//! As a result, system APIs with a numeric postfix (indicating the data bit width) are bound to names without the postfix.
//! For example, `ic0::msg_cycles_available128` is bound to [`msg_cycles_available`], while `ic0::msg_cycles_available` has no binding.
//!
//! Functions that provide bindings for a single system API method share the same name as the system API.
//! For example, `ic0::msg_reject_code` is bound to [`msg_reject_code`].
//!
//! Functions that wrap multiple system API methods are named using the common prefix of the wrapped methods.
//! For example, [`msg_arg_data`] wraps both `ic0::msg_arg_data_size` and `ic0::msg_arg_data_copy`.

use candid::Principal;
use std::{convert::TryFrom, num::NonZeroU64};

#[doc(hidden)]
#[deprecated(
    since = "0.18.0",
    note = "The `api::call` module is deprecated. Individual items within this module have their own deprecation notices with specific migration guidance."
)]
pub mod call;
pub mod management_canister;
#[doc(hidden)]
#[deprecated(
    since = "0.18.0",
    note = "The `api::stable` module has been moved to `stable` (crate root)."
)]
pub mod stable;

/// Gets the message argument data.
pub fn msg_arg_data() -> Vec<u8> {
    // SAFETY: ic0.msg_arg_data_size is always safe to call.
    let len = unsafe { ic0::msg_arg_data_size() };
    let mut bytes = Vec::with_capacity(len);
    // SAFETY:
    // `bytes`, being mutable and allocated to `len` bytes, is safe to pass to ic0.msg_arg_data_copy with no offset
    // ic0.msg_arg_data_copy writes to all of `bytes[0..len]`, so `set_len` is safe to call with the new len.
    unsafe {
        ic0::msg_arg_data_copy(bytes.as_mut_ptr() as usize, 0, len);
        bytes.set_len(len);
    }
    bytes
}

/// Gets the identity of the caller, which may be a canister id or a user id.
///
/// During canister installation or upgrade, this is the id of the user or canister requesting the installation or upgrade.
/// During a system task (heartbeat or global timer), this is the id of the management canister.
pub fn msg_caller() -> Principal {
    // SAFETY: ic0.msg_caller_size is always safe to call.
    let len = unsafe { ic0::msg_caller_size() };
    let mut bytes = vec![0u8; len];
    // SAFETY: Because `bytes` is mutable, and allocated to `len` bytes, it is safe to be passed to `ic0.msg_caller_copy` with a 0-offset.
    unsafe {
        ic0::msg_caller_copy(bytes.as_mut_ptr() as usize, 0, len);
    }
    // Trust that the system always returns a valid principal.
    Principal::try_from(&bytes).unwrap()
}

/// Returns the reject code, if the current function is invoked as a reject callback.
pub fn msg_reject_code() -> u32 {
    // SAFETY: ic0.msg_reject_code is always safe to call.
    unsafe { ic0::msg_reject_code() }
}

/// Gets the reject message.
///
/// This function can only be called in the reject callback.
///
/// Traps if there is no reject message (i.e. if reject_code is 0).
pub fn msg_reject_msg() -> String {
    // SAFETY: ic0.msg_reject_msg_size is always safe to call.
    let len = unsafe { ic0::msg_reject_msg_size() };
    let mut bytes = vec![0u8; len];
    // SAFETY: `bytes`, being mutable and allocated to `len` bytes, is safe to pass to ic0.msg_reject_msg_copy with no offset
    unsafe {
        ic0::msg_reject_msg_copy(bytes.as_mut_ptr() as usize, 0, len);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Gets the deadline, in nanoseconds since 1970-01-01, after which the caller might stop waiting for a response.
///
/// For calls to update methods with best-effort responses and their callbacks,
/// the deadline is computed based on the time the call was made,
/// and the `timeout_seconds` parameter provided by the caller.
/// In such cases, the deadline value will be converted to `NonZeroU64` and wrapped in `Some`.
/// To get the deadline value as a `u64`, call `get()` on the `NonZeroU64` value.
///
/// ```rust,no_run
/// use ic_cdk::api::msg_deadline;
/// if let Some(deadline) = msg_deadline() {
///     let deadline_value : u64 = deadline.get();
/// }
/// ```
///
/// For other calls (ingress messages and all calls to query and composite query methods,
/// including calls in replicated mode), a `None` is returned.
/// Please note that the raw `msg_deadline` system API returns 0 in such cases.
/// This function is a wrapper around the raw system API that provides more semantic information through the return type.
pub fn msg_deadline() -> Option<NonZeroU64> {
    // SAFETY: ic0.msg_deadline is always safe to call.
    let nano_seconds = unsafe { ic0::msg_deadline() };
    match nano_seconds {
        0 => None,
        _ => Some(NonZeroU64::new(nano_seconds).unwrap()),
    }
}

/// Replies to the sender with the data.
pub fn msg_reply<T: AsRef<[u8]>>(data: T) {
    let buf = data.as_ref();
    if !buf.is_empty() {
        // SAFETY: `buf`, being &[u8], is a readable sequence of bytes, and therefore valid to pass to ic0.msg_reply.
        unsafe { ic0::msg_reply_data_append(buf.as_ptr() as usize, buf.len()) }
    };
    // SAFETY: ic0.msg_reply is always safe to call.
    unsafe { ic0::msg_reply() };
}

/// Rejects the call with a diagnostic message.
pub fn msg_reject<T: AsRef<str>>(message: T) {
    let buf = message.as_ref();
    // SAFETY: `buf`, being &str, is a readable sequence of UTF8 bytes and therefore can be passed to ic0.msg_reject.
    unsafe {
        ic0::msg_reject(buf.as_ptr() as usize, buf.len());
    }
}

/// Gets the number of cycles transferred by the caller of the current call, still available in this message.
pub fn msg_cycles_available() -> u128 {
    let mut recv = 0u128;
    // SAFETY: recv is writable and sixteen bytes wide, and therefore is safe to pass to ic0.msg_cycles_available128
    unsafe {
        ic0::msg_cycles_available128(&mut recv as *mut u128 as usize);
    }
    recv
}

/// Gets the amount of cycles that came back with the response as a refund
///
/// This function can only be used in a callback handler (reply or reject).
/// The refund has already been added to the canister balance automatically.
pub fn msg_cycles_refunded() -> u128 {
    let mut recv = 0u128;
    // SAFETY: recv is writable and sixteen bytes wide, and therefore is safe to pass to ic0.msg_cycles_refunded128
    unsafe {
        ic0::msg_cycles_refunded128(&mut recv as *mut u128 as usize);
    }
    recv
}

/// Moves cycles from the call to the canister balance.
///
/// The actual amount moved will be returned.
pub fn msg_cycles_accept(max_amount: u128) -> u128 {
    let high = (max_amount >> 64) as u64;
    let low = (max_amount & u64::MAX as u128) as u64;
    let mut recv = 0u128;
    // SAFETY: `recv` is writable and sixteen bytes wide, and therefore safe to pass to ic0.msg_cycles_accept128
    unsafe {
        ic0::msg_cycles_accept128(high, low, &mut recv as *mut u128 as usize);
    }
    recv
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

/// Gets canister's own identity.
pub fn canister_self() -> Principal {
    // SAFETY: ic0.canister_self_size is always safe to call.
    let len = unsafe { ic0::canister_self_size() };
    let mut bytes = vec![0u8; len];
    // SAFETY: Because `bytes` is mutable, and allocated to `len` bytes, it is safe to be passed to `ic0.canister_self_copy` with a 0-offset.
    unsafe {
        ic0::canister_self_copy(bytes.as_mut_ptr() as usize, 0, len);
    }
    // Trust that the system always returns a valid principal.
    Principal::try_from(&bytes).unwrap()
}

/// Gets the current cycle balance of the canister
pub fn canister_cycle_balance() -> u128 {
    let mut recv = 0u128;
    // SAFETY: recv is writable and the size expected by ic0.canister_cycle_balance128.
    unsafe { ic0::canister_cycle_balance128(&mut recv as *mut u128 as usize) }
    recv
}

/// Gets the status of the canister.
///
/// The status is one of the following:
/// * 1: Running
/// * 2: Stopping
/// * 3: Stopped
pub fn canister_status() -> CanisterStatusCode {
    // SAFETY: ic0.canister_status is always safe to call.
    unsafe { ic0::canister_status() }.into()
}

/// The status of a canister.
///
/// See [Canister status](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-canister-status).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum CanisterStatusCode {
    /// Running.
    Running = 1,
    /// Stopping.
    Stopping = 2,
    /// Stopped.
    Stopped = 3,
    /// A status code that is not recognized by this library.
    Unrecognized(u32),
}

impl From<u32> for CanisterStatusCode {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::Running,
            2 => Self::Stopping,
            3 => Self::Stopped,
            _ => Self::Unrecognized(value),
        }
    }
}

impl From<CanisterStatusCode> for u32 {
    fn from(value: CanisterStatusCode) -> Self {
        match value {
            CanisterStatusCode::Running => 1,
            CanisterStatusCode::Stopping => 2,
            CanisterStatusCode::Stopped => 3,
            CanisterStatusCode::Unrecognized(value) => value,
        }
    }
}

impl PartialEq<u32> for CanisterStatusCode {
    fn eq(&self, other: &u32) -> bool {
        let self_as_u32: u32 = (*self).into();
        self_as_u32 == *other
    }
}

/// Gets the canister version.
///
/// See [Canister version](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-canister-version).
pub fn canister_version() -> u64 {
    // SAFETY: ic0.canister_version is always safe to call.
    unsafe { ic0::canister_version() }
}

/// Gets the name of the method to be inspected.
///
/// This function is only available in the `canister_inspect_message` context.
pub fn msg_method_name() -> String {
    // SAFETY: ic0.msg_method_name_size is always safe to call.
    let len: u32 = unsafe { ic0::msg_method_name_size() as u32 };
    let mut bytes = vec![0u8; len as usize];
    // SAFETY: `bytes` is writable and allocated to `len` bytes, and therefore can be safely passed to ic0.msg_method_name_copy
    unsafe {
        ic0::msg_method_name_copy(bytes.as_mut_ptr() as usize, 0, len as usize);
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Accepts the message in `canister_inspect_message`.
///
/// This function is only available in the `canister_inspect_message` context.
/// This function traps if invoked twice.
pub fn accept_message() {
    // SAFETY: ic0.accept_message is always safe to call.
    unsafe { ic0::accept_message() }
}

/// Gets the current size of the stable memory (in WebAssembly pages).
///
/// One WebAssembly page is 64KiB.
pub fn stable_size() -> u64 {
    // SAFETY: ic0.stable64_size is always safe to call.
    unsafe { ic0::stable64_size() }
}

/// Attempts to grow the stable memory by `new_pages` many pages containing zeroes.
///
/// One WebAssembly page is 64KiB.
///
/// If successful, returns the previous size of the memory (in pages).
/// Otherwise, returns `u64::MAX`.
pub fn stable_grow(new_pages: u64) -> u64 {
    // SAFETY: ic0.stable64_grow is always safe to call.
    unsafe { ic0::stable64_grow(new_pages) }
}

/// Writes data to the stable memory location specified by an offset.
///
/// # Warning
/// This will panic if `offset + buf.len()` exceeds the current size of stable memory.
/// Call [`stable_grow`] to request more stable memory if needed.
pub fn stable_write(offset: u64, buf: &[u8]) {
    // SAFETY: `buf`, being &[u8], is a readable sequence of bytes, and therefore valid to pass to ic0.stable64_write.
    unsafe {
        ic0::stable64_write(offset, buf.as_ptr() as u64, buf.len() as u64);
    }
}

/// Reads data from the stable memory location specified by an offset.
///
/// # Warning
/// This will panic if `offset + buf.len()` exceeds the current size of stable memory.
pub fn stable_read(offset: u64, buf: &mut [u8]) {
    // SAFETY: `buf`, being &mut [u8], is a writable sequence of bytes, and therefore valid to pass to ic0.stable64_read.
    unsafe {
        ic0::stable64_read(buf.as_ptr() as u64, offset, buf.len() as u64);
    }
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
pub fn certified_data_set<T: AsRef<[u8]>>(data: T) {
    let buf = data.as_ref();
    // SAFETY:  uf is a slice ref, its pointer and length are valid to pass to ic0.certified_data_set.
    unsafe { ic0::certified_data_set(buf.as_ptr() as usize, buf.len()) }
}

/// When called from a query call, returns the data certificate authenticating
/// certified_data set by this canister.
///
/// Returns `None` if called not from a query call.
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

/// Gets current timestamp, in nanoseconds since the epoch (1970-01-01)
pub fn time() -> u64 {
    // SAFETY: ic0.time is always safe to call.
    unsafe { ic0::time() }
}

/// Sets global timer.
///
/// The canister can set a global timer to make the system
/// schedule a call to the exported `canister_global_timer`
/// Wasm method after the specified time.
/// The time must be provided as nanoseconds since 1970-01-01.
///
/// The function returns the previous value of the timer.
/// If no timer is set before invoking the function, then the function returns zero.
///
/// Passing zero as an argument to the function deactivates the timer and thus
/// prevents the system from scheduling calls to the canister's `canister_global_timer` Wasm method.
pub fn global_timer_set(timestamp: u64) -> u64 {
    // SAFETY: ic0.global_timer_set is always safe to call.
    unsafe { ic0::global_timer_set(timestamp) }
}

/// Gets the value of specified performance counter.
///
/// See [`PerformanceCounterType`] for available counter types.
#[inline]
pub fn performance_counter(counter_type: impl Into<PerformanceCounterType>) -> u64 {
    let counter_type: u32 = counter_type.into().into();
    // SAFETY: ic0.performance_counter is always safe to call.
    unsafe { ic0::performance_counter(counter_type) }
}

/// The type of performance counter.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum PerformanceCounterType {
    /// Current execution instruction counter.
    ///
    /// The number of WebAssembly instructions the canister has executed
    /// since the beginning of the current Message execution.
    InstructionCounter,
    /// Call context instruction counter
    ///
    /// The number of WebAssembly instructions the canister has executed
    /// within the call context of the current Message execution
    /// since Call context creation.
    /// The counter monotonically increases across all message executions
    /// in the call context until the corresponding call context is removed.
    CallContextInstructionCounter,
    /// A performance counter type that is not recognized by this library.
    Unrecognized(u32),
}

impl From<u32> for PerformanceCounterType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::InstructionCounter,
            1 => Self::CallContextInstructionCounter,
            _ => Self::Unrecognized(value),
        }
    }
}

impl From<PerformanceCounterType> for u32 {
    fn from(value: PerformanceCounterType) -> Self {
        match value {
            PerformanceCounterType::InstructionCounter => 0,
            PerformanceCounterType::CallContextInstructionCounter => 1,
            PerformanceCounterType::Unrecognized(value) => value,
        }
    }
}

impl PartialEq<u32> for PerformanceCounterType {
    fn eq(&self, other: &u32) -> bool {
        let self_as_u32: u32 = (*self).into();
        self_as_u32 == *other
    }
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

/// Determines if a Principal is a controller of the canister.
pub fn is_controller(principal: &Principal) -> bool {
    let slice = principal.as_slice();
    // SAFETY: `principal.as_bytes()`, being `&[u8]`, is a readable sequence of bytes and therefore safe to pass to `ic0.is_controller`.
    unsafe { ic0::is_controller(slice.as_ptr() as usize, slice.len()) != 0 }
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

/// Emits textual trace messages.
///
/// On the "real" network, these do not do anything.
///
/// When executing in an environment that supports debugging, this copies out the data
/// and logs, prints or stores it in an environment-appropriate way.
pub fn debug_print<T: AsRef<str>>(data: T) {
    let buf = data.as_ref();
    // SAFETY: `buf` is a readable sequence of bytes and therefore can be passed to ic0.debug_print.
    unsafe {
        ic0::debug_print(buf.as_ptr() as usize, buf.len());
    }
}

/// Traps with the given message.
///
/// The environment may copy out the data and log, print or store it in an environment-appropriate way,
/// or include it in system-generated reject messages where appropriate.
pub fn trap<T: AsRef<str>>(data: T) -> ! {
    let buf = data.as_ref();
    // SAFETY: `buf` is a readable sequence of bytes and therefore can be passed to ic0.trap.
    unsafe {
        ic0::trap(buf.as_ptr() as usize, buf.len());
    }
    unreachable!()
}

// # Deprecated API bindings
//
// The following functions are deprecated and will be removed in the future.
// They are kept here for compatibility with existing code.

/// Prints the given message.
#[deprecated(since = "0.18.0", note = "Use `debug_print` instead")]
pub fn print<S: std::convert::AsRef<str>>(s: S) {
    let s = s.as_ref();
    // SAFETY: `s`, being &str, is a readable sequence of bytes and therefore can be passed to ic0.debug_print.
    unsafe {
        ic0::debug_print(s.as_ptr() as usize, s.len());
    }
}

/// Returns the caller of the current call.
#[deprecated(since = "0.18.0", note = "Use `msg_caller` instead")]
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
#[deprecated(since = "0.18.0", note = "Use `canister_self` instead")]
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
#[deprecated(since = "0.18.0", note = "Use `canister_cycle_balance` instead")]
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
#[deprecated(since = "0.18.0", note = "Use `certified_data_set` instead")]
pub fn set_certified_data(data: &[u8]) {
    // SAFETY: because data is a slice ref, its pointer and length are valid to pass to ic0.certified_data_set.
    unsafe { ic0::certified_data_set(data.as_ptr() as usize, data.len()) }
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
#[deprecated(since = "0.18.0", note = "Use `global_timer_set` instead")]
pub fn set_global_timer(timestamp: u64) -> u64 {
    // SAFETY: ic0.global_timer_set is always safe to call.
    unsafe { ic0::global_timer_set(timestamp) }
}
