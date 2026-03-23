use ic_cdk::call::Error;
use ic_cdk::update;
use ic_cdk_bitcoin_canister::*;

// The Bitcoin canister validates that addresses match the requested network, so each network
// requires a distinct address. The mainnet and testnet addresses are the official BIP173 test
// vectors (P2WPKH of the genesis block public key):
// https://github.com/bitcoin/bips/blob/master/bip-0173.mediawiki#test-vectors
// No official test vector exists for regtest, so a known-valid bcrt1 address is used instead.
const BTC_ADDRESS_MAINNET: &str = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4";
const BTC_ADDRESS_TESTNET: &str = "tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx";
const BTC_ADDRESS_REGTEST: &str = "bcrt1qu58aj62urda83c00eylc6w34yl2s6e5rkzqet7";

#[update]
async fn execute_all_methods(network: NetworkInRequest) {
    let network_enum = Network::from(network);

    let btc_address = match network_enum {
        Network::Mainnet => BTC_ADDRESS_MAINNET,
        Network::Testnet => BTC_ADDRESS_TESTNET,
        Network::Regtest => BTC_ADDRESS_REGTEST,
    };

    let _info = get_blockchain_info(network_enum).await.unwrap();

    let arg = GetUtxosRequest {
        address: btc_address.to_string(),
        network,
        filter: Some(UtxosFilterInRequest::MinConfirmations(1)),
    };
    let _response = bitcoin_get_utxos(&arg).await.unwrap();

    let arg = GetBalanceRequest {
        network,
        address: btc_address.to_string(),
        min_confirmations: Some(1),
    };
    let _balance = bitcoin_get_balance(&arg).await.unwrap();

    let arg = GetCurrentFeePercentilesRequest { network };
    let _percentiles = bitcoin_get_current_fee_percentiles(&arg).await.unwrap();

    let arg = GetBlockHeadersRequest {
        network,
        start_height: 0,
        end_height: None,
    };
    let _response = bitcoin_get_block_headers(&arg).await.unwrap();

    let arg = SendTransactionRequest {
        transaction: vec![],
        network,
    };
    let err = bitcoin_send_transaction(&arg).await.unwrap_err();
    assert!(matches!(err, Error::CallRejected { .. }));
}

fn main() {}
