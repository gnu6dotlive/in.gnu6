#!/usr/bin/env python3
"""W14 network cross-matrix using ephemeral local HTTP service."""
import hashlib,json,pathlib,re,subprocess,tempfile,threading
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
ROOT=pathlib.Path(__file__).resolve().parents[2]
B=pathlib.Path("/mnt/workbench/workspaces/gnu.in.labs/.probes/upstream-rd-01")
BIN=B/"mxc-fixture-target/debug/gnu6-mxc-deno-net"
DENO=B/"deno-v2.9.7/runtime/deno"
SCRIPT=ROOT/"tools/upstream-rd-01/composite_deno_net.ts"
assert BIN.is_file() and DENO.is_file()
class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        self.send_response(200);self.end_headers();self.wfile.write(b"GNU6_LOCAL_ONLY")
    def log_message(self,*a): pass
results=[]
with tempfile.TemporaryDirectory(prefix="gnu6-mxc-deno-only-",dir=B) as t:
    with ThreadingHTTPServer(("127.0.0.1",0),Handler) as server:
        thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
        port=str(server.server_port)
        for m,p in [("mxc-allow","deno-allow"),("mxc-deny","deno-allow"),("mxc-allow","deno-deny"),("mxc-deny","deno-deny")]:
            proc=subprocess.run([str(BIN),m,p,t,port,str(DENO),str(SCRIPT)],
                capture_output=True,text=True,timeout=15)
            found=re.findall(r'\{[^\n]*"ok"\s*:\s*(?:true|false)[^\n]*\}',proc.stdout)
            try: decision=json.loads(found[-1]) if found else None
            except ValueError: decision=None
            results.append({"mxc":m,"deno":p,"exit":proc.returncode,
                "decision":decision,"stdout":proc.stdout[:700],"stderr":proc.stderr[:700]})
        server.shutdown()
for row in results:
    d=row["decision"] or {}
    if row["mxc"]=="mxc-allow" and row["deno"]=="deno-allow":
        row["pass"]=row["exit"]==0 and d.get("ok") is True and d.get("status")==200
    elif row["deno"]=="deno-deny":
        row["pass"]=row["exit"]==0 and d.get("ok") is False and d.get("name")=="NotCapable"
    else:
        row["pass"]=row["exit"]==0 and d.get("ok") is False and d.get("name")!="NotCapable"
if all(x["pass"] for x in results):
    gate="PASS_4_OF_4"
elif (not results[0]["pass"]) and all(x["pass"] for x in results[1:]):
    gate="INCONCLUSIVE_HOST_LOOPBACK_POSITIVE_CONTROL"
else:
    gate="RED_OR_INCONCLUSIVE"
report={"gate":gate,"cases":results,"target":"ephemeral localhost HTTP, no public egress",
        "mxc_source_pin":"c6f301d53a1430c4c921a05c57af838f7392348f",
        "deno_version":"2.9.7","source_runtime_match":False,
        "limitations":["Process/Bubblewrap on one physical Linux machine",
                       "Deno broker IPC not exercised","No Judgment Plane integration",
                       "A TCP localhost fixture does not qualify universal network confinement",
                       "MXC Bubblewrap always creates a private network namespace; host-loopback is denied by design",
                       "Without a supported MXC-positive network endpoint this is NOT a pass of a bidirectional network oracle"],
        "cause_source":"https://github.com/microsoft/mxc/blob/c6f301d53a1430c4c921a05c57af838f7392348f/docs/backends/bwrap/bubblewrap-backend.md"}
dst=ROOT/"evidence/upstream-rd-01/composite-01/network-qualification.json"
dst.parent.mkdir(parents=True,exist_ok=True)
dst.write_text(json.dumps(report,indent=2,ensure_ascii=False)+"\n")
print("W14_NETWORK",gate,[(x["mxc"],x["deno"],x["decision"],x["pass"],x["stderr"][:75]) for x in results])
raise SystemExit(0 if gate=="PASS_4_OF_4" else 2)
