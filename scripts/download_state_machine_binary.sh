#!/usr/bin/env bash

set -euo pipefail

# Make sure we always run from the root
SCRIPTS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPTS_DIR/.."

uname_sys=$(uname -s | tr '[:upper:]' '[:lower:]')
echo "uname_sys: $uname_sys"
commit_sha="b314222935b7d06c70036b0b54aa80a33252d79c"

curl -sLO "https://download.dfinity.systems/ic/$commit_sha/binaries/x86_64-$uname_sys/ic-test-state-machine.gz"
gzip -d ic-test-state-machine.gz
chmod a+x ic-test-state-machine
./ic-test-state-machine --version
