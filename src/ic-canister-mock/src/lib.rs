#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_arg_data_size() -> usize {
    panic!("msg_arg_data_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_arg_data_copy(dst: usize, offset: usize, size: usize) {
    panic!("msg_arg_data_copy should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_caller_size() -> usize {
    panic!("msg_caller_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_caller_copy(dst: usize, offset: usize, size: usize) {
    panic!("msg_caller_copy should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reject_code() -> u32 {
    panic!("msg_reject_code should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reject_msg_size() -> usize {
    panic!("msg_reject_msg_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reject_msg_copy(dst: usize, offset: usize, size: usize) {
    panic!("msg_reject_msg_copy should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reply_data_append(src: usize, size: usize) {
    panic!("msg_reply_data_append should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reply() {
    panic!("msg_reply should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_reject(src: usize, size: usize) {
    panic!("msg_reject should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_available() -> u64 {
    panic!("msg_cycles_available should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_available128(dst: usize) {
    panic!("msg_cycles_available128 should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_refunded() -> u64 {
    panic!("msg_cycles_refunded should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_refunded128(dst: usize) {
    panic!("msg_cycles_refunded128 should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_accept(max_amount: u64) -> u64 {
    panic!("msg_cycles_accept should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_cycles_accept128(
    max_amount_high: u64,
    max_amount_low: u64,
    dst: usize,
) {
    panic!("msg_cycles_accept128 should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_cycles_burn128(amount_high: i64, amount_low: i64, dst: usize) {
    panic!("cycles_burn128 should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_self_size() -> usize {
    panic!("canister_self_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_self_copy(dst: usize, offset: usize, size: usize) {
    panic!("canister_self_copy should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_cycle_balance() -> u64 {
    panic!("canister_cycle_balance should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_cycle_balance128(dst: usize) {
    panic!("canister_cycle_balance128 should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_status() -> u32 {
    panic!("canister_status should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_canister_version() -> u64 {
    panic!("canister_version should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_method_name_size() -> usize {
    panic!("msg_method_name_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_msg_method_name_copy(dst: usize, offset: usize, size: usize) {
    panic!("msg_method_name_copy should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_accept_message() {
    panic!("accept_message should only be called inside canisters.");
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
    panic!("call_new should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_on_cleanup(fun: usize, env: usize) {
    panic!("call_on_cleanup should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_data_append(src: usize, size: usize) {
    panic!("call_data_append should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_cycles_add(amount: u64) {
    panic!("call_cycles_add should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_cycles_add128(amount_high: u64, amount_low: u64) {
    panic!("call_cycles_add128 should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_call_perform() -> u32 {
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
unsafe extern "C-unwind" fn ic0_stable_write(offset: u32, src: usize, size: u32) {
    panic!("stable_write should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable_read(dst: usize, offset: u32, size: u32) {
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
unsafe extern "C-unwind" fn ic0_stable64_write(offset: u64, src: u64, size: u64) {
    panic!("stable64_write should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_stable64_read(dst: u64, offset: u64, size: u64) {
    panic!("stable64_read should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_certified_data_set(src: usize, size: usize) {
    panic!("certified_data_set should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_data_certificate_present() -> u32 {
    panic!("data_certificate_present should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_data_certificate_size() -> usize {
    panic!("data_certificate_size should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_data_certificate_copy(dst: usize, offset: usize, size: usize) {
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
unsafe extern "C-unwind" fn ic0_performance_counter(counter_type: u32) -> i64 {
    panic!("performance_counter should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_is_controller(src: usize, size: usize) -> usize {
    panic!("is_controller should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_debug_print(src: usize, size: usize) {
    panic!("debug_print should only be called inside canisters.");
}
#[no_mangle]
unsafe extern "C-unwind" fn ic0_trap(src: usize, size: usize) {
    panic!("trap should only be called inside canisters.");
}
