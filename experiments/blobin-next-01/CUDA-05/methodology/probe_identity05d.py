#!/usr/bin/env python3
"""Bounded Linux peer-identity experiment. No privileged operations, accounts or GPU reset."""
from __future__ import annotations
from datetime import datetime, timezone
import json
import os
from pathlib import Path
import platform
import socket
import stat
import struct
import subprocess
import sys
import tempfile

HERE=Path(__file__).resolve().parent
SO_PEERCRED=17
# A child reports *its view*; the server measures *kernel outside peer credentials*.
CLIENT = """
import json,os,socket,sys
p=sys.argv[1]
s=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
s.settimeout(4)
s.connect(p)
s.sendall(json.dumps({'self_euid':os.geteuid(),'self_egid':os.getegid()}).encode())
s.close()
"""
def bounded(cmd):
    try:
        result=subprocess.run(cmd,timeout=7,check=False,capture_output=True,text=True)
        return {"exit_code": result.returncode,
                "stdout":result.stdout[:240],
                "stderr":result.stderr[:240],
                "timeout":False}
    except subprocess.TimeoutExpired:
        return {"exit_code":None,"stdout":"","stderr":"TIMEOUT","timeout":True}

def round_trip(listener,command):
    done=bounded(command)
    result={"command_kind":"baseline" if command[0]==sys.executable else
            ("namespace-root" if "--map-root-user" in command else "subordinate-auto"),
            "process":done,"kernel_peer":None,"client_claim":None}
    if done["exit_code"]!=0:
        return result
    try:
        conn,_=listener.accept()
    except socket.timeout:
        result["process"]["stderr"]="PEER_CONNECTION_MISSING"
        return result
    with conn:
        payload=conn.recv(256)
        creds=conn.getsockopt(socket.SOL_SOCKET,SO_PEERCRED,struct.calcsize("3i"))
    pid,uid,gid=struct.unpack("3i",creds)
    result["kernel_peer"]={"pid":pid,"uid":uid,"gid":gid}
    try:
        result["client_claim"]=json.loads(payload)
    except (ValueError,UnicodeError):
        result["client_claim"]={"malformed":True}
    return result

def main():
    if sys.platform!="linux":
        raise SystemExit("CUDA05D ERROR: LINUX_REQUIRED")
    with tempfile.TemporaryDirectory(prefix="cuda05d-",dir="/tmp") as dirname:
        directory=Path(dirname)
        os.chmod(directory,0o700)
        sock=directory/"identity.sock"
        server=socket.socket(socket.AF_UNIX,socket.SOCK_STREAM)
        with server:
            server.settimeout(3)
            server.bind(str(sock))
            os.chmod(sock,0o600)
            server.listen(4)
            baseline=round_trip(server,[sys.executable,"-c",CLIENT,str(sock)])
            rootless=round_trip(server,["unshare","--user","--map-root-user",
                                              sys.executable,"-c",CLIENT,str(sock)])
            subordinate=round_trip(server,["unshare","--user","--map-auto",
                                                  sys.executable,"-c",CLIENT,str(sock)])
            recorded={
                "schema":"CUDA05D.IDENTITIES.v1",
                "recorded_utc":datetime.now(timezone.utc).isoformat(),
                "platform":{"kernel_release":platform.release(),"python":platform.python_version()},
                "host_euid":os.geteuid(),"host_egid":os.getegid(),
                "socket_directory_mode":oct(stat.S_IMODE(directory.stat().st_mode)),
                "socket_mode":oct(stat.S_IMODE(sock.stat().st_mode)),
                "runs":{"baseline":baseline,"namespace_root":rootless,"subordinate_auto":subordinate},
                "limitations":[
                    "SO_PEERCRED from host user namespace, not hardware attestation",
                    "Client self-report is informational and may be forged",
                    "No distinct host account created or service ownership delegated",
                    "A successful namespace mapping alone cannot establish agent authorization",
                ],
            }
    output=HERE/"evidence"/"identity-observations.json"
    output.parent.mkdir(parents=True,exist_ok=True)
    output.write_text(json.dumps(recorded,indent=2,ensure_ascii=False,sort_keys=True)+"\n")
    print(f"CUDA05D_ACQUISITION_CAPTURED file={output.name} controls=baseline,namespace_root,subordinate_auto")

if __name__=="__main__":
    main()
