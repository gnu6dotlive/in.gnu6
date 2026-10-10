#!/usr/bin/env python3
"""CUDA-05F preregistered one-run actual Rust service -> worker -> GPU experiment."""
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import shutil
import socket
import stat
import subprocess
import tempfile
import time
import uuid

HERE=Path(__file__).resolve().parent
BUILD=Path("/mnt/workbench/build")
BIN=Path("/mnt/workbench/target/release")
LIB=Path("/mnt/workbench/build/blobin-cuda04d/libblobin_cuda03.so")
IMAGE="debian:trixie-slim"
SERVICE_UID=65534
REJECT_UID=65533
DAC_UID=65532
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def command(args,timeout=35):
    try:
        p=subprocess.run(args,capture_output=True,text=True,timeout=timeout,check=False)
        return {"exit":p.returncode,"stdout":p.stdout[:9000],"stderr":p.stderr[:1100]}
    except subprocess.TimeoutExpired:
        return {"exit":None,"stdout":"","stderr":"TIMEOUT"}
def docker(*args,timeout=35):
    return command(["docker",*args],timeout=timeout)
def assert_ok(result,label):
    if result["exit"]!=0:raise RuntimeError(label+": "+result["stderr"][:350])
def connect(sock,msg):
    try:
        with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as s:
            s.settimeout(20)
            s.connect(str(sock))
            s.sendall((msg+"\n").encode())
            return {"exit":0,"reply":s.recv(256).decode().strip()}
    except Exception as e:
        return {"exit":1,"reply":None,"error":str(e)}
PERL_CLIENT='''use strict;use warnings;use IO::Socket::UNIX;use Socket qw(SOCK_STREAM);
my $s=IO::Socket::UNIX->new(Type=>SOCK_STREAM,Peer=>$ARGV[0]) or die "CONNECT_FAILED $!\n";
print $s "STATUS\n";
my $reply=<$s>//"";print $reply;close($s);
'''
def another_uid(folder,uid,gid):
    return docker("run","--rm","--pull=never","--network","none","--read-only",
        "--cap-drop","ALL","--security-opt","no-new-privileges","--pids-limit","32",
        "--memory","128m","--user",f"{uid}:{gid}",
        "--mount",f"type=bind,source={folder},target=/lab,readonly",
        IMAGE,"perl","-e",PERL_CLIENT,"/lab/service.sock")
def host_pid_uid(pid):
    for line in Path(f"/proc/{pid}/status").read_text().splitlines():
        if line.startswith("Uid:"):return [int(n) for n in line.split()[1:]]
    raise RuntimeError("MISSING_HOST_UID_OBSERVATION")
def mount_options(folder):
    return ["--mount",f"type=bind,source={folder},target=/lab",
        "--mount","type=bind,source=/usr/lib,target=/hostlib,readonly",
        "--mount","type=bind,source=/usr/share/vulkan,target=/usr/share/vulkan,readonly"]
def prepare(folder):
    files={"supervise05f":BIN/"supervise05f","worker05c":BIN/"worker05c",
        "libblobin_cuda03.so":LIB}
    for name,source in files.items():
        if not source.is_file():raise RuntimeError("MISSING_BINARY "+name)
        shutil.copyfile(source,folder/name)
    for name in ("supervise05f","worker05c"):
        target=folder/name
        assert_ok(command(["patchelf","--set-interpreter","/hostlib/ld-linux-x86-64.so.2",
            "--force-rpath","--set-rpath","/hostlib:/usr/lib/x86_64-linux-gnu",str(target)]),"PATCH_ELF "+name)
        target.chmod(0o755)
    (folder/"libblobin_cuda03.so").chmod(0o644)
    for name in ("worker05c","libblobin_cuda03.so"):
        content=(folder/name).read_bytes()
        (folder/(name+".restore")).write_bytes(content)
        altered=bytearray(content);altered[0]^=255
        (folder/(name+".bad")).write_bytes(altered)
        (folder/(name+".restore")).chmod(0o755 if name=="worker05c" else 0o644)
        (folder/(name+".bad")).chmod(0o755 if name=="worker05c" else 0o644)
    return {name:digest(folder/name) for name in files}
def mutate(name,path,destination):
    result=docker("exec","--user",f"{SERVICE_UID}:{os.getegid()}",
        name,"cp",f"/lab/{path}",f"/lab/{destination}.new")
    assert_ok(result,"MUTATE_COPY")
    result=docker("exec","--user",f"{SERVICE_UID}:{os.getegid()}",
        name,"mv",f"/lab/{destination}.new",f"/lab/{destination}")
    assert_ok(result,"MUTATE_REPLACE")

def scenario(backend,kind):
    label="cuda05f-"+uuid.uuid4().hex[:12]
    name=label+"-svc"
    host_uid=os.geteuid();gid=os.getegid()
    commands=(
        ["STATUS","RENDER","ACQUIRE","RENDER","XREJECT","XDAC","RENDER","REVOKE",
            "RENDER","ACQUIRE","RENDER"] if kind=="normal" else
        ["STATUS","ACQUIRE","RENDER","RENDER","ACQUIRE","RENDER","STATUS"] if kind in ("crash","hang") else
        ["ACQUIRE","RENDER","RENDER","RENDER","ACQUIRE","RENDER"]
    )
    accepted=sum(1 for item in commands if item!="XDAC")
    fault_index=2 if kind in ("crash","hang") else None
    evidence={"backend":backend,"scenario":kind,"commands":commands,
        "timestamp_utc":dt.datetime.now(dt.timezone.utc).isoformat(),
        "observations":[],"service":{},"files":{},"setup":None}
    with tempfile.TemporaryDirectory(prefix=label+"-",dir=BUILD) as tmp:
        folder=Path(tmp)
        file_hashes=prepare(folder)
        evidence["files"]=file_hashes
        folder.chmod(0o710)
        setup=docker("run","--rm","--pull=never","--network","none","--read-only",
            "--cap-drop","ALL","--cap-add","CHOWN","--security-opt","no-new-privileges",
            "--pids-limit","32","--user","0:0",
            "--mount",f"type=bind,source={folder},target=/lab",IMAGE,
            "chown",f"{SERVICE_UID}:{gid}","/lab")
        evidence["setup"]=setup
        assert_ok(setup,"DIRECTORY_SETUP")
        created=False
        try:
            runtime=[
                "run","--detach","--pull=never","--network","none","--read-only",
                "--cap-drop","ALL","--security-opt","no-new-privileges",
                "--pids-limit","64","--memory","1024m","--gpus","all",
                "--device","/dev/dri/renderD128",
                "--env","NVIDIA_DRIVER_CAPABILITIES=graphics,compute,utility",
                "--user",f"{SERVICE_UID}:{gid}","--name",name,
                "--label","org.gnu6.lab=cuda05f",
                *mount_options(folder),IMAGE,
                "/lab/supervise05f","--socket","/lab/service.sock",
                "--worker","/lab/worker05c","--worker-sha256",file_hashes["worker05c"],
                "--backend",backend,"--allowed-uid",str(host_uid),
                "--max-requests",str(accepted),"--timeout-ms",
                "1000" if kind=="hang" else "12000"
            ]
            if backend=="cuda":
                runtime+=["--cuda-lib","/lab/libblobin_cuda03.so",
                          "--sha256",file_hashes["libblobin_cuda03.so"]]
            if fault_index is not None:
                runtime+=["--fault-index",str(fault_index),"--fault",kind]
            started=docker(*runtime)
            evidence["service"]["start"]=started
            assert_ok(started,"SERVICE_START")
            created=True
            sock=folder/"service.sock"
            for _ in range(150):
                if sock.exists():break
                time.sleep(.04)
            if not sock.exists():
                evidence["service"]["start_logs"]=docker("logs",name)
                raise RuntimeError("RUST_SOCKET_NOT_READY")
            inspection=docker("inspect","--format",
                "{{.Config.User}}|{{.HostConfig.UsernsMode}}|{{.HostConfig.NetworkMode}}|{{.State.Pid}}",
                name)
            assert_ok(inspection,"INSPECT")
            parts=inspection["stdout"].strip().split("|")
            pid=int(parts[3])
            evidence["service"]["inspect"]=inspection
            evidence["service"]["host_uid_quad"]=host_pid_uid(pid)
            evidence["service"]["directory"]={"mode":oct(stat.S_IMODE(folder.stat().st_mode)),
                "owner":folder.stat().st_uid,"group":folder.stat().st_gid}
            evidence["service"]["socket"]={"mode":oct(stat.S_IMODE(sock.stat().st_mode)),
                "owner":sock.stat().st_uid,"group":sock.stat().st_gid}
            for idx,item in enumerate(commands):
                if item=="XREJECT":
                    response=another_uid(folder,REJECT_UID,gid)
                elif item=="XDAC":
                    response=another_uid(folder,DAC_UID,DAC_UID)
                else:
                    if kind in ("worker_tamper","provider_tamper") and idx==2:
                        target="worker05c" if kind=="worker_tamper" else "libblobin_cuda03.so"
                        mutate(name,target+".bad",target)
                    if kind in ("worker_tamper","provider_tamper") and idx==3:
                        target="worker05c" if kind=="worker_tamper" else "libblobin_cuda03.so"
                        mutate(name,target+".restore",target)
                    response=connect(sock,item)
                evidence["observations"].append({"index":idx,"intent":item,"response":response})
            for _ in range(80):
                state=docker("inspect","--format","{{.State.Running}}",name)
                if state["stdout"].strip()=="false":break
                time.sleep(.08)
            evidence["service"]["log"]=docker("logs",name)
            evidence["service"]["exit"]=docker("inspect","--format","{{.State.ExitCode}}",name)
        finally:
            if created:
                evidence["cleanup_container"]=docker("rm","--force",name)
            evidence["cleanup_ownership"]=docker("run","--rm","--pull=never",
                "--network","none","--read-only","--cap-drop","ALL","--cap-add","CHOWN",
                "--security-opt","no-new-privileges","--pids-limit","32",
                "--user","0:0","--mount",f"type=bind,source={folder},target=/lab",
                IMAGE,"chown",f"{host_uid}:{gid}","/lab")
            folder.chmod(0o700)
    return evidence
def main():
    BUILD.mkdir(parents=True,exist_ok=True)
    scenarios=[(b,k) for b in ("cuda","vulkan") for k in ("normal","crash","hang")]
    scenarios += [("cuda","worker_tamper"),("cuda","provider_tamper")]
    result={"schema":"CUDA05F.RAW.v1","experiment_id":"CUDA-05F-P01",
        "timestamp_utc":dt.datetime.now(dt.timezone.utc).isoformat(),
        "contract_sha256":digest(HERE/"GPU-EVIDENCE-CONTRACT-v1.json"),
        "environment":{"image":IMAGE,"service_uid":SERVICE_UID,
            "authorized_uid":os.geteuid(),"gid":os.getegid(),
            "rootful_docker":True,"gpu":"RTX3070 active Wayland display; no reset"},
        "scenarios":[]}
    out=HERE/"evidence"/"05F";out.mkdir(exist_ok=True,parents=True)
    for backend,kind in scenarios:
        item=scenario(backend,kind)
        result["scenarios"].append(item)
        print("CUDA05F_SCENARIO_CAPTURED backend="+backend+" kind="+kind+
              " outcomes="+str(len(item["observations"])))
        (out/"05F-raw.json").write_text(json.dumps(result,indent=2,sort_keys=True)+"\n")
    print("CUDA05F_RAW_CAPTURED scenarios="+str(len(result["scenarios"])))
if __name__=="__main__":
    main()
