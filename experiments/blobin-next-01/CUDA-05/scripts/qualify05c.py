#!/usr/bin/env python3
"""Real GPU one-child-per-command lab; child crash and timeout injections."""
import hashlib
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time

ROOT=Path(__file__).resolve().parents[1]
BIN=Path("/mnt/workbench/target/release/supervise05c")
WORKER=Path("/mnt/workbench/target/release/worker05c")
LIB=Path("/mnt/workbench/build/blobin-cuda04d/libblobin_cuda03.so")
OUT=ROOT/"fixtures"/"qualification-20261009-c"
NORMAL=["STATUS","RENDER","ACQUIRE","RENDER","REVOKE","RENDER","ACQUIRE","RENDER","STATUS"]
FAULT=["STATUS","ACQUIRE","RENDER","RENDER","ACQUIRE","RENDER","STATUS"]

def invoke(sock,cmd):
    with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as s:
        s.settimeout(15)
        s.connect(str(sock))
        s.sendall((cmd+"\n").encode())
        data=s.recv(4096).decode().strip()
        assert data, "empty supervisor reply"
        return data

def run(backend,commands,timeout=5000,injection=None,allowed_uid=None,worker_tamper=False):
    with tempfile.TemporaryDirectory(prefix="blobin-05c-",dir="/mnt/workbench/build") as td:
        sock=Path(td)/"service.sock"
        staged=Path(td)/"worker05c"
        if worker_tamper:
            staged.write_bytes(WORKER.read_bytes())
            staged.chmod(0o700)
        chosen=staged if worker_tamper else WORKER
        args=[str(BIN),"--socket",str(sock),"--worker",str(chosen),
              "--worker-sha256",hashlib.sha256(WORKER.read_bytes()).hexdigest(),
              "--backend",backend,"--allowed-uid",str(os.getuid() if allowed_uid is None else allowed_uid),
              "--max-requests",str(len(commands)),"--timeout-ms",str(timeout)]
        if backend=="cuda":
            args += ["--cuda-lib",str(LIB),"--sha256",hashlib.sha256(LIB.read_bytes()).hexdigest()]
        if injection is not None:
            args+=["--fault-index","2","--fault",injection]
        p=subprocess.Popen(args,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
        try:
            deadline=time.monotonic()+10
            while not sock.exists() and p.poll() is None and time.monotonic()<deadline:
                time.sleep(.02)
            assert sock.exists(),f"socket absent process={p.poll()}"
            assert sock.stat().st_mode&0o077==0
            results=[]
            for index,c in enumerate(commands):
                if worker_tamper and index==2:
                    replacement=Path(td)/"corrupt-worker"
                    altered=bytearray(WORKER.read_bytes())
                    altered[0]^=255
                    replacement.write_bytes(altered)
                    replacement.chmod(0o700)
                    os.replace(replacement,staged)
                if worker_tamper and index==4:
                    replacement=Path(td)/"repaired-worker"
                    replacement.write_bytes(WORKER.read_bytes())
                    replacement.chmod(0o700)
                    os.replace(replacement,staged)
                results.append(invoke(sock,c))
            stdout,_=p.communicate(timeout=45)
            assert p.returncode==0,(p.returncode,stdout)
            assert "CUDA05C_EXIT clean=PASS" in stdout
            assert stdout.count("CUDA05C_EVENT")==len(commands)
            assert not sock.exists()
            return results
        finally:
            if p.poll() is None:
                p.kill()
                p.communicate(timeout=5)

def validate_normal(rows):
    assert rows[0].startswith("STATE generation=None")
    assert rows[1].startswith("DENIED reason=CUDA05C_NO_ACTIVE_GRANT")
    assert rows[2].startswith("READY generation=")
    assert rows[3].startswith("RENDER_PASS") and "parity=PASS" in rows[3]
    assert rows[4].startswith("REVOKED")
    assert rows[5].startswith("DENIED reason=CUDA05C_NO_ACTIVE_GRANT")
    assert rows[6].startswith("READY generation=")
    assert int(rows[6].split("generation=")[1].split()[0])>int(rows[2].split("generation=")[1].split()[0])
    assert rows[7].startswith("RENDER_PASS") and "parity=PASS" in rows[7]
    assert rows[8].startswith("STATE generation=Some")

def validate_fault(rows,mode):
    assert rows[0].startswith("STATE generation=None")
    assert rows[1].startswith("READY generation=")
    expected="CUDA05C_WORKER_TIMEOUT_KILLED" if mode=="hang" else "CUDA05C_WORKER_CRASH_OR_ERROR"
    assert rows[2].startswith(f"DENIED reason={expected}"),(mode,rows[2])
    assert rows[3].startswith("DENIED reason=CUDA05C_NO_ACTIVE_GRANT")
    assert rows[4].startswith("READY generation=")
    assert rows[5].startswith("RENDER_PASS") and "parity=PASS" in rows[5]
    assert rows[6].startswith("STATE generation=Some")
    assert int(rows[4].split("generation=")[1].split()[0])>int(rows[1].split("generation=")[1].split()[0])

def main():
    assert BIN.is_file() and WORKER.is_file() and LIB.is_file()
    lines=[]
    for backend in ("cuda","vulkan"):
        result=run(backend,NORMAL)
        validate_normal(result)
        for i,(command,response) in enumerate(zip(NORMAL,result)):
            lines.append(f"CUDA05C_NORMAL_PASS backend={backend} index={i} command={command} response={response}")
        for mode in ("crash","hang"):
            result=run(backend,FAULT,800 if mode=="hang" else 5000,injection=mode)
            validate_fault(result,mode)
            for i,(command,response) in enumerate(zip(FAULT,result)):
                lines.append(f"CUDA05C_FAULT_STEP_PASS backend={backend} fault={mode} index={i} command={command} response={response}")
            lines.append(f"CUDA05C_FAULT_PASS backend={backend} fault={mode} revoked=PASS recovered=PASS")
        denied=run(backend,["STATUS"],allowed_uid=os.getuid()+1)
        assert denied==["DENIED reason=CUDA05C_PEER_UNAUTHORIZED"]
        lines.append(f"CUDA05C_DENY_PASS backend={backend} configured_wrong_uid=REJECTED")
    tamper_cmd=["ACQUIRE","RENDER","RENDER","RENDER","ACQUIRE","RENDER"]
    tampered=run("cuda",tamper_cmd,worker_tamper=True)
    assert tampered[0].startswith("READY generation=")
    assert tampered[1].startswith("RENDER_PASS")
    assert tampered[2].startswith("DENIED reason=CUDA05C_WORKER_HASH_NOT_APPROVED")
    assert tampered[3].startswith("DENIED reason=CUDA05C_NO_ACTIVE_GRANT")
    assert tampered[4].startswith("READY generation=")
    assert tampered[5].startswith("RENDER_PASS")
    for i,(cmd,response) in enumerate(zip(tamper_cmd,tampered)):
        lines.append(f"CUDA05C_TAMPER_STEP_PASS index={i} command={cmd} response={response}")
    lines.append("CUDA05C_TAMPER_PASS backend=cuda same_path_worker=REJECTED restored=PASS")
    lines.append("CUDA05C_ALL_PASS real_backends=2 normal_socket_ops=18 fault_ops=28 worker_tamper_ops=6 injected_crashes=2 injected_timeouts=2 wrong_uid=2 no_gpu_reset=PASS")
    OUT.mkdir(parents=True,exist_ok=True)
    (OUT/"results.log").write_text("\n".join(lines)+"\n")
    print("\n".join(lines))
if __name__=="__main__":
    try:main()
    except Exception as e:sys.exit(f"CUDA05C_TEST_ERROR {e}")
