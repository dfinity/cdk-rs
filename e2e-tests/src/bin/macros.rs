use candid::utils::{decode_args, decode_one};
use ic_cdk::api::msg_arg_data;
use ic_cdk::{export_candid, update};
use std::marker::PhantomData;

#[update(decode_with = "decode_arg0")]
fn arg0() {}
fn decode_arg0() {}

#[update(decode_with = "decode_arg1")]
fn arg1(a: u32) {
    assert_eq!(a, 1)
}
fn decode_arg1() -> u32 {
    let arg_bytes = msg_arg_data();
    decode_one(&arg_bytes).unwrap()
}

#[update(decode_with = "decode_arg2")]
fn arg2(a: u32, b: u32) {
    assert_eq!(a, 1);
    assert_eq!(b, 2);
}
fn decode_arg2() -> (u32, u32) {
    let arg_bytes = msg_arg_data();
    decode_args(&arg_bytes).unwrap()
}

#[update(encode_with = "encode_ret0")]
fn ret0() {}
fn encode_ret0() -> Vec<u8> {
    vec![0]
}

#[update(encode_with = "encode_ret1")]
fn ret1() -> u32 {
    42
}
fn encode_ret1(v1: u32) -> Vec<u8> {
    vec![v1 as u8]
}

#[update(encode_with = "encode_ret2")]
fn ret2() -> (u32, u32) {
    (1, 2)
}
fn encode_ret2(ret: (u32, u32)) -> Vec<u8> {
    vec![ret.0 as u8, ret.1 as u8]
}

#[update(manual_reply = true)]
fn manual_reply() -> PhantomData<u32> {
    let v: u32 = 1;
    let reply_bytes = candid::encode_one(v).unwrap();
    ic_cdk::api::msg_reply(reply_bytes);
    PhantomData
}

export_candid! {}

fn main() {
    println!("{}", __export_service());
}

#[cfg(test)]
mod tests {
    use candid_parser::utils::{service_equal, CandidSource};

    #[test]
    fn candid_equality_test() {
        let expected = "service : {
            arg0 : () -> ();
            arg1 : (nat32) -> ();
            arg2 : (nat32, nat32) -> ();
            ret0 : () -> ();
            ret1 : () -> (nat32);
            ret2 : () -> (nat32, nat32);
            manual_reply : () -> (nat32);
          }";
        let expected_candid = CandidSource::Text(expected);

        let actual = super::__export_service();
        let actual_candid = CandidSource::Text(&actual);

        let result = service_equal(expected_candid, actual_candid);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
    }
}
