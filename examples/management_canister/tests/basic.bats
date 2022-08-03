# Executed before each test.
setup() {
  cd examples/management_canister
  # Make sure the directory is clean.
  dfx start --clean --background --host "127.0.0.1:0"
  local webserver_port=$(cat .dfx/webserver-port)
  cp dfx.json dfx.json.bk
  cat <<<$(jq .networks.local.bind=\"127.0.0.1:${webserver_port}\" dfx.json) >dfx.json
}

# executed after each test
teardown() {
  dfx stop
  mv dfx.json.bk dfx.json
}

@test "All management canister methods succeed" {
  dfx deploy
  run dfx canister call caller execute_main_methods
  run dfx canister call caller execute_provisional_methods
  run dfx canister call caller http_request_example
  run dfx canister call caller execute_threshold_ecdsa_methods
  run dfx canister call caller execute_bitcoin_methods
}
