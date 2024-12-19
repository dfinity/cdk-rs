use pocket_ic::call_candid;
use pocket_ic::common::rest::RawEffectivePrincipal;

mod test_utilities;
use test_utilities::{cargo_build_canister, pocket_ic};

#[test]
fn test_management_canister() {
    let pic = pocket_ic();

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
