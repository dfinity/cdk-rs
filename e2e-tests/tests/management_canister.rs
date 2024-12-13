use ic_cdk_e2e_tests::cargo_build_canister;
use pocket_ic::call_candid;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::PocketIcBuilder;

#[test]
fn test_call_management() {
    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .with_nonmainnet_features(true)
        .build();

    let wasm = cargo_build_canister("management_canister");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 300_000_000_000_000_000_000_000_000u128);
    pic.install_canister(canister_id, wasm, vec![], None);
    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "test_basic",
        (),
    )
    .unwrap();
}
