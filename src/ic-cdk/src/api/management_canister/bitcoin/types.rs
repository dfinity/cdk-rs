use candid::CandidType;
use serde::{Deserialize, Serialize};

pub type Satoshi = u64;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Eq, Hash, CandidType, Copy)]
pub enum BitcoinNetwork {
    // TODO: The variants are Capitalized while they are lowercase in the spec
    Mainnet,
    Testnet,
    // TODO: the spec doesn't have this type of network
    Regtest,
}

pub type BitcoinAddress = String;

pub type BlockHash = Vec<u8>;

pub type MillisatoshiPerByte = u64;

#[derive(Clone, CandidType, Deserialize, Debug, Default)]
pub struct Outpoint {
    pub txid: Vec<u8>,
    pub vout: u32,
}

#[derive(Clone, CandidType, Deserialize, Debug, Default)]
pub struct Utxo {
    pub outpoint: Outpoint,
    pub value: Satoshi,
    pub height: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Eq, Hash, CandidType)]
pub enum UtxoFilter {
    // TODO: In the spec, variants are in snake case
    // #[serde(rename = "min_confirmations")]
    MinConfirmations(u32),
    // #[serde(rename = "page")]
    Page(Vec<u8>),
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct GetBalanceRequest {
    pub address: BitcoinAddress,
    pub network: BitcoinNetwork,
    pub min_confirmations: Option<u32>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct GetUtxosRequest {
    pub address: BitcoinAddress,
    pub network: BitcoinNetwork,
    pub filter: Option<UtxoFilter>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct GetUtxosResponse {
    pub utxos: Vec<Utxo>,
    pub tip_block_hash: BlockHash,
    pub tip_height: u32,
    pub next_page: Option<Vec<u8>>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct SendTransactionRequest {
    pub transaction: Vec<u8>,
    pub network: BitcoinNetwork,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct GetCurrentFeePercentilesRequest {
    pub network: BitcoinNetwork,
}
