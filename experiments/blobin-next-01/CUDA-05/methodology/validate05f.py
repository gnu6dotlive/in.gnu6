#!/usr/bin/env python3
"""Independent CUDA-05F verification; does NOT import the acquisition or derivation scripts."""
from hashlib import sha256
import json
from pathlib import Path
import re
import subprocess
import sys

ROOT=Path(__file__).resolve().parent
E=ROOT/"evidence"/"05F"
PROTO="03ba7945fe6740b2cf35dc28656ba866f66c33fb"
def digest(data):return sha256(data).hexdigest()
def verify_run(raw):
    assert raw["schema"]=="CUDA05F.RAW.v1" and raw["experiment_id"]=="CUDA-05F-P01"
    assert raw["contract_sha256"]==digest((ROOT/"GPU-EVIDENCE-CONTRACT-v1.json").read_bytes())
    env=raw["environment"]; uid=env["authorized_uid"];service=env["service_uid"]; gid=env["gid"]
    assert uid!=service and service==65534 and env["rootful_docker"] is True
    scenarios={(x["backend"],x["scenario"]):x for x in raw["scenarios"]}
    expected={(b,k) for b in ("cuda","vulkan") for k in ("normal","crash","hang")}
    expected|={("cuda","worker_tamper"),("cuda","provider_tamper")}
    assert len(raw["scenarios"])==len(expected) and set(scenarios)==expected
    for (backend,kind),s in scenarios.items():
        assert s["backend"]==backend and s["scenario"]==kind
        assert s["setup"]["exit"]==0
        assert s["service"]["start"]["exit"]==0
        inspect=s["service"]["inspect"]
        assert inspect["exit"]==0
        values=inspect["stdout"].strip().split("|")
        assert len(values)==4 and values[0]==f"{service}:{gid}" and values[1]=="" and values[2]=="none"
        assert s["service"]["host_uid_quad"]==[service]*4
        assert s["service"]["directory"]=={"mode":"0o710","owner":service,"group":gid}
        assert s["service"]["socket"]=={"mode":"0o660","owner":service,"group":gid}
        assert s["cleanup_container"]["exit"]==0 and s["cleanup_ownership"]["exit"]==0
        assert s["service"]["exit"]["stdout"].strip()=="0"
        logs=s["service"]["log"]
        assert logs["exit"]==0 and "CUDA05F_EXIT clean=PASS" in logs["stdout"]
        assert "CUDA05C_ERROR " not in logs["stderr"]
        # Parse independent kernel peer-observation receipt emitted for every accepted socket.
        peers={}
        for line in logs["stdout"].splitlines():
            if line.startswith("CUDA05F_PEER_OBS "):
                m=re.fullmatch(r"CUDA05F_PEER_OBS index=(\d+) peer_uid=(\d+) allowed=(true|false)",line)
                assert m is not None, "malformed peer event"
                ix=int(m[1]);assert ix not in peers
                peers[ix]=(int(m[2]),m[3]=="true")
        non_dac=[obs for obs in s["observations"] if obs["intent"]!="XDAC"]
        assert len(peers)==len(non_dac)
        for index,o in enumerate(non_dac):
            # Service index counts accepted sockets. XDAC never reaches accept.
            assert peers[index]==((65533,False) if o["intent"]=="XREJECT" else (uid,True))
        for o in s["observations"]:
            assert o["index"]==s["observations"].index(o)
        expected_count=11 if kind=="normal" else 7 if kind in ("crash","hang") else 6
        assert len(s["observations"])==expected_count
    def r(b,k,index):
        return scenarios[(b,k)]["observations"][index]["response"]
    def ok(b,k,index,needle):
        v=r(b,k,index)
        return v.get("exit")==0 and needle in v.get("reply",v.get("stdout",""))
    for b in ("cuda","vulkan"):
        s=scenarios[(b,"normal")]
        assert s["observations"][4]["intent"]=="XREJECT"
        assert s["observations"][5]["intent"]=="XDAC"
        assert ok(b,"normal",0,"STATE generation=None")
        assert ok(b,"normal",1,"NO_ACTIVE_GRANT")
        for index,generation in ((2,1),(9,2)):
            assert ok(b,"normal",index,f"READY generation={generation}")
        for index,generation in ((3,1),(6,1),(10,2)):
            assert ok(b,"normal",index,
                f"RENDER_PASS generation={generation} worker=FRESH parity=PASS")
        assert ok(b,"normal",7,"REVOKED") and ok(b,"normal",8,"NO_ACTIVE_GRANT")
        assert r(b,"normal",4)["stdout"].strip()=="DENIED reason=CUDA05C_PEER_UNAUTHORIZED"
        assert r(b,"normal",5)["exit"]!=0 and "Permission denied" in r(b,"normal",5)["stderr"]
        assert (4,65533,False) in [(i,*v) for i,v in parse_peers(s).items()]
        # An attempted DAC-denied connection must not advance server accept indexing.
        assert len(parse_peers(s))==10 and parse_peers(s)[5]==(uid,True)
        for kind,code in (("crash","CUDA05C_WORKER_CRASH_OR_ERROR"),
                          ("hang","CUDA05C_WORKER_TIMEOUT_KILLED")):
            assert ok(b,kind,0,"STATE generation=None")
            assert ok(b,kind,1,"READY generation=1")
            assert ok(b,kind,2,code)
            assert ok(b,kind,3,"NO_ACTIVE_GRANT")
            assert ok(b,kind,4,"READY generation=2")
            assert ok(b,kind,5,"RENDER_PASS generation=2 worker=FRESH parity=PASS")
            assert ok(b,kind,6,"STATE generation=Some(2)")
    for kind,denial in (("worker_tamper","CUDA05C_WORKER_HASH_NOT_APPROVED"),
                        ("provider_tamper","CUDA05C_WORKER_CRASH_OR_ERROR")):
        assert ok("cuda",kind,0,"READY generation=1")
        assert ok("cuda",kind,1,"RENDER_PASS generation=1")
        assert ok("cuda",kind,2,denial)
        assert ok("cuda",kind,3,"NO_ACTIVE_GRANT")
        assert ok("cuda",kind,4,"READY generation=2")
        assert ok("cuda",kind,5,"RENDER_PASS generation=2 worker=FRESH parity=PASS")
    return {key:("NOT_QUALIFIED" if key=="release_authority" else "PASS")
            for key in json.loads((ROOT/"GPU-EVIDENCE-CONTRACT-v1.json").read_text())["required_gates"]}
def parse_peers(s):
    rows={}
    for line in s["service"]["log"]["stdout"].splitlines():
        if line.startswith("CUDA05F_PEER_OBS "):
            m=re.fullmatch(r"CUDA05F_PEER_OBS index=(\d+) peer_uid=(\d+) allowed=(true|false)",line)
            assert m
            rows[int(m[1])]=(int(m[2]),m[3]=="true")
    return rows
def check_evidence():
    raw_bytes=(E/"05F-raw.json").read_bytes()
    raw=json.loads(raw_bytes)
    states=verify_run(raw)
    d=json.loads((E/"05F-derived.json").read_text())
    p=json.loads((E/"05F-public.json").read_text())
    assert d["schema"]=="CUDA05F.DERIVED.v1"
    assert p["schema"]=="gnu6.gpu-evidence-public.v1"
    assert d["contract_sha256"]==p["contract_sha256"]==raw["contract_sha256"]
    assert d["protocol_commit"]==p["protocol_commit"]==PROTO
    assert d["raw_sha256"]==p["raw_sha256"]==digest(raw_bytes)
    assert d["cleanup"]=="PASS"
    assert d["production_authorization"]==p["production_authorization"]=="DENIED"
    assert p["review_status"]=="draft"
    assert set(d["gates"])==set(p["gates"])==set(states)
    assert all(d["gates"][key]["state"]==p["gates"][key]["state"]==val
               for key,val in states.items())
    assert all(isinstance(v["observation_ids"],list) for v in p["gates"].values())
    allowed=set(json.loads((ROOT/"GPU-EVIDENCE-CONTRACT-v1.json").read_text())["public_allowlist"])
    assert set(p)==allowed
    public=json.dumps(p)
    for item in ("/mnt/workbench","peer_uid=",'"service_host_pid"','"authorized_uid"','"socket_owner"'):
        assert item not in public
    first=json.loads((E/"05F-run01-negative.json").read_text())
    assert any(s["backend"]=="vulkan" and s["scenario"]=="normal"
        and "WORKER_CRASH_OR_ERROR" in json.dumps(s["observations"]) for s in first["scenarios"])
    second=json.loads((E/"05F-run02-rpath.json").read_text())
    assert all("CUDA05F_PEER_OBS" not in s["service"]["log"]["stdout"]
               for s in second["scenarios"])
    third=json.loads((E/"05F-run03-peer-disconnect.json").read_text())
    assert any(s["backend"]=="vulkan" and s["scenario"]=="crash"
        and s["service"]["exit"]["stdout"].strip()=="1"
        and "Broken pipe" in s["service"]["log"]["stderr"] for s in third["scenarios"])
    env=json.loads((E/"environment.json").read_text())
    assert env["schema"]=="CUDA05F.ENVIRONMENT.v1"
    assert env["contract_sha256"]==raw["contract_sha256"]
    assert env["preregistration_commit"]==PROTO
    assert env["device"]["exit"]==0 and "RTX 3070" in env["device"]["output"]
    assert env["gpu_device_loss"]=="NOT_TESTED"
    cumulative=(E/"cumulative-tests.log").read_text().splitlines()
    expected_phases=["F1","F2","F3","F4","F4B","F4C","CUDA-02","CUDA-03","CUDA-04","CUDA-05"]
    completed=[x.split("phase=",1)[1] for x in cumulative if x.startswith("PHASE_PASS phase=")]
    assert completed==expected_phases
    assert cumulative.count("CUMULATIVE_ALL_PASS phases=10")==1
    assert sum(int(x.split()[3]) for x in cumulative if x.startswith("test result: ok."))==69
    manifest=(E/"SHA256SUMS.txt").read_text().splitlines()
    expected={
        "05F-run01-negative.json","05F-run02-rpath.json","05F-run03-peer-disconnect.json",
        "05F-raw.json","05F-derived.json","05F-public.json",
        "environment.json","cumulative-tests.log",
        "../../GPU-EVIDENCE-CONTRACT-v1.json","../../05F-PREREG.md",
        "../../05F-AMENDMENT-01.md","../../05F-AMENDMENT-02.md","../../05F-AMENDMENT-03.md",
        "../../qualify05f.py","../../evaluate05f.py","../../validate05f.py",
    }
    present=set()
    for line in manifest:
        h,path=line.split("  ",1)
        assert path in expected and path not in present
        assert digest((E/path).read_bytes())==h
        present.add(path)
    assert present==expected
    repo=subprocess.check_output(["git","-C",str(ROOT),"rev-parse","--show-toplevel"],text=True).strip()
    original=subprocess.check_output(["git","-C",repo,"show",PROTO+
      ":experiments/blobin-next-01/CUDA-05/methodology/05F-PREREG.md"])
    assert original==(ROOT/"05F-PREREG.md").read_bytes()
    corrupt=json.loads(json.dumps(raw))
    corrupt["scenarios"][0]["service"]["host_uid_quad"]=[corrupt["environment"]["authorized_uid"]]*4
    try:verify_run(corrupt)
    except AssertionError:pass
    else:raise AssertionError("FALSIFIER FAILED: forged host UID accepted")
    print("CUDA05F_INDEPENDENT_PASS scenarios=8 backends=2 gates=10/10 operational release=NOT_QUALIFIED"+
          " history=3 negative_runs retained hashes=16 falsifier=REJECTED")
if __name__=="__main__":
    try:check_evidence()
    except (AssertionError,KeyError,ValueError,OSError) as e:sys.exit("CUDA05F_INDEPENDENT_FAIL "+str(e))
