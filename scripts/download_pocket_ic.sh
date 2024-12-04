#!/usr/bin/env bash

set -euo pipefail

# Make sure we always run from the root
SCRIPTS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPTS_DIR/../e2e-tests"

uname_sys=$(uname -s | tr '[:upper:]' '[:lower:]')
echo "uname_sys: $uname_sys"

tag="release-2024-11-28_03-15-base"

curl -sL "https://github.com/dfinity/ic/releases/download/$tag/pocket-ic-x86_64-$uname_sys.gz" --output pocket-ic.gz
gzip -df pocket-ic.gz
chmod a+x pocket-ic
./pocket-ic --version

if [[ "$uname_sys" == "darwin" ]]; then
    xattr -dr com.apple.quarantine pocket-ic
fi
