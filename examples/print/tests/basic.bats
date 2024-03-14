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
