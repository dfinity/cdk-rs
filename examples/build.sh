#!/usr/bin/env bash
set -e

name="$1"
package="$2"
root="$(dirname "$0")/.."
example_root="$(dirname "$0")/$name"
did_file="/tmp/a.did"

# This script generates the did file, build the project (passed as $1) and then run the ic-wasm to shrink and attach metadata.
cargo build --manifest-path="$example_root/Cargo.toml" \
    --target wasm32-unknown-unknown \
    --release \
    --package "$package" --features "ic-cdk/wasi ic-cdk-macros/export_candid"

wasmtime "$example_root/target/wasm32-unknown-unknown/release/$package.wasm" > $did_file

cargo build --manifest-path="$example_root/Cargo.toml" \
    --target wasm32-unknown-unknown \
    --release \
    --package "$package"

ic-wasm "$example_root/target/wasm32-unknown-unknown/release/$package.wasm" \
    -o "$example_root/target/wasm32-unknown-unknown/release/$package.wasm" \
    metadata candid:service -v public -f $did_file

ic-wasm "$example_root/target/wasm32-unknown-unknown/release/$package.wasm" \
    -o "$example_root/target/wasm32-unknown-unknown/release/$package-opt.wasm" \
    shrink

