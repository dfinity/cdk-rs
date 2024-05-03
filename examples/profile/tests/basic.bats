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

@test "profile_inter_rs build.rs can generate type bindings in specified dir" {
  dfx build --check
  run test -f src/profile_inter_rs/declarations/type/profile_inter_rs.rs
  assert_success
}
