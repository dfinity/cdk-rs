use candid::CandidType;
use serde::{Deserialize, Serialize};

/// 10^8 Satoshi = 1 Bitcoin.
pub type Satoshi = u64;

/// Bitcoin Network.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy,
)]
pub enum BitcoinNetwork {
    /// Mainnet.
    #[serde(rename = "mainnet")]
    Mainnet,
    /// Testnet.
    #[serde(rename = "testnet")]
    Testnet,
    /// Regtest.
    ///
    /// This is only available when developing with local replica.
    #[serde(rename = "regtest")]
    Regtest,
}

impl Default for BitcoinNetwork {
    fn default() -> Self {
        Self::Regtest
    }
}

/// Bitcoin Address.
pub type BitcoinAddress = String;

/// Block Hash.
pub type BlockHash = Vec<u8>;

/// Element in the Response of [`bitcoin_get_current_fee_percentiles`](super::bitcoin_get_current_fee_percentiles).
pub type MillisatoshiPerByte = u64;

/// Identifier of [`Utxo`].
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct Outpoint {
    /// Transaction Identifier.
    pub txid: Vec<u8>,
    /// A implicit index number.
    pub vout: u32,
}

/// Unspent transaction output (UTXO).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct Utxo {
    /// See [`Outpoint`].
    pub outpoint: Outpoint,
    /// Value in the units of satoshi.
    pub value: Satoshi,
    /// Height in the chain.
    pub height: u32,
}

/// Filter for requesting UTXOs.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub enum UtxoFilter {
    /// Minimum number of confirmations. There is an upper bound of 144. Typically set to a value around 6 in practice.
    #[serde(rename = "min_confirmations")]
    MinConfirmations(u32),
    /// Page reference.
    ///
    /// DON'T construct it from scratch.
    /// Only get it from the `next_page` field of [`GetUtxosResponse`].
    #[serde(rename = "page")]
    Page(Vec<u8>),
}

/// Argument type of [`bitcoin_get_balance`](super::bitcoin_get_balance).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct GetBalanceRequest {
    /// See [`BitcoinAddress`].
    pub address: BitcoinAddress,
    /// See [`BitcoinNetwork`].
    pub network: BitcoinNetwork,
    /// Minimum number of confirmations. There is an upper bound of 144. Typically set to a value around 6 in practice.
    pub min_confirmations: Option<u32>,
}

/// Argument type of [`bitcoin_get_utxos`](super::bitcoin_get_utxos).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct GetUtxosRequest {
    /// See [`BitcoinAddress`].
    pub address: BitcoinAddress,
    /// See [`BitcoinNetwork`].
    pub network: BitcoinNetwork,
    /// See [`UtxoFilter`].
    pub filter: Option<UtxoFilter>,
}

/// Response type of [`bitcoin_get_utxos`](super::bitcoin_get_utxos).
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
    /// To be used in [`UtxoFilter::Page`].
    pub next_page: Option<Vec<u8>>,
}

/// Argument type of [`bitcoin_send_transaction`](super::bitcoin_send_transaction).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SendTransactionRequest {
    /// The serialized transaction data.
    ///
    /// Several checks are performed.
    /// See [IC method `bitcoin_send_transaction`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_send_transaction).
    pub transaction: Vec<u8>,
    /// See [`BitcoinNetwork`].
    pub network: BitcoinNetwork,
}

/// Argument type of [`bitcoin_get_current_fee_percentiles`](super::bitcoin_get_current_fee_percentiles).
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
    /// See [`BitcoinNetwork`].
    pub network: BitcoinNetwork,
}

/// Argument type of [`bitcoin_get_block_headers`](super::bitcoin_get_block_headers).
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
pub struct GetBlockHeadersRequest {
    /// The starting block height for the request.
    pub start_height: u32,
    /// The ending block height for the request, or `None` for the current tip.
    pub end_height: Option<u32>,
    /// See [`BitcoinNetwork`].
    pub network: BitcoinNetwork,
}

/// Response type of [`bitcoin_get_block_headers`](super::bitcoin_get_block_headers).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct GetBlockHeadersResponse {
    /// The tip of the blockchain when this request was filled.
    pub tip_height: u32,
    /// The requested block headers.
    pub block_headers: Vec<Vec<u8>>,
}
