#!/usr/bin/env python3
"""Integrate W14B receipts only into the upstream research dispatch."""
import json,pathlib,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[2]
B=ROOT/"contracts/agent-batches/2026-10-10-upstream-rd-01"
get=lambda p:json.loads((ROOT/p).read_text())
E="evidence/upstream-rd-01/broker-net-02/"
broker=get(E+"deno-broker-live.json")
net=get(E+"mxc-network-refusal.json")
joint=get(E+"mxc-deno-broker.json")
assert broker["gate"]=="PASS_9_OF_9" and len(broker["cases"])==9
assert net["gate"]=="PASS_3_OF_3" and len(net["cases"])==3
assert joint["gate"]=="PASS_5_OF_5" and len(joint["cases"])==5
assert broker["broker_source_git_blob_v2_9_7"]==broker["broker_source_git_blob_pinned_commit"]
assert joint["source_mxc_commit"]==net["source_pin"]=="c6f301d53a1430c4c921a05c57af838f7392348f"
assert broker["exact_pinned_binary"] is False and joint["deno_pinned_full_tree_executed"] is False
d=json.loads((B/"dispatch.json").read_text())
assert subprocess.check_output(["git","branch","--show-current"],cwd=ROOT,text=True).strip()==d["branch"]
d["w14b_broker_net_02"]={"deno_broker_ipc":"LAB_PASS_9_OF_9",
    "mxc_network_policy_refusal":"LAB_PASS_3_OF_3_POSITIVE_SPAWN_AND_TWO_REFUSALS",
    "mxc_deno_broker_composite":"LAB_PASS_5_OF_5",
    "mxc_egress_allow":"NOT_RUN_SLIRP4NETNS_MISSING",
    "judgment_plane_grant":"NOT_RUN",
    "deno_full_pinned_commit_runtime":"NOT_RUN_BROKER_SOURCE_BLOB_MATCHES_ONLY",
    "product_mutations":"NONE","production_mutations":"NONE"}
(B/"dispatch.json").write_text(json.dumps(d,indent=2,ensure_ascii=False)+"\n")
print("DISPATCH_W14B=VALIDATED; 9+3+5 passing bounded cases; open gates preserved")
