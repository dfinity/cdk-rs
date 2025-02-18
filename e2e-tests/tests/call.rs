use pocket_ic::call_candid;
use pocket_ic::common::rest::RawEffectivePrincipal;

mod test_utilities;
use test_utilities::{cargo_build_canister, pocket_ic};

#[test]
fn call() {
    let pic = pocket_ic();
    let wasm = cargo_build_canister("call");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);
    let _: () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "call_foo",
        (),
    )
    .unwrap();
    let _: () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "call_echo",
        (),
    )
    .unwrap();
    let _: () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "retry_calls",
        (),
    )
    .unwrap();
    let _: () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "join_calls",
        (),
    )
    .unwrap();
    let _: () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "call_error_ext",
        (),
    )
    .unwrap();
}
