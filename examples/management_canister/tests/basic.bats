# Executed before each test.
setup() {
  cd examples/management_canister
  bitcoind -regtest -daemonwait
  # Make sure the directory is clean.
  dfx start --clean --background
}

# executed after each test
teardown() {
  dfx stop
  bitcoin-cli -regtest stop
}

@test "All management canister methods succeed" {
  dfx deploy
  run dfx canister call caller execute_main_methods
  run dfx canister call caller execute_provisional_methods
  run dfx canister call caller http_request_example
  run dfx canister call caller execute_ecdsa_methods
  run dfx canister call caller execute_bitcoin_methods
}
