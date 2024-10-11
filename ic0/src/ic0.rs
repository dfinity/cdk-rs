// This file is generated from ic0.txt.
// Don't manually modify it.
#[cfg(target_family = "wasm")]
#[link(wasm_import_module = "ic0")]
extern "C" {
    pub fn msg_arg_data_size() -> usize;
    pub fn msg_arg_data_copy(dst: usize, offset: usize, size: usize);
    pub fn msg_caller_size() -> usize;
    pub fn msg_caller_copy(dst: usize, offset: usize, size: usize);
    pub fn msg_reject_code() -> u32;
    pub fn msg_reject_msg_size() -> usize;
    pub fn msg_reject_msg_copy(dst: usize, offset: usize, size: usize);
    pub fn msg_deadline() -> u64;
    pub fn msg_reply_data_append(src: usize, size: usize);
    pub fn msg_reply();
    pub fn msg_reject(src: usize, size: usize);
    pub fn msg_cycles_available128(dst: usize);
    pub fn msg_cycles_refunded128(dst: usize);
    pub fn msg_cycles_accept128(max_amount_high: u64, max_amount_low: u64, dst: usize);
    pub fn cycles_burn128(amount_high: u64, amount_low: u64, dst: usize);
    pub fn canister_self_size() -> usize;
    pub fn canister_self_copy(dst: usize, offset: usize, size: usize);
    pub fn canister_cycle_balance128(dst: usize);
    pub fn canister_status() -> u32;
    pub fn canister_version() -> u64;
    pub fn msg_method_name_size() -> usize;
    pub fn msg_method_name_copy(dst: usize, offset: usize, size: usize);
    pub fn accept_message();
    pub fn call_new(
        callee_src: usize,
        callee_size: usize,
        name_src: usize,
        name_size: usize,
        reply_fun: usize,
        reply_env: usize,
        reject_fun: usize,
        reject_env: usize,
    );
    pub fn call_on_cleanup(fun: usize, env: usize);
    pub fn call_data_append(src: usize, size: usize);
    pub fn call_with_best_effort_response(timeout_seconds: u32);
    pub fn call_cycles_add128(amount_high: u64, amount_low: u64);
    pub fn call_perform() -> u32;
    pub fn stable64_size() -> u64;
    pub fn stable64_grow(new_pages: u64) -> u64;
    pub fn stable64_write(offset: u64, src: u64, size: u64);
    pub fn stable64_read(dst: u64, offset: u64, size: u64);
    pub fn certified_data_set(src: usize, size: usize);
    pub fn data_certificate_present() -> u32;
    pub fn data_certificate_size() -> usize;
    pub fn data_certificate_copy(dst: usize, offset: usize, size: usize);
    pub fn time() -> u64;
    pub fn global_timer_set(timestamp: u64) -> u64;
    pub fn performance_counter(counter_type: u32) -> u64;
    pub fn is_controller(src: usize, size: usize) -> u32;
    pub fn in_replicated_execution() -> u32;
    pub fn debug_print(src: usize, size: usize);
    pub fn trap(src: usize, size: usize);
}

#[cfg(not(target_family = "wasm"))]
#[allow(unused_variables)]
#[allow(clippy::missing_safety_doc)]
#[allow(clippy::too_many_arguments)]
mod non_wasm {
    pub unsafe fn msg_arg_data_size() -> usize {
        panic!("msg_arg_data_size should only be called inside canisters.");
    }
    pub unsafe fn msg_arg_data_copy(dst: usize, offset: usize, size: usize) {
        panic!("msg_arg_data_copy should only be called inside canisters.");
    }
    pub unsafe fn msg_caller_size() -> usize {
        panic!("msg_caller_size should only be called inside canisters.");
    }
    pub unsafe fn msg_caller_copy(dst: usize, offset: usize, size: usize) {
        panic!("msg_caller_copy should only be called inside canisters.");
    }
    pub unsafe fn msg_reject_code() -> u32 {
        panic!("msg_reject_code should only be called inside canisters.");
    }
    pub unsafe fn msg_reject_msg_size() -> usize {
        panic!("msg_reject_msg_size should only be called inside canisters.");
    }
    pub unsafe fn msg_reject_msg_copy(dst: usize, offset: usize, size: usize) {
        panic!("msg_reject_msg_copy should only be called inside canisters.");
    }
    pub unsafe fn msg_deadline() -> u64 {
        panic!("msg_deadline should only be called inside canisters.");
    }
    pub unsafe fn msg_reply_data_append(src: usize, size: usize) {
        panic!("msg_reply_data_append should only be called inside canisters.");
    }
    pub unsafe fn msg_reply() {
        panic!("msg_reply should only be called inside canisters.");
    }
    pub unsafe fn msg_reject(src: usize, size: usize) {
        panic!("msg_reject should only be called inside canisters.");
    }
    pub unsafe fn msg_cycles_available128(dst: usize) {
        panic!("msg_cycles_available128 should only be called inside canisters.");
    }
    pub unsafe fn msg_cycles_refunded128(dst: usize) {
        panic!("msg_cycles_refunded128 should only be called inside canisters.");
    }
    pub unsafe fn msg_cycles_accept128(max_amount_high: u64, max_amount_low: u64, dst: usize) {
        panic!("msg_cycles_accept128 should only be called inside canisters.");
    }
    pub unsafe fn cycles_burn128(amount_high: u64, amount_low: u64, dst: usize) {
        panic!("cycles_burn128 should only be called inside canisters.");
    }
    pub unsafe fn canister_self_size() -> usize {
        panic!("canister_self_size should only be called inside canisters.");
    }
    pub unsafe fn canister_self_copy(dst: usize, offset: usize, size: usize) {
        panic!("canister_self_copy should only be called inside canisters.");
    }
    pub unsafe fn canister_cycle_balance128(dst: usize) {
        panic!("canister_cycle_balance128 should only be called inside canisters.");
    }
    pub unsafe fn canister_status() -> u32 {
        panic!("canister_status should only be called inside canisters.");
    }
    pub unsafe fn canister_version() -> u64 {
        panic!("canister_version should only be called inside canisters.");
    }
    pub unsafe fn msg_method_name_size() -> usize {
        panic!("msg_method_name_size should only be called inside canisters.");
    }
    pub unsafe fn msg_method_name_copy(dst: usize, offset: usize, size: usize) {
        panic!("msg_method_name_copy should only be called inside canisters.");
    }
    pub unsafe fn accept_message() {
        panic!("accept_message should only be called inside canisters.");
    }
    pub unsafe fn call_new(
        callee_src: usize,
        callee_size: usize,
        name_src: usize,
        name_size: usize,
        reply_fun: usize,
        reply_env: usize,
        reject_fun: usize,
        reject_env: usize,
    ) {
        panic!("call_new should only be called inside canisters.");
    }
    pub unsafe fn call_on_cleanup(fun: usize, env: usize) {
        panic!("call_on_cleanup should only be called inside canisters.");
    }
    pub unsafe fn call_data_append(src: usize, size: usize) {
        panic!("call_data_append should only be called inside canisters.");
    }
    pub unsafe fn call_with_best_effort_response(timeout_seconds: u32) {
        panic!("call_with_best_effort_response should only be called inside canisters.");
    }
    pub unsafe fn call_cycles_add128(amount_high: u64, amount_low: u64) {
        panic!("call_cycles_add128 should only be called inside canisters.");
    }
    pub unsafe fn call_perform() -> u32 {
        panic!("call_perform should only be called inside canisters.");
    }
    pub unsafe fn stable64_size() -> u64 {
        panic!("stable64_size should only be called inside canisters.");
    }
    pub unsafe fn stable64_grow(new_pages: u64) -> u64 {
        panic!("stable64_grow should only be called inside canisters.");
    }
    pub unsafe fn stable64_write(offset: u64, src: u64, size: u64) {
        panic!("stable64_write should only be called inside canisters.");
    }
    pub unsafe fn stable64_read(dst: u64, offset: u64, size: u64) {
        panic!("stable64_read should only be called inside canisters.");
    }
    pub unsafe fn certified_data_set(src: usize, size: usize) {
        panic!("certified_data_set should only be called inside canisters.");
    }
    pub unsafe fn data_certificate_present() -> u32 {
        panic!("data_certificate_present should only be called inside canisters.");
    }
    pub unsafe fn data_certificate_size() -> usize {
        panic!("data_certificate_size should only be called inside canisters.");
    }
    pub unsafe fn data_certificate_copy(dst: usize, offset: usize, size: usize) {
        panic!("data_certificate_copy should only be called inside canisters.");
    }
    pub unsafe fn time() -> u64 {
        panic!("time should only be called inside canisters.");
    }
    pub unsafe fn global_timer_set(timestamp: u64) -> u64 {
        panic!("global_timer_set should only be called inside canisters.");
    }
    pub unsafe fn performance_counter(counter_type: u32) -> u64 {
        panic!("performance_counter should only be called inside canisters.");
    }
    pub unsafe fn is_controller(src: usize, size: usize) -> u32 {
        panic!("is_controller should only be called inside canisters.");
    }
    pub unsafe fn in_replicated_execution() -> u32 {
        panic!("in_replicated_execution should only be called inside canisters.");
    }
    pub unsafe fn debug_print(src: usize, size: usize) {
        panic!("debug_print should only be called inside canisters.");
    }
    pub unsafe fn trap(src: usize, size: usize) {
        panic!("trap should only be called inside canisters.");
    }
}

#[cfg(not(target_family = "wasm"))]
pub use non_wasm::*;
