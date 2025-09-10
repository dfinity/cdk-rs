mod test_utilities;
use test_utilities::{cargo_build_canister, pic_base, update};

#[test]
fn bindgen() {
    let wasm = cargo_build_canister("bindgen");
    let pic = pic_base().build();
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 100_000_000_000_000);
    pic.install_canister(canister_id, wasm, vec![], None);
    let _: () = update(&pic, canister_id, "call_management_canister", ()).unwrap();
}
