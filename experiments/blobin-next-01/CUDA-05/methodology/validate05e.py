#!/usr/bin/env python3
"""Independent, read-only CUDA-05E validation. Never import its generator."""
from hashlib import sha256
import json
from pathlib import Path
import re
import subprocess
import sys
BASE=Path(__file__).resolve().parent
E=BASE/"evidence"/"05E"
FROZEN="84f82a786d80981edd824a2d9c81f4bbfd105c71"
AMEND="fbd752d322a148f6d83279bef5cf04b4f9436374"
FILES={"05E-raw.json","05E-derived.json","05E-public.json",
    "../../05E-PREREG.md","../../05E-AMENDMENT-01.md","../../05e-service.pl",
    "../../probe_principals05e.py","../../evaluate05e.py"}

def audit(raw,derived,pub):
    assert raw["schema"]=="CUDA05E.PEERS.v1"
    assert derived["schema"]=="CUDA05E.INTERPRETATION.v1"
    assert pub["schema"]=="CUDA05E.PUBLIC.v1"
    e=raw["expected"]; i=raw["inspection"]; o=raw["observations"]
    assert e["service_uid"]!=e["authorized_uid"]!=e["rejected_uid"]
    assert raw["image"]=="debian:trixie-slim"
    assert len(raw["commands"])==2
    assert all(row["result"]["code"]==0 for row in raw["commands"])
    assert i["command"]["code"]==0 and i["userns_mode"]==""
    assert i["directory_owner"]==i["socket_owner"]==e["service_uid"]
    assert i["directory_group"]==i["socket_group"]==e["shared_gid"]
    assert i["service_host_uid_quad"]==[e["service_uid"]]*4
    assert i["directory_mode"]=="0o710" and i["socket_mode"]=="0o660"
    parts=i["command"]["stdout"].strip().split("|")
    assert len(parts)==4 and parts[0]==str(e["service_uid"])+":"+str(e["shared_gid"])
    assert parts[1]=="" and parts[2]=="none" and parts[3].isdigit()
    assert o["authorized_before"]["code"]==o["authorized_after"]["code"]==0
    assert o["authorized_before"]["reply"]==o["authorized_after"]["reply"]=="ALLOWED"
    assert o["rejected_connected"]["code"]==0
    assert o["rejected_connected"]["stdout"].strip()=="DENIED"
    assert o["rejected_dac"]["code"]!=0
    assert "Permission denied" in o["rejected_dac"]["stderr"]
    logfile=raw["service_log"]
    assert logfile["code"]==0
    lines=logfile["stdout"].splitlines()
    peer_rows=[line for line in lines if line.startswith("SERVICE_OBS ")]
    assert len(peer_rows)==3
    expected=[(0,e["authorized_uid"],e["shared_gid"],"ALLOWED"),
        (1,e["rejected_uid"],e["shared_gid"],"DENIED"),
        (2,e["authorized_uid"],e["shared_gid"],"ALLOWED")]
    for line,(num,uid,gid,decision) in zip(peer_rows,expected):
        words=dict(word.split("=",1) for word in line.split()[1:])
        assert int(words["index"])==num and int(words["peer_uid"])==uid
        assert int(words["peer_gid"])==gid and words["decision"]==decision
        # PID 0 may be reported through a non-shared PID namespace. UID is authoritative here.
    assert lines[-1]=="SERVICE_EXIT clean=PASS"
    assert raw["service_exit"]["stdout"].strip()=="0"
    assert raw["cleanup_container"]["code"]==0 and raw["cleanup_ownership"]["code"]==0
    gates=("host_service_principal","container_configuration","kernel_peer_authorization",
        "group_dac_rejection","post_denial_service_continuity","lifecycle_cleanup")
    assert all(derived[g]==pub[g]=="PASS" for g in gates)
    assert derived["protocol_commit"]==pub["protocol_commit"]==FROZEN
    assert derived["amendment_commit"]==pub["amendment_commit"]==AMEND
    assert derived["production_authorization"]==pub["production_authorization"]=="DENIED"
    assert pub["review_status"]=="draft"
    assert pub["external_reproducibility"].startswith("not independently")
    assert derived["raw_sha256"]==pub["raw_sha256"]
    assert pub["observed_utc"]==raw["recorded_utc"]
    for field in ("authorized_uid","rejected_uid","service_uid","peer_pid","host_pid",
                  "socket_path","socket_owner"):
        assert field not in pub
    exported=json.dumps(pub)
    for forbidden in ("tension_atoi","/mnt/","SERVICE_OBS","newuidmap","peer_uid="):
        assert forbidden not in exported

def main():
    manifest=(E/"SHA256SUMS.txt").read_text().splitlines()
    present=set()
    for line in manifest:
        hashval,path=line.split("  ",1)
        assert path in FILES and path not in present
        assert sha256((E/path).read_bytes()).hexdigest()==hashval
        present.add(path)
    assert present==FILES
    bytes_raw=(E/"05E-raw.json").read_bytes()
    raw=json.loads(bytes_raw)
    derived=json.loads((E/"05E-derived.json").read_text())
    public=json.loads((E/"05E-public.json").read_text())
    assert derived["raw_sha256"]==sha256(bytes_raw).hexdigest()
    root=subprocess.check_output(["git","-C",str(BASE),"rev-parse","--show-toplevel"],text=True).strip()
    proto=subprocess.check_output(["git","-C",root,"show",FROZEN+
        ":experiments/blobin-next-01/CUDA-05/methodology/05E-PREREG.md"])
    assert proto==(BASE/"05E-PREREG.md").read_bytes()
    audit(raw,derived,public)
    forged=json.loads(json.dumps(raw))
    forged["inspection"]["service_host_uid_quad"]=[forged["expected"]["authorized_uid"]]*4
    try:audit(forged,derived,public)
    except AssertionError:pass
    else:raise AssertionError("forged service host UID accepted")
    print("CUDA05E_INDEPENDENT_PASS host_principals=distinct allow=2 reject_peer=1 reject_dac=1"+
          " raw_files=3 hashes=8 falsifier=REJECTED prod=DENIED")

if __name__=="__main__":
    try:main()
    except (AssertionError,KeyError,ValueError,OSError) as error:
        sys.exit("CUDA05E_AUDIT_FAIL "+str(error))
