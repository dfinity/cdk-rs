# Executed before each test.
setup() {
  cd examples/profile
  # Make sure the directory is clean.
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

@test "Can get, update, search (profile_rs)" {
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
