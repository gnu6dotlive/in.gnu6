#!/usr/bin/env python3
"""PACER-06 same-binary first-feedback gate A/B; never ratifies CP03."""
import argparse
import json
import re
from pathlib import Path
from gpu05_pacer04_analyze import analyze, load

def inspect(path, mode):
    path=Path(path)
    log=(path/"run.log").read_text()
    marker=re.findall(r"GPU05_PACER06_GATE mode=([\w-]+) max_first_wait_ms=250",log)
    if marker != [mode]:
        raise ValueError("UNQUALIFIED_STARTUP_MODE")
    if "GPU05_PACER06_XDG_FIRST_CONFIGURE_ACK" not in log or "GPU05_PACER06_CONFIGURED_BEFORE_VULKAN_SWAPCHAIN status=true" not in log:
        raise ValueError("MISSING_INITIAL_XDG_LIFECYCLE")
    if "GPU05_PACER06_FIRST_FRAME_FEEDBACK_ARMED seq=1" not in log:
        raise ValueError("FIRST_FEEDBACK_NOT_ARMED")
    if "GPU05_PACER06_EXIT" not in log or "aborted=false" not in log:
        raise ValueError("UNQUALIFIED_NATIVE_EXIT")
    monitor=json.loads((path/"monitors.json").read_text())
    outputs=[m for m in monitor if m.get("name")=="HDMI-A-1"]
    if len(outputs)!=1 or outputs[0].get("width")!=2560 or outputs[0].get("height")!=1440 or abs(outputs[0].get("refreshRate",0)-74.964)>.1:
        raise ValueError("MONITOR_MISMATCH")
    window=json.loads((path/"target-window.json").read_text())
    if len(window)!=1 or window[0].get("monitor")!=0 or window[0].get("size")!=[2560,1440] or window[0].get("fullscreen")!=2:
        raise ValueError("WINDOW_CONFIG_MISMATCH")
    report=analyze(load(path/"stages.jsonl"),load(path/"presentations.jsonl"),load(path/"callbacks.jsonl"))
    fb=load(path/"presentations.jsonl")[1:]
    first=[x for x in fb if x["frame_sequence"]==1]
    if len(first)!=1:
        raise ValueError("MISSING_FIRST_OUTCOME")
    report["first_frame_outcome"]=first[0]["outcome"]
    report["mode"]=mode
    report["monitor"]={key: outputs[0].get(key) for key in ("name","width","height","refreshRate","x","y")}
    return report

def compare(a,b,binary_hash,fixture_hash):
    if len(binary_hash)!=64 or len(fixture_hash)!=64:
        raise ValueError("MISSING_BINARY_OR_FIXTURE_HASH")
    left=inspect(a,"current")
    right=inspect(b,"await-first-feedback")
    if left["monitor"]!=right["monitor"]:
        raise ValueError("RUNS_DIFFERENT_MONITORS")
    if min(left["frame_count"],right["frame_count"])/max(left["frame_count"],right["frame_count"])<.8:
        raise ValueError("DIFFERENT_DWELL_TIME")
    return {"status":"STARTUP_GATE_AB_OBSERVED_CP03_OPEN",
            "binary_sha256":binary_hash,"fixture_sha256":fixture_hash,
            "current":left,"await_first_feedback":right,
            "first_frame_outcome_changed":left["first_frame_outcome"]!=right["first_frame_outcome"],
            "cp03_pass":False,
            "caveat":"Sequential short tests are not independently audited and are not statistical replication"}
def main():
    p=argparse.ArgumentParser()
    for k in ("a","b","output"):p.add_argument("--"+k,required=True,type=Path)
    p.add_argument("--binary-sha256",required=True)
    p.add_argument("--fixture-sha256",required=True)
    a=p.parse_args()
    result=compare(a.a,a.b,a.binary_sha256,a.fixture_sha256)
    payload=json.dumps(result,indent=2)+"\n"
    a.output.write_text(payload)
    print(payload,end="")
if __name__=="__main__":main()
