load ../../bats/bats-support/load.bash
load ../../bats/bats-assert/load.bash

# Executed before each test.
setup() {
  cd examples/management_canister
}

# executed after each test
teardown() {
  dfx stop
  bitcoin-cli -regtest stop
}

@test "http_request example succeed" {
  dfx start --clean --background --enable-canister-http
  dfx deploy
  run dfx canister call caller http_request_example
  assert_success
}

@test "ecdsa methods succeed" {
  dfx start --clean --background
  dfx deploy
  run dfx canister call caller execute_ecdsa_methods
  assert_success
}

@test "bitcoin methods succeed" {
  bitcoind -regtest -daemonwait

  wget https://github.com/dfinity/bitcoin-canister/releases/download/release%2F2023-10-13/ic-btc-canister.wasm.gz
  DFX_BITCOIN_WASM=ic-btc-canister.wasm.gz dfx start --clean --background --enable-bitcoin

  dfx deploy
  run dfx canister call caller execute_bitcoin_methods
  assert_success
}
