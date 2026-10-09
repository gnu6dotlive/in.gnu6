#!/usr/bin/env python3
"""Derive interpretations from observations, preserving negative and missing evidence."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import subprocess

BASE=Path(__file__).resolve().parent
EVID=BASE/"evidence"

def analyze(raw):
    runs=raw["runs"]
    a,b,c=(runs[k] for k in ("baseline","namespace_root","subordinate_auto"))
    host_uid=raw["host_euid"]
    baseline_ok=a["process"]["exit_code"]==0 and a["kernel_peer"] is not None and \
        a["kernel_peer"]["uid"]==host_uid and a["client_claim"]["self_euid"]==host_uid
    namespace_ok=b["process"]["exit_code"]==0 and b["kernel_peer"] is not None
    root_ns_is_host_distinct=bool(namespace_ok and b["kernel_peer"]["uid"] != host_uid)
    root_ns_is_inside_root=bool(namespace_ok and b["client_claim"]["self_euid"] == 0)
    auto_status=("observed_peer" if c["kernel_peer"] is not None and c["process"]["exit_code"]==0
                 else "mapping_or_connect_unavailable")
    auto_outside_distinct=bool(c["kernel_peer"] is not None and c["kernel_peer"]["uid"] != host_uid)
    # A client-only test never satisfies the separate SERVER principal criterion.
    return {
        "schema":"CUDA05D.ANALYSIS.v1",
        "baseline_control":"PASS" if baseline_ok else "FAILED",
        "namespace_root_probe":"OBSERVED" if namespace_ok else "UNAVAILABLE",
        "namespace_inside_root":root_ns_is_inside_root,
        "namespace_distinct_host_uid":root_ns_is_host_distinct,
        "subordinate_mapping":auto_status,
        "subordinate_peer_distinct_host_uid":auto_outside_distinct,
        "distinct_service_identity_gate":"NOT_QUALIFIED",
        "production_authorization":"DENIED",
        "observed_not_proved":[
            "No supervisor process with a second host UID was executed.",
            "No cross-UID authorized and unauthorized socket paths were exercised.",
            "No GPU fault, driver reset or production service configuration was changed.",
        ],
        "methodological_decision":"INCOMPLETE_EVIDENCE_DO_NOT_PROMOTE",
    }

def main():
    EVID.mkdir(exist_ok=True)
    raw_bytes=(EVID/"identity-observations.json").read_bytes()
    raw=json.loads(raw_bytes)
    result=analyze(raw)
    parent_commit=subprocess.check_output(["git","-C",str(BASE),"rev-parse","HEAD"],text=True).strip()
    result["protocol_commit"]=parent_commit
    result["raw_sha256"]=hashlib.sha256(raw_bytes).hexdigest()
    (EVID/"interpretation.json").write_text(json.dumps(result,indent=2,sort_keys=True)+"\n")
    public={
        "schema":"CUDA05D.PUBLIC.v1",
        "protocol_commit":parent_commit,
        "observed_utc":raw["recorded_utc"],
        "baseline_control":result["baseline_control"],
        "namespace_root_probe":result["namespace_root_probe"],
        "namespace_inside_root":result["namespace_inside_root"],
        "namespace_distinct_host_uid":result["namespace_distinct_host_uid"],
        "subordinate_mapping":result["subordinate_mapping"],
        "subordinate_peer_distinct_host_uid":result["subordinate_peer_distinct_host_uid"],
        "distinct_service_identity_gate":result["distinct_service_identity_gate"],
        "production_authorization":result["production_authorization"],
        "raw_sha256":result["raw_sha256"],
        "source_visibility":"experiment source not necessarily publicly accessible",
        "limits":"No second host service UID; no cross-UID access proof; no physical GPU fault",
    }
    (EVID/"public-results.json").write_text(json.dumps(public,indent=2,sort_keys=True)+"\n")
    print("CUDA05D_ANALYSIS_GENERATED baseline="+result["baseline_control"]+
          " principal_gate="+result["distinct_service_identity_gate"]+
          " protocol_commit="+parent_commit[:12])

if __name__=="__main__":
    main()
