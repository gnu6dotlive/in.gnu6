#!/usr/bin/env python3
"""Preregistered, single-trial Docker disposable host UID / SO_PEERCRED experiment."""
import datetime as dt
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

ROOT=Path(__file__).resolve().parent
BUILD=Path("/mnt/workbench/build")
IMAGE="debian:trixie-slim"
SERVICE_UID=65534
REJECT_UID=65533
DAC_UID=65532
def execute(args,timeout=20):
    try:
        p=subprocess.run(args,check=False,capture_output=True,text=True,timeout=timeout)
        return {"code":p.returncode,"stdout":p.stdout[:2500],"stderr":p.stderr[:1200]}
    except subprocess.TimeoutExpired:
        return {"code":None,"stdout":"","stderr":"SUBPROCESS_TIMEOUT"}
def docker(*args,timeout=20):
    return execute(["docker",*args],timeout=timeout)
def run_client(sock):
    try:
        with socket.socket(socket.AF_UNIX,socket.SOCK_STREAM) as peer:
            peer.settimeout(4)
            peer.connect(str(sock))
            peer.sendall(b"STATUS\n")
            return {"code":0,"reply":peer.recv(128).decode().strip()}
    except Exception as e:
        return {"code":1,"reply":None,"error":str(e)}
PERL_CLIENT='''use strict;use warnings;use IO::Socket::UNIX;use Socket qw(SOCK_STREAM);
my $x=IO::Socket::UNIX->new(Type=>SOCK_STREAM,Peer=>$ARGV[0]) or die "CONNECT_FAILED $!\n";
print $x "STATUS\n";
my $line=<$x> // "";
print $line;
close($x);
'''
def isolated_client(path,uid,gid):
    return docker("run","--rm","--pull=never","--network","none","--read-only",
                  "--cap-drop","ALL","--security-opt","no-new-privileges",
                  "--pids-limit","32","--memory","128m",
                  "--user",f"{uid}:{gid}",
                  "--mount",f"type=bind,source={path.parent},target=/lab,readonly",
                  IMAGE,"perl","-e",PERL_CLIENT,"/lab/identity.sock")
def uid_of_host_pid(pid):
    if not isinstance(pid,int) or pid<=0:return None
    for line in Path(f"/proc/{pid}/status").read_text().splitlines():
        if line.startswith("Uid:"):
            return [int(x) for x in line.split()[1:]]
    return None
def main():
    BUILD.mkdir(parents=True,exist_ok=True)
    label="cuda05e-"+uuid.uuid4().hex[:12]
    host_uid,group=os.geteuid(),os.getegid()
    raw={"schema":"CUDA05E.PEERS.v1","recorded_utc":dt.datetime.now(dt.timezone.utc).isoformat(),
         "image":IMAGE,"expected":{"service_uid":SERVICE_UID,"authorized_uid":host_uid,
             "rejected_uid":REJECT_UID,"dac_rejected_uid":DAC_UID,"shared_gid":group},
         "limits":["single non-randomized trial","Docker daemon is privileged","no GPU operation"],
         "commands":[]}
    name=label+"-svc"
    with tempfile.TemporaryDirectory(prefix=label+"-",dir=BUILD) as td:
        folder=Path(td)
        service_script=folder/"service.pl"
        shutil.copyfile(ROOT/"05e-service.pl",service_script)
        service_script.chmod(0o644)
        mount=f"type=bind,source={folder},target=/lab"
        setup=docker("run","--rm","--pull=never","--network","none","--read-only",
                     "--cap-drop","ALL","--cap-add","CHOWN",
                     "--security-opt","no-new-privileges","--pids-limit","32",
                     "--user","0:0","--mount",mount,IMAGE,
                     "sh","-c",f"chown {SERVICE_UID}:{group} /lab && chmod 0710 /lab")
        raw["commands"].append({"operation":"bounded_directory_ownership","result":setup})
        if setup["code"]!=0:raise RuntimeError("SETUP_FAILED "+setup["stderr"])
        created=False
        try:
            started=docker("run","--detach","--pull=never","--network","none",
                           "--read-only","--cap-drop","ALL",
                           "--security-opt","no-new-privileges","--pids-limit","32",
                           "--memory","128m","--user",f"{SERVICE_UID}:{group}",
                           "--name",name,"--label","org.gnu6.lab=cuda05e",
                           "--mount",mount,IMAGE,"perl","/lab/service.pl",
                           "/lab/identity.sock",str(host_uid),"3")
            raw["commands"].append({"operation":"start_disposable_supervisor","result":started})
            if started["code"]!=0:raise RuntimeError("SERVICE_START_FAILED "+started["stderr"])
            created=True
            sock=folder/"identity.sock"
            for _ in range(140):
                if sock.exists():break
                time.sleep(.05)
            if not sock.exists():raise RuntimeError("SERVICE_SOCKET_NOT_CREATED")
            inspect=docker("inspect","--format",
                "{{.Config.User}}|{{.HostConfig.UsernsMode}}|{{.HostConfig.NetworkMode}}|{{.State.Pid}}",
                name)
            raw["inspection"]={"command":inspect,"socket_mode":oct(stat.S_IMODE(sock.stat().st_mode)),
                "directory_mode":oct(stat.S_IMODE(folder.stat().st_mode)),
                "directory_owner":folder.stat().st_uid,"directory_group":folder.stat().st_gid,
                "socket_owner":sock.stat().st_uid,"socket_group":sock.stat().st_gid}
            parts=inspect["stdout"].strip().split("|")
            raw["inspection"]["userns_mode"]=parts[1] if len(parts)==4 else "UNKNOWN"
            raw["inspection"]["service_host_pid"]=int(parts[3]) if len(parts)==4 else None
            raw["inspection"]["service_host_uid_quad"]=uid_of_host_pid(raw["inspection"]["service_host_pid"])
            raw["observations"]={}
            raw["observations"]["authorized_before"]=run_client(sock)
            raw["observations"]["rejected_connected"]=isolated_client(sock,REJECT_UID,group)
            raw["observations"]["rejected_dac"]=isolated_client(sock,DAC_UID,DAC_UID)
            raw["observations"]["authorized_after"]=run_client(sock)
            for _ in range(100):
                status=docker("inspect","--format","{{.State.Running}}",name)
                if status["stdout"].strip()=="false":break
                time.sleep(.05)
            raw["service_log"]=docker("logs",name)
            raw["service_exit"]=docker("inspect","--format","{{.State.ExitCode}}",name)
        finally:
            if created:
                raw["cleanup_container"]=docker("rm","--force",name)
            # DAC cleanup is restricted to the experiment's own directory.
            cleanup=docker("run","--rm","--pull=never","--network","none","--read-only",
                           "--cap-drop","ALL","--cap-add","CHOWN",
                           "--security-opt","no-new-privileges","--pids-limit","32",
                           "--user","0:0","--mount",mount,IMAGE,"chown",
                           f"{host_uid}:{group}","/lab")
            raw["cleanup_ownership"]=cleanup
            os.chmod(folder,0o700)
    evidence=ROOT/"evidence"/"05E"
    evidence.mkdir(parents=True,exist_ok=True)
    (evidence/"05E-raw.json").write_text(json.dumps(raw,indent=2,sort_keys=True)+"\n")
    print("CUDA05E_RAW_CAPTURED trials=1 controls=4 container_cleanup="+
          str(raw.get("cleanup_container",{}).get("code")))
if __name__=="__main__":
    main()
