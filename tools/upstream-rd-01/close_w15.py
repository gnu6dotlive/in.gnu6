#!/usr/bin/env python3
"""Validate W15 sealed grant and Deno broker before dispatch promotion."""
import json,pathlib,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[2]
B=ROOT/"contracts/agent-batches/2026-10-10-upstream-rd-01"
E=ROOT/"evidence/upstream-rd-01/w15-authority-03"
read=lambda p:json.loads(p.read_text())
first,last,forge=(read(E/name) for name in ("qualification-first.json","qualification.json","forgery.json"))
for obj in (first,last):
    assert obj["gate"]=="PASS_11_OF_11" and len(obj["cases"])==11
    assert all(row["pass"] for row in obj["cases"])
    assert obj["bootstrapping"]=="TEST_SUPPORT_ONLY_NOT_PRODUCTION"
assert forge["gate"]=="PASS_2_OF_2" and len(forge["cases"])==2
dispatch=read(B/"dispatch.json")
assert subprocess.check_output(["git","branch","--show-current"],cwd=ROOT,text=True).strip()==dispatch["branch"]
dispatch["w15_authority_03"]={
    "source_core":"de096640d52c75734813611aca4f858fa7ff47d9",
    "source_judgment_plane":"745e962bb3628d52f0c98386ace232a35003db69",
    "local_live_deno_broker":"LAB_PASS_11_OF_11_REPEATED_TWICE",
    "external_forgery_compiler":"DENIED_2_OF_2",
    "execution_grant_issuer":"TEST_SUPPORT_ONLY",
    "mxc_composition":"NOT_RUN",
    "production_issuer":"NOT_RUN",
    "hardened_inode_binding":"NOT_RUN",
    "product_mutations":"NONE","production_mutations":"NONE"
}
(B/"dispatch.json").write_text(json.dumps(dispatch,indent=2,ensure_ascii=False)+"\n")
print("DISPATCH_W15=VALIDATED; 22 execution cases and 2 external compile-fails")
