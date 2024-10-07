#!/usr/bin/env bash

set -euo pipefail

build_canister() {
    cargo build --target wasm32-unknown-unknown --release -p $1 --locked
    wasm-opt ./target/wasm32-unknown-unknown/release/$1.wasm -o ./target/wasm32-unknown-unknown/release/$1.wasm -O3 -Os
    gzip -f -1 ./target/wasm32-unknown-unknown/release/$1.wasm
}

build_canister individual_user_template
build_canister user_index
build_canister post_cache
build_canister platform_orchestrator

POCKET_IC_BIN=$PWD/pocket-ic cargo test --no-fail-fast