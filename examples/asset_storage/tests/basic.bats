# Executed before each test.
setup() {
  cd examples/asset_storage
  # Make sure the directory is clean.
  dfx start --clean --background --host "127.0.0.1:0"
  local webserver_port=$(cat .dfx/webserver-port)
  cat <<<$(jq .networks.local.bind=\"127.0.0.1:${webserver_port}\" dfx.json) >dfx.json

  run dfx identity new alice
  run dfx identity new bob
  run dfx identity new charlie
}

# executed after each test
teardown() {
  dfx stop
}

@test "Can store and restore assets" {
  dfx deploy
  dfx canister call asset_storage store '("asset_name", vec { 1; 2; 3; })'
  dfx canister call asset_storage retrieve '("asset_name")'
  run dfx canister call asset_storage retrieve '("unknown")'
  [ "$status" != 0 ]
}

@test "Will fails on invalid identities" {
  dfx identity use alice
  dfx deploy
  dfx canister call asset_storage store '("asset_name", vec { 1; 2; 3; })'
  dfx canister call asset_storage retrieve '("asset_name")'

  dfx canister call asset_storage add_user "(principal \"$(dfx --identity charlie identity get-principal)\")"

  dfx identity use bob
  dfx canister call asset_storage retrieve '("asset_name")'

  # Test that an unknown asset fails.
  run dfx canister call asset_storage retrieve '("unknown")'
  [ "$status" != 0 ]

  # Test that cannot upload assets as bob.
  run dfx canister call asset_storage store '("asset_name", vec { 1; })'
  [ "$status" != 0 ]

  # Test we can upload assets as charlie.
  dfx identity use charlie
  run dfx canister call asset_storage store '("asset_name_2", vec { 1; 2; 3; })'
  [ "$status" == 0 ]
}

@test "Can upgrade and keep ACLs" {
  dfx identity use alice
  dfx deploy

  dfx canister call asset_storage store '("asset_name", vec { 1; 2; 3; })'
  dfx identity use bob
  run dfx canister call asset_storage retrieve '("unknown")'
  [ "$status" != 0 ]

  dfx identity use alice
  dfx canister call asset_storage add_user "(principal \"$(dfx --identity charlie identity get-principal)\")"

  dfx build
  dfx canister install --all --mode=upgrade

  dfx identity use charlie
  run dfx canister call asset_storage store '("asset_name_2", vec { 1; 2; 3; })'
  [ "$status" == 0 ]
}
