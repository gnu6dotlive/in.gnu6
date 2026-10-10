#!/usr/bin/env python3
"""Small causal Linux host bwrap oracle. This is NOT an MXC SDK security qualification."""
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile

ROOT = pathlib.Path(__file__).resolve().parents[2]
OUT = ROOT / "evidence/upstream-rd-01/intake"
OUT.mkdir(parents=True, exist_ok=True)
exe = shutil.which("bwrap")
assert exe, "NO_BWRAP"
control = b"GNU6 upstream-rd-01 sealed baseline\n"
payload = """
import json, os, pathlib, socket, sys, tempfile
p=pathlib.Path(sys.argv[1])
before=p.read_bytes()
write_error=None
try:
    p.write_bytes(b"VIOLATION!")
except OSError as e:
    write_error={"errno":e.errno,"error":str(e)}
with tempfile.NamedTemporaryFile(prefix="rd01-",dir="/tmp") as f:
    f.write(b"private temp")
    f.flush()
    tmp_ok=f.readable() and pathlib.Path(f.name).read_bytes()==b"private temp"
network_error=None
try:
    sock=socket.create_connection(("192.0.2.1",9), timeout=0.5)
    sock.close()
except OSError as e:
    network_error={"errno":e.errno,"error":str(e)}
print(json.dumps({"read_only_write_denied":write_error is not None,
                  "write_error":write_error,"content_preserved":p.read_bytes()==before,
                  "private_tmp_writable":tmp_ok,"network_connect_failed":network_error is not None,
                  "network_error":network_error,"namespace":os.readlink("/proc/self/ns/net")}))
"""
with tempfile.TemporaryDirectory(prefix="rd01-oracle-", dir=OUT) as td:
    sentinel=pathlib.Path(td) / "sentinel.txt"
    sentinel.write_bytes(control)
    host_network=os.readlink("/proc/self/ns/net")
    cmd=[exe,"--unshare-user","--unshare-pid","--unshare-net",
         "--ro-bind","/","/","--tmpfs","/tmp","--dev-bind","/dev","/dev",
         "--proc","/proc","--","/usr/bin/python3","-B","-c",payload,str(sentinel)]
    result=subprocess.run(cmd,capture_output=True,text=True,timeout=10)
    observed=json.loads(result.stdout.strip()) if result.returncode==0 else None
    integrity=sentinel.read_bytes()==control
    assert integrity, "Host sentinel unexpectedly changed!"
    report={"classification":"HOST_BWRAP_ONLY", "exitcode":result.returncode,
            "host_net_namespace":host_network,"stderr":result.stderr[:500],
            "observed":observed,"host_sentinel_sha256":hashlib.sha256(control).hexdigest(),
            "host_sentinel_preserved":integrity,
            "limitations":["not MXC SDK behavior","ro root remains readable",
                           "single unroutable test endpoint is not exhaustive network proof",
                           "only a bounded negative and positive fixture, not a security certification"]}
    report["gate"]="PASS_BOUNDARY_FIXTURE" if (
        observed and observed["read_only_write_denied"]
        and observed["content_preserved"] and observed["private_tmp_writable"]
        and observed["network_connect_failed"]
        and observed["namespace"]!=host_network
        ) else "FAIL_OR_INCONCLUSIVE"
(OUT/"host-denial-oracle.json").write_text(json.dumps(report,indent=2,ensure_ascii=False)+"\n")
print(json.dumps({"gate":report["gate"],"exitcode":result.returncode,"observed":observed,
                  "host_sentinel_preserved":integrity}))
sys.exit(0 if report["gate"]=="PASS_BOUNDARY_FIXTURE" else 2)
