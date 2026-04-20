#!/bin/bash

set -euo pipefail

case $(uname -s) in
    Linux*)     os="linux";;
    Darwin*)    os="darwin";;
    *)          echo "Unsupported OS $(uname -s)"; exit 1;;
esac

case $(uname -m) in
    x86_64*)    arch="x86_64";;
    arm64*)     arch="arm64";;
    aarch64*)   arch="arm64";;
    *)          echo "Unsupported architecture $(uname -m)"; exit 1;;
esac

echo "os: $os, arch: $arch"

SCRIPTS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPTS_DIR/.."

# Extract the source URL of the pocket-ic dependency via cargo metadata.
# With --no-deps only workspace packages are emitted; .dependencies[] still
# carries the declared source (e.g. git+https://...?rev=<hash>#<hash>).
pocket_ic_source=$(cargo metadata --format-version=1 --no-deps \
    | jq -r '.packages[].dependencies[] | select(.name=="pocket-ic") | .source' \
    | head -1)
echo "pocket-ic source: $pocket_ic_source"

ARTIFACTS_DIR="$SCRIPTS_DIR/../target/e2e-tests-artifacts"
mkdir -p "$ARTIFACTS_DIR"
cd "$ARTIFACTS_DIR"

if [[ "$pocket_ic_source" == *"?rev="* ]]; then
    rev=$(printf '%s' "$pocket_ic_source" | sed 's/.*?rev=\([^#&]*\).*/\1/')
    echo "Using git rev: $rev"
    echo -n "$rev" > pocket-ic-tag
    curl -sL "https://download.dfinity.systems/ic/$rev/binaries/${arch}-${os}/pocket-ic.gz" --output pocket-ic.gz
elif [[ "$pocket_ic_source" == *"?tag="* ]]; then
    tag=$(printf '%s' "$pocket_ic_source" | sed 's/.*?tag=\([^#&]*\).*/\1/')
    echo "Using git tag: $tag"
    echo -n "$tag" > pocket-ic-tag
    curl -sL "https://github.com/dfinity/ic/releases/download/$tag/pocket-ic-${arch}-${os}.gz" --output pocket-ic.gz
else
    echo "Error: unexpected pocket-ic source: $pocket_ic_source"
    exit 1
fi
gzip -df pocket-ic.gz
chmod a+x pocket-ic
./pocket-ic --version

if [[ "$os" == "darwin" ]]; then
    xattr -dr com.apple.quarantine pocket-ic
fi
