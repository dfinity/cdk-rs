// This file is generated from ic0.txt.
// Don't manually modify it.
#[cfg(any(feature = "mock", target_arch = "wasm32"))]
#[link(wasm_import_module = "ic0")]
extern "C" {
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_arg_data_size")]
    pub fn msg_arg_data_size() -> usize;
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_arg_data_copy")]
    pub fn msg_arg_data_copy(dst: usize, offset: usize, size: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_caller_size")]
    pub fn msg_caller_size() -> usize;
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_caller_copy")]
    pub fn msg_caller_copy(dst: usize, offset: usize, size: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_reject_code")]
    pub fn msg_reject_code() -> u32;
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_reject_msg_size")]
    pub fn msg_reject_msg_size() -> usize;
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_reject_msg_copy")]
    pub fn msg_reject_msg_copy(dst: usize, offset: usize, size: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_reply_data_append")]
    pub fn msg_reply_data_append(src: usize, size: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_reply")]
    pub fn msg_reply();
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_reject")]
    pub fn msg_reject(src: usize, size: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_cycles_available")]
    pub fn msg_cycles_available() -> u64;
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_cycles_available128")]
    pub fn msg_cycles_available128(dst: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_cycles_refunded")]
    pub fn msg_cycles_refunded() -> u64;
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_cycles_refunded128")]
    pub fn msg_cycles_refunded128(dst: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_cycles_accept")]
    pub fn msg_cycles_accept(max_amount: u64) -> u64;
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_cycles_accept128")]
    pub fn msg_cycles_accept128(max_amount_high: u64, max_amount_low: u64, dst: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_cycles_burn128")]
    pub fn cycles_burn128(amount_high: u64, amount_low: u64, dst: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_canister_self_size")]
    pub fn canister_self_size() -> usize;
    #[cfg_attr(feature = "mock", link_name = "ic0_canister_self_copy")]
    pub fn canister_self_copy(dst: usize, offset: usize, size: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_canister_cycle_balance")]
    pub fn canister_cycle_balance() -> u64;
    #[cfg_attr(feature = "mock", link_name = "ic0_canister_cycle_balance128")]
    pub fn canister_cycle_balance128(dst: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_canister_status")]
    pub fn canister_status() -> u32;
    #[cfg_attr(feature = "mock", link_name = "ic0_canister_version")]
    pub fn canister_version() -> u64;
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_method_name_size")]
    pub fn msg_method_name_size() -> usize;
    #[cfg_attr(feature = "mock", link_name = "ic0_msg_method_name_copy")]
    pub fn msg_method_name_copy(dst: usize, offset: usize, size: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_accept_message")]
    pub fn accept_message();
    #[cfg_attr(feature = "mock", link_name = "ic0_call_new")]
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
    #[cfg_attr(feature = "mock", link_name = "ic0_call_on_cleanup")]
    pub fn call_on_cleanup(fun: usize, env: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_call_data_append")]
    pub fn call_data_append(src: usize, size: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_call_cycles_add")]
    pub fn call_cycles_add(amount: u64);
    #[cfg_attr(feature = "mock", link_name = "ic0_call_cycles_add128")]
    pub fn call_cycles_add128(amount_high: u64, amount_low: u64);
    #[cfg_attr(feature = "mock", link_name = "ic0_call_perform")]
    pub fn call_perform() -> u32;
    #[cfg_attr(feature = "mock", link_name = "ic0_stable_size")]
    pub fn stable_size() -> i32;
    #[cfg_attr(feature = "mock", link_name = "ic0_stable_grow")]
    pub fn stable_grow(new_pages: i32) -> i32;
    #[cfg_attr(feature = "mock", link_name = "ic0_stable_write")]
    pub fn stable_write(offset: u32, src: usize, size: u32);
    #[cfg_attr(feature = "mock", link_name = "ic0_stable_read")]
    pub fn stable_read(dst: usize, offset: u32, size: u32);
    #[cfg_attr(feature = "mock", link_name = "ic0_stable64_size")]
    pub fn stable64_size() -> i64;
    #[cfg_attr(feature = "mock", link_name = "ic0_stable64_grow")]
    pub fn stable64_grow(new_pages: i64) -> i64;
    #[cfg_attr(feature = "mock", link_name = "ic0_stable64_write")]
    pub fn stable64_write(offset: u64, src: u64, size: u64);
    #[cfg_attr(feature = "mock", link_name = "ic0_stable64_read")]
    pub fn stable64_read(dst: u64, offset: u64, size: u64);
    #[cfg_attr(feature = "mock", link_name = "ic0_certified_data_set")]
    pub fn certified_data_set(src: usize, size: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_data_certificate_present")]
    pub fn data_certificate_present() -> u32;
    #[cfg_attr(feature = "mock", link_name = "ic0_data_certificate_size")]
    pub fn data_certificate_size() -> usize;
    #[cfg_attr(feature = "mock", link_name = "ic0_data_certificate_copy")]
    pub fn data_certificate_copy(dst: usize, offset: usize, size: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_time")]
    pub fn time() -> i64;
    #[cfg_attr(feature = "mock", link_name = "ic0_global_timer_set")]
    pub fn global_timer_set(timestamp: i64) -> i64;
    #[cfg_attr(feature = "mock", link_name = "ic0_performance_counter")]
    pub fn performance_counter(counter_type: u32) -> u64;
    #[cfg_attr(feature = "mock", link_name = "ic0_is_controller")]
    pub fn is_controller(src: usize, size: usize) -> usize;
    #[cfg_attr(feature = "mock", link_name = "ic0_in_replicated_execution")]
    pub fn in_replicated_execution() -> u32;
    #[cfg_attr(feature = "mock", link_name = "ic0_debug_print")]
    pub fn debug_print(src: usize, size: usize);
    #[cfg_attr(feature = "mock", link_name = "ic0_trap")]
    pub fn trap(src: usize, size: usize);
}

#[cfg(not(any(feature = "mock", target_arch = "wasm32")))]
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
    pub unsafe fn msg_reply_data_append(src: usize, size: usize) {
        panic!("msg_reply_data_append should only be called inside canisters.");
    }
    pub unsafe fn msg_reply() {
        panic!("msg_reply should only be called inside canisters.");
    }
    pub unsafe fn msg_reject(src: usize, size: usize) {
        panic!("msg_reject should only be called inside canisters.");
    }
    pub unsafe fn msg_cycles_available() -> u64 {
        panic!("msg_cycles_available should only be called inside canisters.");
    }
    pub unsafe fn msg_cycles_available128(dst: usize) {
        panic!("msg_cycles_available128 should only be called inside canisters.");
    }
    pub unsafe fn msg_cycles_refunded() -> u64 {
        panic!("msg_cycles_refunded should only be called inside canisters.");
    }
    pub unsafe fn msg_cycles_refunded128(dst: usize) {
        panic!("msg_cycles_refunded128 should only be called inside canisters.");
    }
    pub unsafe fn msg_cycles_accept(max_amount: u64) -> u64 {
        panic!("msg_cycles_accept should only be called inside canisters.");
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
    pub unsafe fn canister_cycle_balance() -> u64 {
        panic!("canister_cycle_balance should only be called inside canisters.");
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
    pub unsafe fn call_cycles_add(amount: u64) {
        panic!("call_cycles_add should only be called inside canisters.");
    }
    pub unsafe fn call_cycles_add128(amount_high: u64, amount_low: u64) {
        panic!("call_cycles_add128 should only be called inside canisters.");
    }
    pub unsafe fn call_perform() -> u32 {
        panic!("call_perform should only be called inside canisters.");
    }
    pub unsafe fn stable_size() -> i32 {
        panic!("stable_size should only be called inside canisters.");
    }
    pub unsafe fn stable_grow(new_pages: i32) -> i32 {
        panic!("stable_grow should only be called inside canisters.");
    }
    pub unsafe fn stable_write(offset: u32, src: usize, size: u32) {
        panic!("stable_write should only be called inside canisters.");
    }
    pub unsafe fn stable_read(dst: usize, offset: u32, size: u32) {
        panic!("stable_read should only be called inside canisters.");
    }
    pub unsafe fn stable64_size() -> i64 {
        panic!("stable64_size should only be called inside canisters.");
    }
    pub unsafe fn stable64_grow(new_pages: i64) -> i64 {
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
    pub unsafe fn time() -> i64 {
        panic!("time should only be called inside canisters.");
    }
    pub unsafe fn global_timer_set(timestamp: i64) -> i64 {
        panic!("global_timer_set should only be called inside canisters.");
    }
    pub unsafe fn performance_counter(counter_type: u32) -> u64 {
        panic!("performance_counter should only be called inside canisters.");
    }
    pub unsafe fn is_controller(src: usize, size: usize) -> usize {
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

#[cfg(not(any(feature = "mock", target_arch = "wasm32")))]
pub use non_wasm::*;
