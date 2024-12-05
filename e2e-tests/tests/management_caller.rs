use pocket_ic::call_candid;
use pocket_ic::common::rest::RawEffectivePrincipal;

mod test_utilities;
use test_utilities::{cargo_build_canister, pocket_ic};

#[test]
fn test_call_management() {
    let pic = pocket_ic();
    let wasm = cargo_build_canister("management_caller");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 300_000_000_000_000_000_000_000_000u128);
    pic.install_canister(canister_id, wasm, vec![], None);
    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "execute_main_methods",
        (),
    )
    .expect("Error calling execute_main_methods");
    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "execute_provisional_methods",
        (),
    )
    .expect("Error calling execute_provisional_methods");
}

#[test]
fn test_snapshot() {
    let pic = pocket_ic();
    let wasm = cargo_build_canister("management_caller");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 300_000_000_000_000_000_000_000_000u128);
    pic.install_canister(canister_id, wasm, vec![], None);
    let () = call_candid(
        &pic,
        canister_id,
        RawEffectivePrincipal::None,
        "execute_snapshot_methods",
        (),
    )
    .expect("Error calling execute_snapshot_methods");
}
