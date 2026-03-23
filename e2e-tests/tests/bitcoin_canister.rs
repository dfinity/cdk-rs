use candid::Principal;
use cargo_metadata::MetadataCommand;
use ic_btc_interface::{CanisterArg, Fees, Flag, InitConfig, Network, NetworkInRequest};
use std::path::PathBuf;

mod test_utilities;
use test_utilities::{cargo_build_canister, pic_base, update};

#[test]
fn test_bitcoin_canister() {
    let blocks_source = Principal::from_text("aaaaa-aa").unwrap();

    // Mainnet
    let mainnet_id = Principal::from_slice(&[0, 0, 0, 0, 1, 160, 0, 4, 1, 1]);
    let mainnet_init = CanisterArg::Init(InitConfig {
        network: Some(Network::Mainnet),
        blocks_source: Some(blocks_source),
        stability_threshold: Some(100),
        syncing: Some(Flag::Enabled),
        api_access: Some(Flag::Enabled),
        disable_api_if_not_fully_synced: Some(Flag::Enabled),
        burn_cycles: Some(Flag::Enabled),
        lazily_evaluate_fee_percentiles: Some(Flag::Enabled),
        fees: Some(Fees::mainnet()),
        watchdog_canister: None,
    });
    test_network(NetworkInRequest::Mainnet, mainnet_id, mainnet_init);

    // Testnet
    let testnet_id = Principal::from_slice(&[0, 0, 0, 0, 1, 160, 0, 1, 1, 1]);
    let testnet_init = CanisterArg::Init(InitConfig {
        network: Some(Network::Testnet),
        blocks_source: Some(blocks_source),
        stability_threshold: Some(144),
        syncing: Some(Flag::Enabled),
        api_access: Some(Flag::Enabled),
        disable_api_if_not_fully_synced: Some(Flag::Enabled),
        burn_cycles: Some(Flag::Enabled),
        lazily_evaluate_fee_percentiles: Some(Flag::Enabled),
        fees: Some(Fees::testnet()),
        watchdog_canister: None,
    });
    test_network(NetworkInRequest::Testnet, testnet_id, testnet_init);

    // Regtest
    let regtest_id = testnet_id;
    let regtest_init = CanisterArg::Init(InitConfig {
        network: Some(Network::Regtest),
        blocks_source: Some(blocks_source),
        stability_threshold: Some(144),
        syncing: Some(Flag::Enabled),
        api_access: Some(Flag::Enabled),
        disable_api_if_not_fully_synced: Some(Flag::Enabled),
        burn_cycles: Some(Flag::Enabled),
        lazily_evaluate_fee_percentiles: Some(Flag::Enabled),
        fees: Some(Fees::default()),
        watchdog_canister: None,
    });
    test_network(NetworkInRequest::Regtest, regtest_id, regtest_init);
}

fn test_network(network: NetworkInRequest, btc_id: Principal, init_arg: CanisterArg) {
    let wasm = cargo_build_canister("bitcoin_canister");
    // The Bitcoin canisters can still function without connecting to a `bitcoind` node.
    // The interface check and the cycles cost logic are still valid.
    let pic = pic_base().with_bitcoin_subnet().build();
    let canister_id = pic.create_canister();
    pic.add_cycles(canister_id, 10_000_000_000_000u128); // 10 T
    pic.install_canister(canister_id, wasm, vec![], None);

    let btc_canister_wasm = std::fs::read(cache_btc_canister_wasm()).unwrap();
    let _ = pic.create_canister_with_id(None, None, btc_id).unwrap();
    pic.add_cycles(btc_id, 10_000_000_000_000u128);
    let encoded_args = candid::encode_one(init_arg).expect("failed to encode init args");
    pic.install_canister(btc_id, btc_canister_wasm.clone(), encoded_args, None);
    let () = update(&pic, canister_id, "execute_all_methods", (network,)).unwrap();
}

fn cache_btc_canister_wasm() -> PathBuf {
    const EXPECTED_TAG: &str = "release%2F2026-03-06"; // The slash is encoded as %2F in the URL
    let dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let cargo_toml_path = dir.join("Cargo.toml");
    let target_dir = MetadataCommand::new()
        .manifest_path(&cargo_toml_path)
        .exec()
        .expect("failed to run cargo metadata")
        .target_directory;
    let artifact_dir = target_dir.join("e2e-tests-artifacts");
    std::fs::create_dir_all(&artifact_dir).expect("failed to create artifact directory");
    let tag_file = artifact_dir.join("ic-btc-canister-tag");
    let binary_file = artifact_dir.join("ic-btc-canister.wasm.gz");
    if let Ok(tag) = std::fs::read_to_string(&tag_file)
        && tag == EXPECTED_TAG
        && binary_file.exists()
    {
        return binary_file.into();
    }
    let url = format!(
        " https://github.com/dfinity/bitcoin-canister/releases/download/{EXPECTED_TAG}/ic-btc-canister.wasm.gz"
    );
    let gz_bytes = reqwest::blocking::get(url)
        .expect("failed to download ic-btc-canister.wasm.gz")
        .bytes()
        .expect("failed to get bytes of ic-btc-canister.wasm.gz")
        .to_vec();
    std::fs::write(&binary_file, gz_bytes).expect("failed to write binary file");
    std::fs::write(tag_file, EXPECTED_TAG).expect("failed to write tag file");
    binary_file.into()
}
