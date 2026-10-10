import unittest
from gpu05_oracle03_gap_audit import audit

def trace(ms:int=13):
    token="test"
    return [
        {"type":"header","schema":"blobin.gpu05.presentation.v1",
         "feedback_origin":"wp_presentation","wl_surface_token":token},
        *[
            {"type":"feedback","frame_sequence":i,
             "wl_surface_token":token,"outcome":"presented",
             "tv_sec_hi":0,"tv_sec_lo":0,"tv_nsec":ts*1_000_000}
            for i,ts in ((1,0),(2,ms))
        ],
    ]

def stages(ms:int=13):
    data=[]
    for frame,step in ((1,0),(2,ms*1_000_000)):
        for kind,offset in (("after_queue_present",0),
                            ("frame_callback_observed",1_000_000),
                            ("compositor_present_observed",1_100_000)):
            data.append({"type":"cpu_stage","wl_surface_token":"test",
                         "stage":kind,"frame_sequence":frame,
                         "time_since_host_ready_ns":step+offset})
    return data

class WaylandPresentationGapAuditTests(unittest.TestCase):
    def test_small_gap_is_not_cp03(self):
        result=audit(trace(),stages())
        self.assertEqual(result["verdict"],"NO_MAX_GAP_VIOLATION_IN_THIS_TRACE")
        self.assertEqual(result["long_intervals_over_33_4ms"],0)
        self.assertFalse(result["cp03_pass"])
        self.assertTrue(result["same_clock_subtractions_only"])

    def test_one_large_gap_is_explicitly_red(self):
        result=audit(trace(400),stages(400))
        self.assertEqual(result["verdict"],"COMPOSITOR_MAX_GAP_GATE_RED")
        self.assertEqual(result["max_wp_gap_prev_sequence"],1)
        self.assertEqual(result["max_wp_gap_next_sequence"],2)

    def test_surface_mismatch_is_rejected(self):
        rows=trace()
        rows[2]["wl_surface_token"]="another"
        with self.assertRaisesRegex(ValueError,"CROSS_SURFACE"):
            audit(rows,stages())

    def test_timestamp_rollback_is_rejected(self):
        rows=trace()
        rows[2]["tv_nsec"]=0
        with self.assertRaisesRegex(ValueError,"NONMONOTONIC_PRESENTATION"):
            audit(rows,stages())

    def test_missing_correlated_host_stage_is_rejected(self):
        items=[x for x in stages() if not (x["frame_sequence"]==1 and x["stage"]=="frame_callback_observed")]
        with self.assertRaisesRegex(ValueError,"MISSING_OR_REVERSED"):
            audit(trace(),items)

    def test_compositor_trace_header_required(self):
        rows=trace()
        rows[0]["feedback_origin"]="queue.present"
        with self.assertRaisesRegex(ValueError,"INVALID_WP_PRESENTATION_HEADER"):
            audit(rows,stages())

    def test_host_clock_cannot_reverse(self):
        items=stages()
        items[0]["time_since_host_ready_ns"]=1_500_000
        with self.assertRaisesRegex(ValueError,"MISSING_OR_REVERSED"):
            audit(trace(),items)

if __name__=="__main__":
    unittest.main()
