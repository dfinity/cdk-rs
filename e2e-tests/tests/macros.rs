use candid::Principal;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::{call_candid, WasmResult};

mod test_utilities;
use test_utilities::{cargo_build_canister, pocket_ic};

#[test]
fn call_macros() {
    let pic = pocket_ic();
    let wasm = cargo_build_canister("macros");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);
    let _: () = call_candid(&pic, canister_id, RawEffectivePrincipal::None, "arg0", ()).unwrap();
    let _: () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "arg1",
        (1u32,),
    )
    .unwrap();
    let _: () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "arg2",
        (1u32, 2u32),
    )
    .unwrap();
    let sender = Principal::anonymous();
    let res = pic
        .update_call(canister_id, sender, "ret0", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![0]));
    let res = pic
        .update_call(canister_id, sender, "ret1", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![42]));
    let res = pic
        .update_call(canister_id, sender, "ret2", vec![])
        .unwrap();
    assert_eq!(res, WasmResult::Reply(vec![1, 2]));
    let _: (u32,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "manual_reply",
        (),
    )
    .unwrap();
}
