# Executed before each test.
setup() {
  cd examples/chess
  # Make sure the directory is clean.
  npm install

  dfx start --clean --background --host "127.0.0.1:0"
  local webserver_port=$(cat .dfx/webserver-port)
  cp dfx.json dfx.json.bk
  cat <<<$(jq .networks.local.bind=\"127.0.0.1:${webserver_port}\" dfx.json) >dfx.json
}

# executed after each test
teardown() {
  dfx stop
  mv dfx.json.bk dfx.json
}

@test "Can play chess against AI" {
  dfx deploy --no-wallet
  run dfx canister --no-wallet call chess_rs new '("test", true)'
  [ "$output" == "()" ]
  run dfx canister --no-wallet call chess_rs move '("test", "e2e4")'
  [ "$output" == "(true)" ]
  run dfx canister --no-wallet call chess_rs move '("test", "d2d3")'
  [ "$output" == "(true)" ]
  run dfx canister --no-wallet call chess_rs getFen '("test")'
  [ "$output" == '(opt "rnb1kbnr/pp1ppppp/1qp5/8/4P3/3P4/PPP2PPP/RNBQKBNR w KQkq - 1 3")' ]
}
