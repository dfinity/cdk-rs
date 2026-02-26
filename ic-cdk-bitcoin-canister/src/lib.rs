#![doc = include_str!("../README.md")]

pub use ic_btc_interface::*;

use candid::Principal;
use ic_cdk::call::{Call, CallResult};

const MAINNET_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 1, 160, 0, 4, 1, 1]); // "ghsi2-tqaaa-aaaan-aaaca-cai"
const TESTNET_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 1, 160, 0, 1, 1, 1]); // "g4xu7-jiaaa-aaaan-aaaaq-cai"
const REGTEST_ID: Principal = Principal::from_slice(&[0, 0, 0, 0, 1, 160, 0, 1, 1, 1]); // "g4xu7-jiaaa-aaaan-aaaaq-cai"

// The cycles costs below are from the [API fees & Pricing](https://internetcomputer.org/docs/references/bitcoin-how-it-works#api-fees-and-pricing) documentation.
// They are unlikely to change, so hardcoded here for simplicity.
const GET_UTXO_MAINNET: u128 = 10_000_000_000;
const GET_UTXO_TESTNET: u128 = 4_000_000_000;

const GET_BALANCE_MAINNET: u128 = 100_000_000;
const GET_BALANCE_TESTNET: u128 = 40_000_000;

const GET_CURRENT_FEE_PERCENTILES_MAINNET: u128 = 100_000_000;
const GET_CURRENT_FEE_PERCENTILES_TESTNET: u128 = 40_000_000;

const GET_BLOCK_HEADERS_MAINNET: u128 = 10_000_000_000;
const GET_BLOCK_HEADERS_TESTNET: u128 = 4_000_000_000;

const SEND_TRANSACTION_SUBMISSION_MAINNET: u128 = 5_000_000_000;
const SEND_TRANSACTION_SUBMISSION_TESTNET: u128 = 2_000_000_000;

const SEND_TRANSACTION_PAYLOAD_MAINNET: u128 = 20_000_000;
const SEND_TRANSACTION_PAYLOAD_TESTNET: u128 = 8_000_000;

/// Gets the canister ID of the Bitcoin canister for the specified network.
pub fn get_bitcoin_canister_id(network: Network) -> Principal {
    match network {
        Network::Mainnet => MAINNET_ID,
        Network::Testnet => TESTNET_ID,
        Network::Regtest => REGTEST_ID,
    }
}

/// Gets all unspent transaction outputs (UTXOs) associated with the provided address.
///
/// **Bounded-wait call**
///
/// Check the [Bitcoin Canisters Interface Specification](https://github.com/dfinity/bitcoin-canister/blob/master/INTERFACE_SPECIFICATION.md#bitcoin_get_utxos) for more details.
pub async fn bitcoin_get_utxos(arg: &GetUtxosRequest) -> CallResult<GetUtxosResponse> {
    let canister_id = get_bitcoin_canister_id(arg.network.into());
    let cycles = cost_get_utxos(arg);
    Ok(Call::bounded_wait(canister_id, "bitcoin_get_utxos")
        .with_arg(arg)
        .with_cycles(cycles)
        .await?
        .candid()?)
}

/// Gets the cycles cost for the [`bitcoin_get_utxos`] function.
///
/// # Note
///
/// [`bitcoin_get_utxos`] calls this function internally so it's not necessary to call this function directly.
/// When it is preferred to construct a [`Call`] manually, this function can be used to get the cycles cost.
pub fn cost_get_utxos(arg: &GetUtxosRequest) -> u128 {
    match Network::from(arg.network) {
        Network::Mainnet => GET_UTXO_MAINNET,
        Network::Testnet => GET_UTXO_TESTNET,
        Network::Regtest => GET_UTXO_MAINNET,
    }
}

/// Gets the current balance of a Bitcoin address in Satoshi.
///
/// **Bounded-wait call**
///
/// Check the [Bitcoin Canisters Interface Specification](https://github.com/dfinity/bitcoin-canister/blob/master/INTERFACE_SPECIFICATION.md#bitcoin_get_balance) for more details.
pub async fn bitcoin_get_balance(arg: &GetBalanceRequest) -> CallResult<Satoshi> {
    let canister_id = get_bitcoin_canister_id(arg.network.into());
    let cycles = cost_get_balance(arg);
    Ok(Call::bounded_wait(canister_id, "bitcoin_get_balance")
        .with_arg(arg)
        .with_cycles(cycles)
        .await?
        .candid()?)
}

/// Gets the cycles cost for the [`bitcoin_get_balance`] function.
///
/// # Note
///
/// [`bitcoin_get_balance`] calls this function internally so it's not necessary to call this function directly.
/// When it is preferred to construct a [`Call`] manually, this function can be used to get the cycles cost.
pub fn cost_get_balance(arg: &GetBalanceRequest) -> u128 {
    match Network::from(arg.network) {
        Network::Mainnet => GET_BALANCE_MAINNET,
        Network::Testnet => GET_BALANCE_TESTNET,
        Network::Regtest => GET_BALANCE_MAINNET,
    }
}

/// Gets the Bitcoin transaction fee percentiles.
///
/// **Bounded-wait call**
///
/// The percentiles are measured in millisatoshi/byte (1000 millisatoshi = 1 satoshi),
/// over the last 10,000 transactions in the specified network,
/// i.e., over the transactions in the last approximately 4-10 blocks.
pub async fn bitcoin_get_current_fee_percentiles(
    arg: &GetCurrentFeePercentilesRequest,
) -> CallResult<Vec<MillisatoshiPerByte>> {
    let canister_id = get_bitcoin_canister_id(arg.network.into());
    let cycles = cost_get_current_fee_percentiles(arg);
    Ok(
        Call::bounded_wait(canister_id, "bitcoin_get_current_fee_percentiles")
            .with_arg(arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

/// Gets the cycles cost for the [`bitcoin_get_current_fee_percentiles`] function.
///
/// # Note
///
/// [`bitcoin_get_current_fee_percentiles`] calls this function internally so it's not necessary to call this function directly.
/// When it is preferred to construct a [`Call`] manually, this function can be used to get the cycles cost.
pub fn cost_get_current_fee_percentiles(arg: &GetCurrentFeePercentilesRequest) -> u128 {
    match Network::from(arg.network) {
        Network::Mainnet => GET_CURRENT_FEE_PERCENTILES_MAINNET,
        Network::Testnet => GET_CURRENT_FEE_PERCENTILES_TESTNET,
        Network::Regtest => GET_CURRENT_FEE_PERCENTILES_MAINNET,
    }
}

/// Gets the block headers in the provided range of block heights.
///
/// **Bounded-wait call**
///
/// Check the [Bitcoin Canisters Interface Specification](https://github.com/dfinity/bitcoin-canister/blob/master/INTERFACE_SPECIFICATION.md#bitcoin_get_block_headers) for more details.
pub async fn bitcoin_get_block_headers(
    arg: &GetBlockHeadersRequest,
) -> CallResult<GetBlockHeadersResponse> {
    let canister_id = get_bitcoin_canister_id(arg.network.into());
    let cycles = cost_get_block_headers(arg);
    Ok(Call::bounded_wait(canister_id, "bitcoin_get_block_headers")
        .with_arg(arg)
        .with_cycles(cycles)
        .await?
        .candid()?)
}

/// Gets the cycles cost for the [`bitcoin_get_block_headers`] function.
///
/// # Note
///
/// [`bitcoin_get_block_headers`] calls this function internally so it's not necessary to call this function directly.
/// When it is preferred to construct a [`Call`] manually, this function can be used to get the cycles cost.
pub fn cost_get_block_headers(arg: &GetBlockHeadersRequest) -> u128 {
    match Network::from(arg.network) {
        Network::Mainnet => GET_BLOCK_HEADERS_MAINNET,
        Network::Testnet => GET_BLOCK_HEADERS_TESTNET,
        Network::Regtest => GET_BLOCK_HEADERS_MAINNET,
    }
}

/// Sends a Bitcoin transaction to the Bitcoin network.
///
/// **Unbounded-wait call**
///
/// Check the [Bitcoin Canisters Interface Specification](https://github.com/dfinity/bitcoin-canister/blob/master/INTERFACE_SPECIFICATION.md#bitcoin_send_transaction) for more details.
pub async fn bitcoin_send_transaction(arg: &SendTransactionRequest) -> CallResult<()> {
    let canister_id = get_bitcoin_canister_id(arg.network.into());
    let cycles = cost_send_transaction(arg);
    Ok(
        Call::unbounded_wait(canister_id, "bitcoin_send_transaction")
            .with_arg(arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

/// Gets the cycles cost for the [`bitcoin_send_transaction`] function.
///
/// # Note
///
/// [`bitcoin_send_transaction`] calls this function internally so it's not necessary to call this function directly.
/// When it is preferred to construct a [`Call`] manually, this function can be used to get the cycles cost.
pub fn cost_send_transaction(arg: &SendTransactionRequest) -> u128 {
    let (submission, payload) = match Network::from(arg.network) {
        Network::Mainnet => (
            SEND_TRANSACTION_SUBMISSION_MAINNET,
            SEND_TRANSACTION_PAYLOAD_MAINNET,
        ),
        Network::Testnet => (
            SEND_TRANSACTION_SUBMISSION_TESTNET,
            SEND_TRANSACTION_PAYLOAD_TESTNET,
        ),
        Network::Regtest => (
            SEND_TRANSACTION_SUBMISSION_MAINNET,
            SEND_TRANSACTION_PAYLOAD_MAINNET,
        ),
    };
    submission + payload * arg.transaction.len() as u128
}
