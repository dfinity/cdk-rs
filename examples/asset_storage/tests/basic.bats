# Executed before each test.
setup() {
  cd examples/asset_storage
  # Make sure the directory is clean.
  dfx start --clean --background --host "127.0.0.1:0"
  local webserver_port=$(cat .dfx/webserver-port)
  cp dfx.json dfx.json.bk
  cat <<<$(jq .networks.local.bind=\"127.0.0.1:${webserver_port}\" dfx.json) >dfx.json

  run dfx identity new alice
  run dfx identity new bob
  run dfx identity new charlie
}

# executed after each test
teardown() {
  dfx stop
  mv dfx.json.bk dfx.json
}

@test "Can store and restore assets" {
  dfx deploy --no-wallet
  dfx canister --no-wallet call asset_storage store '("asset_name", vec { 1; 2; 3; })'
  dfx canister --no-wallet call asset_storage retrieve '("asset_name")'
  run dfx canister --no-wallet call asset_storage retrieve '("unknown")'
  # As of dfx 0.8.1, above command results in following error message:
  # > The Replica returned an error: code 5, message: "IC0502: Canister rwlgt-iiaaa-aaaaa-aaaaa-cai trapped: unreachable"
  [ "$status" != 0 ]
}

@test "Will fails on invalid identities" {
  dfx identity use alice
  dfx deploy --no-wallet
  dfx canister --no-wallet call asset_storage store '("asset_name", vec { 1; 2; 3; })'
  dfx canister --no-wallet call asset_storage retrieve '("asset_name")'

  dfx canister --no-wallet call asset_storage add_user "(principal \"$(dfx --identity charlie identity get-principal)\")"

  dfx identity use bob
  dfx canister --no-wallet call asset_storage retrieve '("asset_name")'

  # Test that an unknown asset fails.
  run dfx canister --no-wallet call asset_storage retrieve '("unknown")'
  [ "$status" != 0 ]

  # Test that cannot upload assets as bob.
  run dfx canister --no-wallet call asset_storage store '("asset_name", vec { 1; })'
  [ "$status" != 0 ]

  # Test we can upload assets as charlie.
  dfx identity use charlie
  run dfx canister --no-wallet call asset_storage store '("asset_name_2", vec { 1; 2; 3; })'
  [ "$status" == 0 ]
}

@test "Can upgrade and keep ACLs" {
  dfx identity use alice
  dfx deploy --no-wallet

  dfx canister --no-wallet call asset_storage store '("asset_name", vec { 1; 2; 3; })'
  dfx identity use bob
  run dfx canister --no-wallet call asset_storage retrieve '("unknown")'
  [ "$status" != 0 ]

  dfx identity use alice
  dfx canister --no-wallet call asset_storage add_user "(principal \"$(dfx --identity charlie identity get-principal)\")"

  dfx build
  dfx canister --no-wallet install --all --mode=upgrade

  dfx identity use charlie
  run dfx canister --no-wallet call asset_storage store '("asset_name_2", vec { 1; 2; 3; })'
  [ "$status" == 0 ]
}
