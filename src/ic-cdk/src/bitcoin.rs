//! The IC Bitcoin API.
//!
//! Check [Bitcoin integration](https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/bitcoin-how-it-works/#api) for more details.

use crate::api::call::{call_with_payment128, CallResult};
use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};

const GET_UTXO_MAINNET: u128 = 10_000_000_000;
const GET_UTXO_TESTNET: u128 = 4_000_000_000;

const GET_CURRENT_FEE_PERCENTILES_MAINNET: u128 = 100_000_000;
const GET_CURRENT_FEE_PERCENTILES_TESTNET: u128 = 40_000_000;

const GET_BALANCE_MAINNET: u128 = 100_000_000;
const GET_BALANCE_TESTNET: u128 = 40_000_000;

const SEND_TRANSACTION_SUBMISSION_MAINNET: u128 = 5_000_000_000;
const SEND_TRANSACTION_SUBMISSION_TESTNET: u128 = 2_000_000_000;

const SEND_TRANSACTION_PAYLOAD_MAINNET: u128 = 20_000_000;
const SEND_TRANSACTION_PAYLOAD_TESTNET: u128 = 8_000_000;

/// TODO: doc
#[derive(Debug)]
pub struct BitcoinCanister {
    canister_id: Principal,
    get_balance_fee: u128,
    get_utxos_fee: u128,
    get_current_fee_percentiles_fee: u128,
    send_transaction_submission_fee: u128,
    send_transaction_payload_fee: u128,
}

impl BitcoinCanister {
    /// Get access to the canister of Bitcoin mainnet.
    pub fn mainnet() -> Self {
        Self {
            canister_id: Principal::management_canister(), // TODO: replace with the actual canister id
            get_balance_fee: GET_BALANCE_MAINNET,
            get_utxos_fee: GET_UTXO_MAINNET,
            get_current_fee_percentiles_fee: GET_CURRENT_FEE_PERCENTILES_MAINNET,
            send_transaction_submission_fee: SEND_TRANSACTION_SUBMISSION_MAINNET,
            send_transaction_payload_fee: SEND_TRANSACTION_PAYLOAD_MAINNET,
        }
    }

    /// Get access to the canister of Bitcoin testnet.
    pub fn testnet() -> Self {
        Self {
            canister_id: Principal::management_canister(), // TODO: replace with the actual canister id
            get_balance_fee: GET_BALANCE_TESTNET,
            get_utxos_fee: GET_UTXO_TESTNET,
            get_current_fee_percentiles_fee: GET_CURRENT_FEE_PERCENTILES_TESTNET,
            send_transaction_submission_fee: SEND_TRANSACTION_SUBMISSION_TESTNET,
            send_transaction_payload_fee: SEND_TRANSACTION_PAYLOAD_TESTNET,
        }
    }

    /// Get access to the canister of Bitcoin regtest.
    pub fn regtest() -> Self {
        Self {
            canister_id: Principal::management_canister(), // TODO: replace with the actual canister id
            get_balance_fee: 0,
            get_utxos_fee: 0,
            get_current_fee_percentiles_fee: 0,
            send_transaction_submission_fee: 0,
            send_transaction_payload_fee: 0,
        }
    }

    /// Get the balance of a Bitcoin address.
    pub async fn get_balance(&self, args: GetBalanceArgs) -> CallResult<GetBalanceResult> {
        call_with_payment128(
            self.canister_id,
            "bitcoin_get_balance",
            (args,),
            self.get_balance_fee,
        )
        .await
        .map(|r: (GetBalanceResult,)| r.0)
    }
}

// ============================================================================

// ----------------------------------------------------------------------------
// Types definition below
// ----------------------------------------------------------------------------

/// `type satoshi = nat64;`
pub type Satoshi = u64;

/// Bitcoin Network.
///
/// ```text
/// type network = variant {
///     mainnet;
///     testnet;
///     regtest;
/// };
/// ```
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
    /// Mainnet.
    #[serde(rename = "mainnet")]
    #[default]
    Mainnet,
    /// Testnet.
    #[serde(rename = "testnet")]
    Testnet,
    /// Regtest.
    #[serde(rename = "regtest")]
    Regtest,
}

/// `type address = text;`
pub type Address = String;

/// Argument type of [bitcoin_get_balance](super::bitcoin_get_balance).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct GetBalanceArgs {
    /// See [Address].
    address: Address,
    /// See [Network].
    network: Network,
    /// Minimum number of confirmations. There is an upper bound of 144. Typically set to a value around 6 in practice.
    min_confirmations: Option<u32>,
}

impl GetBalanceArgs {
    /// Create a new instance.
    pub fn new(address: Address, network: Network) -> Self {
        Self {
            address,
            network,
            min_confirmations: None,
        }
    }

    /// Set the minimum number of confirmations.
    pub fn with_min_confirmations(mut self, min_confirmations: u32) -> Self {
        self.min_confirmations = Some(min_confirmations);
        self
    }

    /// Get the address.
    pub fn get_address(&self) -> &Address {
        &self.address
    }

    /// Get the network.
    pub fn get_network(&self) -> Network {
        self.network
    }

    /// Get the minimum number of confirmations.
    pub fn get_min_confirmations(&self) -> Option<u32> {
        self.min_confirmations
    }
}

/// Result type of [BitcoinCanister::get_balance].
pub type GetBalanceResult = Satoshi;
