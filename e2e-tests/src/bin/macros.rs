use candid::utils::{decode_args, decode_one};
use ic_cdk::api::msg_arg_data;
use ic_cdk::update;

#[update(decode_with = "decode_u0")]
fn u0() {}
fn decode_u0() {}

#[update(decode_with = "decode_u1")]
fn u1(a: u32) {
    assert_eq!(a, 1)
}
fn decode_u1() -> u32 {
    let arg_bytes = msg_arg_data();
    decode_one(&arg_bytes).unwrap()
}

#[update(decode_with = "decode_u2")]
fn u2(a: u32, b: u32) {
    assert_eq!(a, 1);
    assert_eq!(b, 2);
}
fn decode_u2() -> (u32, u32) {
    let arg_bytes = msg_arg_data();
    decode_args(&arg_bytes).unwrap()
}

fn main() {}
