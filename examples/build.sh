#!/usr/bin/env bash
set -e

name="$1"
package="$2"
root="$(dirname "$0")/.."
example_root="$(dirname "$0")/$name"

# This script builds an example project (passed as $1) and then run the ic-cdk-optimizer on it.
cargo build --manifest-path="$example_root/Cargo.toml" \
    --target wasm32-unknown-unknown \
    --release \
    --package "$package" --features "ic-cdk/wasi ic-cdk-macros/export_candid"

ic-wasm "$example_root/target/wasm32-unknown-unknown/release/$package.wasm" \
    -o "$example_root/target/wasm32-unknown-unknown/release/$package-opt.wasm" \
    shrink
