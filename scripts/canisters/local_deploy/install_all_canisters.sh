#!/usr/bin/env bash
set -euo pipefail

usage() {
  printf "Usage: \n[-s Skip test] \n[-h Display help] \n";
  exit 0;
}

skip_test=false

while getopts "sih" arg; do
  case $arg in
    s)
      skip_test=true
      ;;
    h)
      usage
      ;;
  esac
done

scripts/canisters/local_deploy/setup_icp_ledger.sh
scripts/canisters/local_deploy/create_sns_wasm.sh

dfx canister create --no-wallet individual_user_template
dfx canister create --no-wallet post_cache
dfx canister create --no-wallet user_index
dfx canister create --no-wallet platform_orchestrator

# HACK: dfx doesn't support specifying feature flags....
rebuild_canister() {
  cargo build --target wasm32-unknown-unknown --release --features local -p $1 --locked
  wasm-opt ./target/wasm32-unknown-unknown/release/$1.wasm -o ./target/wasm32-unknown-unknown/release/$1.wasm -Oz
  gzip -fk -1 ./target/wasm32-unknown-unknown/release/$1.wasm
  cp ./target/wasm32-unknown-unknown/release/$1.wasm.gz .dfx/local/canisters/$1/$1.wasm.gz
}

scripts/candid_generator.sh

rebuild_canister individual_user_template
rebuild_canister user_index
rebuild_canister post_cache
rebuild_canister platform_orchestrator

if [[ $skip_test != true ]]
then
  cargo test
fi

dfx canister install post_cache --argument "(record {
  known_principal_ids = opt vec {
    record {
      variant { UserIdGlobalSuperAdmin };
      principal \"$(dfx identity get-principal)\";
    };
    record {
      variant { CanisterIdPostCache };
      principal \"$(dfx canister id post_cache)\";
    };
    record {
      variant { CanisterIdUserIndex };
      principal \"$(dfx canister id user_index)\";
    };
    record {
      variant { CanisterIdSnsWasm };
      principal \"$(dfx canister id sns_wasm)\";
    };
  };
  version= \"v1.0.0\"
})"

dfx canister install user_index --argument "(record {
  known_principal_ids = opt vec {
    record {
      variant { UserIdGlobalSuperAdmin };
      principal \"$(dfx identity get-principal)\";
    };
    record {
      variant { CanisterIdPostCache };
      principal \"$(dfx canister id post_cache)\";
    };
    record {
      variant { CanisterIdUserIndex };
      principal \"$(dfx canister id user_index)\";
    };
    record {
      variant { CanisterIdSnsWasm };
      principal \"$(dfx canister id sns_wasm)\";
    };
  };
  access_control_map = opt vec {
    record {
      principal \"$(dfx identity get-principal)\";
      vec { variant { CanisterAdmin }; variant { CanisterController }; }
    };
  };
  proof_of_participation = opt record {
    chain = vec {};
  };
  version= \"v1.0.0\"
})"

scripts/canisters/local_deploy/create_pool_of_individual_canister_user_index.sh
