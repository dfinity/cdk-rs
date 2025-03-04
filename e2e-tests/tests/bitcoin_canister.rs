use pocket_ic::PocketIcBuilder;

mod test_utilities;
use test_utilities::cargo_build_canister;

#[test]
fn test_bitcoin_canister() {
    let pic = PocketIcBuilder::new()
        .with_application_subnet()
        .with_nonmainnet_features(true)
        .with_bitcoin_subnet()
        .with_bitcoind_addrs(vec![])
        .build();
    let wasm = cargo_build_canister("bitcoin_canister");
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 10_000_000_000_000u128); // 10 T
    pic.install_canister(canister_id, wasm, vec![], None);
}
