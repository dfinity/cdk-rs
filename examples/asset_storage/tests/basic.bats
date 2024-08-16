load ../../bats/bats-assert/load.bash

# Executed before each test.
setup() {
  cd examples/asset_storage
  # Make sure the directory is clean.
  dfx start --clean --background

  x=$(mktemp -d -t cdk-XXXXXXXX)
  export DFX_CONFIG_ROOT="$x"
  dfx identity new alice --storage-mode=plaintext
  dfx identity new bob --storage-mode=plaintext
  dfx identity new charlie --storage-mode=plaintext
}

# executed after each test
teardown() {
  dfx stop
  rm -rf "$DFX_CONFIG_ROOT"
}

@test "Can store and restore assets" {
  dfx --identity alice deploy
  run dfx --identity alice canister call asset_storage store '("asset_name", vec { 1; 2; 3; })'
  assert_success
  run dfx --identity alice canister call asset_storage retrieve '("asset_name")'
  assert_success

  # Test that an unknown asset fails.
  run dfx --identity alice canister call asset_storage retrieve '("unknown")'
  assert_failure
}

@test "Unauthorized identity cannot store" {
  dfx --identity alice deploy
  dfx --identity alice canister call asset_storage store '("asset_name", vec { 1; 2; 3; })'
  dfx --identity alice canister call asset_storage retrieve '("asset_name")'

  # add charlie
  run dfx --identity alice canister call asset_storage add_user "(principal \"$(dfx --identity charlie identity get-principal)\")"
  assert_success

  # bob cannot upload assets
  run dfx --identity bob canister call asset_storage store '("asset_name", vec { 1; })'
  assert_failure

  # charlie can upload assets
  run dfx --identity charlie canister call asset_storage store '("asset_name_2", vec { 1; 2; 3; })'
  assert_success
}

@test "Can upgrade and keep the access control list" {
  dfx --identity alice deploy
  dfx --identity alice canister call asset_storage store '("asset_name", vec { 1; 2; 3; })'
  dfx --identity alice canister call asset_storage add_user "(principal \"$(dfx --identity charlie identity get-principal)\")"

  dfx canister install --all --mode=upgrade

  # bob still cannot upload assets
  run dfx --identity bob canister call asset_storage store '("asset_name", vec { 1; })'
  assert_failure

  # charlie still can upload assets
  run dfx --identity charlie canister call asset_storage store '("asset_name_2", vec { 1; 2; 3; })'
  assert_success
}
