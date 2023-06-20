//! The IC Bitcoin API.

use crate::api::call::{call_with_payment128, CallResult};
use candid::Principal;

mod types;
pub use types::*;

/// See [IC method `bitcoin_get_balance`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_get_balance).
///
/// This API charges cycles. Please check [`API fees`](https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/bitcoin-how-it-works/#api-fees).
pub async fn bitcoin_get_balance(arg: GetBalanceRequest, cycles: u128) -> CallResult<(Satoshi,)> {
    call_with_payment128(
        Principal::management_canister(),
        "bitcoin_get_balance",
        (arg,),
        cycles,
    )
    .await
}

/// See [IC method `bitcoin_get_utxos`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_get_utxos).
///
/// This API charges cycles. Please check [`API fees`](https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/bitcoin-how-it-works/#api-fees).
pub async fn bitcoin_get_utxos(
    arg: GetUtxosRequest,
    cycles: u128,
) -> CallResult<(GetUtxosResponse,)> {
    call_with_payment128(
        Principal::management_canister(),
        "bitcoin_get_utxos",
        (arg,),
        cycles,
    )
    .await
}

/// See [IC method `bitcoin_send_transaction`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_send_transaction).
///
/// This API charges cycles. Please check [`API fees`](https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/bitcoin-how-it-works/#api-fees).
pub async fn bitcoin_send_transaction(arg: SendTransactionRequest, cycles: u128) -> CallResult<()> {
    call_with_payment128(
        Principal::management_canister(),
        "bitcoin_send_transaction",
        (arg,),
        cycles,
    )
    .await
}

/// See [IC method `bitcoin_get_current_fee_percentiles`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_get_current_fee_percentiles).
///
/// This API charges cycles. Please check [`API fees`](https://internetcomputer.org/docs/current/developer-docs/integrations/bitcoin/bitcoin-how-it-works/#api-fees).
pub async fn bitcoin_get_current_fee_percentiles(
    arg: GetCurrentFeePercentilesRequest,
    cycles: u128,
) -> CallResult<(Vec<MillisatoshiPerByte>,)> {
    call_with_payment128(
        Principal::management_canister(),
        "bitcoin_get_current_fee_percentiles",
        (arg,),
        cycles,
    )
    .await
}
