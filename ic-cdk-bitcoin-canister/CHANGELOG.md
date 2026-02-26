# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [unreleased]

### Changed

- [BREAKING] All Bitcoin interface types are now re-exported from the [`ic-btc-interface`](https://crates.io/crates/ic-btc-interface) crate instead of being defined locally. This aligns with the canonical types published by the Bitcoin Canister team. The Candid-level interface remains compatible, but several Rust-level breaking changes are introduced.

#### Migration guide from `ic_cdk::bitcoin_canister`

##### Renamed types

| Before | After |
|---|---|
| `Outpoint` | `OutPoint` (capital `P`) |
| `BlockHeight` | `Height` |

##### Changed field types in request structs

The `network` field in all request structs (`GetUtxosRequest`, `GetBalanceRequest`, `GetCurrentFeePercentilesRequest`, `GetBlockHeadersRequest`, `SendTransactionRequest`) changed from `Network` to `NetworkInRequest`. Convert using `.into()`:

```rust
// Before
let arg = GetUtxosRequest {
    network: Network::Mainnet,
    ..
};

// After
let arg = GetUtxosRequest {
    network: Network::Mainnet.into(), // or NetworkInRequest::Mainnet
    ..
};
```

Similarly, `GetUtxosRequest::filter` changed from `Option<UtxosFilter>` to `Option<UtxosFilterInRequest>`:

```rust
// Before
filter: Some(UtxosFilter::MinConfirmations(6))

// After
filter: Some(UtxosFilterInRequest::MinConfirmations(6))
```

##### Changed field types in structs

- `OutPoint::txid`: `Vec<u8>` → `Txid` (a `[u8; 32]` newtype). Convert with `Txid::from(bytes: [u8; 32])` or `txid.as_ref()` / `<[u8; 32]>::from(txid)`.
- `Utxo::height`: `BlockHeight` (`u32`) → `Height` (`u32`). Same underlying type, just renamed.
- `GetUtxosResponse::tip_height`: `u32` → `Height` (`u32`). Same underlying type.
- `GetUtxosResponse::next_page`: `Option<Vec<u8>>` → `Option<Page>` where `Page = serde_bytes::ByteBuf`.
- `GetBlockHeadersResponse::tip_height`: `BlockHeight` → `Height` (`u32`). Same underlying type.

##### Fewer trait derivations

The `ic-btc-interface` types derive fewer traits than the previous local definitions. Notably:

- Most types no longer derive `PartialOrd`, `Ord`, or `Hash` (except `OutPoint`, `Utxo`, `Network`, `NetworkInRequest`).
- Request types (`GetUtxosRequest`, `GetBalanceRequest`, etc.) no longer derive `Clone`, `Serialize`, `Default`, `PartialOrd`, `Ord`, or `Hash`.
- `Network` no longer derives `Default`, `PartialOrd`, or `Ord`.

##### `get_bitcoin_canister_id` signature change

The function now takes `Network` by value instead of by reference:

```rust
// Before
get_bitcoin_canister_id(&network)

// After
get_bitcoin_canister_id(network) // Network is Copy
```
