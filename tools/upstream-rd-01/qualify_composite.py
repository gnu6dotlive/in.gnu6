#!/usr/bin/env python3
"""W14: bounded *executed* MXC SDK × Deno fixture, independent policy gates."""
import hashlib, json, pathlib, re, subprocess, tempfile
ROOT=pathlib.Path(__file__).resolve().parents[2]
B=pathlib.Path("/mnt/workbench/workspaces/gnu.in.labs/.probes/upstream-rd-01")
MXC=B/"mxc-fixture-target/debug/gnu6-mxc-deno-composite"
DENO=B/"deno-v2.9.7/runtime/deno"
ARCHIVE=B/"deno-v2.9.7/deno-x86_64-unknown-linux-gnu.zip"
SCRIPT=ROOT/"tools/upstream-rd-01/composite_deno_fixture.ts"
SRC=B/"mxc-c6f301"
PIN="c6f301d53a1430c4c921a05c57af838f7392348f"
assert subprocess.check_output(["git","rev-parse","HEAD"],cwd=SRC,text=True).strip()==PIN
assert MXC.is_file() and DENO.is_file()
assert hashlib.sha256(ARCHIVE.read_bytes()).hexdigest()=="c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490"
assert subprocess.check_output([str(DENO),"--version"],text=True,timeout=5).startswith("deno 2.9.7 ")
out=[]
with tempfile.TemporaryDirectory(prefix="gnu6-mxc-deno-only-",dir=B) as t:
    d=pathlib.Path(t)
    sentinel=d/"sentinel.txt"
    sentinel.write_text("GNU6_COMPOSITE_DISPOSABLE\n")
    for mode, permission in [("rw","allow"),("ro","allow"),("rw","deny"),("ro","deny")]:
        marker=d/"marker.txt"
        marker.unlink(missing_ok=True)
        command=[str(MXC),mode,permission,t,str(DENO),str(SCRIPT)]
        p=subprocess.run(command,capture_output=True,text=True,timeout=15)
        found=re.findall(r'\{[^\n]*"ok"\s*:\s*(?:true|false)[^\n]*\}',p.stdout)
        try: decision=json.loads(found[-1]) if found else None
        except ValueError: decision=None
        out.append({"mode":mode,"permission":permission,
            "exitcode":p.returncode,"stdout":p.stdout[:800],"stderr":p.stderr[:800],
            "decision":decision,"marker":marker.read_text() if marker.exists() else None})
    sentinel_ok=sentinel.read_text()=="GNU6_COMPOSITE_DISPOSABLE\n"
def good(row):
    decision=row["decision"] or {}
    if row["exitcode"]!=0: return False
    key=(row["mode"],row["permission"])
    if key==("rw","allow"):
        return decision.get("ok") is True and row["marker"]=="COMPOSITE_ALLOWED\n"
    if key==("ro","allow"):
        return decision.get("ok") is False and decision.get("name")!="NotCapable" and row["marker"] is None
    return decision.get("ok") is False and decision.get("name")=="NotCapable" and row["marker"] is None
for entry in out: entry["pass"]=good(entry)
gate="PASS_4_OF_4" if all(x["pass"] for x in out) and sentinel_ok else "RED_OR_INCONCLUSIVE"
receipt={"gate":gate,"source_commit_mxc":PIN,"mxc_sdk":"v1.0.0",
         "deno_binary_release":"v2.9.7","deno_matches_pinned_source_commit":False,
         "cases":out,"sentinel_preserved":sentinel_ok,
         "limitations":["No Deno external permission broker IPC configured",
                        "MXC Process/Bubblewrap backend on one Linux machine only",
                        "No real Judgment Plane ExecutionGrant or token minting",
                        "No general security certification or deployment"]}
path=ROOT/"evidence/upstream-rd-01/composite-01/qualification.json"
path.parent.mkdir(parents=True,exist_ok=True)
path.write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+"\n")
print("W14",gate,[(x["mode"],x["permission"],x["decision"],x["pass"]) for x in out])
raise SystemExit(0 if gate=="PASS_4_OF_4" else 2)
