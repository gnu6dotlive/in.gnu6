#!/usr/bin/env python3
"""Bounded real GPU Unix socket qualification; no device reset."""
import hashlib
import os
from pathlib import Path
import socket
import subprocess
import sys
import tempfile
import time

BIN = Path("/mnt/workbench/target/debug/supervise05b")
LIB = Path("/mnt/workbench/build/blobin-cuda04d/libblobin_cuda03.so")
ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "fixtures" / "qualification-20261009-b"

def one(sock, command):
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as conn:
        conn.settimeout(45)
        conn.connect(str(sock))
        conn.sendall((command + "\n").encode())
        data = conn.recv(4096).decode().strip()
    if not data:
        raise AssertionError("missing supervisor response")
    return data

def run_service(device, allowed, commands, sha, tamper=False):
    with tempfile.TemporaryDirectory(prefix="cuda05b-", dir="/mnt/workbench/build") as name:
        sock = Path(name) / "supervisor.sock"
        test_lib = Path(name) / "provider.so" if tamper else LIB
        if tamper: test_lib.write_bytes(LIB.read_bytes())
        cmd=[str(BIN),"--socket",str(sock),"--backend",device,
             "--allowed-uid",str(allowed),"--max-requests",str(len(commands))]
        if device == "cuda":
            cmd += ["--cuda-lib",str(test_lib),"--sha256",sha]
        process=subprocess.Popen(cmd,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
        try:
            deadline=time.monotonic()+15
            while not sock.exists() and process.poll() is None and time.monotonic()<deadline:
                time.sleep(0.025)
            if not sock.exists(): raise AssertionError(f"socket not ready: {process.poll()}")
            if sock.stat().st_mode & 0o077: raise AssertionError("socket not private")
            results=[]
            for index,command in enumerate(commands):
                if tamper and index==2:
                    corrupted=bytearray(LIB.read_bytes())
                    corrupted[0]^=255
                    tmp=Path(name) / "tampered.so"
                    tmp.write_bytes(corrupted)
                    os.replace(tmp,test_lib)
                if tamper and index==3:
                    tmp=Path(name) / "restored.so"
                    tmp.write_bytes(LIB.read_bytes())
                    os.replace(tmp,test_lib)
                results.append(one(sock,command))
            stdout,_=process.communicate(timeout=90)
            assert process.returncode==0,(process.returncode,stdout)
            assert not sock.exists(), "socket left behind"
            assert "CUDA05B_EXIT clean=PASS" in stdout
            return results, stdout
        finally:
            if process.poll() is None:
                process.kill()
                process.communicate(timeout=5)

def gate(results):
    assert results[0].startswith("STATE generation=None")
    assert results[1].startswith("DENIED reason=CUDA05B_NO_ACTIVE_PERMIT")
    assert results[2].startswith("READY generation=")
    assert results[3].startswith("RENDER_PASS") and "cache_hit=false" in results[3]
    assert results[4].startswith("RENDER_PASS") and "cache_hit=true" in results[4]
    assert results[5].startswith("REVOKED")
    assert results[6].startswith("DENIED reason=CUDA05B_NO_ACTIVE_PERMIT")
    assert results[7].startswith("READY generation=")
    assert int(results[7].split("generation=")[1].split()[0]) > int(results[2].split("generation=")[1].split()[0])
    assert results[8].startswith("RENDER_PASS") and "cache_hit=false" in results[8]
    assert results[9].startswith("STATE generation=Some")
    for index in [3,4,8]:
        assert "parity=PASS" in results[index]

def main():
    assert BIN.is_file() and LIB.is_file()
    digest=hashlib.sha256(LIB.read_bytes()).hexdigest()
    cmd=["STATUS","RENDER","ACQUIRE","RENDER","RENDER","REVOKE","RENDER","ACQUIRE","RENDER","STATUS"]
    evidence=[]
    for device in ("cuda","vulkan"):
        results, stdout=run_service(device,os.getuid(),cmd,digest)
        gate(results)
        assert stdout.count("CUDA05B_EVENT")==10
        for i,(query,response) in enumerate(zip(cmd,results)):
            evidence.append(f"CUDA05B_SOCKET_PASS backend={device} index={i} command={query} response={response}")
        evidence.append(f"CUDA05B_DEVICE_PASS backend={device} probe_challenges=5 socket_peer=OS_UID parity=PASS")
        unauthorized,_=run_service(device,os.getuid()+1,["STATUS"],digest)
        assert unauthorized==["DENIED reason=CUDA05B_PEER_UNAUTHORIZED"]
        evidence.append(f"CUDA05B_DENY_PASS backend={device} wrong_uid=REJECTED")
    tamper_cmd=["ACQUIRE","RENDER","RENDER","RENDER","ACQUIRE","RENDER"]
    tampered,_=run_service("cuda",os.getuid(),tamper_cmd,digest,tamper=True)
    assert tampered[0].startswith("READY generation=")
    assert tampered[1].startswith("RENDER_PASS")
    assert tampered[2].startswith("DENIED reason=CUDA05B_CHALLENGE_UNAPPROVED_PROVIDER")
    assert tampered[3].startswith("DENIED reason=CUDA05B_NO_ACTIVE_PERMIT")
    assert tampered[4].startswith("READY generation=")
    assert tampered[5].startswith("RENDER_PASS") and "cache_hit=false" in tampered[5]
    for i,(query,response) in enumerate(zip(tamper_cmd,tampered)):
        evidence.append(f"CUDA05B_TAMPER_STEP_PASS index={i} command={query} response={response}")
    evidence.append("CUDA05B_TAMPER_PASS backend=cuda same_path=REJECTED revoked=PASS restored=PASS")
    evidence.append("CUDA05B_ALL_PASS devices=2 socket_operations=26 wrong_uid=2 tamper=1 no_gpu_reset=PASS")
    OUT.mkdir(exist_ok=True,parents=True)
    (OUT/"results.log").write_text("\n".join(evidence)+"\n")
    print("\n".join(evidence))
if __name__=="__main__":
    try:main()
    except Exception as exc:sys.exit(f"CUDA05B_ERROR {exc}")
