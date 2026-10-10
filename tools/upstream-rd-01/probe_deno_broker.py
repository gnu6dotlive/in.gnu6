#!/usr/bin/env python3
"""Actual Deno 2.9.7 Unix IPC broker fixture, temp-only."""
import hashlib,json,os,pathlib,socket,subprocess,tempfile,threading
ROOT=pathlib.Path(__file__).resolve().parents[2]
B=pathlib.Path("/mnt/workbench/workspaces/gnu.in.labs/.probes/upstream-rd-01")
EXE=B/"deno-v2.9.7/runtime/deno"
SCRIPT=ROOT/"tools/upstream-rd-01/deno_broker_fixture.ts"
def trial(label,answer,grant_flag):
    with tempfile.TemporaryDirectory(prefix="gnu6-deno-broker-only-",dir=B) as td:
        d=pathlib.Path(td); data=d/"data.txt"; data.write_text("GNU6_DENO_BROKER_ONLY\n")
        sockpath=d/"broker.sock"; request=[]; errors=[]
        with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as server:
            server.bind(str(sockpath));server.listen(1);server.settimeout(7)
            def handler():
                try:
                    with server.accept()[0] as conn:
                        conn.settimeout(7)
                        by=b""
                        while not by.endswith(b"\n") and len(by)<8192:
                            part=conn.recv(1024)
                            if not part:break
                            by+=part
                        if by:
                            q=json.loads(by);request.append(q)
                            value={"id":q.get("id"),"result":answer}
                            if answer=="wrong_id":value={"id":q["id"]+100,"result":"allow"}
                            if answer=="unknown":value={"id":q["id"],"result":"mystery"}
                            if answer!="disconnect":
                                conn.sendall((json.dumps(value)+"\n").encode())
                except Exception as exc: errors.append(str(exc)[:150])
            th=threading.Thread(target=handler,daemon=True);th.start()
            env=os.environ.copy()
            env.update(DENO_PERMISSION_BROKER_PATH=str(sockpath),
                       DENO_NO_UPDATE_CHECK="1",DENO_NO_PROMPT="1",DENO_DIR=str(d/"cache"))
            args=[str(EXE),"run","--no-config","--no-prompt"]
            if grant_flag:args.append("--allow-read="+str(data))
            args += [str(SCRIPT),str(data)]
            proc=subprocess.run(args,env=env,capture_output=True,text=True,timeout=12)
            th.join(timeout=2)
        return {"label":label,"answer":answer,"flag":grant_flag,
                "exit":proc.returncode,"stdout":proc.stdout[:400],
                "stderr":proc.stderr[:400],"request":request,"server_errors":errors,
                "host_file_intact":data.read_text()=="GNU6_DENO_BROKER_ONLY\n"}
if __name__=="__main__":
    cases=[
      trial("broker_allow_default_deny","allow",False),
      trial("broker_deny_over_cli_allow","deny",True),
      trial("broker_deny_without_cli_allow","deny",False),
      trial("broker_wrong_response_id","wrong_id",False),
      trial("broker_unknown_response","unknown",False),
      trial("broker_socket_disconnect","disconnect",False)]
    for c in cases:
        print(c["label"],"exit",c["exit"],"out",c["stdout"][:110],"err",c["stderr"][-140:],
              "requests",len(c["request"]))
