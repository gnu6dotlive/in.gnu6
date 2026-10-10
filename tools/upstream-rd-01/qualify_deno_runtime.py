#!/usr/bin/env python3
"""Bounded real-Deno 2.9.7 file/network permissions regression fixture."""
import hashlib, json, os, pathlib, subprocess, tempfile, threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

ROOT=pathlib.Path(__file__).resolve().parents[2]
B=pathlib.Path("/mnt/workbench/workspaces/gnu.in.labs/.probes/upstream-rd-01/deno-v2.9.7")
EXE=B/"runtime/deno"
ARCHIVE=B/"deno-x86_64-unknown-linux-gnu.zip"
EXPECTED_SHA="c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490"
assert hashlib.sha256(ARCHIVE.read_bytes()).hexdigest()==EXPECTED_SHA
version=subprocess.check_output([str(EXE),"--version"],text=True,timeout=10).splitlines()[0]
assert version.startswith("deno 2.9.7 "), version
SCRIPT=ROOT/"tools/upstream-rd-01/deno_permission_fixture.ts"
ENV=os.environ.copy()
ENV.update(DENO_DIR=str(B/"cache"),DENO_NO_UPDATE_CHECK="1",NO_COLOR="1",DENO_NO_PROMPT="1")
class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200);self.end_headers();self.wfile.write(b"GNU6 ephemeral local")
    def log_message(self,*args): pass
def probe(name,grants,operation,target,expected):
    cmd=[str(EXE),"run","--no-config","--no-prompt",*grants,str(SCRIPT),operation,target]
    p=subprocess.run(cmd,capture_output=True,text=True,timeout=14,env=ENV)
    try: observed=json.loads(p.stdout.strip().splitlines()[-1])
    except (ValueError,IndexError): observed=None
    assert observed is not None, f"Missing JSON in {name}: {p.stdout[:160]} {p.stderr[:160]}"
    ok=(p.returncode==0 and observed["ok"] is expected)
    if expected and operation=="read": ok=ok and observed.get("matches_fixture") is True
    if expected and operation=="net": ok=ok and observed.get("status")==200
    return {"case":name,"status":"PASS" if ok else "RED","expected_allow":expected,
            "observed":observed,"exit_code":p.returncode,"stderr":p.stderr[:200],
            "argv_display":["deno","run","--no-config","--no-prompt",*grants,"<fixture>",operation,"<temp_target>"]}
results=[]
with tempfile.TemporaryDirectory(prefix="gnu6-deno-only-",dir=B) as tmp:
    work=pathlib.Path(tmp)
    allowed=work/"synthetic.txt"
    decoy=work/"decoy.txt"
    allowed.write_text("GNU6-DENO-TEMP-FIXTURE\n")
    decoy.write_text("DECOY\n")
    results.append(probe("read_default_deny",[],"read",str(allowed),False))
    results.append(probe("read_exact_allow",["--allow-read="+str(allowed)],"read",str(allowed),True))
    results.append(probe("read_other_path_allow_only",["--allow-read="+str(decoy)],"read",str(allowed),False))
    results.append(probe("read_explicit_deny_wins",["--allow-read="+str(work),"--deny-read="+str(allowed)],"read",str(allowed),False))
    with ThreadingHTTPServer(("127.0.0.1",0),Handler) as server:
        srv=threading.Thread(target=server.serve_forever,daemon=True)
        srv.start()
        url=f"http://127.0.0.1:{server.server_port}/"
        results.append(probe("network_default_deny",[],"net",url,False))
        results.append(probe("network_exact_allow",["--allow-net=127.0.0.1:"+str(server.server_port)],"net",url,True))
        server.shutdown()
    content_preserved=allowed.read_text()=="GNU6-DENO-TEMP-FIXTURE\n"
gate=all(r["status"]=="PASS" for r in results) and content_preserved
report={"gate":"PASS_6_OF_6" if gate else "RED","runtime":version,
        "release_tag":"v2.9.7","archive_sha256":EXPECTED_SHA,
        "pinned_source_commit":"a18ce33715e30cd2b0d99c7e322ef11a65490e4d",
        "source_runtime_match":False,"fixtures":results,"synthetic_file_intact":content_preserved,
        "limitations":["release v2.9.7 precedes pinned source commit of 2026-10-06",
                       "no Deno permission broker IPC process configured",
                       "Deno application permission gate, not OS sandbox enforcement",
                       "host loopback only, no external network"]}
out=ROOT/"evidence/upstream-rd-01/deno-runtime-02/qualification.json"
out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(report,indent=2,ensure_ascii=False)+"\n")
print("DENO_RUNTIME_GATE=",report["gate"],"VERSION=",version,
      "CASES=",[(r["case"],r["status"],r["observed"].get("name")) for r in results])
raise SystemExit(0 if gate else 2)
