use ic_cdk::{export_candid, update};
use prost::Message;
use std::marker::PhantomData;

#[update(decode_with = "decode_arg0")]
fn arg0() {}
fn decode_arg0(_arg_bytes: Vec<u8>) {}

#[update(decode_with = "decode_arg1")]
fn arg1(a: u32) {
    assert_eq!(a, 1)
}
fn decode_arg1(arg_bytes: Vec<u8>) -> u32 {
    candid::utils::decode_one(&arg_bytes).unwrap()
}

#[update(decode_with = "decode_arg2")]
fn arg2(a: u32, b: u32) {
    assert_eq!(a, 1);
    assert_eq!(b, 2);
}
fn decode_arg2(arg_bytes: Vec<u8>) -> (u32, u32) {
    candid::utils::decode_args(&arg_bytes).unwrap()
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

/// The following two endpoints demonstrate how to use generic decode/encode functions.
/// The endpoints take different types of arguments and return values.
/// While the decode/encode functions are generic and can be used for both endpoints.
#[update(decode_with = "from_proto_bytes", encode_with = "to_proto_bytes")]
fn protobuf_onwire1(a: u32) -> u32 {
    a + 42
}
#[update(decode_with = "from_proto_bytes", encode_with = "to_proto_bytes")]
fn protobuf_onwire2(a: String) -> String {
    (a + " world!").to_string()
}
fn to_proto_bytes<T: Message>(msg: T) -> Vec<u8> {
    msg.encode_to_vec()
}
fn from_proto_bytes<T: Message + Default>(msg: Vec<u8>) -> T {
    Message::decode(&msg[..]).unwrap()
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
        // If `decode_with` is specified, the argument type would be `blob` in Candid.
        // If `encode_with` is specified, the return type would be `blob` in Candid.
        let expected = "service : {
            arg0 : (blob) -> ();
            arg1 : (blob) -> ();
            arg2 : (blob) -> ();
            ret0 : () -> (blob);
            ret1 : () -> (blob);
            ret2 : () -> (blob);
            protobuf_onwire1 : (blob) -> (blob);
            protobuf_onwire2 : (blob) -> (blob);
            manual_reply : () -> (nat32);
          }";
        let expected_candid = CandidSource::Text(expected);

        let actual = super::__export_service();
        let actual_candid = CandidSource::Text(&actual);

        let result = service_equal(expected_candid, actual_candid);
        assert!(result.is_ok(), "{:?}", result.unwrap_err());
    }
}
