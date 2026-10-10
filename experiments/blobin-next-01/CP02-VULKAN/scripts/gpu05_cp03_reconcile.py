#!/usr/bin/env python3
"""Evidence-only CP03 reconciliation. Does not rerun GPU benchmarks or grant CP03.

Validates previously captured artifacts against their own recorded hashes.
Recorded hashes and JSON reports are not independent hardware attestation.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

MIN_FRAMES=7_200
MIN_SPAN_S=120
MIN_FPS=60

def require(ok:bool, code:str)->None:
    if not ok:
        raise ValueError(code)

def hash_file(path:Path)->str:
    sha=hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda:f.read(1024*1024),b""):
            sha.update(chunk)
    return sha.hexdigest()

def verify_trace_hashes(folder:Path, name:str,include_startup_clock:bool=False)->dict[str,str]:
    manifest=folder/"traces.sha256"
    require(manifest.is_file(),f"{name}_MISSING_TRACE_MANIFEST")
    seen={}
    expected={"presentations.jsonl","callbacks.jsonl","stages.jsonl"}
    if include_startup_clock:
        expected.add("startup-clock.json")
    for line in manifest.read_text().splitlines():
        fields=line.split(maxsplit=1)
        require(len(fields)==2,f"{name}_BAD_TRACE_MANIFEST")
        digest,recorded_path=fields
        filename=Path(recorded_path.strip("*")).name
        require(filename not in seen and len(digest)==64,
                f"{name}_DUPLICATE_OR_INVALID_HASH")
        require(filename in expected,f"{name}_UNEXPECTED_TRACE")
        path=folder/filename
        require(path.is_file(),f"{name}_MISSING_{filename}")
        require(hash_file(path)==digest,f"{name}_TRACE_HASH_MISMATCH_{filename}")
        seen[filename]=digest
    require(set(seen)==expected,
            f"{name}_INCOMPLETE_TRACE_MANIFEST")
    return seen

def reconcile_reports(endurance:dict,startup:dict,optical:dict|None=None)->dict:
    require(endurance.get("status")=="EVIDENCE_ANALYZED_NOT_CP03",
            "ENDURANCE_NOT_NATIVE_COMPOSITOR_ANALYSIS")
    require(endurance.get("cp03_pass") is False,"ENDURANCE_CANNOT_CLAIM_CP03")
    require(startup.get("verdict")=="REVERSED_PAIR_OBSERVED_NOT_CP03",
            "STARTUP_NOT_REVERSED_A_B")
    require(startup.get("cp03_pass") is False and
            startup.get("input_to_photon_qualified") is False,
            "STARTUP_INVALID_TRUTH_BOUNDARY")
    n=endurance.get("presented")
    physical_cadence=(
        type(n) is int and n>=MIN_FRAMES and
        endurance.get("frame_count")==n and
        endurance.get("callback_count")==n and
        endurance.get("discarded_count")==0 and
        type(endurance.get("actual_presented_fps")) in (int,float) and
        endurance["actual_presented_fps"]>=MIN_FPS and
        endurance.get("compositor_gap_p95_ms",100)>0 and
        endurance["compositor_gap_p95_ms"]<=17 and
        endurance.get("compositor_gap_p99_ms",100)<=20 and
        endurance.get("compositor_gap_max_ms",100)<=33.4
    )
    first_frame=(
        startup.get("execution_order")==["await-first-feedback","current"] and
        startup.get("first_frame_outcomes")=={
            "await_first_feedback":"presented","current":"discarded"
        } and startup.get("discarded")=={
            "await_first_feedback":0,"current":1
        }
    )
    optical_state="NOT_CAPTURED"
    optical_numeric_candidate=False
    if optical is not None:
        require(optical.get("schema")=="blobin.gpu05.optical-input.v1",
                "UNTRUSTED_OPTICAL_SCHEMA")
        require(optical.get("cp03_pass") is False and
                optical.get("independent_capture_provenance_qualified") is False,
                "UNAUTHORIZED_OPTICAL_PROVENANCE_PROMOTION")
        optical_numeric_candidate=optical.get("numeric_candidate") is True
        optical_state="NUMERIC_CANDIDATE_UNATTESTED" if optical_numeric_candidate else "RED_OR_INCOMPLETE"
    return {
        "verdict":"GPU05_LOCAL_EVIDENCE_RECONCILED_CP03_OPEN",
        "endurance_compositor_metrics_local_green":bool(physical_cadence),
        "endurance_120s_span_checked_externally":False,
        "reversed_startup_pair_local_green":bool(first_frame),
        "optical_input_to_photon":optical_state,
        "optical_numeric_candidate":optical_numeric_candidate,
        "optical_physical_provenance_qualified":False,
        "production_supervised_ingress_qualified":False,
        "independent_audit_qualified":False,
        "cp03_pass":False,
        "limits":[
            "Preserved summaries have local attribution, not an external independent audit",
            "Actual native Wayland input-driven photodiode experiment not executed",
            "No full CP03 promotion from either software timing or synthetic sensor data",
        ]
    }

def reconcile(endurance_dir:Path,startup_dir:Path,optical_path:Path|None)->dict:
    hashes={
        "endurance":verify_trace_hashes(endurance_dir,"ENDURANCE"),
        "startup_current":verify_trace_hashes(
            startup_dir/"current","STARTUP_CURRENT",include_startup_clock=True),
        "startup_await_first_feedback":verify_trace_hashes(
            startup_dir/"await-first-feedback","STARTUP_GATED",include_startup_clock=True),
    }
    summary=json.loads((endurance_dir/"analysis.json").read_text())
    numeric=json.loads((endurance_dir/"cp03-numeric-gate.json").read_text())
    require(numeric.get("presented_count")==summary.get("presented") and
            numeric.get("discarded_count")==summary.get("discarded_count") and
            numeric.get("observed_span_seconds",0)>=MIN_SPAN_S and
            numeric.get("cp03_pass") is False and
            numeric.get("metrics_pass") is False and
            numeric.get("input_samples_count")==0,
            "ENDURANCE_CROSS_ANALYZER_DIVERGENCE")
    startup=json.loads((startup_dir/"comparison.json").read_text())
    optical=json.loads(optical_path.read_text()) if optical_path else None
    verdict=reconcile_reports(summary,startup,optical)
    verdict["endurance_120s_span_checked_externally"]=True
    verdict["verified_trace_sha256"]=hashes
    # Hashes cannot authenticate causal origin, but detect changes since
    # the original run's SHA-256 evidence manifest.
    return verdict

def main()->int:
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--endurance",required=True,type=Path)
    p.add_argument("--startup",required=True,type=Path)
    p.add_argument("--optical-report",type=Path)
    p.add_argument("--output",required=True,type=Path)
    args=p.parse_args()
    try:
        out=reconcile(args.endurance,args.startup,args.optical_report)
    except (OSError,ValueError,json.JSONDecodeError) as error:
        print(json.dumps({"verdict":"RECONCILIATION_REJECTED",
                          "error":str(error),"cp03_pass":False},indent=2))
        return 2
    data=json.dumps(out,indent=2)+"\n"
    args.output.write_text(data)
    print(data,end="")
    return 0

if __name__=="__main__":
    raise SystemExit(main())
