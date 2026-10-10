#!/usr/bin/env python3
"""Reconcile only W14 research receipts into the in.gnu6 batch dispatch."""
import json, pathlib, subprocess
ROOT=pathlib.Path(__file__).resolve().parents[2]
B=ROOT/"contracts/agent-batches/2026-10-10-upstream-rd-01"
read=lambda p:json.loads((ROOT/p).read_text())
manifest=read("contracts/agent-batches/2026-10-10-upstream-rd-01/sources.json")["sources"]
mx=read("evidence/upstream-rd-01/composite-01/qualification.json")
old=read("evidence/upstream-rd-01/composite-01/qualification-first.json")
net=read("evidence/upstream-rd-01/composite-01/network-qualification.json")
assert mx["source_commit_mxc"]==old["source_commit_mxc"]==manifest[0]["commit_sha"]
assert mx["deno_matches_pinned_source_commit"] is False
assert mx["deno_binary_release"]=="v2.9.7"
for receipt in (mx,old):
    assert receipt["gate"]=="PASS_4_OF_4" and receipt["sentinel_preserved"]
    assert len(receipt["cases"])==4 and all(x["pass"] for x in receipt["cases"])
assert net["gate"]=="INCONCLUSIVE_HOST_LOOPBACK_POSITIVE_CONTROL"
assert len(net["cases"])==4 and not net["cases"][0]["pass"]
assert all(x["pass"] for x in net["cases"][1:])
dispatch=read("contracts/agent-batches/2026-10-10-upstream-rd-01/dispatch.json")
assert subprocess.check_output(["git","branch","--show-current"],cwd=ROOT,text=True).strip()==dispatch["branch"]
dispatch["w14_composite"]={
    "mxc":"SDK_1_0_0_FROM_PINNED_SOURCE",
    "deno":"RELEASE_V2_9_7_SHA256_VERIFIED_SOURCE_PIN_DIFFERS",
    "filesystem":"LAB_PASS_4_OF_4_REPEATED_TWICE",
    "network":"INCONCLUSIVE_INVALID_HOST_LOOPBACK_POSITIVE_CONTROL",
    "deno_broker_ipc":"NOT_RUN",
    "judgment_plane":"NOT_RUN",
    "product_transfer":"NOT_RUN",
    "public_security_claim":"NOT_AUTHORIZED"
}
(B/"dispatch.json").write_text(json.dumps(dispatch,indent=2,ensure_ascii=False)+"\n")
print("W14_FILESYSTEM=PASS_4_OF_4_REPEATED")
print("W14_NETWORK=INCONCLUSIVE")
print("W14_DENO_BROKER_AND_JUDGMENT_PLANE=NOT_RUN")
