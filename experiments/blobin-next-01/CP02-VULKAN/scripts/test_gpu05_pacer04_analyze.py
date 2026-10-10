"""PACER04: synthetic negative trace qualification only (not real GPU tests)."""
import copy
import unittest
from gpu05_pacer04_analyze import analyze, STEPS

TOKEN = "pacer04-surface"


def sample():
    fb = [{"type": "header", "feedback_origin": "wp_presentation",
           "wl_surface_token": TOKEN}]
    cb = []
    stages = []
    for seq in (1, 2):
        fb.append({"type": "feedback", "frame_sequence": seq,
                   "wl_surface_token": TOKEN, "outcome": "presented",
                   "tv_sec_hi": 0, "tv_sec_lo": 20, "tv_nsec": 13_333_333 * seq,
                   "refresh_ns": 13_333_333})
        cb.append({"frame_sequence": seq, "wl_surface_token": TOKEN})
        for i, step in enumerate(STEPS):
            stages.append({"type": "cpu_stage", "frame_sequence": seq,
                           "stage": step, "wl_surface_token": TOKEN,
                           "time_since_host_ready_ns": seq*20_000_000+i*1_000_000})
        for step in ("frame_callback_observed", "compositor_present_observed"):
            stages.append({"type": "cpu_stage", "frame_sequence": seq,
                           "stage": step, "wl_surface_token": TOKEN,
                           "time_since_host_ready_ns": seq*20_000_000+8_000_000})
    return stages, fb, cb


class Pacer04AnalyzerTests(unittest.TestCase):
    def test_valid_stage_receipts_do_not_grant_cp03(self):
        report = analyze(*sample())
        self.assertEqual(report["presented"], 2)
        self.assertEqual(report["discarded_count"], 0)
        self.assertFalse(report["cp03_pass"])

    def test_missing_stage_rejected(self):
        st, fb, cb = sample()
        st.remove(next(x for x in st if x["stage"] == "after_acquire"))
        with self.assertRaisesRegex(ValueError, "MISSING_STAGE"):
            analyze(st, fb, cb)

    def test_wrong_surface_rejected(self):
        st, fb, cb = sample()
        st[0]["wl_surface_token"] = "foreign"
        with self.assertRaisesRegex(ValueError, "MIXED_SURFACE"):
            analyze(st, fb, cb)

    def test_duplicate_stage_rejected(self):
        st, fb, cb = sample()
        st.append(copy.deepcopy(st[0]))
        with self.assertRaisesRegex(ValueError, "DUPLICATED_STAGE"):
            analyze(st, fb, cb)

    def test_callback_partition_rejected(self):
        st, fb, cb = sample()
        cb.pop()
        with self.assertRaisesRegex(ValueError, "IDENTITY_MISMATCH"):
            analyze(st, fb, cb)

    def test_wrong_feedback_origin_rejected(self):
        st, fb, cb = sample()
        fb[0]["feedback_origin"] = "queue.present"
        with self.assertRaisesRegex(ValueError, "NOT_WP_PRESENTATION"):
            analyze(st, fb, cb)

    def test_clock_reversal_rejected(self):
        st, fb, cb = sample()
        st[1]["time_since_host_ready_ns"] = 0
        with self.assertRaisesRegex(ValueError, "STAGE_CLOCK_INVALID"):
            analyze(st, fb, cb)

    def test_discards_stay_red_and_identified(self):
        st, fb, cb = sample()
        fb[1]["outcome"] = "discarded"
        del fb[1]["tv_sec_hi"]
        st[7]["stage"] = "compositor_discard_observed"
        # This two-sample synthetic oracle must refuse lack of timeline.
        with self.assertRaisesRegex(ValueError, "INVALID_COMPOSITOR_TIMELINE"):
            analyze(st, fb, cb)


if __name__ == "__main__":
    unittest.main()
