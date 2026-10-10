#!/usr/bin/env python3
"""W15 receipt integrity gate; fails closed on missing or misclassified proof."""
import json,pathlib
ROOT=pathlib.Path(__file__).resolve().parents[2]
E=ROOT/"evidence/upstream-rd-01/w15-authority-03"
read=lambda name:json.loads((E/name).read_text())
first,second,forge=read("qualification-first.json"),read("qualification.json"),read("forgery.json")
expected=["valid","scope_escape","ttl_exceeds","expired","revoked","replay",
          "wrong_digest","wrong_identity","wrong_operation","wrong_peer","symlink_alias"]
for receipt in (first,second):
    assert receipt["gate"]=="PASS_11_OF_11"
    assert receipt["authority_core_sha"]=="de096640d52c75734813611aca4f858fa7ff47d9"
    assert receipt["bootstrapping"]=="TEST_SUPPORT_ONLY_NOT_PRODUCTION"
    assert [r["case"] for r in receipt["cases"]]==expected
    assert all(r["pass"] and r["harness_exit"]==0 for r in receipt["cases"])
    assert receipt["cases"][0]["result"]["decision"]=="allow"
    assert receipt["cases"][0]["result"]["marker_contents"]=="COMPOSITE_ALLOWED\n"
    for r in receipt["cases"][1:]:
        out=r["result"]
        assert out["decision"]=="deny" and out["marker_exists"] is False
        assert out["peer_uid_matches"] is True and out["request_pid_matches"] is True
        assert out["exit_code"]==0 and '"NotCapable"' in out["stdout"]
assert forge["gate"]=="PASS_2_OF_2"
assert len(forge["cases"])==2 and all(x["pass"] and x["cargo_exit"]!=0 for x in forge["cases"])
print("W15_LIVE_DENO_AUTHORITY=PASS_11_OF_11_X2")
print("W15_EXTERNAL_CRATE_FORGERY=COMPILER_DENIES_2_OF_2")
print("W15_PRODUCTION_ISSUER=NOT_RUN; W15_FULL_MXC_COMPOSITION=NOT_RUN")
