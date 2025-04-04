#!/bin/bash

set -euo pipefail

uname_sys=$(uname -s | tr '[:upper:]' '[:lower:]')
echo "uname_sys: $uname_sys"

SCRIPTS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPTS_DIR/../e2e-tests"
# extract the tag from e2e-tests/Cargo.toml
tag=$(grep -E 'pocket-ic.*tag' Cargo.toml | sed -n "s/.*tag *= *\"\([^\"]*\)\".*/\1/p")

ARTIFACTS_DIR="$SCRIPTS_DIR/../target/e2e-tests-artifacts"
mkdir -p "$ARTIFACTS_DIR"
cd "$ARTIFACTS_DIR"
echo -n "$tag" > pocket-ic-tag

if [ "$uname_sys" = "linux" ]; then
    # TODO: this is a temporary link to the pocket-ic binary that supports the ic0.root_key_* API (linux only)
    curl -sL "https://download.dfinity.systems/ic/ab123e32fefce0953bc2bae615f7fff3a9ec7d30/binaries/x86_64-linux/pocket-ic.gz" --output pocket-ic.gz
else
    curl -sL "https://github.com/dfinity/ic/releases/download/$tag/pocket-ic-x86_64-$uname_sys.gz" --output pocket-ic.gz
fi
gzip -df pocket-ic.gz
chmod a+x pocket-ic
./pocket-ic --version

if [[ "$uname_sys" == "darwin" ]]; then
    xattr -dr com.apple.quarantine pocket-ic
fi
