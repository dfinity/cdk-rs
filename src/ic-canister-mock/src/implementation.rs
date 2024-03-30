#![allow(unused)]

use std::cell::RefCell;

use candid::Principal;
use time::OffsetDateTime;

use crate::{
    enqueue_message, with_active_message, with_active_message_mut, Callback, Message,
    MessageStatus, LOADED_CANISTERS,
};

thread_local! {
    static MESSAGE_BUILDER: RefCell<Option<Message>> = <_>::default();
}

pub unsafe extern "C-unwind" fn msg_arg_data_size() -> usize {
    with_active_message(|msg| {
        let payload = msg
            .payload
            .as_ref()
            .unwrap_or_else(|| panic!("msg_arg_data not accessible in this context"));
        payload.len()
    })
}
pub unsafe extern "C-unwind" fn msg_arg_data_copy(dst: usize, offset: usize, size: usize) {
    with_active_message(|msg| {
        let dst = dst as *mut u8;
        let payload = msg
            .payload
            .as_ref()
            .unwrap_or_else(|| panic!("msg_arg_data not accessible in this context"));
        unsafe { dst.copy_from_nonoverlapping(payload[offset..offset + size].as_ptr(), size) };
    });
}
pub unsafe extern "C-unwind" fn msg_caller_size() -> usize {
    with_active_message(|msg| msg.from.as_slice().len())
}
pub unsafe extern "C-unwind" fn msg_caller_copy(dst: usize, offset: usize, size: usize) {
    with_active_message(|msg| {
        let dst = dst as *mut u8;
        unsafe {
            dst.copy_from_nonoverlapping(msg.from.as_slice()[offset..offset + size].as_ptr(), size)
        };
    });
}
pub unsafe extern "C-unwind" fn msg_reject_code() -> u32 {
    panic!("msg_reject_code should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn msg_reject_msg_size() -> usize {
    panic!("msg_reject_msg_size should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn msg_reject_msg_copy(dst: usize, offset: usize, size: usize) {
    panic!("msg_reject_msg_copy should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn msg_reply_data_append(src: usize, size: usize) {
    panic!("msg_reply_data_append should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn msg_reply() {
    panic!("msg_reply should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn msg_reject(src: usize, size: usize) {
    panic!("msg_reject should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn msg_cycles_available() -> u64 {
    panic!("msg_cycles_available should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn msg_cycles_available128(dst: usize) {
    panic!("msg_cycles_available128 should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn msg_cycles_refunded() -> u64 {
    panic!("msg_cycles_refunded should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn msg_cycles_refunded128(dst: usize) {
    panic!("msg_cycles_refunded128 should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn msg_cycles_accept(max_amount: u64) -> u64 {
    panic!("msg_cycles_accept should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn msg_cycles_accept128(
    max_amount_high: u64,
    max_amount_low: u64,
    dst: usize,
) {
    panic!("msg_cycles_accept128 should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn cycles_burn128(amount_high: u64, amount_low: u64, dst: usize) {
    panic!("cycles_burn128 should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn canister_self_size() -> usize {
    with_active_message(|m| m.to.as_slice().len())
}
pub unsafe extern "C-unwind" fn canister_self_copy(dst: usize, offset: usize, size: usize) {
    with_active_message(|m| {
        let dst = dst as *mut u8;
        unsafe {
            dst.copy_from_nonoverlapping(m.to.as_slice()[offset..offset + size].as_ptr(), size)
        };
    })
}
pub unsafe extern "C-unwind" fn canister_cycle_balance() -> u64 {
    panic!("canister_cycle_balance should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn canister_cycle_balance128(dst: usize) {
    panic!("canister_cycle_balance128 should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn canister_status() -> u32 {
    panic!("canister_status should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn canister_version() -> u64 {
    panic!("canister_version should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn msg_method_name_size() -> usize {
    with_active_message(|msg| msg.method.len())
}
pub unsafe extern "C-unwind" fn msg_method_name_copy(dst: usize, offset: usize, size: usize) {
    with_active_message(|msg| {
        let dst = dst as *mut u8;
        dst.copy_from_nonoverlapping(msg.method[offset..offset + size].as_ptr(), size)
    })
}
pub unsafe extern "C-unwind" fn accept_message() {
    panic!("accept_message should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn call_new(
    callee_src: usize,
    callee_size: usize,
    name_src: usize,
    name_size: usize,
    reply_fun: usize,
    reply_env: usize,
    reject_fun: usize,
    reject_env: usize,
) {
    if MESSAGE_BUILDER.with_borrow(|m| m.is_some()) {
        panic!("call_new is already in progress");
    }
    let callee = Principal::from_slice(std::slice::from_raw_parts(
        callee_src as *const u8,
        callee_size,
    ));
    let name = std::str::from_utf8(std::slice::from_raw_parts(name_src as *const u8, name_size))
        .unwrap_or_else(|_| panic!("invalid canister method"));
    // todo calls made outside context
    let message = with_active_message(|m| Message {
        from: m.to,
        to: callee,
        method: name.to_string(),
        cycles: 0,
        payload: Some(vec![]),
        status: MessageStatus::Pending,
        callback: Some(Callback {
            reply_callback: callback_or_dummy(reply_fun),
            reply_env: reply_env as *mut (),
            reject_callback: callback_or_dummy(reject_fun),
            reject_env: reject_env as *mut (),
            cleanup_callback: None,
            cleanup_env: None,
            message_in_progress: None,
        }),
    });
    MESSAGE_BUILDER.with_borrow_mut(|m| *m = Some(message));
}
pub unsafe extern "C-unwind" fn call_on_cleanup(fun: usize, env: usize) {
    MESSAGE_BUILDER.with_borrow_mut(|m| {
        let m = m
            .as_mut()
            .unwrap_or_else(|| panic!("call_on_cleanup called before call_new"));
        let callback = m.callback.as_mut().unwrap();
        callback.cleanup_callback = Some(callback_or_dummy(fun));
        callback.cleanup_env = Some(env as *mut ());
    })
}
pub unsafe extern "C-unwind" fn call_data_append(src: usize, size: usize) {
    MESSAGE_BUILDER.with_borrow_mut(|m| {
        let m = m
            .as_mut()
            .unwrap_or_else(|| panic!("call_data_append called before call_new"));
        let data = unsafe { std::slice::from_raw_parts(src as *const u8, size) };
        m.payload.as_mut().unwrap().extend_from_slice(data);
    });
}
pub unsafe extern "C-unwind" fn call_cycles_add(amount: u64) {
    panic!("call_cycles_add should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn call_cycles_add128(amount_high: u64, amount_low: u64) {
    panic!("call_cycles_add128 should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn call_perform() -> u32 {
    let mut message = MESSAGE_BUILDER
        .with_borrow_mut(|m| m.take())
        .unwrap_or_else(|| panic!("call_perform called before call_new "));
    with_active_message_mut(|m| {
        message.callback.as_mut().unwrap().message_in_progress = Some(Box::new(Message {
            callback: m.callback.take(), // todo this is wrong. the callback needs to be run after the *last* branch, not the first. probably have to reference-count.
            cycles: m.cycles,
            from: m.from,
            to: m.to,
            method: m.method.clone(),
            payload: m.payload.clone(),
            status: MessageStatus::Pending,
        }));
    });
    enqueue_message(message);
    0
}
pub unsafe extern "C-unwind" fn stable_size() -> i32 {
    panic!("stable_size should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn stable_grow(new_pages: i32) -> i32 {
    panic!("stable_grow should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn stable_write(offset: u32, src: usize, size: u32) {
    panic!("stable_write should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn stable_read(dst: usize, offset: u32, size: u32) {
    panic!("stable_read should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn stable64_size() -> i64 {
    panic!("stable64_size should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn stable64_grow(new_pages: i64) -> i64 {
    panic!("stable64_grow should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn stable64_write(offset: u64, src: u64, size: u64) {
    panic!("stable64_write should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn stable64_read(dst: u64, offset: u64, size: u64) {
    panic!("stable64_read should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn certified_data_set(src: usize, size: usize) {
    panic!("certified_data_set should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn data_certificate_present() -> u32 {
    panic!("data_certificate_present should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn data_certificate_size() -> usize {
    panic!("data_certificate_size should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn data_certificate_copy(dst: usize, offset: usize, size: usize) {
    panic!("data_certificate_copy should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn time() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp_nanos() as i64
}
pub unsafe extern "C-unwind" fn global_timer_set(timestamp: i64) -> i64 {
    panic!("global_timer_set should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn performance_counter(counter_type: u32) -> u64 {
    panic!("performance_counter should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn is_controller(src: usize, size: usize) -> usize {
    panic!("is_controller should only be called inside canisters.");
}
pub unsafe extern "C-unwind" fn debug_print(src: usize, size: usize) {
    let current = with_active_message(|msg| msg.to);
    let s = String::from_utf8_lossy(std::slice::from_raw_parts(src as *const u8, size));
    eprintln!("[{current}] {s}");
}
pub unsafe extern "C-unwind" fn trap(src: usize, size: usize) {
    let s = String::from_utf8_lossy(std::slice::from_raw_parts(src as *const u8, size));
    panic!("{s}");
}

fn callback_or_dummy(cb: usize) -> unsafe extern "C-unwind" fn(env: *mut ()) {
    if cb == usize::MAX {
        dummy
    } else {
        unsafe { std::mem::transmute(cb) }
    }
}

unsafe extern "C-unwind" fn dummy(env: *mut ()) {}
