#!/usr/bin/env python3
"""GPU05-ORACLE-01: strict dual-channel hardware input-to-light verifier.

It analyzes physically captured edges. A JSON file is NOT proof that the
capture was genuine: an independent witness must attest device and source.
Never merge these values into a compositor trace as self-reported input events.
"""
from __future__ import annotations
import argparse
import json
import math
import sys
from pathlib import Path

SCHEMA="blobin.gpu05.optical-input.v1"
MAX_BYTES=2_000_000
MAX_RECORDS=20_000
MIN_TRIALS=30
P95_BUDGET_MS=50.0
MAX_TOTAL_UNCERTAINTY_US=5_000
MIN_TICK_HZ=100_000
MAX_TICK_HZ=100_000_000

def pctl(values:list[float], frac:float)->float:
    ordered=sorted(values)
    return ordered[math.ceil(len(ordered)*frac)-1]

def rows_from_path(path:Path)->list[dict]:
    if path.stat().st_size>MAX_BYTES:
        raise ValueError("TRACE_OVER_SIZE_LIMIT")
    out=[]
    with path.open(encoding="utf-8") as f:
        for n,line in enumerate(f,1):
            if not line.strip():
                raise ValueError(f"EMPTY_LINE_{n}")
            if len(out)>=MAX_RECORDS:
                raise ValueError("TOO_MANY_EVENTS")
            item=json.loads(line)
            if not isinstance(item,dict):
                raise ValueError(f"NON_OBJECT_{n}")
            out.append(item)
    return out

def require_int(value, code, minimum=0, maximum=2**63-1):
    if type(value) is not int or not minimum<=value<=maximum:
        raise ValueError(code)
    return value

def analyze(rows:list[dict])->dict:
    if not rows or rows[0].get("type")!="header" or rows[0].get("schema")!=SCHEMA:
        raise ValueError("MISSING_OPTICAL_HEADER")
    h=rows[0]
    if h.get("timebase")!="shared-hardware-counter":
        raise ValueError("SOFTWARE_OR_MIXED_CLOCK_NOT_ALLOWED")
    hz=require_int(h.get("tick_hz"),"INVALID_TICK_HZ",MIN_TICK_HZ,MAX_TICK_HZ)
    uncertainty=require_int(h.get("total_calibrated_uncertainty_us"),
                            "INVALID_CALIBRATED_UNCERTAINTY",0,MAX_TOTAL_UNCERTAINTY_US)
    unit=h.get("capture_unit_id")
    calibration=h.get("calibration_record_id")
    if not all(isinstance(x,str) and 8<=len(x)<=160 for x in (unit,calibration)):
        raise ValueError("MISSING_HARDWARE_AND_CALIBRATION_IDENTIFIERS")
    if h.get("input_sensor")!="physical-switch-contact" or h.get("light_sensor")!="external-photodiode":
        raise ValueError("NOT_INDEPENDENT_PHYSICAL_CHANNELS")
    if h.get("display_transition")!="dark-to-bright":
        raise ValueError("UNSUPPORTED_DISPLAY_TRANSITION")
    if h.get("scene_response")!="physical-switch-causes-target-pixel-brightening":
        raise ValueError("NO_INPUT_RESPONSE_BINDING")
    trials:dict[int,dict[str,int]]={}
    last_tick=-1
    last_trial=0
    for event in rows[1:]:
        if event.get("type")!="edge" or event.get("edge")!="rising":
            raise ValueError("UNEXPECTED_CAPTURE_EVENT")
        if event.get("capture_unit_id")!=unit:
            raise ValueError("MIXED_CAPTURE_DEVICES")
        trial=require_int(event.get("trial"),"BAD_TRIAL",1,MAX_RECORDS)
        tick=require_int(event.get("tick"),"BAD_TICK",0)
        channel=event.get("channel")
        if channel not in ("switch","light"):
            raise ValueError("UNSUPPORTED_CHANNEL")
        if tick<=last_tick:
            raise ValueError("GLOBAL_CAPTURE_TIME_NOT_INCREASING")
        last_tick=tick
        if trial<last_trial:
            raise ValueError("TRIAL_SEQUENCE_REVERSED")
        last_trial=trial
        record=trials.setdefault(trial,{})
        if channel in record:
            raise ValueError("DUPLICATE_SIGNAL_FOR_TRIAL")
        record[channel]=tick
    if not trials:
        raise ValueError("NO_PHYSICAL_TRIALS")
    if set(trials)!=set(range(1,max(trials)+1)):
        raise ValueError("MISSING_TRIAL")
    latencies=[]
    for seq,signals in sorted(trials.items()):
        if set(signals)!={"switch","light"}:
            raise ValueError(f"INCOMPLETE_PHYSICAL_TRIAL_{seq}")
        if signals["light"]<=signals["switch"]:
            raise ValueError(f"PHOTON_NOT_AFTER_INPUT_{seq}")
        latency_ms=(signals["light"]-signals["switch"])*1_000/hz
        if latency_ms>10_000:
            raise ValueError("UNBOUNDED_RESPONSE_DELAY")
        latencies.append(latency_ms)
    p95=pctl(latencies,.95)
    margin_ms=uncertainty/1_000
    candidate=(len(latencies)>=MIN_TRIALS and p95+margin_ms<=P95_BUDGET_MS)
    return {
        "schema":SCHEMA,
        "trial_count":len(latencies),
        "latency_mean_ms":round(sum(latencies)/len(latencies),5),
        "latency_p50_ms":round(pctl(latencies,.5),5),
        "latency_p95_ms":round(p95,5),
        "latency_p99_ms":round(pctl(latencies,.99),5),
        "latency_max_ms":round(max(latencies),5),
        "calibration_uncertainty_bound_ms":margin_ms,
        "conservative_p95_upper_bound_ms":round(p95+margin_ms,5),
        "numeric_candidate":candidate,
        "capture_unit_id":unit,
        "calibration_record_id":calibration,
        "independent_capture_provenance_qualified":False,
        "native_input_to_scene_causality_qualified":False,
        "cp03_pass":False,
        "verdict":"NUMERIC_CANDIDATE_AWAITING_INDEPENDENT_PHYSICAL_REVIEW" if candidate
                  else "PHYSICAL_METRICS_NOT_QUALIFIED",
        "limitations":[
            "A capture file does not independently authenticate the physical wiring, clock, calibration or trial source",
            "A switch and sensor must be observed by an independent witness, with distinct electrical/optical channels",
            "Target pixel must causally change due to the physical input, not free-running animation or a timer",
            "A webcam and wp_presentation timestamps alone are not this oracle",
            "All other ratified CP03 gates remain independently open",
        ],
    }

def main()->int:
    cli=argparse.ArgumentParser(description=__doc__)
    cli.add_argument("capture",type=Path)
    cli.add_argument("--output",type=Path)
    args=cli.parse_args()
    try:
        result=analyze(rows_from_path(args.capture))
    except (ValueError,OSError,json.JSONDecodeError) as e:
        print(json.dumps({"verdict":"TRACE_REJECTED","cp03_pass":False,"error":str(e)},indent=2))
        return 2
    output=json.dumps(result,indent=2)+"\n"
    if args.output:
        args.output.write_text(output)
    print(output,end="")
    return 0 if result["numeric_candidate"] else 1

if __name__=="__main__":
    sys.exit(main())
