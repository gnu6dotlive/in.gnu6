#!/usr/bin/env python3
"""W14B receipt-level gates: real Deno IPC, MXC policy refusals, composite."""
import json,pathlib
ROOT=pathlib.Path(__file__).resolve().parents[2]
E=ROOT/"evidence/upstream-rd-01/broker-net-02"
load=lambda n: json.loads((E/n).read_text())
broker=load("deno-broker-live.json")
mxc=load("mxc-network-refusal.json")
joint=load("mxc-deno-broker.json")
assert broker["gate"]=="PASS_9_OF_9" and len(broker["cases"])==9
assert mxc["gate"]=="PASS_3_OF_3" and len(mxc["cases"])==3
assert joint["gate"]=="PASS_5_OF_5" and len(joint["cases"])==5
assert all(r["pass"] and r["host_file_intact"] for r in broker["cases"])
assert all(r["pass"] for r in mxc["cases"]) and all(r["pass"] for r in joint["cases"])
assert joint["source_mxc_commit"]==mxc["source_pin"]=="c6f301d53a1430c4c921a05c57af838f7392348f"
assert broker["broker_source_git_blob_v2_9_7"]==broker["broker_source_git_blob_pinned_commit"]
assert broker["exact_pinned_binary"] is False and joint["deno_pinned_full_tree_executed"] is False
j={(r["mode"],r["broker"]):r for r in joint["cases"]}
assert j["rw","allow"]["marker"]=="COMPOSITE_ALLOWED\n"
assert j["ro","allow"]["marker"] is None and "os error 30" in j["ro","allow"]["result"]["message"]
assert j["rw","deny"]["result"]["name"]=="NotCapable"
assert j["ro","deny"]["result"]["name"]=="NotCapable"
assert j["rw","wrong_id"]["exit"]==87 and j["rw","wrong_id"]["marker"] is None
assert mxc["egress_supported_here"] is False
print("W14B_BROKER=PASS_9_OF_9; MXC_NET_REFUSAL=PASS_3_OF_3; COMPOSITE_BROKER=PASS_5_OF_5")
print("MXC_POSITIVE_EGRESS=NOT_RUN; DENO_EXACT_FULL_TREE=NOT_RUN; JUDGMENT_PLANE=NOT_RUN")
