#!/usr/bin/env python3
"""A small agent that asks near-guard before it pays.
The decision comes from the can_spend view (unsigned RPC). The payment is made
with near-cli, so this script needs no key handling of its own.
Run: python3 demo/agent_demo.py
"""
import base64, json, subprocess, urllib.request

RPC = "https://rpc.testnet.near.org"
CONTRACT = "zouguard-11141131.testnet"
AGENT = "agent-11141131.testnet"
SHOP = "shop-11141131.testnet"


def rpc(method, params):
    req = urllib.request.Request(RPC, data=json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}).encode(),
                                 headers={"content-type": "application/json"})
    return json.load(urllib.request.urlopen(req, timeout=25))


def can_spend(receiver, amount):
    r = rpc("query", {"request_type": "call_function", "finality": "final",
                      "account_id": CONTRACT, "method_name": "can_spend",
                      "args_base64": base64.b64encode(json.dumps({"agent": AGENT, "receiver": receiver, "amount": str(amount)}).encode()).decode()})
    return json.loads(bytes(r["result"]["result"]).decode())


def pay(receiver, amount):
    out = subprocess.run(["npx", "--yes", "near-cli", "call", CONTRACT, "spend",
                          json.dumps({"receiver": receiver, "amount": str(amount)}),
                          "--accountId", AGENT, "--depositYocto", str(amount)],
                         capture_output=True, text=True, timeout=180).stdout
    return "EVENT_JSON" in out, out[-300:]


plan = [(SHOP, 10 ** 23), ("stranger.testnet", 10 ** 23), (SHOP, 3 * 10 ** 24), (SHOP, 2 * 10 ** 23)]
for receiver, amount in plan:
    ans = can_spend(receiver, amount)
    print(f"ask  can_spend({receiver}, {amount/1e24:.2f} NEAR) -> ok={ans['ok']}  reason={ans['reason']}")
    if not ans["ok"]:
        print("     agent stops. no money moves.\n")
        continue
    ok, tail = pay(receiver, amount)
    print("     agent pays ->", "EVENT_JSON spend logged" if ok else tail.strip().splitlines()[-1][:100], "\n")
