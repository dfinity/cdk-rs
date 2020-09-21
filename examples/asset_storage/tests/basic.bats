# Executed before each test.
setup() {
  cd examples/asset_storage
  # Make sure the directory is clean.
  dfx start --clean --background

  run dfx identity new alice
  run dfx identity new bob
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

  dfx identity use bob
  dfx canister call asset_storage retrieve '("asset_name")'

  # Test that an unknown asset fails.
  run dfx canister call asset_storage retrieve '("unknown")'
  [ "$status" != 0 ]

  # Test that cannot upload assets as bob.
  run dfx canister call asset_storage store '("asset_name", vec { 1; })'
  [ "$status" != 0 ]

  run dfx canister call asset_storage store '("asset_name_2", vec { 1; 2; 3; })'
  [ "$status" != 0 ]
}
