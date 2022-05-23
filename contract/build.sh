#!/bin/bash

RUSTFLAGS='-C link-arg=-s' cargo build --target wasm32-unknown-unknown --release
wasm-opt -Oz -o target/randomness_contract.wasm target/wasm32-unknown-unknown/release/randomness_contract.wasm
