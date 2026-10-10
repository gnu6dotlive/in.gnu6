#!/usr/bin/env python3
"""CUDA-05F derived claims. Never automatically promote a laboratory gate to production."""
from hashlib import sha256
from pathlib import Path
import json
import re

HERE=Path(__file__).resolve().parent
E=HERE/"evidence"/"05F"
PROTOCOL="03ba7945fe6740b2cf35dc28656ba866f66c33fb"
def sha(p):return sha256(p.read_bytes()).hexdigest()
def rows(s,source):
    return {r["index"]:r["response"] for r in s["observations"] if r["intent"]==source}
def result(s,index):
    return s["observations"][index]["response"]
def success(s,index,needle):
    r=result(s,index)
    return r.get("exit")==0 and needle in r.get("reply",r.get("stdout",""))
def observations(s):
    return [line for line in s["service"]["log"]["stdout"].splitlines()
            if line.startswith("CUDA05F_PEER_OBS ")]
def peer(s,index,uid,allowed):
    hit="CUDA05F_PEER_OBS index={} peer_uid={} allowed={}".format(
        index,uid,"true" if allowed else "false")
    return hit in observations(s)
def decide(raw):
    sc={(s["backend"],s["scenario"]):s for s in raw["scenarios"]}
    need={(b,k) for b in ("cuda","vulkan") for k in ("normal","crash","hang")}
    need|={("cuda","worker_tamper"),("cuda","provider_tamper")}
    if set(sc)!=need:raise ValueError("SCENARIO_INVENTORY_MISMATCH")
    host=raw["environment"]
    suid,uid,gid=host["service_uid"],host["authorized_uid"],host["gid"]
    all_sc=list(sc.values())
    principal=all(
        s["service"]["host_uid_quad"]==[suid]*4
        and s["service"]["directory"]=={"mode":"0o710","owner":suid,"group":gid}
        and s["service"]["socket"]=={"mode":"0o660","owner":suid,"group":gid}
        and s["service"]["inspect"]["stdout"].split("|")[1:3]==["","none"]
        for s in all_sc) and suid!=uid
    identity=all(peer(s,0,uid,True) for s in all_sc)
    denies=all(
        peer(sc[(b,"normal")],4,65533,False)
        and sc[(b,"normal")]["observations"][4]["response"]["stdout"].strip() ==
            "DENIED reason=CUDA05C_PEER_UNAUTHORIZED"
        for b in ("cuda","vulkan"))
    dac=all(
        "Permission denied" in sc[(b,"normal")]["observations"][5]["response"]["stderr"]
        and len(observations(sc[(b,"normal")]))==10
        for b in ("cuda","vulkan"))
    parity={}
    for b in ("cuda","vulkan"):
        s=sc[(b,"normal")]
        parity[b]=all([success(s,1,"NO_ACTIVE_GRANT"),
            success(s,2,"READY generation=1"),
            success(s,3,"RENDER_PASS generation=1 worker=FRESH parity=PASS"),
            success(s,6,"RENDER_PASS generation=1 worker=FRESH parity=PASS"),
            success(s,7,"REVOKED"),success(s,8,"NO_ACTIVE_GRANT"),
            success(s,9,"READY generation=2"),
            success(s,10,"RENDER_PASS generation=2 worker=FRESH parity=PASS"),
            "CUDA05F_EXIT clean=PASS" in s["service"]["log"]["stdout"]])
    recovered={}
    for kind,expected in (("crash","CUDA05C_WORKER_CRASH_OR_ERROR"),
                          ("hang","CUDA05C_WORKER_TIMEOUT_KILLED")):
        recovered[kind]=all(all([
            success(sc[(b,kind)],1,"READY generation=1"),
            success(sc[(b,kind)],2,expected),
            success(sc[(b,kind)],3,"NO_ACTIVE_GRANT"),
            success(sc[(b,kind)],4,"READY generation=2"),
            success(sc[(b,kind)],5,"RENDER_PASS generation=2 worker=FRESH parity=PASS"),
            success(sc[(b,kind)],6,"STATE generation=Some(2)")
        ]) for b in ("cuda","vulkan"))
    worker=sc[("cuda","worker_tamper")]
    native=sc[("cuda","provider_tamper")]
    worker_ok=all([success(worker,0,"READY generation=1"),
        success(worker,1,"RENDER_PASS"),
        success(worker,2,"CUDA05C_WORKER_HASH_NOT_APPROVED"),
        success(worker,3,"NO_ACTIVE_GRANT"),success(worker,4,"READY generation=2"),
        success(worker,5,"RENDER_PASS generation=2")])
    provider_ok=all([success(native,0,"READY generation=1"),
        success(native,1,"RENDER_PASS"),
        success(native,2,"CUDA05C_WORKER_CRASH_OR_ERROR"),
        success(native,3,"NO_ACTIVE_GRANT"),success(native,4,"READY generation=2"),
        success(native,5,"RENDER_PASS generation=2")])
    cleanup=all(
        s["service"]["exit"]["stdout"].strip()=="0"
        and s["cleanup_container"]["exit"]==0
        and s["cleanup_ownership"]["exit"]==0
        for s in all_sc)
    values={
        "host_service_uid":principal,
        "authorized_host_peer":identity,
        "rejected_connected_peer":denies,
        "dac_rejected_peer":dac,
        "cuda_gpu_parity":parity["cuda"],
        "vulkan_gpu_parity":parity["vulkan"],
        "worker_crash_recovery":recovered["crash"],
        "worker_timeout_recovery":recovered["hang"],
        "worker_binary_integrity":worker_ok,
        "provider_integrity":provider_ok,
        "release_authority":False,
    }
    gates={}
    for key,passed in values.items():
        gates[key]={
            "state":"NOT_QUALIFIED" if key=="release_authority" else ("PASS" if passed else "FAIL"),
            "observation_ids":(
                ["cuda/normal","vulkan/normal"] if key in (
                    "host_service_uid","authorized_host_peer","rejected_connected_peer",
                    "dac_rejected_peer") else
                [key.split("_")[0]+"/normal"] if key.endswith("gpu_parity") else
                ["cuda/"+key.split("_")[1],"vulkan/"+key.split("_")[1]] if key in (
                    "worker_crash_recovery","worker_timeout_recovery") else
                ["cuda/"+("worker_tamper" if key=="worker_binary_integrity" else "provider_tamper")]
                if key.endswith("integrity") else []
            ),
        }
    return gates,cleanup
def main():
    original=(E/"05F-raw.json").read_bytes()
    raw=json.loads(original)
    contract=sha(HERE/"GPU-EVIDENCE-CONTRACT-v1.json")
    assert raw["contract_sha256"]==contract
    gates,cleanup=decide(raw)
    allowed=set(json.loads((HERE/"GPU-EVIDENCE-CONTRACT-v1.json").read_text())["required_gates"])
    assert set(gates)==allowed
    derived={"schema":"CUDA05F.DERIVED.v1","experiment_id":"CUDA-05F-P01",
       "contract_sha256":contract,"protocol_commit":PROTOCOL,
       "raw_sha256":sha(E/"05F-raw.json"),"gates":gates,
       "cleanup":"PASS" if cleanup else "FAIL",
       "production_authorization":"DENIED",
       "scope":"Eight one-shot disposable Rust services, real CUDA/Vulkan workers and CPU oracle; one completed third acquisition."}
    (E/"05F-derived.json").write_text(json.dumps(derived,indent=2,sort_keys=True)+"\n")
    public={k:derived[k] for k in ("experiment_id","contract_sha256",
        "protocol_commit","raw_sha256","gates","production_authorization")}
    public.update({"schema":"gnu6.gpu-evidence-public.v1","review_status":"draft",
        "limitations":[
            "Rootful Docker privileged daemon and writable staged binary directory",
            "One completed qualifying acquisition after earlier negative and amended trials",
            "Actual device-loss and host-level malicious worker confinement not tested",
            "SO_PEERCRED identifies UID, not an independently authenticated agent",
            "Provider and worker digests from local builds are not signed provenance",
            "Native-provider substitution failure is behaviorally observed; worker stderr was suppressed",
            "Private raw observations not externally verifiable by a public checksum alone",
            "Production integration and publication explicitly unauthorized"]})
    (E/"05F-public.json").write_text(json.dumps(public,indent=2,sort_keys=True)+"\n")
    print("CUDA05F_DERIVED statuses="+",".join(k+":"+v["state"] for k,v in gates.items())+
          " cleanup="+derived["cleanup"])
if __name__=="__main__":main()
