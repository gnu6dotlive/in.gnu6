"""PACER05 comparator negative controls; no physical test duplication."""
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from gpu05_pacer05_compare import compare, verify_run


class ComparatorLawTests(unittest.TestCase):
    def test_first_party_policy_required(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            (root/"run.log").write_text("GPU05_PACER05_STARTUP variant=other settle_ms=0 window_ready=true")
            with self.assertRaisesRegex(ValueError,"STARTUP_POLICY_NOT_PROVEN"):
                verify_run(root,"immediate")

    def test_binary_fingerprint_required(self):
        with self.assertRaisesRegex(ValueError,"MISSING_BINARY_FINGERPRINT"):
            compare("a","b","", "1"*64)

    def test_fixture_fingerprint_required(self):
        with self.assertRaisesRegex(ValueError,"MISSING_FIXTURE_FINGERPRINT"):
            compare("a","b","a"*64,"")

    def test_identical_topology_is_mandatory(self):
        base = {"frame_count":100,"startup_frames":[{},
            {"queue_present_call_ms":31.0}], "discarded_sequences":[]}
        other = {**base, "startup_frames":[{}, {"queue_present_call_ms":1.0}]}
        monitor={"name":"HDMI-A-1","width":2560,"height":1440,
                 "refreshRate":74.964,"x":-630,"y":590}
        with patch("gpu05_pacer05_compare.verify_run",
                   side_effect=[(base,monitor),(other,{**monitor,"width":1920})]):
            with self.assertRaisesRegex(ValueError,"MONITOR_TOPOLOGY_CHANGED"):
                compare("a","b","a"*64,"b"*64)

    def test_one_changing_variable_reports_delta_but_never_cp03(self):
        base={"frame_count":100,"startup_frames":[{},
              {"queue_present_call_ms":31.0}],"discarded_sequences":[1]}
        other={**base,"startup_frames":[{},{"queue_present_call_ms":1.0}],
               "discarded_sequences":[]}
        monitor={"name":"HDMI-A-1","width":2560,"height":1440,
                 "refreshRate":74.964,"x":-630,"y":590}
        with patch("gpu05_pacer05_compare.verify_run",
                   side_effect=[(base,monitor),(other,monitor)]):
            result=compare("a","b","a"*64,"b"*64)
        self.assertFalse(result["cp03_pass"])
        self.assertEqual(result["frame2_queue_present_delta_ms"],-30.0)
        self.assertEqual(result["first_frame_discards"],
                         {"immediate":True,"settle75":False})


if __name__=="__main__":
    unittest.main()
