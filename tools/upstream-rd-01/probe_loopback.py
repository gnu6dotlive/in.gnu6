#!/usr/bin/env python3
"""Positive and negative loopback oracle for host bwrap net namespace; NOT MXC."""
import json
import os
import pathlib
import socket
import subprocess
import sys

ROOT=pathlib.Path(__file__).resolve().parents[2]
CHILD=r"""
import json,os,socket,sys
port=int(sys.argv[1])
s=socket.socket()
s.settimeout(2)
try:
    s.connect(("127.0.0.1",port))
    result={"connected":True,"error":None}
except OSError as e:
    result={"connected":False,"error":str(e),"errno":e.errno}
finally:
    s.close()
result["net_namespace"]=os.readlink("/proc/self/ns/net")
print(json.dumps(result))
"""
def attempt(port, isolated):
    cmd=["/usr/bin/bwrap","--unshare-user","--unshare-pid"]
    if isolated:cmd += ["--unshare-net"]
    cmd += ["--ro-bind","/","/","--proc","/proc","--dev-bind","/dev","/dev",
            "--tmpfs","/tmp","--","/usr/bin/python3","-B","-c",CHILD,str(port)]
    p=subprocess.run(cmd,capture_output=True,text=True,timeout=9)
    return {"command":cmd[:4]+["...truncated-argv"],"exitcode":p.returncode,
            "stderr":p.stderr[:200],"result":json.loads(p.stdout) if p.returncode==0 else None}
with socket.socket() as server:
    server.setsockopt(socket.SOL_SOCKET,socket.SO_REUSEADDR,1)
    server.bind(("127.0.0.1",0))
    server.listen(8)
    port=server.getsockname()[1]
    with socket.create_connection(("127.0.0.1",port),timeout=2):pass
    control=attempt(port,False)
    isolated=attempt(port,True)
host_ns=os.readlink("/proc/self/ns/net")
good=(control["exitcode"]==0 and isolated["exitcode"]==0
      and control["result"] is not None and isolated["result"] is not None
      and control["result"]["connected"]
      and not isolated["result"]["connected"]
      and isolated["result"]["net_namespace"]!=host_ns)
result={"gate":"PASS_TWO_SIDED_HOST_ORACLE" if good else "INCONCLUSIVE_OR_FAIL",
        "positive_control":control,"isolated_test":isolated,
        "host_namespace":host_ns,"target":"ephemeral loopback TCP listener only",
        "limitations":["not MXC SDK or Deno runtime","isolation depends on kernel, bwrap and run environment",
                       "ro-bind / / discloses readable host paths, not a confidentiality sandbox"]}
out=ROOT/"evidence/upstream-rd-01/mxc-security-01/loopback-oracle.json"
out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(result,indent=2,ensure_ascii=False)+"\n")
print("MXC_HOST_NET_NAMESPACE_ORACLE",result["gate"],
      "control=",control["result"],"isolated=",isolated["result"])
sys.exit(0 if good else 2)
