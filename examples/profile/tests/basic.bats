load ../../bats/bats-assert/load.bash

# Executed before each test.
setup() {
  cd examples/profile
}

# executed after each test
teardown() {
  dfx stop
}

@test "Can get, update, search (profile_rs)" {
  dfx start --clean --background
  dfx deploy

  run dfx canister call profile-rs getSelf
  assert_output '(record { name = ""; description = ""; keywords = vec {} })'

  run dfx canister call profile-rs update 'record {"name"= "abc"; "description"="123"; "keywords"= vec {} }'
  assert_success

  run dfx canister call profile-rs get abc
  assert_output '(record { name = "abc"; description = "123"; keywords = vec {} })'

  run dfx canister call profile-rs search ab
  assert_output '(opt record { name = "abc"; description = "123"; keywords = vec {} })'

  run dfx canister call profile-inter-rs getSelf
  assert_output '(record { name = ""; description = ""; keywords = vec {} })'

  run dfx canister call profile-inter-rs update 'record {"name"= "def"; "description"="456"; "keywords"= vec {} }'
  assert_success

  run dfx canister call profile-inter-rs get def
  assert_output '(record { name = "def"; description = "456"; keywords = vec {} })'

  run dfx canister call profile-inter-rs search de
  assert_output '(opt record { name = "def"; description = "456"; keywords = vec {} })'
}

@test "ic-cdk-bindgen warns about deprecated env vars when running with dfx v0.13.1" {
  dfxvm install 0.13.1
  run dfx +0.13.1 build --check
  assert_success
  assert_regex "$output" "The environment variable CANISTER_CANDID_PATH_profile_rs is deprecated. Please set CANISTER_CANDID_PATH_PROFILE_RS instead. Upgrading dfx may fix this issue."
  assert_regex "$output" "The environment variable CANISTER_ID_profile_rs is deprecated. Please set CANISTER_ID_PROFILE_RS instead. Upgrading dfx may fix this issue."
}