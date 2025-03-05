//! This module provides functionality for making inter-canister calls to the [Bitcoin canisters][1].
//!
//! The Bitcoin canisters allow for interactions with the Bitcoin network from within the Internet Computer.
//! This module includes functions and types that facilitate these interactions, adhering to the
//! [Bitcoin Canisters Interface Specification][2].
//!
//! [1]: https://github.com/dfinity/bitcoin-canister
//! [2]: https://github.com/dfinity/bitcoin-canister/blob/master/INTERFACE_SPECIFICATION.md

use crate::call::{Call, CallResult};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

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

fn get_canister_id(network: &Network) -> Principal {
    match network {
        Network::Mainnet => MAINNET_ID,
        Network::Testnet => TESTNET_ID,
        Network::Regtest => REGTEST_ID,
    }
}

/// Bitcoin Network.
#[derive(
    CandidType,
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    Copy,
    Default,
)]
pub enum Network {
    /// The Bitcoin mainnet.
    #[default]
    #[serde(rename = "mainnet")]
    Mainnet,
    /// The Bitcoin testnet.
    #[serde(rename = "testnet")]
    Testnet,
    /// The Bitcoin regtest, used for local testing purposes.
    #[serde(rename = "regtest")]
    Regtest,
}

/// Satoshi.
///
/// The smallest unit of Bitcoin, equal to 0.00000001 BTC.
pub type Satoshi = u64;

/// Bitcoin Address.
///
/// Please check the [Bitcoin Canisters Interface Specification](https://github.com/dfinity/bitcoin-canister/blob/master/INTERFACE_SPECIFICATION.md#bitcoin_get_utxos) for supported address formats.
pub type Address = String;

/// Block Hash.
pub type BlockHash = Vec<u8>;

/// Block Height.
pub type BlockHeight = u32;

/// Outpoint.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct Outpoint {
    /// Transaction ID (TxID).
    ///
    /// The hash of the transaction that created the UTXO.
    pub txid: Vec<u8>,
    /// Output Index (vout).
    ///
    /// The index of the specific output within that transaction (since a transaction can have multiple outputs).
    pub vout: u32,
}

/// Unspent Transaction Output (UTXO).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct Utxo {
    /// The outpoint of the UTXO.
    pub outpoint: Outpoint,
    /// The value of the UTXO in satoshis.
    pub value: Satoshi,
    /// The block height at which the UTXO was created.
    pub height: BlockHeight,
}

/// Filter to restrict the set of returned UTXOs.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub enum UtxosFilter {
    /// Filter by minimum number of confirmations.
    #[serde(rename = "min_confirmations")]
    MinConfirmations(u32),
    /// Filter by a page reference.
    #[serde(rename = "page")]
    Page(Vec<u8>),
}

/// Argument type of [`bitcoin_get_utxos`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct GetUtxosRequest {
    /// The Bitcoin network.
    pub network: Network,
    /// The Bitcoin address.
    pub address: Address,
    /// An optional filter to restrict the set of returned UTXOs.
    pub filter: Option<UtxosFilter>,
}

/// Result type of [`bitcoin_get_utxos`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct GetUtxosResponse {
    /// List of UTXOs.
    pub utxos: Vec<Utxo>,
    /// Hash of the tip block.
    pub tip_block_hash: BlockHash,
    /// Height of the tip height.
    pub tip_height: u32,
    /// Page reference when the response needs to be paginated.
    ///
    /// To be used in [`UtxosFilter::Page`].
    pub next_page: Option<Vec<u8>>,
}

/// Gets all unspent transaction outputs (UTXOs) associated with the provided address.
///
/// Check the [Bitcoin Canisters Interface Specification](https://github.com/dfinity/bitcoin-canister/blob/master/INTERFACE_SPECIFICATION.md#bitcoin_get_utxos) for more details.
pub async fn bitcoin_get_utxos(arg: &GetUtxosRequest) -> CallResult<GetUtxosResponse> {
    let canister_id = get_canister_id(&arg.network);
    let cycles = match arg.network {
        Network::Mainnet => GET_UTXO_MAINNET,
        Network::Testnet => GET_UTXO_TESTNET,
        Network::Regtest => 0,
    };
    Ok(Call::unbounded_wait(canister_id, "bitcoin_get_utxos")
        .with_arg(arg)
        .with_cycles(cycles)
        .await?
        .candid()?)
}

/// Argument type of [`bitcoin_get_balance`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct GetBalanceRequest {
    /// The Bitcoin network.
    pub network: Network,
    /// The Bitcoin address.
    pub address: Address,
    /// Minimum number of confirmations.
    ///
    /// There is an upper bound of 144. Typically set to a value around 6 in practice.
    pub min_confirmations: Option<u32>,
}

/// Gets the current balance of a Bitcoin address in Satoshi.
///
/// Check the [Bitcoin Canisters Interface Specification](https://github.com/dfinity/bitcoin-canister/blob/master/INTERFACE_SPECIFICATION.md#bitcoin_get_balance) for more details.
pub async fn bitcoin_get_balance(arg: &GetBalanceRequest) -> CallResult<Satoshi> {
    let canister_id = get_canister_id(&arg.network);
    let cycles = match arg.network {
        Network::Mainnet => GET_BALANCE_MAINNET,
        Network::Testnet => GET_BALANCE_TESTNET,
        Network::Regtest => 0,
    };
    Ok(Call::unbounded_wait(canister_id, "bitcoin_get_balance")
        .with_arg(arg)
        .with_cycles(cycles)
        .await?
        .candid()?)
}

/// Argument type of the [`bitcoin_get_current_fee_percentiles`] function.
#[derive(
    CandidType,
    Serialize,
    Deserialize,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    Copy,
    Default,
)]
pub struct GetCurrentFeePercentilesRequest {
    /// The Bitcoin network.
    pub network: Network,
}

/// Unit of Bitcoin transaction fee.
///
/// This is the element in the [`bitcoin_get_current_fee_percentiles`] response.
pub type MillisatoshiPerByte = u64;

/// Gets the Bitcoin transaction fee percentiles.
///
/// The percentiles are measured in millisatoshi/byte (1000 millisatoshi = 1 satoshi),
/// over the last 10,000 transactions in the specified network,
/// i.e., over the transactions in the last approximately 4-10 blocks.
pub async fn bitcoin_get_current_fee_percentiles(
    arg: &GetCurrentFeePercentilesRequest,
) -> CallResult<Vec<MillisatoshiPerByte>> {
    let canister_id = get_canister_id(&arg.network);
    let cycles = match arg.network {
        Network::Mainnet => GET_CURRENT_FEE_PERCENTILES_MAINNET,
        Network::Testnet => GET_CURRENT_FEE_PERCENTILES_TESTNET,
        Network::Regtest => 0,
    };
    Ok(
        Call::unbounded_wait(canister_id, "bitcoin_get_current_fee_percentiles")
            .with_arg(arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

/// Argument type of the [`bitcoin_get_block_headers`] function.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct GetBlockHeadersRequest {
    /// The starting block height for the request.
    pub start_height: BlockHeight,
    /// The ending block height for the request, or `None` for the current tip.
    pub end_height: Option<BlockHeight>,
    /// The Bitcoin network.
    pub network: Network,
}

/// Block Header.
pub type BlockHeader = Vec<u8>;

/// Response type of the [`bitcoin_get_block_headers`] function.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct GetBlockHeadersResponse {
    /// The tip of the blockchain when this request was filled.
    pub tip_height: BlockHeight,
    /// The requested block headers.
    pub block_headers: Vec<BlockHeader>,
}

/// Gets the block headers in the provided range of block heights.
///
/// Check the [Bitcoin Canisters Interface Specification](https://github.com/dfinity/bitcoin-canister/blob/master/INTERFACE_SPECIFICATION.md#bitcoin_get_block_headers) for more details.
pub async fn bitcoin_get_block_headers(
    arg: &GetBlockHeadersRequest,
) -> CallResult<GetBlockHeadersResponse> {
    let canister_id = get_canister_id(&arg.network);
    let cycles = match arg.network {
        Network::Mainnet => GET_BLOCK_HEADERS_MAINNET,
        Network::Testnet => GET_BLOCK_HEADERS_TESTNET,
        Network::Regtest => 0,
    };
    Ok(
        Call::unbounded_wait(canister_id, "bitcoin_get_block_headers")
            .with_arg(arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

/// Argument type of the [`bitcoin_send_transaction`] function.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SendTransactionRequest {
    /// The Bitcoin network.
    pub network: Network,
    /// The Bitcoin transaction.
    pub transaction: Vec<u8>,
}

/// Sends a Bitcoin transaction to the Bitcoin network.
///
/// Check the [Bitcoin Canisters Interface Specification](https://github.com/dfinity/bitcoin-canister/blob/master/INTERFACE_SPECIFICATION.md#bitcoin_send_transaction) for more details.
pub async fn bitcoin_send_transaction(arg: &SendTransactionRequest) -> CallResult<()> {
    let canister_id = get_canister_id(&arg.network);
    let cycles = send_transaction_fee(arg);
    Ok(
        Call::unbounded_wait(canister_id, "bitcoin_send_transaction")
            .with_arg(arg)
            .with_cycles(cycles)
            .await?
            .candid()?,
    )
}

fn send_transaction_fee(arg: &SendTransactionRequest) -> u128 {
    let (submission, payload) = match arg.network {
        Network::Mainnet => (
            SEND_TRANSACTION_SUBMISSION_MAINNET,
            SEND_TRANSACTION_PAYLOAD_MAINNET,
        ),
        Network::Testnet => (
            SEND_TRANSACTION_SUBMISSION_TESTNET,
            SEND_TRANSACTION_PAYLOAD_TESTNET,
        ),
        Network::Regtest => (0, 0),
    };
    submission + payload * arg.transaction.len() as u128
}
