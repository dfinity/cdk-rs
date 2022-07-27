use candid::Principal;
use ic_cdk::api::call::{call, CallResult};

mod types;
pub use types::*;

/// bitcoin_get_balance: (get_balance_request) -> (satoshi);
pub async fn bitcoin_get_balance(arg: GetBalanceRequest) -> CallResult<(Satoshi,)> {
    call(
        Principal::management_canister(),
        "bitcoin_get_balance",
        (arg,),
    )
    .await
}

/// bitcoin_get_utxos: (get_utxos_request) -> (get_utxos_response);
pub async fn bitcoin_get_utxos(arg: GetUtxosRequest) -> CallResult<(GetUtxosResponse,)> {
    call(
        Principal::management_canister(),
        "bitcoin_get_utxos",
        (arg,),
    )
    .await
}

/// bitcoin_send_transaction: (send_transaction_request) -> ();
pub async fn bitcoin_send_transaction(arg: SendTransactionRequest) -> CallResult<()> {
    call(
        Principal::management_canister(),
        "bitcoin_send_transaction",
        (arg,),
    )
    .await
}

/// bitcoin_get_current_fee_percentiles: (get_current_fee_percentiles_request) -> (vec millisatoshi_per_byte);
pub async fn bitcoin_get_current_fee_percentiles(
    arg: GetCurrentFeePercentilesRequest,
) -> CallResult<(Vec<MillisatoshiPerByte>,)> {
    call(
        Principal::management_canister(),
        "bitcoin_get_current_fee_percentiles",
        (arg,),
    )
    .await
}
