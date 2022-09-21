use candid::{types::reference::Func, CandidType, Principal};
use ic_cdk::api::call::CallResult;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha2::Digest;
use std::convert::TryFrom;
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// The subaccont that is used by default.
pub const DEFAULT_SUBACCOUNT: Subaccount = Subaccount([0; 32]);

/// The default fee for ledger transactions.
pub const DEFAULT_FEE: Tokens = Tokens { e8s: 10_000 };

/// Id of the ledger canister on the IC.
pub const MAINNET_LEDGER_CANISTER_ID: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x01, 0x01]);

/// Id of the governance canister on the IC.
pub const MAINNET_GOVERNANCE_CANISTER_ID: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01, 0x01]);

/// Id of the cycles minting canister on the IC.
pub const MAINNET_CYCLES_MINTING_CANISTER_ID: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x04, 0x01, 0x01]);

/// Number of nanoseconds from the UNIX epoch in UTC timezone.
#[derive(
    CandidType, Serialize, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Timestamp {
    pub timestamp_nanos: u64,
}

/// A type for representing amounts of Tokens.
///
/// # Panics
///
/// * Arithmetics (addition, subtraction) on the Tokens type panics if the underlying type
///   overflows.
#[derive(
    CandidType, Serialize, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Tokens {
    e8s: u64,
}

impl Tokens {
    /// The maximum number of Tokens we can hold on a single account.
    pub const MAX: Self = Tokens { e8s: u64::MAX };
    /// Zero Tokens.
    pub const ZERO: Self = Tokens { e8s: 0 };
    /// How many times can Tokenss be divided
    pub const SUBDIVIDABLE_BY: u64 = 100_000_000;

    /// Constructs an amount of Tokens from the number of 10^-8 Tokens.
    pub const fn from_e8s(e8s: u64) -> Self {
        Self { e8s }
    }

    /// Returns the number of 10^-8 Tokens in this amount.
    pub const fn e8s(&self) -> u64 {
        self.e8s
    }
}

impl Add for Tokens {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let e8s = self.e8s.checked_add(other.e8s).unwrap_or_else(|| {
            panic!(
                "Add Tokens {} + {} failed because the underlying u64 overflowed",
                self.e8s, other.e8s
            )
        });
        Self { e8s }
    }
}

impl AddAssign for Tokens {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for Tokens {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let e8s = self.e8s.checked_sub(other.e8s).unwrap_or_else(|| {
            panic!(
                "Subtracting Tokens {} - {} failed because the underlying u64 underflowed",
                self.e8s, other.e8s
            )
        });
        Self { e8s }
    }
}

impl SubAssign for Tokens {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl fmt::Display for Tokens {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{:08}",
            self.e8s / Tokens::SUBDIVIDABLE_BY,
            self.e8s % Tokens::SUBDIVIDABLE_BY
        )
    }
}

/// Subaccount is an arbitrary 32-byte byte array.
/// Ledger uses subaccounts to compute account address, which enables one
/// principal to control multiple ledger accounts.
#[derive(
    CandidType, Serialize, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Subaccount(pub [u8; 32]);

/// AccountIdentifier is a 32-byte array.
/// The first 4 bytes is big-endian encoding of a CRC32 checksum of the last 28 bytes.
#[derive(
    CandidType, Serialize, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct AccountIdentifier([u8; 32]);

impl AccountIdentifier {
    pub fn new(owner: &Principal, subaccount: &Subaccount) -> Self {
        let mut hasher = sha2::Sha224::new();
        hasher.update(b"\x0Aaccount-id");
        hasher.update(owner.as_slice());
        hasher.update(&subaccount.0[..]);
        let hash: [u8; 28] = hasher.finalize().into();

        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&hash);
        let crc32_bytes = hasher.finalize().to_be_bytes();

        let mut result = [0u8; 32];
        result[0..4].copy_from_slice(&crc32_bytes[..]);
        result[4..32].copy_from_slice(hash.as_ref());
        Self(result)
    }
}

impl TryFrom<[u8; 32]> for AccountIdentifier {
    type Error = String;

    fn try_from(bytes: [u8; 32]) -> Result<Self, Self::Error> {
        let hash = &bytes[4..];
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(hash);
        let crc32_bytes = hasher.finalize().to_be_bytes();
        if bytes[0..4] == crc32_bytes[0..4] {
            Ok(Self(bytes))
        } else {
            Err("CRC-32 checksum failed to verify".to_string())
        }
    }
}

impl AsRef<[u8]> for AccountIdentifier {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for AccountIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.as_ref()))
    }
}

/// Arguments for the `account_balance` call.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct AccountBalanceArgs {
    pub account: AccountIdentifier,
}

/// An arbitrary number associated with a transaction.
/// The caller can set it in a `transfer` call as a correlation identifier.
#[derive(
    CandidType, Serialize, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Memo(pub u64);

/// Arguments for the `transfer` call.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct TransferArgs {
    pub memo: Memo,
    pub amount: Tokens,
    pub fee: Tokens,
    pub from_subaccount: Option<Subaccount>,
    pub to: AccountIdentifier,
    pub created_at_time: Option<Timestamp>,
}

/// The sequence number of a block in the Tokens ledger blockchain.
pub type BlockIndex = u64;

/// Result of the `transfer` call.
pub type TransferResult = Result<BlockIndex, TransferError>;

/// Error of the `transfer` call.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TransferError {
    BadFee { expected_fee: Tokens },
    InsufficientFunds { balance: Tokens },
    TxTooOld { allowed_window_nanos: u64 },
    TxCreatedInFuture,
    TxDuplicate { duplicate_of: BlockIndex },
}

impl fmt::Display for TransferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadFee { expected_fee } => {
                write!(f, "transaction fee should be {}", expected_fee)
            }
            Self::InsufficientFunds { balance } => {
                write!(
                    f,
                    "the debit account doesn't have enough funds to complete the transaction, current balance: {}",
                    balance
                )
            }
            Self::TxTooOld {
                allowed_window_nanos,
            } => write!(
                f,
                "transaction is older than {} seconds",
                allowed_window_nanos / 1_000_000_000
            ),
            Self::TxCreatedInFuture => write!(f, "transaction's created_at_time is in future"),
            Self::TxDuplicate { duplicate_of } => write!(
                f,
                "transaction is a duplicate of another transaction in block {}",
                duplicate_of
            ),
        }
    }
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Operation {
    Mint {
        to: AccountIdentifier,
        amount: Tokens,
    },
    Burn {
        from: AccountIdentifier,
        amount: Tokens,
    },
    Transfer {
        from: AccountIdentifier,
        to: AccountIdentifier,
        amount: Tokens,
        fee: Tokens,
    },
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub memo: Memo,
    pub operation: Option<Operation>,
    /// The time at which the client of the ledger constructed the transaction.
    pub created_at_time: Timestamp,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Block {
    /// The hash of the parent block.
    pub parent_hash: Option<[u8; 32]>,
    pub transaction: Transaction,
    /// The time at which the ledger constructed the block.
    pub timestamp: Timestamp,
}

#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct GetBlocksArgs {
    /// The index of the first block to fetch.
    pub start: BlockIndex,
    /// Max number of blocks to fetch.
    pub length: u64,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct QueryBlocksResponse {
    pub chain_length: u64,
    /// The replica certificate for the last block hash (see https://internetcomputer.org/docs/current/references/ic-interface-spec#certification-encoding).
    /// Not available when querying blocks from a canister.
    pub certificate: Option<ByteBuf>,
    pub blocks: Vec<Block>,
    /// The index of the first block in [QueryBlocksResponse::blocks].
    pub first_block_index: BlockIndex,
    pub archived_blocks: Vec<ArchivedBlockRange>,
}

#[derive(CandidType, Deserialize, Clone, Debug)]
pub struct ArchivedBlockRange {
    pub start: BlockIndex,
    pub length: u64,
    pub callback: QueryArchiveFn,
}

#[derive(CandidType, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct BlockRange {
    pub blocks: Vec<Block>,
}

pub type GetBlocksResult = Result<BlockRange, GetBlocksError>;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, CandidType)]
pub enum GetBlocksError {
    BadFirstBlockIndex {
        requested_index: BlockIndex,
        first_valid_index: BlockIndex,
    },
    Other {
        error_code: u64,
        error_message: String,
    },
}

impl fmt::Display for GetBlocksError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadFirstBlockIndex {
                requested_index,
                first_valid_index,
            } => write!(
                f,
                "invalid first block index: requested block = {}, first valid block = {}",
                requested_index, first_valid_index
            ),
            Self::Other {
                error_code,
                error_message,
            } => write!(
                f,
                "failed to query blocks (error code {}): {}",
                error_code, error_message
            ),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(transparent)]
pub struct QueryArchiveFn(Func);

impl From<Func> for QueryArchiveFn {
    fn from(func: Func) -> Self {
        Self(func)
    }
}

impl From<QueryArchiveFn> for Func {
    fn from(query_func: QueryArchiveFn) -> Self {
        query_func.0
    }
}

impl CandidType for QueryArchiveFn {
    fn _ty() -> candid::types::Type {
        candid::types::Type::Func(candid::types::Function {
            modes: vec![candid::parser::types::FuncMode::Query],
            args: vec![GetBlocksArgs::_ty()],
            rets: vec![GetBlocksResult::_ty()],
        })
    }

    fn idl_serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: candid::types::Serializer,
    {
        Func::from(self.clone()).idl_serialize(serializer)
    }
}

/// Calls the "account_balance" method on the specified canister.
///
/// # Example
/// ```no_run
/// use ic_cdk::api::{caller, call::call};
/// use ic_ledger_types::{AccountIdentifier, AccountBalanceArgs, Tokens, DEFAULT_SUBACCOUNT, MAINNET_LEDGER_CANISTER_ID, account_balance};
///
/// async fn check_callers_balance() -> Tokens {
///   account_balance(
///     MAINNET_LEDGER_CANISTER_ID,
///     AccountBalanceArgs {
///       account: AccountIdentifier::new(&caller(), &DEFAULT_SUBACCOUNT)
///     }
///   ).await.expect("call to ledger failed")
/// }
/// ```
pub async fn account_balance(
    ledger_canister_id: Principal,
    args: AccountBalanceArgs,
) -> CallResult<Tokens> {
    let (icp,) = ic_cdk::call(ledger_canister_id, "account_balance", (args,)).await?;
    Ok(icp)
}

/// Calls the "transfer" method on the specified canister.
/// # Example
/// ```no_run
/// use ic_cdk::api::{caller, call::call};
/// use ic_ledger_types::{AccountIdentifier, BlockIndex, Memo, TransferArgs, Tokens, DEFAULT_SUBACCOUNT, DEFAULT_FEE, MAINNET_LEDGER_CANISTER_ID, transfer};
///
/// async fn transfer_to_caller() -> BlockIndex {
///   transfer(
///     MAINNET_LEDGER_CANISTER_ID,
///     TransferArgs {
///       memo: Memo(0),
///       amount: Tokens::from_e8s(1_000_000),
///       fee: DEFAULT_FEE,
///       from_subaccount: None,
///       to: AccountIdentifier::new(&caller(), &DEFAULT_SUBACCOUNT),
///       created_at_time: None,
///     }
///   ).await.expect("call to ledger failed").expect("transfer failed")
/// }
/// ```
pub async fn transfer(
    ledger_canister_id: Principal,
    args: TransferArgs,
) -> CallResult<TransferResult> {
    let (result,) = ic_cdk::call(ledger_canister_id, "transfer", (args,)).await?;
    Ok(result)
}

#[derive(Serialize, Deserialize, CandidType, Clone, Hash, Debug, PartialEq, Eq)]
pub struct Symbol {
    pub symbol: String,
}

/// Calls the "token_symbol" method on the specified canister.
/// # Example
/// ```no_run
/// use candid::Principal;
/// use ic_cdk::api::{caller, call::call};
/// use ic_ledger_types::{Symbol, token_symbol};
///
/// async fn symbol(ledger_canister_id: Principal) -> String {
///   token_symbol(ledger_canister_id).await.expect("call to ledger failed").symbol
/// }
/// ```
pub async fn token_symbol(ledger_canister_id: Principal) -> CallResult<Symbol> {
    let (result,) = ic_cdk::call(ledger_canister_id, "token_symbol", ()).await?;
    Ok(result)
}

/// Calls the "query_block" method on the specified canister.
/// # Example
/// ```no_run
/// use candid::Principal;
/// use ic_cdk::api::call::CallResult;
/// use ic_ledger_types::{BlockIndex, Block, GetBlocksArgs, query_blocks, query_archived_blocks};
///
/// async fn query_one_block(ledger: Principal, block_index: BlockIndex) -> CallResult<Option<Block>> {
///   let args = GetBlocksArgs { start: block_index, length: 1 };
///
///   let blocks_result = query_blocks(ledger, args.clone()).await?;
///
///   if blocks_result.blocks.len() >= 1 {
///       debug_assert_eq!(blocks_result.first_block_index, block_index);
///       return Ok(blocks_result.blocks.into_iter().next());
///   }
///
///   if let Some(func) = blocks_result
///       .archived_blocks
///       .into_iter()
///       .find_map(|b| (b.start <= block_index && (block_index - b.start) < b.length).then(|| b.callback)) {
///       match query_archived_blocks(&func, args).await? {
///           Ok(range) => return Ok(range.blocks.into_iter().next()),
///           _ => (),
///       }
///   }
///   Ok(None)
/// }
pub async fn query_blocks(
    ledger_canister_id: Principal,
    args: GetBlocksArgs,
) -> CallResult<QueryBlocksResponse> {
    let (result,) = ic_cdk::call(ledger_canister_id, "query_blocks", (args,)).await?;
    Ok(result)
}

pub async fn query_archived_blocks(
    func: &QueryArchiveFn,
    args: GetBlocksArgs,
) -> CallResult<GetBlocksResult> {
    let (result,) = ic_cdk::api::call::call(func.0.principal, &func.0.method, (args,)).await?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;

    #[test]
    fn test_account_id() {
        assert_eq!(
            "bdc4ee05d42cd0669786899f256c8fd7217fa71177bd1fa7b9534f568680a938".to_string(),
            AccountIdentifier::new(
                &Principal::from_text(
                    "iooej-vlrze-c5tme-tn7qt-vqe7z-7bsj5-ebxlc-hlzgs-lueo3-3yast-pae"
                )
                .unwrap(),
                &DEFAULT_SUBACCOUNT
            )
            .to_string()
        );
    }

    #[test]
    fn test_account_id_try_from() {
        let mut bytes: [u8; 32] = [0; 32];
        bytes.copy_from_slice(
            &hex::decode("bdc4ee05d42cd0669786899f256c8fd7217fa71177bd1fa7b9534f568680a938")
                .unwrap(),
        );
        assert!(AccountIdentifier::try_from(bytes).is_ok());
        bytes[0] = 0;
        assert!(AccountIdentifier::try_from(bytes).is_err());
    }

    #[test]
    fn test_ledger_canister_id() {
        assert_eq!(
            MAINNET_LEDGER_CANISTER_ID,
            Principal::from_text("ryjl3-tyaaa-aaaaa-aaaba-cai").unwrap()
        );
    }

    #[test]
    fn test_governance_canister_id() {
        assert_eq!(
            MAINNET_GOVERNANCE_CANISTER_ID,
            Principal::from_text("rrkah-fqaaa-aaaaa-aaaaq-cai").unwrap()
        );
    }

    #[test]
    fn test_cycles_minting_canister_id() {
        assert_eq!(
            MAINNET_CYCLES_MINTING_CANISTER_ID,
            Principal::from_text("rkp4c-7iaaa-aaaaa-aaaca-cai").unwrap()
        );
    }
}
