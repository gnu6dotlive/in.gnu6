#!/usr/bin/env python3
"""Derived decision record from immutable CUDA-05E raw observation."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
ROOT=Path(__file__).resolve().parent
OUT=ROOT/"evidence"/"05E"
REL="experiments/blobin-next-01/CUDA-05/methodology/05E-PREREG.md"
def protocol_commit():
    repo=subprocess.check_output(["git","-C",str(ROOT),"rev-parse","--show-toplevel"],text=True).strip()
    matches=subprocess.check_output(["git","-C",repo,"log","--diff-filter=A","--format=%H",
        "--",REL],text=True).splitlines()
    if len(matches)!=1:raise RuntimeError("FROZEN_PROTOCOL_UNRESOLVED")
    return matches[0]
def analyze(d):
    e=d["expected"]
    v=d["observations"]
    inspect=d["inspection"]
    lines=d["service_log"]["stdout"].splitlines()
    peers=[]
    for line in lines:
        if line.startswith("SERVICE_OBS "):
            m=re.fullmatch(r"SERVICE_OBS index=(\d+) peer_uid=(\d+) peer_gid=(\d+) peer_pid=(-?\d+) decision=(ALLOWED|DENIED)",line)
            if not m:raise RuntimeError("PEER_RECORD_MALFORMED")
            peers.append(dict(index=int(m[1]),uid=int(m[2]),gid=int(m[3]),pid=int(m[4]),decision=m[5]))
    host_service=(inspect["service_host_uid_quad"]==[e["service_uid"]]*4
        and inspect["directory_owner"]==inspect["socket_owner"]==e["service_uid"])
    config_verified=(inspect["userns_mode"]=="" and
        inspect["directory_group"]==inspect["socket_group"]==e["shared_gid"] and
        inspect["directory_mode"]=="0o710" and inspect["socket_mode"]=="0o660" and
        inspect["command"]["code"]==0 and inspect["command"]["stdout"].startswith(f'{e["service_uid"]}:{e["shared_gid"]}||none|'))
    connected=(len(peers)==3 and
        [(p["index"],p["uid"],p["decision"]) for p in peers]==[
            (0,e["authorized_uid"],"ALLOWED"),(1,e["rejected_uid"],"DENIED"),(2,e["authorized_uid"],"ALLOWED")])
    replies=(v["authorized_before"]=={"code":0,"reply":"ALLOWED"}
        and v["authorized_after"]=={"code":0,"reply":"ALLOWED"}
        and v["rejected_connected"]["code"]==0
        and v["rejected_connected"]["stdout"].strip()=="DENIED")
    dac=(v["rejected_dac"]["code"]!=0 and
        "Permission denied" in v["rejected_dac"]["stderr"])
    lifetime=(d["service_exit"]["stdout"].strip()=="0" and
        "SERVICE_EXIT clean=PASS" in lines and
        d["cleanup_container"]["code"]==d["cleanup_ownership"]["code"]==0)
    distinct=e["authorized_uid"]!=e["service_uid"] and e["rejected_uid"]!=e["authorized_uid"]
    return {"schema":"CUDA05E.INTERPRETATION.v1",
        "host_service_principal":"PASS" if host_service and distinct else "NOT_QUALIFIED",
        "container_configuration":"PASS" if config_verified else "NOT_QUALIFIED",
        "kernel_peer_authorization":"PASS" if connected and replies else "NOT_QUALIFIED",
        "group_dac_rejection":"PASS" if dac else "NOT_QUALIFIED",
        "post_denial_service_continuity":"PASS" if connected and replies else "NOT_QUALIFIED",
        "lifecycle_cleanup":"PASS" if lifetime else "NOT_QUALIFIED",
        "test_population":"one disposable server, three accepted socket connections, one DAC-blocked attempt",
        "production_authorization":"DENIED",
        "limitations":["Privileged Docker daemon is an ambient authority",
            "An ephemeral scripted service, not the GPU production supervisor",
            "Only one lab repetition; no hostile daemon or hardware fault",
            "The service's PID namespace masks external peer PIDs; UID is the tested signal",
            "Digest does not provide independent access to private raw evidence"]}
def main():
    bytes_raw=(OUT/"05E-raw.json").read_bytes()
    d=json.loads(bytes_raw)
    result=analyze(d)
    result["protocol_commit"]=protocol_commit()
    result["amendment_commit"]="fbd752d322a148f6d83279bef5cf04b4f9436374"
    result["raw_sha256"]=hashlib.sha256(bytes_raw).hexdigest()
    (OUT/"05E-derived.json").write_text(json.dumps(result,indent=2,sort_keys=True)+"\n")
    public={key:result[key] for key in ("schema","protocol_commit","amendment_commit",
        "raw_sha256","host_service_principal","container_configuration",
        "kernel_peer_authorization","group_dac_rejection","post_denial_service_continuity",
        "lifecycle_cleanup","test_population","production_authorization","limitations")}
    public["schema"]="CUDA05E.PUBLIC.v1"
    public["observed_utc"]=d["recorded_utc"]
    public["external_reproducibility"]="not independently demonstrated; raw log remains internal"
    public["review_status"]="draft"
    (OUT/"05E-public.json").write_text(json.dumps(public,indent=2,sort_keys=True)+"\n")
    print("CUDA05E_DERIVED "+ " ".join(k+"="+result[k] for k in (
        "host_service_principal","kernel_peer_authorization","group_dac_rejection","lifecycle_cleanup")))
if __name__=="__main__":main()
