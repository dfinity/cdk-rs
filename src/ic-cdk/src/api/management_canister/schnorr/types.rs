use candid::CandidType;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use super::super::main::CanisterId;

/// Argument Type of [schnorr_public_key](super::schnorr_public_key).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SchnorrPublicKeyArgument {
    /// Canister id, default to the canister id of the caller if None.
    pub canister_id: Option<CanisterId>,
    /// A vector of variable length byte strings.
    pub derivation_path: Vec<Vec<u8>>,
    /// See [SchnorrKeyId].
    pub key_id: SchnorrKeyId,
}

/// Response Type of [schnorr_public_key](super::schnorr_public_key).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SchnorrPublicKeyResponse {
    /// An Schnorr public key encoded in SEC1 compressed form.
    pub public_key: Vec<u8>,
    /// Can be used to deterministically derive child keys of the public_key.
    pub chain_code: Vec<u8>,
}

/// SignWithBip341 auxiliary parameter
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SignWithBip341Aux {
    /// The merkle_root_hash must be generated in accordance with BIP341's specification for taproot_output_script. 
    /// 
    /// Specifically it should be either an empty bytestring (for the script == None case)
    /// or else 32 bytes generated using the procedure documented as taproot_tree_helper.
    pub merkle_root_hash: ByteBuf,
}

/// The auxiliary parameter type SignWithSchnorrAux is an enumeration.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone,
)]
pub enum SignWithSchnorrAux {
    ///  The only currently supported variant is bip341 which allows passing a Merkle tree root hash, 
    ///  which is required to implement Taproot signatures as defined in BIP341. 
    /// 
    ///  The bip341 variant is only allowed for bip340secp256k1 signatures.
    #[serde(rename = "bip341")]
    Bip341(SignWithBip341Aux),
}

/// Argument Type of [sign_with_schnorr](super::sign_with_schnorr).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SignWithSchnorrArgument {
    /// Message to be signed.
    pub message: Vec<u8>,
    /// A vector of variable length byte strings.
    pub derivation_path: Vec<Vec<u8>>,
    /// See [SchnorrKeyId].
    pub key_id: SchnorrKeyId,
    /// An optional auxiliary parameter. 
    /// 
    /// If no auxiliary parameter is provided, then bip340secp256k1 signatures are generated in accordance with BIP340.
    /// 
    /// Check [sign_with_schnorr](https://internetcomputer.org/docs/references/ic-interface-spec#ic-sign_with_schnorr) for more details.
    pub aux: Option<SignWithSchnorrAux>,
}

/// Response Type of [sign_with_schnorr](super::sign_with_schnorr).
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SignWithSchnorrResponse {
    /// The encoding of the signature depends on the key ID's algorithm.
    pub signature: Vec<u8>,
}

/// Schnorr KeyId.
#[derive(
    CandidType, Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Default,
)]
pub struct SchnorrKeyId {
    /// See [SchnorrAlgorithm].
    pub algorithm: SchnorrAlgorithm,
    /// Name.
    pub name: String,
}

/// Schnorr Algorithm.
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
pub enum SchnorrAlgorithm {
    /// BIP-340 secp256k1.
    #[serde(rename = "bip340secp256k1")]
    #[default]
    Bip340secp256k1,
    /// ed25519.
    #[serde(rename = "ed25519")]
    Ed25519,
}
