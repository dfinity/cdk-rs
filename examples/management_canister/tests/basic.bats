# Executed before each test.
setup() {
  cd examples/management_canister
}

# executed after each test
teardown() {
  dfx stop
}

@test "http_request example succeed" {
  dfx start --clean --background --enable-canister-http
  dfx deploy
  dfx canister call caller http_request_example
}

@test "ecdsa methods succeed" {
  dfx start --clean --background
  dfx deploy
  dfx canister call caller execute_ecdsa_methods
}

@test "bitcoin methods succeed" {
  bitcoind -regtest -daemonwait
  dfx start --clean --background --enable-bitcoin
  dfx deploy
  dfx canister call caller execute_bitcoin_methods
  bitcoin-cli -regtest stop
}
