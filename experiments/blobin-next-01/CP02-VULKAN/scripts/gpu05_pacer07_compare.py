#!/usr/bin/env python3
"""PACER-07 reverse-order replay and CLOCK_MONOTONIC startup observer."""
import argparse
import json
from pathlib import Path
from gpu05_pacer06_compare import inspect

REFERENCE_REFRESH_NS=13_339_736
REVERSE_ORDER=["await-first-feedback","current"]

def check_latency(path,mode):
    r=Path(path)
    report=inspect(r,mode)
    clocks=json.loads((r/"startup-clock.json").read_text())
    presentation=json.loads((r/"presentations.jsonl").read_text().splitlines()[0])
    if clocks.get("schema")!="blobin.gpu05.startup-clock.v1":
        raise ValueError("INVALID_STARTUP_CLOCK_SCHEMA")
    if clocks.get("clock_id")!=1 or clocks.get("matched_clock")!="CLOCK_MONOTONIC":
        raise ValueError("NOT_SAME_MONOTONIC_CLOCK")
    if presentation.get("clock_id")!=clocks["clock_id"]:
        raise ValueError("CLOCK_HEADER_MISMATCH")
    if clocks.get("wl_surface_token")!=presentation.get("wl_surface_token"):
        raise ValueError("STARTUP_CLOCK_SURFACE_MISMATCH")
    if clocks.get("is_input_to_photon_latency") is not False or clocks.get("qualifies_cp03") is not False:
        raise ValueError("FORBIDDEN_LATENCY_PROMOTION")
    if clocks.get("first_frame_outcome")!=report["first_frame_outcome"]:
        raise ValueError("FIRST_FRAME_OUTCOME_MISMATCH")
    first_present=next(x for x in (json.loads(l) for l in
         (r/"presentations.jsonl").read_text().splitlines()[1:])
         if x["outcome"]=="presented")
    if first_present["frame_sequence"]!=clocks["first_compositor_present_sequence"]:
        raise ValueError("FIRST_PRESENT_SEQUENCE_MISMATCH")
    presentation_ns=((first_present["tv_sec_hi"]<<32)|first_present["tv_sec_lo"])*1_000_000_000 + first_present["tv_nsec"]
    if presentation_ns!=int(clocks["first_compositor_present_mono_ns"]):
        raise ValueError("FIRST_PRESENT_TIMESTAMP_MISMATCH")
    ready=int(clocks["app_ready_mono_ns"])
    begin=int(clocks["first_submit_begin_mono_ns"])
    returned=int(clocks["first_submit_return_mono_ns"])
    if not ready<=begin<=returned:
        raise ValueError("CPU_MONOTONIC_REVERSED")
    if presentation_ns<begin:
        raise ValueError("PRESENTED_BEFORE_QUEUED")
    measured=(presentation_ns-begin)/1e6
    if abs(measured-clocks["submit_begin_to_present_ms"])>0.00001:
        raise ValueError("BAD_FIRST_PRESENT_ARITHMETIC")
    report["latency"]=clocks
    return report

def compare(root,binary_sha256,fixture_sha256):
    p=Path(root)
    run=json.loads((p/"order.json").read_text())
    if run.get("execution_order")!=REVERSE_ORDER:
        raise ValueError("NOT_REVERSED_ORDER")
    if not all(isinstance(v,str) and len(v)==64 for v in (binary_sha256,fixture_sha256)):
        raise ValueError("MISSING_IMMUTABLE_FINGERPRINTS")
    a=check_latency(p/"await-first-feedback","await-first-feedback")
    b=check_latency(p/"current","current")
    if a["monitor"]!=b["monitor"]:
        raise ValueError("MONITOR_MISMATCH")
    if min(a["frame_count"],b["frame_count"])/max(a["frame_count"],b["frame_count"])<.8:
        raise ValueError("DURATION_NOT_MATCHED")
    return {"verdict":"REVERSED_PAIR_OBSERVED_NOT_CP03",
        "execution_order":REVERSE_ORDER,"binary_sha256":binary_sha256,
        "fixture_sha256":fixture_sha256,
        "first_frame_outcomes":{"await_first_feedback":a["first_frame_outcome"],
                                "current":b["first_frame_outcome"]},
        "actual_presented_fps":{"await_first_feedback":a["actual_presented_fps"],
                                "current":b["actual_presented_fps"]},
        "first_present_latency_ms":{"await_first_feedback":a["latency"]["submit_begin_to_present_ms"],
                                    "current":b["latency"]["submit_begin_to_present_ms"]},
        "first_present_sequences":{"await_first_feedback":a["latency"]["first_compositor_present_sequence"],
                                    "current":b["latency"]["first_compositor_present_sequence"]},
        "discarded":{"await_first_feedback":a["discarded_count"],
                     "current":b["discarded_count"]},
        "runs":{"await_first_feedback":a,"current":b},
        "input_to_photon_qualified":False,"cp03_pass":False,
        "limitations":"Only one reversed-order short pair. No independently calibrated user input-to-display latency."}

def main():
    cli=argparse.ArgumentParser()
    cli.add_argument("--root",type=Path,required=True)
    cli.add_argument("--binary-sha256",required=True)
    cli.add_argument("--fixture-sha256",required=True)
    cli.add_argument("--output",type=Path,required=True)
    args=cli.parse_args()
    obj=compare(args.root,args.binary_sha256,args.fixture_sha256)
    out=json.dumps(obj,indent=2)+"\n"
    args.output.write_text(out)
    print(out,end="")
if __name__=="__main__":main()
