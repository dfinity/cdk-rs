# Executed before each test.
setup() {
  cd examples/counter
  # Make sure the directory is clean.
  dfx start --clean --background
}

# executed after each test
teardown() {
  dfx stop
}

@test "Can counter (counter_rs)" {
  dfx deploy
  run dfx canister call counter_rs read
  [ "$output" == '(0 : nat)' ]
  dfx canister call counter_rs inc
  run dfx canister call counter_rs read
  [ "$output" == '(1 : nat)' ]
  run dfx canister call counter_rs write '(5)'
  dfx canister call counter_rs inc
  run dfx canister call counter_rs read
  [ "$output" == '(6 : nat)' ]
}

@test "Can counter (inter_rs)" {
  dfx deploy
  run dfx canister call inter_rs read
  [ "$output" == '(0 : nat)' ]
  dfx canister call inter_rs inc
  run dfx canister call inter_rs read
  [ "$output" == '(1 : nat)' ]
  run dfx canister call inter_rs write '(5)'
  dfx canister call inter_rs inc
  run dfx canister call inter_rs read
  [ "$output" == '(6 : nat)' ]
}

@test "Can counter (inter2_rs)" {
  dfx deploy

  run dfx canister call counter_rs read
  [ "$output" == '(0 : nat)' ]

  run dfx canister call inter2_rs read
  [ "$output" == '(0 : nat)' ]
  dfx canister call inter2_rs inc
  run dfx canister call inter2_rs read
  [ "$output" == '(1 : nat)' ]
  run dfx canister call inter2_rs write '(5)'
  dfx canister call inter2_rs inc
  run dfx canister call inter2_rs read
  [ "$output" == '(6 : nat)' ]

  # Check that counter_rs has 6 too.
  run dfx canister call counter_rs read
  [ "$output" == '(6 : nat)' ]
}
