# candid-extractor

A CLI tool to extract candid definition from canister WASM.

## What

`ic0` is simply an unsafe Rust translation of Internet Computer System API as described in the [Specification](https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-imports).

## Update ic_mock.wat

`candid-extractor` requires a mock WASM (`ic_mock.wat`) which provides ic0 imports.

Such `ic_mock.wat` is directly generated from the [system API][1].

When interface-spec releases a new version that modify ic0 system API:

1. replace `ic0.txt` in the root of this project;
2. execute `cargo run --example=generate_mock_wat`;

`ic_mock.wat` should be updated.

[1]: https://internetcomputer.org/docs/current/references/ic-interface-spec/#system-api-imports
