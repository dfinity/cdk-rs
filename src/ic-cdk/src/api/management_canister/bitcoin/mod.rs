use crate::api::call::{call_with_payment128, CallResult};
use candid::Principal;

mod types;
pub use types::*;

// The fees for the various bitcoin endpoints.
// TODO: where is the public doc of these parameters?
const GET_BALANCE_CYCLES: u128 = 100_000_000;
const GET_UTXOS_CYCLES: u128 = 100_000_000;
const GET_CURRENT_FEE_PERCENTILES_CYCLES: u128 = 100_000_000;
const SEND_TRANSACTION_BASE_CYCLES: u128 = 5_000_000_000;
const SEND_TRANSACTION_PER_BYTE_CYCLES: u128 = 20_000_000;

/// bitcoin_get_balance: (get_balance_request) -> (satoshi);
pub async fn bitcoin_get_balance(arg: GetBalanceRequest) -> CallResult<(Satoshi,)> {
    call_with_payment128(
        Principal::management_canister(),
        "bitcoin_get_balance",
        (arg,),
        GET_BALANCE_CYCLES,
    )
    .await
}

/// bitcoin_get_utxos: (get_utxos_request) -> (get_utxos_response);
pub async fn bitcoin_get_utxos(arg: GetUtxosRequest) -> CallResult<(GetUtxosResponse,)> {
    call_with_payment128(
        Principal::management_canister(),
        "bitcoin_get_utxos",
        (arg,),
        GET_UTXOS_CYCLES,
    )
    .await
}

/// bitcoin_send_transaction: (send_transaction_request) -> ();
pub async fn bitcoin_send_transaction(arg: SendTransactionRequest) -> CallResult<()> {
    let cycles = SEND_TRANSACTION_BASE_CYCLES
        + (*&arg.transaction.len() as u128) * SEND_TRANSACTION_PER_BYTE_CYCLES;
    call_with_payment128(
        Principal::management_canister(),
        "bitcoin_send_transaction",
        (arg,),
        cycles,
    )
    .await
}

/// bitcoin_get_current_fee_percentiles: (get_current_fee_percentiles_request) -> (vec millisatoshi_per_byte);
pub async fn bitcoin_get_current_fee_percentiles(
    arg: GetCurrentFeePercentilesRequest,
) -> CallResult<(Vec<MillisatoshiPerByte>,)> {
    call_with_payment128(
        Principal::management_canister(),
        "bitcoin_get_current_fee_percentiles",
        (arg,),
        GET_CURRENT_FEE_PERCENTILES_CYCLES,
    )
    .await
}
