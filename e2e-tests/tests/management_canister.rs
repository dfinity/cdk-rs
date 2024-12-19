use ic_cdk_e2e_tests::cargo_build_canister;
use pocket_ic::call_candid;
use pocket_ic::common::rest::RawEffectivePrincipal;
use pocket_ic::PocketIcBuilder;

#[test]
fn test_management_canister() {
    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .with_nonmainnet_features(true)
        .with_ii_subnet() // Required for ecdsa and schnorr
        .build();

    let wasm = cargo_build_canister("management_canister");
    let canister_id = pic.create_canister();
    let subnet_id = pic.get_subnet(canister_id).unwrap();
    pic.add_cycles(canister_id, 10_000_000_000_000u128); // 10 T
    pic.install_canister(canister_id, wasm, vec![], None);
    let () = call_candid(&pic, canister_id, RawEffectivePrincipal::None, "basic", ()).unwrap();
    let () = call_candid(&pic, canister_id, RawEffectivePrincipal::None, "ecdsa", ()).unwrap();
    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "schnorr",
        (),
    )
    .unwrap();
    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "metrics",
        (subnet_id,),
    )
    .unwrap();
    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "subnet",
        (subnet_id,),
    )
    .unwrap();
    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "provisional",
        (),
    )
    .unwrap();
    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "snapshots",
        (),
    )
    .unwrap();
}
