#!/usr/bin/env bash
# One-command reproduction: create 3 testnet accounts, deploy, run 3 takes.
# Usage: bash demo/e2e-testnet.sh [suffix]
# Needs: near-cli (npx), a built wasm.
set -u
SUF=${1:-$(date +%m%d%H%M)}
C=zouguard-$SUF.testnet
A=agent-$SUF.testnet
S=shop-$SUF.testnet
WASM=${WASM:-/Users/mike/near-guard-1114/target/wasm32-unknown-unknown/release/near_guard.wasm}
NC="npx --yes near-cli"
echo "hint: run with a new suffix for a clean state, e.g. bash demo/e2e-testnet.sh $(date +%H%M)"
echo "will use: $C owner, $A agent, $S receiver"
for x in "$C" "$A" "$S"; do
  if [ -f "$HOME/.near-credentials/testnet/$x.json" ]; then
    echo "\$ create-account $x  (exists, skip)"
  else
    echo "\$ create-account $x"
    $NC create-account "$x" --useFaucet 2>&1 | tail -1
  fi
done
echo "\$ deploy $C"
$NC deploy "$C" "$WASM" --force 2>&1 | tail -2
echo "\$ new owner=$C"
$NC call "$C" new "{\"owner\":\"$C\"}" --accountId "$C" 2>&1 | grep -E "Log|Failure" | head -1 || echo "(new: state already initialized, set_policy below resets it)"
echo "\$ set_policy agent=$A per_tx=0.25 daily=0.5 receivers=[$S]"
$NC call "$C" set_policy "{\"agent\":\"$A\",\"per_tx\":\"250000000000000000000000\",\"daily\":\"500000000000000000000000\",\"receivers\":[\"$S\"]}" --accountId "$C" 2>&1 | grep -E "Log|Failure" | head -2
echo "--- take 1: 0.1 NEAR inside the policy must pass"
$NC call "$C" spend "{\"receiver\":\"$S\",\"amount\":\"100000000000000000000000\"}" --accountId "$A" --depositYocto 100000000000000000000000 2>&1 | grep -E "Log|Failure" | head -2
echo "--- take 2: 0.3 NEAR over per_tx must fail"
$NC call "$C" spend "{\"receiver\":\"$S\",\"amount\":\"300000000000000000000000\"}" --accountId "$A" --depositYocto 300000000000000000000000 2>&1 | grep -E "Log|Failure" | head -2
echo "--- take 3: unknown receiver must fail"
$NC call "$C" spend "{\"receiver\":\"stranger.testnet\",\"amount\":\"100000000000000000000000\"}" --accountId "$A" --depositYocto 100000000000000000000000 2>&1 | grep -E "Log|Failure" | head -2
echo "--- final policy state"
$NC view "$C" get_policy "{\"agent\":\"$A\"}" 2>&1 | tail -9
