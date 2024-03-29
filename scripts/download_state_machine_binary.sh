#!/usr/bin/env bash

set -euo pipefail

# Make sure we always run from the root
SCRIPTS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPTS_DIR/.."

uname_sys=$(uname -s | tr '[:upper:]' '[:lower:]')
echo "uname_sys: $uname_sys"
# Check https://gitlab.com/dfinity-lab/public/ic/-/commits/master
# Find the most recent commit with a green check mark (the artifacts were built successfully)
commit_sha="f3216c1d7d83a366b4af0cf24708f84819880246"

curl -sLO "https://download.dfinity.systems/ic/$commit_sha/binaries/x86_64-$uname_sys/ic-test-state-machine.gz"
gzip -d ic-test-state-machine.gz
chmod a+x ic-test-state-machine
./ic-test-state-machine --version
