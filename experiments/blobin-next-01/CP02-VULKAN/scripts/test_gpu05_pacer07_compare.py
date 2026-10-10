import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch
from gpu05_pacer07_compare import compare,check_latency

class Laws(unittest.TestCase):
    def test_reversed_order_fails_closed_when_not_reversed(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)
            (p/"order.json").write_text(json.dumps({"execution_order":["current","await-first-feedback"]}))
            with self.assertRaisesRegex(ValueError,"NOT_REVERSED_ORDER"):
                compare(p,"a"*64,"b"*64)
    def test_missing_fingerprints_fail_closed(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)
            (p/"order.json").write_text(json.dumps({"execution_order":["await-first-feedback","current"]}))
            with self.assertRaisesRegex(ValueError,"MISSING_IMMUTABLE_FINGERPRINTS"):
                compare(p,"a"*64,"")
    def test_mixed_clock_unqualified(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)
            (p/"startup-clock.json").write_text(json.dumps({
                "schema":"blobin.gpu05.startup-clock.v1",
                "clock_id":99,"matched_clock":"CLOCK_MONOTONIC"}))
            (p/"presentations.jsonl").write_text(json.dumps({
                "type":"header","clock_id":99,"wl_surface_token":"x"})+"\n")
            with patch("gpu05_pacer07_compare.inspect",return_value={"first_frame_outcome":"presented"}):
                with self.assertRaisesRegex(ValueError,"NOT_SAME_MONOTONIC_CLOCK"):
                    check_latency(p,"current")
    def test_no_input_to_photon_claim(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)
            (p/"startup-clock.json").write_text(json.dumps({
                "schema":"blobin.gpu05.startup-clock.v1",
                "clock_id":1,"matched_clock":"CLOCK_MONOTONIC"}))
            with patch("gpu05_pacer07_compare.inspect",return_value={"first_frame_outcome":"presented"}):
                with self.assertRaises(FileNotFoundError):
                    check_latency(p,"current")
if __name__=="__main__":unittest.main()
