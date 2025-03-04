mod test_utilities;
use test_utilities::{cargo_build_canister, pocket_ic, update};

#[test]
fn test_management_canister() {
    let pic = pocket_ic();

    let wasm = cargo_build_canister("management_canister");
    let canister_id = pic.create_canister();
    let subnet_id = pic.get_subnet(canister_id).unwrap();
    pic.add_cycles(canister_id, 10_000_000_000_000u128); // 10 T
    pic.install_canister(canister_id, wasm, vec![], None);
    let () = update(&pic, canister_id, "basic", ()).unwrap();
    let () = update(&pic, canister_id, "ecdsa", ()).unwrap();
    let () = update(&pic, canister_id, "schnorr", ()).unwrap();
    let () = update(&pic, canister_id, "metrics", (subnet_id,)).unwrap();
    let () = update(&pic, canister_id, "subnet", (subnet_id,)).unwrap();
    let () = update(&pic, canister_id, "provisional", ()).unwrap();
    let () = update(&pic, canister_id, "snapshots", ()).unwrap();
}
