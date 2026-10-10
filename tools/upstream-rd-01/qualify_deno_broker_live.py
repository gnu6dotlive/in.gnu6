#!/usr/bin/env python3
"""Deno v2.9.7 real Unix socket broker: allow/deny/bad protocol/fail closed."""
import hashlib,json,os,pathlib,subprocess,tempfile
from probe_deno_broker import B,EXE,SCRIPT,ROOT,trial
DENO_ZIP=B/"deno-v2.9.7/deno-x86_64-unknown-linux-gnu.zip"
DENO_SHA="c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490"
assert hashlib.sha256(DENO_ZIP.read_bytes()).hexdigest()==DENO_SHA
assert subprocess.check_output([str(EXE),"--version"],text=True).startswith("deno 2.9.7 ")
rows=[
    trial("broker_allow_over_cli_default_deny","allow",False),
    trial("broker_deny_over_cli_allow","deny",True),
    trial("broker_deny_without_cli_allow","deny",False),
    trial("broker_response_id_mismatch","wrong_id",False),
    trial("broker_unknown_response_variant","unknown",False),
    trial("broker_broken_socket_disconnect","disconnect",False),
]
def direct(name,grant,missing_socket):
    with tempfile.TemporaryDirectory(prefix="gnu6-deno-broker-only-",dir=B) as td:
        d=pathlib.Path(td);file=d/"data.txt";file.write_text("GNU6_DENO_BROKER_ONLY\n")
        env=os.environ.copy()
        env.pop("DENO_PERMISSION_BROKER_PATH",None)
        env.update(DENO_NO_UPDATE_CHECK="1",DENO_NO_PROMPT="1",DENO_DIR=str(d/"cache"))
        if missing_socket: env["DENO_PERMISSION_BROKER_PATH"]=str(d/"missing.sock")
        cmd=[str(EXE),"run","--no-config","--no-prompt"]
        if grant: cmd.append("--allow-read="+str(file))
        cmd += [str(SCRIPT),str(file)]
        p=subprocess.run(cmd,env=env,capture_output=True,text=True,timeout=10)
        return {"label":name,"flag":grant,"exit":p.returncode,"stdout":p.stdout[:400],
                "stderr":p.stderr[:450],"request":[],"server_errors":[],
                "host_file_intact":file.read_text()=="GNU6_DENO_BROKER_ONLY\n"}
rows += [
    direct("no_broker_no_cli_allow",False,False),
    direct("no_broker_cli_allow",True,False),
    direct("broker_socket_missing_at_startup",False,True),
]
def parsed(row):
    try: return json.loads(row["stdout"].strip())
    except ValueError: return None
for r in rows:
    d=parsed(r) or {}
    name=r["label"]
    if name in ("broker_allow_over_cli_default_deny","no_broker_cli_allow"):
        good=r["exit"]==0 and d.get("ok") is True and d.get("matches") is True
    elif name in ("broker_deny_over_cli_allow","broker_deny_without_cli_allow","no_broker_no_cli_allow"):
        good=r["exit"]==0 and d.get("ok") is False and d.get("name")=="NotCapable"
    else:
        good=r["exit"]==87 and not r["stdout"]
    if name.startswith("broker_") and name!="broker_socket_missing_at_startup":
        good=good and len(r["request"])==1 and not r["server_errors"]
        if r["request"]:
            q=r["request"][0]
            good=good and q["v"]==1 and q["permission"]=="read" and q["id"]>=1 and q["value"].endswith("/data.txt")
    good=good and r["host_file_intact"]
    r["pass"]=good
gate="PASS_9_OF_9" if len(rows)==9 and all(r["pass"] for r in rows) else "RED_OR_INCONCLUSIVE"
receipt={"gate":gate,"runtime":"official Deno 2.9.7","binary_archive_sha256":DENO_SHA,
         "broker_source_git_blob_v2_9_7":"a7c70e963ed2a6a7bdc7c14bb8581c39b0ce119c",
         "broker_source_git_blob_pinned_commit":"a7c70e963ed2a6a7bdc7c14bb8581c39b0ce119c",
         "exact_pinned_binary":False,
         "socket_protocol":"Unix stream; newline JSON, request id correlated",
         "cases":rows,
         "limitations":["Experimental Deno permission broker","No MAC authentication on test UDS",
                        "Linux local named socket only","Broker behavior does not enforce OS syscalls",
                        "Deno release differs from October 6 pinned full tree"]}
out=ROOT/"evidence/upstream-rd-01/broker-net-02"
out.mkdir(parents=True,exist_ok=True)
(out/"deno-broker-live.json").write_text(json.dumps(receipt,indent=2,ensure_ascii=False)+"\n")
print("DENO_BROKER_LIVE",gate,[(r["label"],r["exit"],r["pass"]) for r in rows])
raise SystemExit(0 if gate=="PASS_9_OF_9" else 2)
