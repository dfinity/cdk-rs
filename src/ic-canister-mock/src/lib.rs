#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_arg_data_size() -> i32 {
    panic!("msg_arg_data_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_arg_data_copy(dst: i32, offset: i32, size: i32) {
    panic!("msg_arg_data_copy should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_caller_size() -> i32 {
    panic!("msg_caller_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_caller_copy(dst: i32, offset: i32, size: i32) {
    panic!("msg_caller_copy should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reject_code() -> i32 {
    panic!("msg_reject_code should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reject_msg_size() -> i32 {
    panic!("msg_reject_msg_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reject_msg_copy(dst: i32, offset: i32, size: i32) {
    panic!("msg_reject_msg_copy should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reply_data_append(src: i32, size: i32) {
    panic!("msg_reply_data_append should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reply() {
    panic!("msg_reply should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reject(src: i32, size: i32) {
    panic!("msg_reject should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_available() -> i64 {
    panic!("msg_cycles_available should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_available128(dst: i32) {
    panic!("msg_cycles_available128 should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_refunded() -> i64 {
    panic!("msg_cycles_refunded should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_refunded128(dst: i32) {
    panic!("msg_cycles_refunded128 should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_accept(max_amount: i64) -> i64 {
    panic!("msg_cycles_accept should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_accept128(
    max_amount_high: i64,
    max_amount_low: i64,
    dst: i32,
) {
    panic!("msg_cycles_accept128 should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_cycles_burn128(amount_high: i64, amount_low: i64, dst: i32) {
    panic!("cycles_burn128 should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_self_size() -> i32 {
    panic!("canister_self_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_self_copy(dst: i32, offset: i32, size: i32) {
    panic!("canister_self_copy should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_cycle_balance() -> i64 {
    panic!("canister_cycle_balance should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_cycle_balance128(dst: i32) {
    panic!("canister_cycle_balance128 should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_status() -> i32 {
    panic!("canister_status should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_version() -> i64 {
    panic!("canister_version should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_method_name_size() -> i32 {
    panic!("msg_method_name_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_method_name_copy(dst: i32, offset: i32, size: i32) {
    panic!("msg_method_name_copy should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_accept_message() {
    panic!("accept_message should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_new(
    callee_src: i32,
    callee_size: i32,
    name_src: i32,
    name_size: i32,
    reply_fun: i32,
    reply_env: i32,
    reject_fun: i32,
    reject_env: i32,
) {
    panic!("call_new should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_on_cleanup(fun: i32, env: i32) {
    panic!("call_on_cleanup should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_data_append(src: i32, size: i32) {
    panic!("call_data_append should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_cycles_add(amount: i64) {
    panic!("call_cycles_add should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_cycles_add128(amount_high: i64, amount_low: i64) {
    panic!("call_cycles_add128 should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_perform() -> i32 {
    panic!("call_perform should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable_size() -> i32 {
    panic!("stable_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable_grow(new_pages: i32) -> i32 {
    panic!("stable_grow should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable_write(offset: i32, src: i32, size: i32) {
    panic!("stable_write should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable_read(dst: i32, offset: i32, size: i32) {
    panic!("stable_read should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable64_size() -> i64 {
    panic!("stable64_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable64_grow(new_pages: i64) -> i64 {
    panic!("stable64_grow should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable64_write(offset: i64, src: i64, size: i64) {
    panic!("stable64_write should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable64_read(dst: i64, offset: i64, size: i64) {
    panic!("stable64_read should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_certified_data_set(src: i32, size: i32) {
    panic!("certified_data_set should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_data_certificate_present() -> i32 {
    panic!("data_certificate_present should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_data_certificate_size() -> i32 {
    panic!("data_certificate_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_data_certificate_copy(dst: i32, offset: i32, size: i32) {
    panic!("data_certificate_copy should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_time() -> i64 {
    panic!("time should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_global_timer_set(timestamp: i64) -> i64 {
    panic!("global_timer_set should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_performance_counter(counter_type: i32) -> i64 {
    panic!("performance_counter should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_is_controller(src: i32, size: i32) -> i32 {
    panic!("is_controller should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_debug_print(src: i32, size: i32) {
    panic!("debug_print should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_trap(src: i32, size: i32) {
    panic!("trap should only be called inside canisters.");
}
