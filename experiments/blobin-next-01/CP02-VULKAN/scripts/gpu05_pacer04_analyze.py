#!/usr/bin/env python3
"""GPU05-PACER-04: independent CPU-stage ↔ compositor-feedback trace checker.

Reports findings, NEVER confers CP03 acceptance. CPU Instant timing and the
wp_presentation clock are distinct domains: do not subtract them directly.
"""
import argparse
import collections
import json
import math
import statistics
from pathlib import Path

STEPS = ("before_acquire", "after_acquire", "before_queue_submit",
         "after_queue_submit", "after_feedback_arm", "after_queue_present")
MAX_SIZE = 32_000_000
MAX_ROWS = 200_000


def load(path: Path):
    if path.stat().st_size > MAX_SIZE:
        raise ValueError("TRACE_OVERSIZE")
    rows = []
    with path.open(encoding="utf8") as fd:
        for text in fd:
            if len(rows) >= MAX_ROWS:
                raise ValueError("TOO_MANY_EVENTS")
            rows.append(json.loads(text))
    return rows


def percentile(xs, quantile):
    if not xs:
        return None
    xs = sorted(xs)
    return round(xs[math.ceil(len(xs) * quantile) - 1], 4)


def analyze(stages, feedback, callbacks):
    if not feedback or feedback[0].get("type") != "header":
        raise ValueError("NO_COMPOSITOR_TRACE_HEADER")
    token = feedback[0]["wl_surface_token"]
    if feedback[0].get("feedback_origin") != "wp_presentation":
        raise ValueError("NOT_WP_PRESENTATION")
    fs = feedback[1:]
    cf = {r["frame_sequence"] for r in fs}
    cs = {r["frame_sequence"] for r in callbacks}
    if len(cf) != len(fs) or len(cs) != len(callbacks) or cf != cs:
        raise ValueError("FEEDBACK_CALLBACK_IDENTITY_MISMATCH")
    stage_by_id = collections.defaultdict(dict)
    for event in stages:
        if event.get("type") != "cpu_stage":
            raise ValueError("UNKNOWN_STAGE_EVENT")
        if event.get("wl_surface_token") != token:
            raise ValueError("MIXED_SURFACE")
        seq, step = event["frame_sequence"], event["stage"]
        if step in stage_by_id[seq]:
            raise ValueError("DUPLICATED_STAGE")
        stage_by_id[seq][step] = event["time_since_host_ready_ns"]
    if set(stage_by_id) != cf:
        raise ValueError("STAGE_FEEDBACK_IDENTITY_MISMATCH")
    for r in fs + callbacks:
        if r["wl_surface_token"] != token:
            raise ValueError("MIXED_SURFACE")
    durations = collections.defaultdict(list)
    for seq, frame in stage_by_id.items():
        if any(step not in frame for step in STEPS):
            raise ValueError(f"MISSING_STAGE_{seq}")
        a = [frame[step] for step in STEPS]
        if any(type(v) is not int or v < 0 for v in a) or any(b < x for x, b in zip(a, a[1:])):
            raise ValueError(f"STAGE_CLOCK_INVALID_{seq}")
        for name, left, right in (
            ("acquire", 0, 1), ("prepare_commands", 1, 2),
            ("queue_submit", 2, 3), ("feedback_arm", 3, 4),
            ("queue_present", 4, 5), ("total_frame_cpu", 0, 5)
        ):
            durations[name].append((a[right] - a[left]) / 1e6)
        if "frame_callback_observed" not in frame:
            raise ValueError(f"FRAME_CALLBACK_MISSING_{seq}")
        if not any(x in frame for x in ("compositor_present_observed", "compositor_discard_observed")):
            raise ValueError(f"COMPOSITOR_OUTCOME_MISSING_{seq}")
    presented = [r for r in fs if r["outcome"] == "presented"]
    discarded = [r["frame_sequence"] for r in fs if r["outcome"] == "discarded"]
    stamps = [(((r["tv_sec_hi"] << 32) | r["tv_sec_lo"]) * 10**9
               + r["tv_nsec"]) for r in presented]
    if stamps != sorted(stamps) or len(stamps) < 2:
        raise ValueError("INVALID_COMPOSITOR_TIMELINE")
    gaps = [(b-a)/1e6 for a, b in zip(stamps, stamps[1:])]
    refreshes = collections.Counter(r["refresh_ns"] for r in presented)
    total_ns = stamps[-1] - stamps[0]
    return {
        "status": "EVIDENCE_ANALYZED_NOT_CP03",
        "frame_count": len(fs),
        "presented": len(presented),
        "discarded_count": len(discarded),
        "discarded_sequences": discarded[:20],
        "callback_count": len(callbacks),
        "refresh_ns_distribution": dict(refreshes),
        "actual_presented_fps": round((len(stamps)-1)*1e9/total_ns,3),
        "compositor_gap_p95_ms": percentile(gaps, 0.95),
        "compositor_gap_p99_ms": percentile(gaps, 0.99),
        "compositor_gap_max_ms": round(max(gaps),4),
        "cpu_stage_ms": {name: {"median": round(statistics.median(xs),4),
                      "p95": percentile(xs,0.95),"p99": percentile(xs,0.99),
                      "max": round(max(xs),4)} for name, xs in durations.items()},
        "limit": "CPU Instant and compositor wp_presentation have distinct clocks; no GPU timestamp queries or independently measured input latency",
        "cp03_pass": False,
    }


def main():
    cli = argparse.ArgumentParser()
    for key in ("stages", "feedback", "callbacks", "output"):
        cli.add_argument("--" + key, type=Path, required=True)
    args = cli.parse_args()
    data = analyze(load(args.stages), load(args.feedback), load(args.callbacks))
    args.output.write_text(json.dumps(data, indent=2) + "\n")
    print(json.dumps(data, indent=2))


if __name__ == "__main__":
    main()
