#!/usr/bin/env python3
"""ORACLE-03 local gap audit: only within-domain intervals; never CP03 PASS."""
from __future__ import annotations
import argparse
import json
import math
from pathlib import Path

MAX_BYTES=4_000_000
MAX_LINES=100_000
MAX_GAP_MS=33.4

def read_jsonl(path:Path)->list[dict]:
    if path.stat().st_size>MAX_BYTES:
        raise ValueError("AUDIT_TRACE_TOO_LARGE")
    rows=[]
    with path.open(encoding="utf-8") as f:
        for line in f:
            if not line.strip() or len(rows)>=MAX_LINES:
                raise ValueError("AUDIT_INVALID_OR_OVERSIZE_TRACE")
            record=json.loads(line)
            if not isinstance(record,dict):
                raise ValueError("AUDIT_NONOBJECT")
            rows.append(record)
    return rows

def strict_int(value,name:str)->int:
    if type(value) is not int or value<0:
        raise ValueError(f"AUDIT_INVALID_{name}")
    return value

def audit(feedback:list[dict],stages:list[dict])->dict:
    presented=[]
    surface_token=None
    all_sequences=[]
    for index,row in enumerate(feedback):
        if index==0:
            if row.get("type")!="header" or row.get("schema")!="blobin.gpu05.presentation.v1" or row.get("feedback_origin")!="wp_presentation":
                raise ValueError("AUDIT_INVALID_WP_PRESENTATION_HEADER")
            surface_token=row.get("wl_surface_token")
            if not isinstance(surface_token,str) or not surface_token:
                raise ValueError("AUDIT_INVALID_SURFACE_TOKEN")
            continue
        if row.get("type")!="feedback":
            raise ValueError("AUDIT_BAD_FEEDBACK_KIND")
        seq=strict_int(row.get("frame_sequence"),"FEEDBACK_SEQUENCE")
        token=row.get("wl_surface_token")
        if not isinstance(token,str) or not token:
            raise ValueError("AUDIT_NO_OWNED_SURFACE_TOKEN")
        if surface_token is None:surface_token=token
        if surface_token!=token:
            raise ValueError("AUDIT_CROSS_SURFACE_FEEDBACK")
        all_sequences.append(seq)
        if row.get("outcome")=="presented":
            sec=(strict_int(row.get("tv_sec_hi"),"TV_HI")<<32)+strict_int(row.get("tv_sec_lo"),"TV_LO")
            ns=strict_int(row.get("tv_nsec"),"TV_NSEC")
            if ns>=1_000_000_000:raise ValueError("AUDIT_INVALID_TV_NSEC")
            presented.append((seq,sec*1_000_000_000+ns))
        elif row.get("outcome")!="discarded":
            raise ValueError("AUDIT_UNKNOWN_OUTCOME")
    if len(presented)<2 or any(y<=x for x,y in zip(all_sequences,all_sequences[1:])):
        raise ValueError("AUDIT_NONMONOTONIC_FEEDBACK_SEQUENCES")
    if any(y[1]<=x[1] for x,y in zip(presented,presented[1:])):
        raise ValueError("AUDIT_NONMONOTONIC_PRESENTATION_CLOCK")
    stage_clock={}
    accepted={"before_acquire","after_queue_present","frame_callback_observed","compositor_present_observed"}
    for row in stages:
        if row.get("type")!="cpu_stage" or row.get("wl_surface_token")!=surface_token:
            raise ValueError("AUDIT_STAGE_OTHER_SURFACE_OR_TYPE")
        key=row.get("stage")
        if key not in accepted:
            # Known stages are not needed for this audit.
            continue
        seq=strict_int(row.get("frame_sequence"),"CPU_SEQUENCE")
        tick=strict_int(row.get("time_since_host_ready_ns"),"CPU_TICK")
        pair=(seq,key)
        if pair in stage_clock:raise ValueError("AUDIT_DUPLICATE_STAGE")
        stage_clock[pair]=tick
    wp_gaps=[]
    for old,new in zip(presented,presented[1:]):
        wp_gaps.append(((new[1]-old[1])/1_000_000,old[0],new[0]))
    wp_gaps.sort(reverse=True)
    max_gap,previous_seq,next_seq=wp_gaps[0]
    details={}
    for kind,left,right,seq in (
        ("host_submit_gap_ms","after_queue_present","after_queue_present",next_seq),
        ("host_prev_callback_delay_ms","after_queue_present","frame_callback_observed",previous_seq),
        ("host_prev_feedback_observation_delay_ms","after_queue_present","compositor_present_observed",previous_seq),
    ):
        a_seq=previous_seq
        a=stage_clock.get((a_seq,left))
        b=stage_clock.get((seq,right))
        if a is None or b is None or b<a:
            raise ValueError("AUDIT_MISSING_OR_REVERSED_HOST_STAGE")
        details[kind]=round((b-a)/1e6,3)
    max_host_feedback=max(
        ((stage_clock.get((seq,"compositor_present_observed"),0)
         -stage_clock.get((seq,"after_queue_present"),0))/1e6,seq)
        for seq,_ in presented
        if (seq,"compositor_present_observed") in stage_clock
        and (seq,"after_queue_present") in stage_clock
    )
    return {
        "verdict":"COMPOSITOR_MAX_GAP_GATE_RED" if max_gap>MAX_GAP_MS
                  else "NO_MAX_GAP_VIOLATION_IN_THIS_TRACE",
        "same_surface_frame_count":len(presented),
        "long_intervals_over_33_4ms":sum(x[0]>MAX_GAP_MS for x in wp_gaps),
        "max_wp_presentation_gap_ms":round(max_gap,3),
        "max_wp_gap_prev_sequence":previous_seq,
        "max_wp_gap_next_sequence":next_seq,
        "second_largest_gap_ms":round(wp_gaps[1][0],3) if len(wp_gaps)>1 else None,
        **details,
        "longest_host_submit_to_feedback_observation_ms":round(max_host_feedback[0],3),
        "longest_host_feedback_sequence":max_host_feedback[1],
        "attribution":"UNDETERMINED_EVENT_LOOP_OR_COMPOSITOR_STALL",
        "same_clock_subtractions_only":True,
        "not_physical_photons":True,
        "cp03_pass":False,
    }

def main()->int:
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--feedback",required=True,type=Path)
    p.add_argument("--stages",required=True,type=Path)
    p.add_argument("--output",required=True,type=Path)
    a=p.parse_args()
    try:out=audit(read_jsonl(a.feedback),read_jsonl(a.stages))
    except (OSError,ValueError,json.JSONDecodeError) as exc:
        print(json.dumps({"verdict":"REJECTED","error":str(exc),"cp03_pass":False}))
        return 2
    data=json.dumps(out,indent=2)+"\n"
    a.output.write_text(data)
    print(data,end="")
    return 0

if __name__=="__main__":
    raise SystemExit(main())
