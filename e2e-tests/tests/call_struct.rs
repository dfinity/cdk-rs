use candid::Principal;
use ic_cdk_e2e_tests::cargo_build_canister;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::{call_candid, PocketIc};

#[test]
fn call_struct() {
    let pic = PocketIc::new();
    let wasm = cargo_build_canister("call_struct");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);
    let _: (Principal,) = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "create_canister_via_struct",
        (),
    )
    .expect("Error calling create_canister_via_struct");
}
