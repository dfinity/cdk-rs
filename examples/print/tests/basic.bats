load ../../bats/bats-assert/load.bash

# Executed before each test.
setup() {
  cd examples/print
  # Make sure the directory is clean.
  dfx start --clean --background
}

# executed after each test
teardown() {
  dfx stop
}

@test "Can print" {
  dfx deploy

  run dfx canister call print print
  assert_success
}

@test "candid-extractor supports version and help" {
  run candid-extractor --version
  assert_success
  run candid-extractor -V
  assert_success
  run candid-extractor --help
  assert_success
  run candid-extractor -h
  assert_success
}
