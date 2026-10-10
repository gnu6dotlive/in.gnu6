#!/usr/bin/env python3
"""W14 integrity/causality receipt gate. Does not touch product authority."""
import json,pathlib,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[2]
BASE=ROOT/"evidence/upstream-rd-01/composite-01"
def load(name):return json.loads((BASE/name).read_text())
first,again=load("qualification-first.json"),load("qualification.json")
for index,receipt in enumerate([first,again],1):
    assert receipt["gate"]=="PASS_4_OF_4",index
    assert receipt["source_commit_mxc"]=="c6f301d53a1430c4c921a05c57af838f7392348f"
    assert receipt["deno_binary_release"]=="v2.9.7"
    assert receipt["deno_matches_pinned_source_commit"] is False
    assert receipt["sentinel_preserved"] is True
    assert len(receipt["cases"])==4
    assert all(x["pass"] and x["exitcode"]==0 for x in receipt["cases"])
    cases={(x["mode"],x["permission"]):x for x in receipt["cases"]}
    assert cases["rw","allow"]["marker"]=="COMPOSITE_ALLOWED\n"
    assert cases["ro","allow"]["marker"] is None
    assert "os error 30" in cases["ro","allow"]["decision"]["message"]
    assert cases["rw","deny"]["decision"]["name"]=="NotCapable"
    assert cases["ro","deny"]["decision"]["name"]=="NotCapable"
    assert all(cases[k]["marker"] is None for k in [("ro","allow"),("rw","deny"),("ro","deny")])
network=load("network-qualification.json")
assert network["gate"]=="INCONCLUSIVE_HOST_LOOPBACK_POSITIVE_CONTROL"
assert len(network["cases"])==4
assert not network["cases"][0]["pass"] and all(x["pass"] for x in network["cases"][1:])
print("W14_FILESYSTEM_COMPOSITE=PASS_4_OF_4_TWICE")
print("W14_NETWORK=INCONCLUSIVE_HOST_LOOPBACK_POSITIVE_CONTROL")
