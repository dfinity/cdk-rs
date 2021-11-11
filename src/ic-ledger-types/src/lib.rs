use candid::{CandidType, Principal};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::fmt;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// The subaccont that is used by default.
pub const DEFAULT_SUBACCOUNT: Subaccount = Subaccount([0; 32]);

/// The default fee for ledger transactions.
pub const DEFAULT_FEE: ICP = ICP { e8s: 10_000 };

/// Id of the ledger canister on the IC.
pub const MAINNET_LEDGER_CANISTER_ID: Principal =
    Principal::from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01]);

/// Number of nanoseconds from the UNIX epoch in UTC timezone.
#[derive(
    CandidType, Serialize, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Timestamp {
    pub timestamp_nanos: u64,
}

/// A type for representing amounts of ICP.
///
/// # Panics
///
/// * Arithmetics (addition, subtraction) on the ICP type panics if the underlying type
///   overflows.
#[derive(
    CandidType, Serialize, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct ICP {
    e8s: u64,
}

impl ICP {
    /// The maximum number of ICP we can hold on a single account.
    pub const MAX: Self = ICP { e8s: u64::MAX };
    /// Zero ICP.
    pub const ZERO: Self = ICP { e8s: 0 };
    /// How many times can ICPs be divided
    pub const SUBDIVIDABLE_BY: u64 = 100_000_000;

    /// Constructs an amount of ICP from the number of 10^-8 ICP.
    pub fn from_e8s(e8s: u64) -> Self {
        Self { e8s }
    }

    /// Returns the number of 10^-8 ICP in this amount.
    pub fn e8s(&self) -> u64 {
        self.e8s
    }
}

impl Add for ICP {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let e8s = self.e8s.checked_add(other.e8s).unwrap_or_else(|| {
            panic!(
                "Add ICP {} + {} failed because the underlying u64 overflowed",
                self.e8s, other.e8s
            )
        });
        Self { e8s }
    }
}

impl AddAssign for ICP {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl Sub for ICP {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        let e8s = self.e8s.checked_sub(other.e8s).unwrap_or_else(|| {
            panic!(
                "Subtracting ICP {} - {} failed because the underlying u64 underflowed",
                self.e8s, other.e8s
            )
        });
        Self { e8s }
    }
}

impl SubAssign for ICP {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl fmt::Display for ICP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}.{:08} ICP",
            self.e8s / ICP::SUBDIVIDABLE_BY,
            self.e8s % ICP::SUBDIVIDABLE_BY
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

#[derive(
    CandidType, Serialize, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct AccountIdentifier([u8; 28]);

impl AccountIdentifier {
    pub fn new(owner: &Principal, subaccount: &Subaccount) -> Self {
        let mut hash = sha2::Sha224::new();
        hash.update(b"\x0Aaccount-id");
        hash.update(owner.as_slice());
        hash.update(&subaccount.0[..]);

        Self(hash.finalize().into())
    }
}

impl AsRef<[u8]> for AccountIdentifier {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Address is a 32-byte array.
/// The first 4 bytes is big-endian encoding of a CRC32 checksum of the last 28 bytes.
#[derive(
    CandidType, Serialize, Deserialize, Clone, Copy, Hash, Debug, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct Address([u8; 32]);

impl Address {
    pub fn new(owner: &Principal, subaccount: &Subaccount) -> Self {
        AccountIdentifier::new(owner, subaccount).into()
    }
}

impl From<AccountIdentifier> for Address {
    fn from(account_id: AccountIdentifier) -> Self {
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&account_id.0);
        let crc32_bytes = hasher.finalize().to_be_bytes();

        let mut result = [0u8; 32];
        result[0..4].copy_from_slice(&crc32_bytes[..]);
        result[4..32].copy_from_slice(account_id.as_ref());
        Self(result)
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.as_ref()))
    }
}

/// Arguments for the `account_balance` call.
/// Example:
/// ```ignore
/// use ic_cdk::api::{caller, call::call};
/// use ic_cdk_macros::update;
/// use ic_ledger_types::{Address, AccountBalanceArgs, ICP, DEFAULT_SUBACCOUNT, MAINNET_LEDGER_CANISTER_ID};
///
/// #[update]
/// async fn check_callers_balance() -> ICP {
///   let (icp,) = call(
///     MAINNET_LEDGER_CANISTER_ID,
///     "account_balance",
///     (AccountBalanceArgs {
///       account: Address::new(caller(), &DEFAULT_SUBACCOUNT)
///     },)
///   ).await.expect("call to ledger failed");
///   icp
/// }
/// ```
#[derive(CandidType, Serialize, Deserialize, Clone, Debug)]
pub struct AccountBalanceArgs {
    pub account: Address,
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
    pub amount: ICP,
    pub fee: ICP,
    pub from_subaccount: Option<Subaccount>,
    pub to: Address,
    pub created_at_time: Option<Timestamp>,
}

/// The sequence number of a block in the ICP ledger blockchain.
pub type BlockIndex = u64;

/// Result of the `transfer` call.
pub type TransferResult = Result<BlockIndex, TransferError>;

/// Error of the `transfer` call.
#[derive(CandidType, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TransferError {
    BadFee { expected_fee: ICP },
    InsufficientFunds { balance: ICP },
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::string::ToString;

    #[test]
    fn test_account_id() {
        assert_eq!(
            "bdc4ee05d42cd0669786899f256c8fd7217fa71177bd1fa7b9534f568680a938".to_string(),
            Address::new(
                &Principal::from_text(
                    "iooej-vlrze-c5tme-tn7qt-vqe7z-7bsj5-ebxlc-hlzgs-lueo3-3yast-pae"
                )
                .unwrap(),
                &DEFAULT_SUBACCOUNT
            )
            .to_string()
        );
    }
}
