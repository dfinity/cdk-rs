use crate::implementation::*;

#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_arg_data_size() -> usize {
    (ic0().msg_arg_data_size)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_arg_data_copy(dst: usize, offset: usize, size: usize) {
    (ic0().msg_arg_data_copy)(dst, offset, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_caller_size() -> usize {
    (ic0().msg_caller_size)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_caller_copy(dst: usize, offset: usize, size: usize) {
    (ic0().msg_caller_copy)(dst, offset, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reject_code() -> u32 {
    (ic0().msg_reject_code)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reject_msg_size() -> usize {
    (ic0().msg_reject_msg_size)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reject_msg_copy(dst: usize, offset: usize, size: usize) {
    (ic0().msg_reject_msg_copy)(dst, offset, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reply_data_append(src: usize, size: usize) {
    (ic0().msg_reply_data_append)(src, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reply() {
    (ic0().msg_reply)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reject(src: usize, size: usize) {
    (ic0().msg_reject)(src, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_available() -> u64 {
    (ic0().msg_cycles_available)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_available128(dst: usize) {
    (ic0().msg_cycles_available128)(dst)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_refunded() -> u64 {
    (ic0().msg_cycles_refunded)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_refunded128(dst: usize) {
    (ic0().msg_cycles_refunded128)(dst)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_accept(max_amount: u64) -> u64 {
    (ic0().msg_cycles_accept)(max_amount)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_accept128(
    max_amount_high: u64,
    max_amount_low: u64,
    dst: usize,
) {
    (ic0().msg_cycles_accept128)(max_amount_high, max_amount_low, dst)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_cycles_burn128(amount_high: u64, amount_low: u64, dst: usize) {
    (ic0().cycles_burn128)(amount_high, amount_low, dst)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_self_size() -> usize {
    (ic0().canister_self_size)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_self_copy(dst: usize, offset: usize, size: usize) {
    (ic0().canister_self_copy)(dst, offset, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_cycle_balance() -> u64 {
    (ic0().canister_cycle_balance)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_cycle_balance128(dst: usize) {
    (ic0().canister_cycle_balance128)(dst)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_status() -> u32 {
    (ic0().canister_status)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_version() -> u64 {
    (ic0().canister_version)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_method_name_size() -> usize {
    (ic0().msg_method_name_size)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_method_name_copy(dst: usize, offset: usize, size: usize) {
    (ic0().msg_method_name_copy)(dst, offset, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_accept_message() {
    (ic0().accept_message)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_new(
    callee_src: usize,
    callee_size: usize,
    name_src: usize,
    name_size: usize,
    reply_fun: usize,
    reply_env: usize,
    reject_fun: usize,
    reject_env: usize,
) {
    (ic0().call_new)(
        callee_src,
        callee_size,
        name_src,
        name_size,
        reply_fun,
        reply_env,
        reject_fun,
        reject_env,
    )
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_on_cleanup(fun: usize, env: usize) {
    (ic0().call_on_cleanup)(fun, env)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_data_append(src: usize, size: usize) {
    (ic0().call_data_append)(src, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_cycles_add(amount: u64) {
    (ic0().call_cycles_add)(amount)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_cycles_add128(amount_high: u64, amount_low: u64) {
    (ic0().call_cycles_add128)(amount_high, amount_low)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_perform() -> u32 {
    (ic0().call_perform)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable_size() -> i32 {
    (ic0().stable_size)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable_grow(new_pages: i32) -> i32 {
    (ic0().stable_grow)(new_pages)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable_write(offset: u32, src: usize, size: u32) {
    (ic0().stable_write)(offset, src, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable_read(dst: usize, offset: u32, size: u32) {
    (ic0().stable_read)(dst, offset, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable64_size() -> i64 {
    (ic0().stable64_size)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable64_grow(new_pages: i64) -> i64 {
    (ic0().stable64_grow)(new_pages)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable64_write(offset: u64, src: u64, size: u64) {
    (ic0().stable64_write)(offset, src, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable64_read(dst: u64, offset: u64, size: u64) {
    (ic0().stable64_read)(dst, offset, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_certified_data_set(src: usize, size: usize) {
    (ic0().certified_data_set)(src, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_data_certificate_present() -> u32 {
    (ic0().data_certificate_present)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_data_certificate_size() -> usize {
    (ic0().data_certificate_size)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_data_certificate_copy(dst: usize, offset: usize, size: usize) {
    (ic0().data_certificate_copy)(dst, offset, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_time() -> i64 {
    (ic0().time)()
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_global_timer_set(timestamp: i64) -> i64 {
    (ic0().global_timer_set)(timestamp)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_performance_counter(counter_type: u32) -> u64 {
    (ic0().performance_counter)(counter_type)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_is_controller(src: usize, size: usize) -> usize {
    (ic0().is_controller)(src, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_debug_print(src: usize, size: usize) {
    (ic0().debug_print)(src, size)
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_trap(src: usize, size: usize) {
    (ic0().trap)(src, size)
}

// DO NOT change the order of fields. New fields should be APPENDED to end of struct.
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub(crate) struct Ic0Vtable {
    pub(crate) size: usize,
    pub(crate) msg_arg_data_size: unsafe extern "C-unwind" fn() -> usize,
    pub(crate) msg_arg_data_copy:
        unsafe extern "C-unwind" fn(dst: usize, offset: usize, size: usize),
    pub(crate) msg_caller_size: unsafe extern "C-unwind" fn() -> usize,
    pub(crate) msg_caller_copy: unsafe extern "C-unwind" fn(dst: usize, offset: usize, size: usize),
    pub(crate) msg_reject_code: unsafe extern "C-unwind" fn() -> u32,
    pub(crate) msg_reject_msg_size: unsafe extern "C-unwind" fn() -> usize,
    pub(crate) msg_reject_msg_copy:
        unsafe extern "C-unwind" fn(dst: usize, offset: usize, size: usize),
    pub(crate) msg_reply_data_append: unsafe extern "C-unwind" fn(src: usize, size: usize),
    pub(crate) msg_reply: unsafe extern "C-unwind" fn(),
    pub(crate) msg_reject: unsafe extern "C-unwind" fn(src: usize, size: usize),
    pub(crate) msg_cycles_available: unsafe extern "C-unwind" fn() -> u64,
    pub(crate) msg_cycles_available128: unsafe extern "C-unwind" fn(dst: usize),
    pub(crate) msg_cycles_refunded: unsafe extern "C-unwind" fn() -> u64,
    pub(crate) msg_cycles_refunded128: unsafe extern "C-unwind" fn(dst: usize),
    pub(crate) msg_cycles_accept: unsafe extern "C-unwind" fn(max_amount: u64) -> u64,
    pub(crate) msg_cycles_accept128:
        unsafe extern "C-unwind" fn(max_amount_high: u64, max_amount_low: u64, dst: usize),
    pub(crate) cycles_burn128:
        unsafe extern "C-unwind" fn(amount_high: u64, amount_low: u64, dst: usize),
    pub(crate) canister_self_size: unsafe extern "C-unwind" fn() -> usize,
    pub(crate) canister_self_copy:
        unsafe extern "C-unwind" fn(dst: usize, offset: usize, size: usize),
    pub(crate) canister_cycle_balance: unsafe extern "C-unwind" fn() -> u64,
    pub(crate) canister_cycle_balance128: unsafe extern "C-unwind" fn(dst: usize),
    pub(crate) canister_status: unsafe extern "C-unwind" fn() -> u32,
    pub(crate) canister_version: unsafe extern "C-unwind" fn() -> u64,
    pub(crate) msg_method_name_size: unsafe extern "C-unwind" fn() -> usize,
    pub(crate) msg_method_name_copy:
        unsafe extern "C-unwind" fn(dst: usize, offset: usize, size: usize),
    pub(crate) accept_message: unsafe extern "C-unwind" fn(),
    pub(crate) call_new: unsafe extern "C-unwind" fn(
        callee_src: usize,
        callee_size: usize,
        name_src: usize,
        name_size: usize,
        reply_fun: usize,
        reply_env: usize,
        reject_fun: usize,
        reject_env: usize,
    ),
    pub(crate) call_on_cleanup: unsafe extern "C-unwind" fn(fun: usize, env: usize),
    pub(crate) call_data_append: unsafe extern "C-unwind" fn(src: usize, size: usize),
    pub(crate) call_cycles_add: unsafe extern "C-unwind" fn(amount: u64),
    pub(crate) call_cycles_add128: unsafe extern "C-unwind" fn(amount_high: u64, amount_low: u64),
    pub(crate) call_perform: unsafe extern "C-unwind" fn() -> u32,
    pub(crate) stable_size: unsafe extern "C-unwind" fn() -> i32,
    pub(crate) stable_grow: unsafe extern "C-unwind" fn(new_pages: i32) -> i32,
    pub(crate) stable_write: unsafe extern "C-unwind" fn(offset: u32, src: usize, size: u32),
    pub(crate) stable_read: unsafe extern "C-unwind" fn(dst: usize, offset: u32, size: u32),
    pub(crate) stable64_size: unsafe extern "C-unwind" fn() -> i64,
    pub(crate) stable64_grow: unsafe extern "C-unwind" fn(new_pages: i64) -> i64,
    pub(crate) stable64_write: unsafe extern "C-unwind" fn(offset: u64, src: u64, size: u64),
    pub(crate) stable64_read: unsafe extern "C-unwind" fn(dst: u64, offset: u64, size: u64),
    pub(crate) certified_data_set: unsafe extern "C-unwind" fn(src: usize, size: usize),
    pub(crate) data_certificate_present: unsafe extern "C-unwind" fn() -> u32,
    pub(crate) data_certificate_size: unsafe extern "C-unwind" fn() -> usize,
    pub(crate) data_certificate_copy:
        unsafe extern "C-unwind" fn(dst: usize, offset: usize, size: usize),
    pub(crate) time: unsafe extern "C-unwind" fn() -> i64,
    pub(crate) global_timer_set: unsafe extern "C-unwind" fn(timestamp: i64) -> i64,
    pub(crate) performance_counter: unsafe extern "C-unwind" fn(counter_type: u32) -> u64,
    pub(crate) is_controller: unsafe extern "C-unwind" fn(src: usize, size: usize) -> usize,
    pub(crate) debug_print: unsafe extern "C-unwind" fn(src: usize, size: usize),
    pub(crate) trap: unsafe extern "C-unwind" fn(src: usize, size: usize),
}

// SAFETY: This must not be written to after any functions are called, and must contain only functions obeying the system spec's implied safety guarantees.
#[export_name = "ic0_interface"]
static mut IC0_INTERFACE: Ic0Vtable = Ic0Vtable {
    size: std::mem::size_of::<Ic0Vtable>(),
    msg_arg_data_size,
    msg_arg_data_copy,
    msg_caller_size,
    msg_caller_copy,
    msg_reject_code,
    msg_reject_msg_size,
    msg_reject_msg_copy,
    msg_reply_data_append,
    msg_reply,
    msg_reject,
    msg_cycles_available,
    msg_cycles_available128,
    msg_cycles_refunded,
    msg_cycles_refunded128,
    msg_cycles_accept,
    msg_cycles_accept128,
    cycles_burn128,
    canister_self_size,
    canister_self_copy,
    canister_cycle_balance,
    canister_cycle_balance128,
    canister_status,
    canister_version,
    msg_method_name_size,
    msg_method_name_copy,
    accept_message,
    call_new,
    call_on_cleanup,
    call_data_append,
    call_cycles_add,
    call_cycles_add128,
    call_perform,
    stable_size,
    stable_grow,
    stable_write,
    stable_read,
    stable64_size,
    stable64_grow,
    stable64_write,
    stable64_read,
    certified_data_set,
    data_certificate_present,
    data_certificate_size,
    data_certificate_copy,
    time,
    global_timer_set,
    performance_counter,
    is_controller,
    debug_print,
    trap,
};

pub(crate) fn ic0() -> Ic0Vtable {
    // SAFETY: This static is not written to while this function is being called, because it is not written to after any function is called.
    let ic0 = unsafe { IC0_INTERFACE };
    assert!(
        ic0.size >= std::mem::size_of::<Ic0Vtable>(),
        "Host canister uses an outdated version of the CDK",
    );
    ic0
}
