#!/usr/bin/env python3
"""PACER05 pairwise comparator: fail closed on mismatched physical controls."""
import argparse
import json
import re
from pathlib import Path

from gpu05_pacer04_analyze import analyze, load

POLICIES = ("immediate", "settle75")


def verify_run(directory, expected_policy):
    path = Path(directory)
    run = (path / "run.log").read_text()
    marker = re.findall(
        r"GPU05_PACER05_STARTUP variant=(\w+) settle_ms=(\d+) window_ready=true", run
    )
    if marker != [(expected_policy, "0" if expected_policy == "immediate" else "75")]:
        raise ValueError("STARTUP_POLICY_NOT_PROVEN")
    if "GPU05_PACER03_EXIT" not in run or "aborted=false" not in run:
        raise ValueError("NATIVE_RUN_NOT_COMPLETED")
    if "GPU05_PINNED_OUTPUT name=HDMI-A-1" not in run:
        raise ValueError("OUTPUT_NOT_PINNED_BY_WAYLAND")
    windows = json.loads((path / "target-window.json").read_text())
    if len(windows) != 1:
        raise ValueError("MISSING_OR_MULTIPLE_TARGET_WINDOW")
    w = windows[0]
    if w.get("monitor") != 0 or w.get("fullscreen") != 2 or w.get("size") != [2560, 1440]:
        raise ValueError("WINDOW_GEOMETRY_MISMATCH")
    monitors = json.loads((path / "monitors.json").read_text())
    displays = [m for m in monitors if m.get("name") == "HDMI-A-1"]
    if len(displays) != 1 or displays[0].get("width") != 2560 or displays[0].get("height") != 1440:
        raise ValueError("OUTPUT_GEOMETRY_MISMATCH")
    if not 74.9 <= displays[0]["refreshRate"] <= 75.1:
        raise ValueError("OUTPUT_REFRESH_NOT_75HZ")
    fb = load(path / "presentations.jsonl")
    cb = load(path / "callbacks.jsonl")
    st = load(path / "stages.jsonl")
    result = analyze(st, fb, cb)
    if set(result["refresh_ns_distribution"]) != {13339736}:
        raise ValueError("COMPOSITOR_REFRESH_CHANGED")
    if result["callback_count"] != result["frame_count"]:
        raise ValueError("CALLBACK_PARTITION_MISMATCH")
    stage = {}
    for item in st:
        stage.setdefault(item["frame_sequence"], {})[item["stage"]] = item["time_since_host_ready_ns"]
    first = []
    for seq in (1, 2, 3, 4, 5):
        if seq not in stage:
            raise ValueError("MISSING_STARTUP_FRAME")
        v = stage[seq]
        first.append({
            "sequence": seq,
            "queue_present_call_ms": round((v["after_queue_present"] -
                v["after_feedback_arm"]) / 1e6, 4),
            "acquire_call_ms": round((v["after_acquire"] - v["before_acquire"]) / 1e6, 4),
        })
    result["startup_frames"] = first
    result["startup_policy"] = expected_policy
    return result, {k: displays[0].get(k) for k in
                    ("name", "width", "height", "refreshRate", "x", "y")}


def compare(a, b, sha256, fixture_sha256):
    if not isinstance(sha256, str) or len(sha256) != 64:
        raise ValueError("MISSING_BINARY_FINGERPRINT")
    if not isinstance(fixture_sha256, str) or len(fixture_sha256) != 64:
        raise ValueError("MISSING_FIXTURE_FINGERPRINT")
    control, monitor_a = verify_run(a, "immediate")
    variant, monitor_b = verify_run(b, "settle75")
    if monitor_a != monitor_b:
        raise ValueError("MONITOR_TOPOLOGY_CHANGED")
    span_ratio = min(control["frame_count"], variant["frame_count"]) / max(
        control["frame_count"], variant["frame_count"])
    if span_ratio < .85:
        raise ValueError("DURATION_OR_FRAME_COUNT_NOT_COMPARABLE")
    return {
        "status": "CONTROLLED_CONFIGURATION_AB_NOT_CP03",
        "binary_sha256": sha256,
        "fixture_sha256": fixture_sha256,
        "monitor": monitor_a,
        "controls": {"shader": "identical packaged binary", "window_mode": "xdg_fullscreen",
                     "changed_parameter": "startup settle 0 versus 75ms",
                     "independent_sessions": "sequential: residual scheduling confounds exist"},
        "immediate": control,
        "settle75": variant,
        "frame2_queue_present_delta_ms": round(
            variant["startup_frames"][1]["queue_present_call_ms"] -
            control["startup_frames"][1]["queue_present_call_ms"],4),
        "first_frame_discards": {
            "immediate": 1 in control["discarded_sequences"],
            "settle75": 1 in variant["discarded_sequences"]
        },
        "cp03_pass": False,
        "limit": "Short A/B cannot qualify 120s, independent input latency, or physical GPU execution timestamps",
    }


def main():
    cli=argparse.ArgumentParser()
    cli.add_argument("--a", required=True, type=Path)
    cli.add_argument("--b", required=True, type=Path)
    cli.add_argument("--binary-sha256", required=True)
    cli.add_argument("--fixture-sha256", required=True)
    cli.add_argument("--output", required=True, type=Path)
    args=cli.parse_args()
    data=compare(args.a,args.b,args.binary_sha256,args.fixture_sha256)
    result=json.dumps(data,indent=2)+"\n"
    args.output.write_text(result)
    print(result,end="")


if __name__=="__main__":
    main()
