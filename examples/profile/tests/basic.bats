load ../../bats/bats-support/load.bash
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

  run dfx canister call profile_rs getSelf
  [ "$output" == '(record { name = ""; description = ""; keywords = vec {} })' ]
  dfx canister call profile_rs update 'record {"name"= "abc"; "description"="123"; "keywords"= vec {} }'
  run dfx canister call profile_rs get abc
  [ "$output" == '(record { name = "abc"; description = "123"; keywords = vec {} })' ]
  run dfx canister call profile_rs search ab
  [ "$output" == '(opt record { name = "abc"; description = "123"; keywords = vec {} })' ]

  run dfx canister call profile_inter_rs getSelf
  [ "$output" == '(record { name = ""; description = ""; keywords = vec {} })' ]
  dfx canister call profile_inter_rs update 'record {"name"= "def"; "description"="456"; "keywords"= vec {} }'
  run dfx canister call profile_inter_rs get def
  [ "$output" == '(record { name = "def"; description = "456"; keywords = vec {} })' ]
  run dfx canister call profile_inter_rs search de
  [ "$output" == '(opt record { name = "def"; description = "456"; keywords = vec {} })' ]
}

@test "ic-cdk-bindgen warns about deprecated env vars when running with dfx v0.13.0" {
  dfxvm install 0.13.0
  run dfx +0.13.0 build --check
  assert_success
  assert_regex "$output" "The environment variable CANISTER_CANDID_PATH_profile_rs is deprecated. Please set CANISTER_CANDID_PATH_PROFILE_RS instead. Upgrading dfx may fix this issue."
  assert_regex "$output" "The environment variable CANISTER_ID_profile_rs is deprecated. Please set CANISTER_ID_PROFILE_RS instead. Upgrading dfx may fix this issue."
}