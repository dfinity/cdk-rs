#!/usr/bin/env bash

set -euo pipefail

# Make sure we always run from the root
SCRIPTS_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPTS_DIR/.."

uname_sys=$(uname -s | tr '[:upper:]' '[:lower:]')
echo "uname_sys: $uname_sys"
# Check https://gitlab.com/dfinity-lab/public/ic/-/commits/master
# Find the most recent commit with a green check mark (the artifacts were built successfully)
tag="release-2024-05-22_23-01-base"
commit_sha="$(git ls-remote https://github.com/dfinity/ic -t $tag | awk '{print $1}' )"

curl -sLO "https://download.dfinity.systems/ic/$commit_sha/binaries/x86_64-$uname_sys/ic-test-state-machine.gz"
gzip -d ic-test-state-machine.gz
chmod a+x ic-test-state-machine
./ic-test-state-machine --version
