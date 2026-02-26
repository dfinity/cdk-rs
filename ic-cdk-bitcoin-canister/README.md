[![Documentation](https://docs.rs/ic-cdk-bitcoin-canister/badge.svg)](https://docs.rs/ic-cdk-bitcoin-canister/)
[![Crates.io](https://img.shields.io/crates/v/ic-cdk-bitcoin-canister.svg)](https://crates.io/crates/ic-cdk-bitcoin-canister)
[![License](https://img.shields.io/crates/l/ic-cdk-bitcoin-canister.svg)](https://github.com/dfinity/cdk-rs/blob/main/LICENSE)
[![Downloads](https://img.shields.io/crates/d/ic-cdk-bitcoin-canister.svg)](https://crates.io/crates/ic-cdk-bitcoin-canister)
[![CI](https://github.com/dfinity/cdk-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/dfinity/cdk-rs/actions/workflows/ci.yml)

# ic-cdk-bitcoin-canister

This crate provides functionality for making inter-canister calls to the [Bitcoin canisters][1].

The Bitcoin canisters allow for interactions with the Bitcoin network from within the Internet Computer.
This crate includes functions and types that facilitate these interactions, adhering to the
[Bitcoin Canisters Interface Specification][2].

The types are re-exported from the [`ic_btc_interface`](https://docs.rs/ic-btc-interface) crate.

## Bounded-wait vs. Unbounded-wait

Interacting with the Bitcoin canisters involves making inter-canister calls,
which can be either [bounded-wait][bounded-wait] or [unbounded-wait][unbounded-wait].

Most of the functions in this crate use the bounded-wait calls because they only read state.
The only function that uses the unbounded-wait call is [`bitcoin_send_transaction`].

If the default behavior is not suitable for a particular use case, the [`Call`] struct can be used directly to make the call.

For example, [`bitcoin_get_utxos`] makes an bounded-wait call. If an unbounded-wait call is preferred, the call can be made as follows:
```rust, no_run
use ic_cdk_bitcoin_canister::{cost_get_utxos, get_bitcoin_canister_id, GetUtxosRequest, GetUtxosResponse, Network};
use ic_cdk::call::Call;

async fn example() -> ic_cdk::call::CallResult<GetUtxosResponse> {
    let arg = GetUtxosRequest {
        address: "bc1qar0srrr7xfkvy5l643lydnw9re59gtzzwf5mdq".to_string(),
        network: Network::Mainnet.into(),
        filter: None,
    };
    let canister_id = get_bitcoin_canister_id(Network::from(arg.network));
    let cycles = cost_get_utxos(&arg);
    let res: GetUtxosResponse = Call::unbounded_wait(canister_id, "bitcoin_get_utxos")
        .with_arg(&arg)
        .with_cycles(cycles)
        .await?
        .candid()?;
    Ok(res)
}
```

### Cycle Cost

All the Bitcoin canister methods require cycles to be attached to the call.
The helper functions in this crate automatically calculate the required cycles and attach them to the call.

For completeness, this crate also provides functions to calculate the cycle cost:
- [`cost_get_utxos`]
- [`cost_get_balance`]
- [`cost_get_current_fee_percentiles`]
- [`cost_get_block_headers`]
- [`cost_send_transaction`]

## Bitcoin Canister ID

The Bitcoin canister ID is determined by the network.
The helper functions in this crate automatically determine the canister ID based on the `network` field in the request.

For completeness, the [`get_bitcoin_canister_id`] function can be used to get the canister ID manually.

[1]: https://github.com/dfinity/bitcoin-canister
[2]: https://github.com/dfinity/bitcoin-canister/blob/master/INTERFACE_SPECIFICATION.md
[bounded-wait]: https://docs.rs/ic-cdk/latest/ic_cdk/call/struct.Call.html#method.bounded_wait
[unbounded-wait]: https://docs.rs/ic-cdk/latest/ic_cdk/call/struct.Call.html#method.unbounded_wait
[`Call`]: https://docs.rs/ic-cdk/latest/ic_cdk/call/struct.Call.html
[`bitcoin_get_utxos`]: https://docs.rs/ic-cdk-bitcoin-canister/latest/ic_cdk_bitcoin_canister/fn.bitcoin_get_utxos.html
[`bitcoin_send_transaction`]: https://docs.rs/ic-cdk-bitcoin-canister/latest/ic_cdk_bitcoin_canister/fn.bitcoin_send_transaction.html
[`cost_get_utxos`]: https://docs.rs/ic-cdk-bitcoin-canister/latest/ic_cdk_bitcoin_canister/fn.cost_get_utxos.html
[`cost_get_balance`]: https://docs.rs/ic-cdk-bitcoin-canister/latest/ic_cdk_bitcoin_canister/fn.cost_get_balance.html
[`cost_get_current_fee_percentiles`]: https://docs.rs/ic-cdk-bitcoin-canister/latest/ic_cdk_bitcoin_canister/fn.cost_get_current_fee_percentiles.html
[`cost_get_block_headers`]: https://docs.rs/ic-cdk-bitcoin-canister/latest/ic_cdk_bitcoin_canister/fn.cost_get_block_headers.html
[`cost_send_transaction`]: https://docs.rs/ic-cdk-bitcoin-canister/latest/ic_cdk_bitcoin_canister/fn.cost_send_transaction.html
[`get_bitcoin_canister_id`]: https://docs.rs/ic-cdk-bitcoin-canister/latest/ic_cdk_bitcoin_canister/fn.get_bitcoin_canister_id.html
