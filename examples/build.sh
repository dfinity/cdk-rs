#!/usr/bin/env bash
set -e

name="$1"
package="$2"
root="$(dirname "$0")/.."
example_root="$(dirname "$0")/$name"
did_file="/tmp/a.did"

cargo update --manifest-path="$example_root/Cargo.toml"

cargo build --manifest-path="$example_root/Cargo.toml" \
    --target wasm32-unknown-unknown \
    --release \
    --package "$package"

candid-extractor "$example_root/target/wasm32-unknown-unknown/release/$package.wasm" 2>/dev/null > $did_file || true

ic-wasm "$example_root/target/wasm32-unknown-unknown/release/$package.wasm" \
    -o "$example_root/target/wasm32-unknown-unknown/release/$package.wasm" \
    metadata candid:service -v public -f $did_file

ic-wasm "$example_root/target/wasm32-unknown-unknown/release/$package.wasm" \
    -o "$example_root/target/wasm32-unknown-unknown/release/$package-opt.wasm" \
    shrink
