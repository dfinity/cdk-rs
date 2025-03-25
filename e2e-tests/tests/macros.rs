use candid::Principal;
use prost::Message;

mod test_utilities;
use test_utilities::{cargo_build_canister, pic_base, update};

#[test]
fn call_macros() {
    let pic = pic_base().build();
    let wasm = cargo_build_canister("macros");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);
    let _: () = update(&pic, canister_id, "arg0", ()).unwrap();
    let _: () = update(&pic, canister_id, "arg1", (1u32,)).unwrap();
    let _: () = update(&pic, canister_id, "arg2", (1u32, 2u32)).unwrap();
    let sender = Principal::anonymous();
    let res = pic
        .update_call(canister_id, sender, "ret0", vec![])
        .unwrap();
    assert_eq!(res, vec![0]);
    let res = pic
        .update_call(canister_id, sender, "ret1", vec![])
        .unwrap();
    assert_eq!(res, vec![42]);
    let res = pic
        .update_call(canister_id, sender, "ret2", vec![])
        .unwrap();
    assert_eq!(res, vec![1, 2]);
    let res = pic
        .update_call(
            canister_id,
            sender,
            "protobuf_onwire1",
            1u32.encode_to_vec(),
        )
        .unwrap();
    assert_eq!(res, 43u32.encode_to_vec());
    let res = pic
        .update_call(
            canister_id,
            sender,
            "protobuf_onwire2",
            "Hello".to_string().encode_to_vec(),
        )
        .unwrap();
    assert_eq!(res, "Hello world!".to_string().encode_to_vec());
    let _: (u32,) = update(&pic, canister_id, "manual_reply", ()).unwrap();
}
