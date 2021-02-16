# ic-cdk-optimizer
Optimizer library to reduce the size of CDK WASMs.

# Installation
To install the ic-cdk-optimizer with cargo run:

`cargo install ic-cdk-optimizer --root target`

# Usage
ic-cdk-optimizer will take your existing `.wasm` files and strip unused data segments. So for a canister named [rust_hello](https://sdk.dfinity.org/docs/rust-guide/rust-quickstart.html) the following command produces an optimized `rust_hello-opt.wasm`:

```
./target/bin/ic-cdk-optimizer ./target/wasm32-unknown-unknown/debug/rust_hello.wasm -o ./target/wasm32-unknown-unknown/release/rust_hello-opt.wasm
```

The `dfx.json` file needs to be updated to reference the new optimized `.wasm` file:

```
% cat dfx.json
{
  "canisters": {
    "rust_hello": {
      "build": "cargo build --target wasm32-unknown-unknown --package rust_hello",
      "candid": "src/rust_hello/rust_hello.did",
      "wasm": "target/wasm32-unknown-unknown/release/rust_hello-opt.wasm",
      "type": "custom"
    }
  },
  "defaults": {
    "build": {
      "packtool": ""
    }
  },
  "dfx": "0.6.23",
  "networks": {
    "local": {
      "bind": "127.0.0.1:8000",
      "type": "ephemeral"
    },
    "ic": {
      "providers": [
        "https://gw.dfinity.network"
      ],
      "type": "persistent"
    }
  },
  "version": 1
}
```

Then simply `dfx deploy` to deploy the optimized version! Happy rusting:

```
% dfx deploy
Deploying all canisters.
All canisters have already been created.
Building canisters...
Executing 'cargo build --target wasm32-unknown-unknown --package rust_hello'
    Finished dev [unoptimized + debuginfo] target(s) in 0.13s
Installing canisters...
Installing code for canister rust_hello, with canister_id rwlgt-iiaaa-aaaaa-aaaaa-cai
Deployed canisters.
```
