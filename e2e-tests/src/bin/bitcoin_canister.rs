use ic_cdk::bitcoin_canister::*;
use ic_cdk::call::Error;
use ic_cdk::update;

#[update]
async fn execute_bitcoin_methods(is_mainnet: bool) {
    let network = match is_mainnet {
        true => Network::Mainnet,
        false => Network::Testnet,
    };
    let address = "bcrt1qu58aj62urda83c00eylc6w34yl2s6e5rkzqet7".to_string();

    let arg = GetUtxosRequest {
        address: address.clone(),
        network,
        filter: Some(UtxosFilter::MinConfirmations(1)),
    };
    let _response = bitcoin_get_utxos(&arg).await.unwrap();

    let arg = GetBalanceRequest {
        network,
        address: address.clone(),
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
    assert!(matches!(err, Error::CallFailed { .. }));
}

fn main() {}
