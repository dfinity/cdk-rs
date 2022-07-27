use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Eq, Hash, CandidType, Copy)]
pub enum EcdsaCurve {
    #[serde(rename = "secp256k1")]
    Secp256k1,
}

pub type Satoshi = u64;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize, Eq, Hash, CandidType, Copy)]
pub enum BitcoinNetwork {
    #[serde(rename = "mainnet")]
    Mainnet,
    #[serde(rename = "testnet")]
    Testnet,
}

pub type BitcoinAddress = String;

pub type BlockHash = Vec<u8>;

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
    #[serde(rename = "min_confirmations")]
    MinConfirmations(u32),
    #[serde(rename = "page")]
    Page(Vec<u8>),
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct GetUtxosRequest {
    pub address: BitcoinAddress,
    pub network: BitcoinNetwork,
    pub filter: Option<UtxoFilter>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct GetCurrentFeePercentilesRequest {
    pub network: BitcoinNetwork,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct GetUtxosResponse {
    pub utxos: Vec<Utxo>,
    pub tip_block_hash: BlockHash,
    pub tip_height: u32,
    pub next_page: Option<Vec<u8>>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct GetBalanceRequest {
    pub address: BitcoinAddress,
    pub network: BitcoinNetwork,
    pub min_confirmations: Option<u32>,
}

#[derive(Clone, CandidType, Deserialize, Debug)]
pub struct SendTransactionRequest {
    pub transaction: Vec<u8>,
    pub network: BitcoinNetwork,
}

pub type MillisatoshiPerByte = u64;
