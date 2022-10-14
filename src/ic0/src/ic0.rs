// This file is generated from ic0.txt.
// Don't manually modify it.
#[link(wasm_import_module = "ic0")]
extern "C" {
    pub fn msg_arg_data_size() -> i32;
    pub fn msg_arg_data_copy(dst: i32, offset: i32, size: i32);
    pub fn msg_caller_size() -> i32;
    pub fn msg_caller_copy(dst: i32, offset: i32, size: i32);
    pub fn msg_reject_code() -> i32;
    pub fn msg_reject_msg_size() -> i32;
    pub fn msg_reject_msg_copy(dst: i32, offset: i32, size: i32);
    pub fn msg_reply_data_append(src: i32, size: i32);
    pub fn msg_reply();
    pub fn msg_reject(src: i32, size: i32);
    pub fn msg_cycles_available() -> i64;
    pub fn msg_cycles_available128(dst: i32);
    pub fn msg_cycles_refunded() -> i64;
    pub fn msg_cycles_refunded128(dst: i32);
    pub fn msg_cycles_accept(max_amount: i64) -> i64;
    pub fn msg_cycles_accept128(max_amount_high: i64, max_amount_low: i64, dst: i32);
    pub fn canister_self_size() -> i32;
    pub fn canister_self_copy(dst: i32, offset: i32, size: i32);
    pub fn canister_cycle_balance() -> i64;
    pub fn canister_cycle_balance128(dst: i32);
    pub fn canister_status() -> i32;
    pub fn msg_method_name_size() -> i32;
    pub fn msg_method_name_copy(dst: i32, offset: i32, size: i32);
    pub fn accept_message();
    pub fn call_new(
        callee_src: i32,
        callee_size: i32,
        name_src: i32,
        name_size: i32,
        reply_fun: i32,
        reply_env: i32,
        reject_fun: i32,
        reject_env: i32,
    );
    pub fn call_on_cleanup(fun: i32, env: i32);
    pub fn call_data_append(src: i32, size: i32);
    pub fn call_cycles_add(amount: i64);
    pub fn call_cycles_add128(amount_high: i64, amount_low: i64);
    pub fn call_perform() -> i32;
    pub fn stable_size() -> i32;
    pub fn stable_grow(new_pages: i32) -> i32;
    pub fn stable_write(offset: i32, src: i32, size: i32);
    pub fn stable_read(dst: i32, offset: i32, size: i32);
    pub fn stable64_size() -> i64;
    pub fn stable64_grow(new_pages: i64) -> i64;
    pub fn stable64_write(offset: i64, src: i64, size: i64);
    pub fn stable64_read(dst: i64, offset: i64, size: i64);
    pub fn certified_data_set(src: i32, size: i32);
    pub fn data_certificate_present() -> i32;
    pub fn data_certificate_size() -> i32;
    pub fn data_certificate_copy(dst: i32, offset: i32, size: i32);
    pub fn time() -> i64;
    pub fn performance_counter(counter_type: i32) -> i64;
    pub fn debug_print(src: i32, size: i32);
    pub fn trap(src: i32, size: i32);
}
