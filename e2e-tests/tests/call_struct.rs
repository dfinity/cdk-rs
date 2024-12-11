use ic_cdk_e2e_tests::cargo_build_canister;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::{call_candid, PocketIcBuilder};

#[test]
fn call_struct() {
    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .with_nonmainnet_features(true)
        .build();
    let wasm = cargo_build_canister("call_struct");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);
    let _: () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "call_struct",
        (),
    )
    .expect("Error calling call_struct");
}
