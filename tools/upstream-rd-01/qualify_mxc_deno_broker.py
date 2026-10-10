#!/usr/bin/env python3
"""W14B: host Unix broker authorizes Deno inside actual MXC read-only/RW policy."""
import hashlib,json,pathlib,socket,subprocess,tempfile,threading
ROOT=pathlib.Path(__file__).resolve().parents[2]
B=pathlib.Path("/mnt/workbench/workspaces/gnu.in.labs/.probes/upstream-rd-01")
BIN=B/"mxc-fixture-target/debug/gnu6-mxc-broker-composite"
DENO=B/"deno-v2.9.7/runtime/deno"
SCRIPT=ROOT/"tools/upstream-rd-01/composite_deno_fixture.ts"
assert BIN.is_file() and DENO.is_file()
rows=[]
for mode,answer in (("rw","allow"),("ro","allow"),("rw","deny"),("ro","deny"),("rw","wrong_id")):
    with tempfile.TemporaryDirectory(prefix="gnu6-mxc-deno-only-",dir=B) as td:
        d=pathlib.Path(td);sockpath=d/"broker.sock";requests=[];server_errors=[]
        with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as server:
            server.bind(str(sockpath));server.listen(1);server.settimeout(7)
            def handle():
                try:
                    with server.accept()[0] as client:
                        client.settimeout(7);incoming=b""
                        while not incoming.endswith(b"\n") and len(incoming)<8192:
                            block=client.recv(512)
                            if not block:break
                            incoming+=block
                        if incoming:
                            q=json.loads(incoming);requests.append(q)
                            resp={"id":q["id"],"result":answer}
                            if answer=="wrong_id": resp={"id":q["id"]+1,"result":"allow"}
                            client.sendall((json.dumps(resp)+"\n").encode())
                except Exception as exc: server_errors.append(str(exc)[:180])
            thread=threading.Thread(target=handle,daemon=True);thread.start()
            proc=subprocess.run([str(BIN),mode,td,str(DENO),str(SCRIPT),str(sockpath)],
                                text=True,capture_output=True,timeout=14)
            thread.join(timeout=2)
        marker=d/"marker.txt";exists=marker.exists()
        stdout=proc.stdout.strip().splitlines()
        try: result=json.loads(next(x for x in stdout if x.startswith('{"ok":')))
        except (ValueError,StopIteration): result=None
        rows.append({"mode":mode,"broker":answer,"exit":proc.returncode,
                     "stdout":proc.stdout[:550],"stderr":proc.stderr[:650],
                     "result":result,"marker":marker.read_text() if exists else None,
                     "broker_requests":requests,"server_errors":server_errors})
for r in rows:
    value=r["result"] or {}; key=(r["mode"],r["broker"])
    if key==("rw","allow"):
        passed=r["exit"]==0 and value.get("ok") is True and r["marker"]=="COMPOSITE_ALLOWED\n"
    elif key==("ro","allow"):
        passed=r["exit"]==0 and value.get("ok") is False and "os error 30" in value.get("message","") and r["marker"] is None
    elif key[1]=="deny":
        passed=r["exit"]==0 and value.get("name")=="NotCapable" and r["marker"] is None
    else:
        passed=r["exit"]==87 and not r["marker"]
    r["pass"]=passed and len(r["broker_requests"])==1 and not r["server_errors"]
    if r["broker_requests"]:
        q=r["broker_requests"][0]
        r["pass"]=r["pass"] and q["permission"]=="write" and q["value"].endswith("/marker.txt")
gate="PASS_5_OF_5" if len(rows)==5 and all(x["pass"] for x in rows) else "RED_OR_INCONCLUSIVE"
receipt={"gate":gate,"source_mxc_commit":"c6f301d53a1430c4c921a05c57af838f7392348f",
         "deno_binary_release":"2.9.7","deno_pinned_full_tree_executed":False,
         "broker_source_blob_same_in_v2_9_7_and_pinned":True,
         "cases":rows,"limitations":["Experimental Unix broker used; no Judgment Plane issuer validation",
                                  "Same-host filesystem Unix socket, no multi-tenant identity",
                                  "Only one Linux platform and MXC Process/Bubblewrap",
                                  "Not a network positive egress qualification"]}
out=ROOT/"evidence/upstream-rd-01/broker-net-02"
out.mkdir(parents=True,exist_ok=True)
(out/"mxc-deno-broker.json").write_text(json.dumps(receipt,indent=2,ensure_ascii=False)+"\n")
print("MXC_DENO_BROKER_COMPOSITE",gate,[(r["mode"],r["broker"],r["exit"],r["result"],r["pass"],r["stderr"][:80]) for r in rows])
raise SystemExit(0 if gate=="PASS_5_OF_5" else 2)
