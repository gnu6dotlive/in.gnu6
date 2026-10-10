#!/usr/bin/env python3
"""Lab-only sealed ExecutionGrant authority-gated real Deno Unix broker."""
import hashlib,json,pathlib,subprocess,tempfile
ROOT=pathlib.Path(__file__).resolve().parents[2]
B=pathlib.Path("/mnt/workbench/workspaces/gnu.in.labs/.probes/upstream-rd-01")
BIN=B/"w15-target/debug/gnu6-w15-grant-broker"
DENO=B/"deno-v2.9.7/runtime/deno"
SCRIPT=ROOT/"tools/upstream-rd-01/composite_deno_fixture.ts"
CORE=pathlib.Path("/mnt/workbench/workspaces/gnu.in.labs/gnosix-authority-core")
GJP=pathlib.Path("/home/tension_atoi/Projects/system-one-rust")
assert subprocess.check_output(["git","rev-parse","HEAD"],cwd=CORE,text=True).strip()=="de096640d52c75734813611aca4f858fa7ff47d9"
assert subprocess.check_output(["git","rev-parse","HEAD"],cwd=GJP,text=True).strip()=="745e962bb3628d52f0c98386ace232a35003db69"
assert not subprocess.check_output(["git","status","--porcelain"],cwd=CORE,text=True).strip()
assert not subprocess.check_output(["git","status","--porcelain"],cwd=GJP,text=True).strip()
assert BIN.is_file() and DENO.is_file()
cases=["valid","scope_escape","ttl_exceeds","expired","revoked","replay",
       "wrong_digest","wrong_identity","wrong_operation","wrong_peer","symlink_alias"]
rows=[]
for case in cases:
    with tempfile.TemporaryDirectory(prefix="gnu6-w15-only-",dir=B) as td:
        proc=subprocess.run([str(BIN),case,td,str(DENO),str(SCRIPT)],
                            capture_output=True,text=True,timeout=17)
        try: result=json.loads(proc.stdout.strip())
        except ValueError: result=None
        rows.append({"case":case,"harness_exit":proc.returncode,"result":result,
                     "harness_stderr":proc.stderr[:240]})
for row in rows:
    result=row["result"] or {}
    case=row["case"]
    baseline=(row["harness_exit"]==0 and result.get("request_seen") is True
              and result.get("peer_uid_matches") is True
              and result.get("request_pid_matches") is True
              and result.get("exit_code")==0)
    if case=="valid":
        good=(baseline and result.get("peer_pid_matches") is True
              and result.get("decision")=="allow" and result.get("marker_exists") is True
              and result.get("marker_contents")=="COMPOSITE_ALLOWED\n"
              and '"ok":true' in result.get("stdout",""))
    else:
        good=(baseline and result.get("decision")=="deny" and result.get("marker_exists") is False
              and '"NotCapable"' in result.get("stdout",""))
        if case=="wrong_peer":good=good and result.get("peer_pid_matches") is False
        else:good=good and result.get("peer_pid_matches") is True
        if case in ("scope_escape","ttl_exceeds"):
            good=good and result.get("grant_issued") is False
        else: good=good and result.get("grant_issued") is True
        if case=="symlink_alias":good=good and result.get("scope_valid") is False
    row["pass"]=good
gate="PASS_11_OF_11" if all(x["pass"] for x in rows) else "RED_OR_INCONCLUSIVE"
receipt={"gate":gate,"authority_core_sha":"de096640d52c75734813611aca4f858fa7ff47d9",
         "judgment_plane_sha_read_only":"745e962bb3628d52f0c98386ace232a35003db69",
         "deno_runtime":"2.9.7","bootstrapping":"TEST_SUPPORT_ONLY_NOT_PRODUCTION",
         "live_integration":"Deno Unix permission broker -> Rust policy adapter -> real sealed ExecutionGrant",
         "cases":rows,"limitations":["Synthetic authority minted only via feature-gated test-support",
                                     "No production bootstrap or issuer authentication",
                                     "Peer UID+PID tested locally, not a signed principal",
                                     "No MXC sandbox in this W15 adapter test",
                                     "Path scope is exact fixture path, not a hardened openat2 race-free resolver",
                                     "No production authority, no public security certification"]}
dst=ROOT/"evidence/upstream-rd-01/w15-authority-03/qualification.json"
dst.parent.mkdir(parents=True,exist_ok=True)
dst.write_text(json.dumps(receipt,indent=2,ensure_ascii=False)+"\n")
print("W15_GATE",gate,[(x["case"],x["pass"],(x["result"] or {}).get("decision"),
                          (x["result"] or {}).get("exit_code")) for x in rows])
raise SystemExit(0 if gate=="PASS_11_OF_11" else 2)
