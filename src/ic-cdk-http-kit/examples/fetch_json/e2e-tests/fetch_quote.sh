#!/usr/bin/env bash
#
# A test that verifies that the `fetch_quote` endpoint works as expected.

# Run dfx stop if we run into errors.
trap "dfx stop" EXIT SIGINT

dfx start --background --clean

# Deploy the watchdog canister.
dfx deploy --no-wallet fetch_json

# Request config.
result=$(dfx canister call fetch_json fetch_quote)
echo "Result: $result"

# Check that the config is correct, eg. by checking it has min_explores field.
if ! [[ $result == *"Kevin Kruse"* ]]; then
  echo "FAIL"
  exit 1
fi

echo "SUCCESS"
exit 0
