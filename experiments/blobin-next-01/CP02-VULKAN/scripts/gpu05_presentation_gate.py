#!/usr/bin/env python3
"""GPU-05 wp_presentation receipt analyzer. Never promotes itself to CP03 PASS.

One JSON object per line. This is an untrusted metrics parser; the independent
capture/origin attestation MUST be verified outside this program.
"""
from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

SCHEMA = "blobin.gpu05.presentation.v1"
MAX_RECEIPTS = 100_000
MAX_BYTES = 32_000_000
MIN_PRESENTED = 7_200
MIN_INPUT_SAMPLES = 30
MIN_SPAN_NS = 120_000_000_000
P95_MS = 17.0
P99_MS = 20.0
MAX_GAP_MS = 33.4
INPUT_P95_MS = 50.0


def percentile(values: list[float], fraction: float) -> float:
    if not values:
        raise ValueError("NO_SAMPLES")
    ordered = sorted(values)
    return ordered[math.ceil(len(ordered) * fraction) - 1]


def timestamp_ns(event: dict) -> int:
    hi, lo, nsec = (event.get(k) for k in ("tv_sec_hi", "tv_sec_lo", "tv_nsec"))
    if not all(type(v) is int and 0 <= v < 2**32 for v in (hi, lo, nsec)):
        raise ValueError("INVALID_TIMESTAMP_COMPONENT")
    if nsec >= 1_000_000_000:
        raise ValueError("NANOSECOND_OUT_OF_RANGE")
    return ((((hi << 32) | lo) * 1_000_000_000) + nsec)


def load_jsonl(path: Path) -> list[dict]:
    if path.stat().st_size > MAX_BYTES:
        raise ValueError("TRACE_OVERSIZE")
    records = []
    with path.open("r", encoding="utf8") as source:
        for line_number, line in enumerate(source, 1):
            if not line.strip():
                raise ValueError(f"EMPTY_LINE_{line_number}")
            if len(records) >= MAX_RECEIPTS:
                raise ValueError("TOO_MANY_RECORDS")
            item = json.loads(line)
            if not isinstance(item, dict):
                raise ValueError(f"NOT_AN_OBJECT_{line_number}")
            records.append(item)
    if not records:
        raise ValueError("EMPTY_TRACE")
    return records


def analyze(records: list[dict]) -> dict:
    header = records[0]
    if header.get("type") != "header" or header.get("schema") != SCHEMA:
        raise ValueError("INVALID_HEADER")
    if header.get("feedback_origin") != "wp_presentation" or header.get(
        "feedback_surface_owner"
    ) != "gpu05-owned-wayland-surface":
        raise ValueError("NOT_SAME_SURFACE_FEEDBACK")
    surface = header.get("wl_surface_token")
    clock = header.get("clock_id")
    if not isinstance(surface, str) or len(surface) < 8 or type(clock) is not int:
        raise ValueError("MISSING_SURFACE_OR_CLOCK_PROVENANCE")
    if not 0 <= clock < 2**32:
        raise ValueError("INVALID_CLOCK")
    events = records[1:]
    seen_feedback_ids: set[int] = set()
    seen_sequences: set[int] = set()
    presented: list[tuple[int, int]] = []
    discarded = 0
    input_ms: list[float] = []
    for event in events:
        kind = event.get("type")
        if kind == "feedback":
            if event.get("wl_surface_token") != surface:
                raise ValueError("MIXED_WAYLAND_SURFACES")
            feedback_id, sequence = event.get("feedback_id"), event.get("frame_sequence")
            if (type(feedback_id) is not int or type(sequence) is not int
                or feedback_id <= 0 or sequence < 0
                or feedback_id in seen_feedback_ids or sequence in seen_sequences):
                raise ValueError("DUPLICATE_OR_INVALID_FEEDBACK_IDENTITY")
            seen_feedback_ids.add(feedback_id)
            seen_sequences.add(sequence)
            outcome = event.get("outcome")
            if outcome == "discarded":
                discarded += 1
            elif outcome == "presented":
                if (type(event.get("refresh_ns")) is not int or
                    event["refresh_ns"] < 0 or event["refresh_ns"] > 500_000_000):
                    raise ValueError("INVALID_PRESENTATION_REFRESH")
                presented.append((sequence, timestamp_ns(event)))
            else:
                raise ValueError("UNTERMINATED_OR_UNKNOWN_FEEDBACK")
        elif kind == "input_latency":
            # Independent input-to-photon oracle. A bare Wayland key time is
            # NOT comparable with the wp_presentation clock_id.
            if event.get("oracle") != "independent-same-clock-calibrated":
                raise ValueError("INPUT_CLOCK_NOT_CALIBRATED")
            value = event.get("latency_ms")
            if not isinstance(value, (int, float)) or isinstance(value, bool):
                raise ValueError("INVALID_INPUT_LATENCY")
            if not math.isfinite(value) or not 0 <= value <= 10_000:
                raise ValueError("INVALID_INPUT_LATENCY")
            input_ms.append(float(value))
        else:
            raise ValueError(f"UNSUPPORTED_EVENT_{kind}")
    if seen_sequences and (max(seen_sequences) - min(seen_sequences) + 1 != len(seen_sequences)):
        raise ValueError("MISSING_FEEDBACK_SEQUENCE")
    if len(presented) < 2:
        raise ValueError("NO_PRESENTATION_TIMELINE")
    presented.sort()
    times = [stamp for _, stamp in presented]
    if any(b <= a for a, b in zip(times, times[1:])):
        raise ValueError("NONMONOTONIC_PRESENTATION_TIME")
    intervals_ms = [(b - a) / 1_000_000 for a, b in zip(times, times[1:])]
    span = (times[-1] - times[0]) / 1_000_000_000
    p95 = percentile(intervals_ms, 0.95)
    p99 = percentile(intervals_ms, 0.99)
    maximum = max(intervals_ms)
    input_p95 = percentile(input_ms, 0.95) if input_ms else None
    metrics_pass = (
        len(presented) >= MIN_PRESENTED and
        discarded == 0 and
        times[-1] - times[0] >= MIN_SPAN_NS and
        p95 <= P95_MS and p99 <= P99_MS and maximum <= MAX_GAP_MS and
        len(input_ms) >= MIN_INPUT_SAMPLES and
        input_p95 is not None and input_p95 <= INPUT_P95_MS
    )
    return {
        "schema": SCHEMA,
        "surface_token": surface,
        "clock_id": clock,
        "feedback_count": len(seen_feedback_ids),
        "presented_count": len(presented),
        "discarded_count": discarded,
        "observed_span_seconds": round(span, 6),
        "interarrival_p95_ms": round(p95, 6),
        "interarrival_p99_ms": round(p99, 6),
        "interarrival_max_ms": round(maximum, 6),
        "input_samples_count": len(input_ms),
        "input_latency_p95_ms": input_p95,
        "metrics_pass": metrics_pass,
        "independent_provenance_qualified": False,
        "cp03_pass": False,
        "verdict": "METRICS_CANDIDATE_PROVENANCE_PENDING" if metrics_pass
                   else "NOT_QUALIFIED",
        "limits": [
            "JSONL source assertions are not independently authenticated",
            "Clock domain and wl_surface linkage require host capture evidence",
            "All other CP03 gates remain external to this metrics parser",
        ],
    }


def main() -> int:
    cli = argparse.ArgumentParser(description=__doc__)
    cli.add_argument("trace", type=Path)
    cli.add_argument("--output", type=Path)
    args = cli.parse_args()
    try:
        result = analyze(load_jsonl(args.trace))
    except (ValueError, OSError, json.JSONDecodeError) as exc:
        print(json.dumps({"verdict": "TRACE_REJECTED", "error": str(exc),
                          "cp03_pass": False}, indent=2))
        return 2
    encoded = json.dumps(result, indent=2) + "\n"
    if args.output:
        args.output.write_text(encoded)
    print(encoded, end="")
    return 0 if result["metrics_pass"] else 1


if __name__ == "__main__":
    sys.exit(main())
