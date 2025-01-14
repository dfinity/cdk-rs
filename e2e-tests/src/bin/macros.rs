use candid::utils::{decode_args, decode_one};
use ic_cdk::api::msg_arg_data;
use ic_cdk::{export_candid, update};
use std::marker::PhantomData;

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

#[update(manual_reply = true)]
fn u_manual_reply() -> PhantomData<u32> {
    let v: u32 = 1;
    let reply_bytes = candid::encode_one(&v).unwrap();
    ic_cdk::api::msg_reply(&reply_bytes);
    PhantomData
}

export_candid! {}

fn main() {
    println!("{}", __export_service());
}

#[cfg(test)]
mod tests {

    #[test]
    fn check_candid() {
        let expected = "service : {
  u0 : () -> ();
  u1 : (nat32) -> ();
  u2 : (nat32, nat32) -> ();
  u_manual_reply : () -> (nat32);
}";
        assert_eq!(expected, super::__export_service());
    }
}
