#!/usr/bin/env python3
"""Synthetic falsification tests for GPU-05 trace gates; NOT physical QA."""
import copy
import unittest

from gpu05_presentation_gate import analyze, timestamp_ns

SURFACE = "gpu05-same-surface-01"
STEP_NS = 16_666_667


def event(seq, *, outcome="presented", step_ns=STEP_NS, surface=SURFACE):
    stamp = 10_000_000_000 + seq * step_ns
    seconds, nanos = divmod(stamp, 1_000_000_000)
    return {
        "type": "feedback",
        "feedback_id": seq + 1,
        "frame_sequence": seq,
        "wl_surface_token": surface,
        "outcome": outcome,
        "tv_sec_hi": seconds >> 32,
        "tv_sec_lo": seconds & 0xFFFFFFFF,
        "tv_nsec": nanos,
        "refresh_ns": STEP_NS,
    }


def header():
    return {
        "type": "header",
        "schema": "blobin.gpu05.presentation.v1",
        "feedback_origin": "wp_presentation",
        "feedback_surface_owner": "gpu05-owned-wayland-surface",
        "wl_surface_token": SURFACE,
        "clock_id": 1,
    }


def perfect_trace():
    trace = [header()]
    trace.extend(event(i) for i in range(7201))
    trace.extend({"type": "input_latency",
                  "oracle": "independent-same-clock-calibrated",
                  "latency_ms": 22.0} for _ in range(40))
    return trace


class GPU05PresentationGate(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.perfect = perfect_trace()

    def test_perfect_trace_cannot_claim_cp03_pass(self):
        result = analyze(self.perfect)
        self.assertTrue(result["metrics_pass"])
        self.assertFalse(result["cp03_pass"])
        self.assertFalse(result["independent_provenance_qualified"])
        self.assertEqual(result["presented_count"], 7201)
        self.assertGreaterEqual(result["observed_span_seconds"], 120)

    def test_wrong_surface_is_rejected(self):
        trace = list(self.perfect)
        trace[200] = event(199, surface="unrelated-wayland-surface")
        with self.assertRaisesRegex(ValueError, "MIXED_WAYLAND_SURFACES"):
            analyze(trace)

    def test_missing_feedback_sequence_is_rejected(self):
        trace = self.perfect[:]
        trace.pop(500)
        with self.assertRaisesRegex(ValueError, "MISSING_FEEDBACK_SEQUENCE"):
            analyze(trace)

    def test_duplicate_feedback_cannot_inflate_frame_count(self):
        trace = self.perfect[:]
        trace.insert(200, copy.deepcopy(trace[199]))
        with self.assertRaisesRegex(ValueError, "DUPLICATE"):
            analyze(trace)

    def test_discarded_frame_is_not_presented(self):
        trace = self.perfect[:]
        trace[200] = event(199, outcome="discarded")
        result = analyze(trace)
        self.assertEqual(result["discarded_count"], 1)
        self.assertEqual(result["presented_count"], 7200)
        self.assertFalse(result["metrics_pass"])

    def test_missing_independent_input_latency_fails_closed(self):
        trace = [e for e in self.perfect if e["type"] != "input_latency"]
        result = analyze(trace)
        self.assertIsNone(result["input_latency_p95_ms"])
        self.assertFalse(result["metrics_pass"])

    def test_uncalibrated_input_clock_is_rejected(self):
        trace = self.perfect[:]
        trace[-1] = {"type": "input_latency", "oracle": "wl_keyboard-event-time",
                     "latency_ms": 20}
        with self.assertRaisesRegex(ValueError, "INPUT_CLOCK_NOT_CALIBRATED"):
            analyze(trace)

    def test_queue_present_is_not_wp_presentation(self):
        trace = self.perfect[:]
        trace[0] = {**header(), "feedback_origin": "queue.present"}
        with self.assertRaisesRegex(ValueError, "NOT_SAME_SURFACE_FEEDBACK"):
            analyze(trace)

    def test_nonmonotonic_presented_timestamps_are_rejected(self):
        trace = self.perfect[:]
        trace[200] = {**trace[200], "tv_sec_lo": 10, "tv_nsec": 1}
        with self.assertRaisesRegex(ValueError, "NONMONOTONIC_PRESENTATION_TIME"):
            analyze(trace)

    def test_p99_jank_blocks_metrics(self):
        trace = perfect_trace()
        for i in range(7201):
            # Smooth 60Hz for most samples; concentrated >33.4ms gap is disqualifying.
            if i >= 3500:
                trace[i + 1]["tv_sec_lo"] += 1
        result = analyze(trace)
        self.assertGreater(result["interarrival_max_ms"], 33.4)
        self.assertFalse(result["metrics_pass"])

    def test_invalid_nanoseconds_rejected(self):
        with self.assertRaisesRegex(ValueError, "NANOSECOND_OUT_OF_RANGE"):
            timestamp_ns({**event(10), "tv_nsec": 1_000_000_000})

    def test_missing_120_second_span_does_not_pass(self):
        trace = [header()]
        trace.extend(event(i, step_ns=8_000_000) for i in range(7201))
        trace.extend(e for e in self.perfect if e["type"] == "input_latency")
        self.assertFalse(analyze(trace)["metrics_pass"])

    def test_input_latency_must_pass(self):
        trace = perfect_trace()
        # At least 3/40 samples must exceed 50 ms to push nearest-rank p95 over the gate.
        for sample in trace[-4:]:
            sample["latency_ms"] = 80
        result = analyze(trace)
        self.assertFalse(result["metrics_pass"])


if __name__ == "__main__":
    unittest.main()
