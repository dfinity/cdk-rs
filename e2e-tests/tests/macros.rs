use pocket_ic::call_candid;
use pocket_ic::common::rest::RawEffectivePrincipal;

mod test_utilities;
use test_utilities::{cargo_build_canister, pocket_ic};

#[test]
fn call_macros() {
    let pic = pocket_ic();
    let wasm = cargo_build_canister("macros");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);
    let _: () = call_candid(&pic, canister_id, RawEffectivePrincipal::None, "u0", ()).unwrap();
    let _: () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "u1",
        (1u32,),
    )
    .unwrap();
    let _: () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "u2",
        (1u32, 2u32),
    )
    .unwrap();
}
