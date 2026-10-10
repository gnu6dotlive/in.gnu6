"""New protocol-order A/B comparison falsification tests."""
import unittest
from unittest.mock import patch
from gpu05_pacer06_compare import compare
class Gates(unittest.TestCase):
    def test_hashes_required(self):
        with self.assertRaisesRegex(ValueError,"MISSING_BINARY_OR_FIXTURE_HASH"):
            compare("a","b","","0"*64)
    def test_monitor_match_is_mandatory(self):
        a={"monitor":{"name":"HDMI-A-1","width":2560},"frame_count":100,
           "first_frame_outcome":"discarded"}
        b={**a,"monitor":{"name":"DP-2","width":1920}}
        with patch("gpu05_pacer06_compare.inspect",side_effect=[a,b]):
            with self.assertRaisesRegex(ValueError,"RUNS_DIFFERENT_MONITORS"):
                compare("a","b","0"*64,"1"*64)
    def test_observed_first_frame_cannot_ratify_cp03(self):
        a={"monitor":{"name":"HDMI-A-1"},"frame_count":100,"first_frame_outcome":"discarded"}
        b={**a,"first_frame_outcome":"presented"}
        with patch("gpu05_pacer06_compare.inspect",side_effect=[a,b]):
            out=compare("a","b","0"*64,"1"*64)
        self.assertTrue(out["first_frame_outcome_changed"])
        self.assertFalse(out["cp03_pass"])
if __name__=="__main__":unittest.main()
