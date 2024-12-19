use candid::Principal;
use ic_cdk::api::*;

#[export_name = "canister_update call_msg_arg_data"]
fn call_msg_arg_data() {
    assert_eq!(msg_arg_data(), vec![42]);
    msg_reply(vec![]);
}

#[export_name = "canister_update call_msg_caller"]
fn call_msg_caller() {
    assert_eq!(msg_caller(), Principal::anonymous());
    msg_reply(vec![]);
}

#[export_name = "canister_update call_msg_reply"]
fn call_msg_reply() {
    msg_reply(vec![42]);
}

#[export_name = "canister_update call_msg_reject"]
fn call_msg_reject() {
    msg_reject("e2e test reject");
}

#[export_name = "canister_update call_msg_cycles_available"]
fn call_msg_cycles_available() {
    assert_eq!(msg_cycles_available(), 0);
    msg_reply(vec![]);
}

#[export_name = "canister_update call_msg_cycles_accept"]
fn call_msg_cycles_accept() {
    // The available cycles are 0, so the actual cycles accepted are 0.
    assert_eq!(msg_cycles_accept(1000), 0);
    msg_reply(vec![]);
}

#[export_name = "canister_update call_cycles_burn"]
fn call_cycles_burn() {
    assert_eq!(cycles_burn(1000), 1000);
    msg_reply(vec![]);
}

#[export_name = "canister_update call_canister_self"]
fn call_canister_self() {
    let self_id = canister_self();
    // The sender sended canister ID
    let data = msg_arg_data();
    assert_eq!(self_id.as_slice(), data);
    msg_reply(vec![]);
}

#[export_name = "canister_update call_canister_cycle_balance"]
fn call_canister_cycle_balance() {
    assert!(canister_cycle_balance() > 0);
    msg_reply(vec![]);
}

#[export_name = "canister_update call_canister_status"]
fn call_canister_status() {
    assert_eq!(canister_status(), 1);
    msg_reply(vec![]);
}

#[export_name = "canister_update call_canister_version"]
fn call_canister_version() {
    assert!(canister_version() > 0);
    msg_reply(vec![]);
}

#[export_name = "canister_inspect_message"]
fn inspect_message() {
    assert!(msg_method_name().starts_with("call_"));
    accept_message();
}

#[export_name = "canister_update call_stable"]
fn call_stable() {
    assert_eq!(stable_size(), 0);
    assert_eq!(stable_grow(1), 0);
    let data = vec![42];
    stable_write(0, &data);
    let mut read_buf = vec![0];
    stable_read(0, &mut read_buf);
    assert_eq!(read_buf, data);
    msg_reply(vec![]);
}

#[export_name = "canister_update call_certified_data_set"]
fn call_certified_data_set() {
    certified_data_set(vec![42]);
    msg_reply(vec![]);
}

#[export_name = "canister_query call_data_certificate"]
fn call_data_certificate() {
    assert!(data_certificate().is_some());
    msg_reply(vec![]);
}

#[export_name = "canister_update call_time"]
fn call_time() {
    assert!(time() > 0);
    msg_reply(vec![]);
}

#[export_name = "canister_update call_performance_counter"]
fn call_performance_counter() {
    let ic0 = performance_counter(0);
    let ic1 = performance_counter(PerformanceCounterType::InstructionCounter);
    let ic2 = instruction_counter();
    assert!(ic0 < ic1);
    assert!(ic1 < ic2);
    let ccic0 = performance_counter(1);
    let ccic1 = performance_counter(PerformanceCounterType::CallContextInstructionCounter);
    let ccic2 = call_context_instruction_counter();
    assert!(ccic0 < ccic1);
    assert!(ccic1 < ccic2);
    msg_reply(vec![]);
}

#[export_name = "canister_update call_is_controller"]
fn call_is_controller() {
    // The canister was created by the anonymous principal.
    assert!(is_controller(&Principal::anonymous()));
    msg_reply(vec![]);
}

/// This entry point will be called by both update and query calls.
/// The query call will return 0, and the update call will return 1.
#[export_name = "canister_query call_in_replicated_execution"]
fn call_in_replicated_execution() {
    let res = match in_replicated_execution() {
        true => 1,
        false => 0,
    };
    msg_reply(vec![res]);
}

#[export_name = "canister_update call_debug_print"]
fn call_debug_print() {
    debug_print("Hello, world!");
    msg_reply(vec![]);
}

#[export_name = "canister_update call_trap"]
fn call_trap() {
    trap("It's a trap!");
}

fn main() {}
