#!/usr/bin/env bash
# near-guard demo: deploy to NEAR testnet and show one paid call and three refused calls.
set -u
C=zouguard-11141131.testnet
A=agent-11141131.testnet
SHOP=shop-11141131.testnet
WASM=/Users/mike/near-guard-1114/target/wasm32-unknown-unknown/release/near_guard.wasm
run() { echo; echo "\$ $*"; "$@" 2>&1; }

run npx --yes near-cli deploy "$C" "$WASM"
run npx --yes near-cli call "$C" new '{"owner":"'"$C"'"}' --accountId "$C"
run npx --yes near-cli call "$C" set_policy '{"agent":"'"$A"'","per_tx":"250000000000000000000000","daily":"500000000000000000000000","receivers":["'"$SHOP"'"]}' --accountId "$C"
echo; echo "--- take 1: a payment inside the policy must pass"
run npx --yes near-cli call "$C" spend '{"receiver":"'"$SHOP"'","amount":"100000000000000000000000"}' --accountId "$A" --depositYocto 100000000000000000000000
echo; echo "--- take 2: a payment over the per-transaction limit must fail"
run npx --yes near-cli call "$C" spend '{"receiver":"'"$SHOP"'","amount":"300000000000000000000000"}' --accountId "$A" --depositYocto 300000000000000000000000
echo; echo "--- take 3: a payment to a receiver outside the allowlist must fail"
run npx --yes near-cli call "$C" spend '{"receiver":"stranger.testnet","amount":"100000000000000000000000"}' --accountId "$A" --depositYocto 100000000000000000000000
echo; echo "--- state after the three takes"
run npx --yes near-cli view "$C" get_policy '{"agent":"'"$A"'"}'
