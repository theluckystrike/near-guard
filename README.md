# near-guard

An on-chain spending policy for autonomous AI agents on NEAR.

An agent can call `spend`. The contract checks the owner's policy first. The contract pays only when every rule passes. The contract writes an `EVENT` line for...[truncated]
## 60-second check for a judge

1. Open https://github.com/theluckystrike/near-guard .
2. Run `cargo test --release`. The output shows 9 passing tests.
3. Open https://testnet.nearblocks.io/address/zouguard-11141131.testnet . The account holds the wasm.
4. Read `demo/transcript.txt`. It shows one payment that passed and two that were refused.
5. Run `bash demo/e2e-testnet.sh $(date +%H%M)` to reproduce all of this from zero.

If any step fails, the claim in this README is false. Report it.

## Live on NEAR testnet (2026-11-14)

The contract is deployed and callable now. These are real transactions, not a mock.

| Item | Value |
|---|---|
| Contract account | `zouguard-11141131.testnet` |
| Owner account | `zouguard-11141131.testnet` |
| Agent account | `agent-11141131.testnet` |
| Receiver account | `shop-11141131.testnet` |
| Policy | per-tx 0.25 NEAR, daily 0.5 NEAR, allowlist `shop-11141131.testnet` |
| Explorer | https://testnet.nearblocks.io/address/zouguard-11141131.testnet |

Run the same demo again with:

```
bash demo/testnet-demo.sh
```

The transcript is in `demo/transcript.txt`. The result of the three takes:

1. A payment of 0.1 NEAR inside the policy passed. The contract logged `EVENT spend`.
2. A payment of 0.3 NEAR over the per-transaction limit was refused.
3. A payment to an account outside the allowlist was refused.

The final `get_policy` call reports `spent_today: 1e+23`. Only take 1 changed the counter. The contract refuses take 2 and take 3 before they touch the counter.

## Limits, stated plainly

- The contract is on testnet. It is not on mainnet. A mainnet deploy needs a funded account and a key that the owner holds.
- A refusal ends in a panic, not a typed error. A calling agent sees `ExecutionError`. The refusal does not return a reason string to the caller.
- The design pays from the deposit that the agent attaches. The contract holds no float. This keeps the contract simple and auditable.
- The allowlist is a plain list. Compare cost grows with list size. Use a set for large lists.
- There is no time window per call and no per-receiver daily limit yet. Track eligibility may need these.

## Baseline: why this exists

An agent that holds a raw key can send any amount to any account. Most agent frameworks keep the limit in the prompt or in the Python process. Both live on the same machine as the agent. A prompt injection or a code bug removes the limit.

near-guard moves the limit into a contract. The agent cannot change the limit. Only the owner can. A refusal is visible on chain.

This is the difference the judges should test: change the policy text in the agent, then call `spend` over the limit. The contract still refuses.

## Track fit

- Agentic payments and commerce: the contract is the payment gate for an agent.
- Safety and guardrails: the policy is visible and the refusal is logged.
- Infrastructure: any NEAR agent can call the same contract.

