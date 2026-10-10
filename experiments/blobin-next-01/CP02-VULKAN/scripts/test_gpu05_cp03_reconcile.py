"""Synthetic negative gates for read-only existing-evidence reconciliation."""
import copy
import hashlib
import tempfile
import unittest
from pathlib import Path
from gpu05_cp03_reconcile import reconcile_reports, verify_trace_hashes

def valid():
    endurance={
        "status":"EVIDENCE_ANALYZED_NOT_CP03",
        "cp03_pass":False,
        "presented":9370,
        "frame_count":9370,
        "callback_count":9370,
        "discarded_count":0,
        "actual_presented_fps":74.956,
        "compositor_gap_p95_ms":13.478,
        "compositor_gap_p99_ms":13.58,
        "compositor_gap_max_ms":26.65
    }
    startup={
        "verdict":"REVERSED_PAIR_OBSERVED_NOT_CP03",
        "cp03_pass":False,
        "input_to_photon_qualified":False,
        "execution_order":["await-first-feedback","current"],
        "first_frame_outcomes":{"await_first_feedback":"presented","current":"discarded"},
        "discarded":{"await_first_feedback":0,"current":1}
    }
    return endurance,startup

class ReconcileContractTests(unittest.TestCase):
    def test_published_local_evidence_not_cp03(self):
        e,s=valid()
        r=reconcile_reports(e,s)
        self.assertTrue(r["endurance_compositor_metrics_local_green"])
        self.assertTrue(r["reversed_startup_pair_local_green"])
        self.assertEqual(r["optical_input_to_photon"],"NOT_CAPTURED")
        self.assertFalse(r["cp03_pass"])

    def test_discarded_frame_remains_red(self):
        e,s=valid()
        e["discarded_count"]=1
        self.assertFalse(reconcile_reports(e,s)["endurance_compositor_metrics_local_green"])

    def test_framerate_failure_does_not_hide_in_summary(self):
        e,s=valid()
        e["actual_presented_fps"]=58
        self.assertFalse(reconcile_reports(e,s)["endurance_compositor_metrics_local_green"])

    def test_startup_order_mutation_fail_closed(self):
        e,s=valid()
        s["execution_order"]=list(reversed(s["execution_order"]))
        self.assertFalse(reconcile_reports(e,s)["reversed_startup_pair_local_green"])

    def test_forged_full_cp03_assertion_rejected(self):
        e,s=valid()
        e["cp03_pass"]=True
        with self.assertRaisesRegex(ValueError,"ENDURANCE_CANNOT_CLAIM_CP03"):
            reconcile_reports(e,s)

    def test_unattested_optical_candidate_still_not_cp03(self):
        e,s=valid()
        o={"schema":"blobin.gpu05.optical-input.v1","numeric_candidate":True,
            "cp03_pass":False,"independent_capture_provenance_qualified":False}
        r=reconcile_reports(e,s,o)
        self.assertEqual(r["optical_input_to_photon"],"NUMERIC_CANDIDATE_UNATTESTED")
        self.assertFalse(r["cp03_pass"])

    def test_physical_source_promotion_is_rejected(self):
        e,s=valid()
        o={"schema":"blobin.gpu05.optical-input.v1","numeric_candidate":True,
            "cp03_pass":True,"independent_capture_provenance_qualified":True}
        with self.assertRaisesRegex(ValueError,"UNAUTHORIZED_OPTICAL_PROVENANCE_PROMOTION"):
            reconcile_reports(e,s,o)

    def test_verified_manifest_accepts_three_and_refuses_stale_hash(self):
        with tempfile.TemporaryDirectory() as temp:
            p=Path(temp)
            files=("presentations.jsonl","callbacks.jsonl","stages.jsonl")
            records=[]
            for name in files:
                data=name.encode()
                (p/name).write_bytes(data)
                records.append(f"{hashlib.sha256(data).hexdigest()}  {p/name}")
            (p/"traces.sha256").write_text("\n".join(records)+"\n")
            self.assertEqual(set(verify_trace_hashes(p,"TEST")),set(files))
            (p/"callbacks.jsonl").write_bytes(b"tamper")
            with self.assertRaisesRegex(ValueError,"TRACE_HASH_MISMATCH"):
                verify_trace_hashes(p,"TEST")

    def test_startup_manifest_requires_fourth_clock_record(self):
        with tempfile.TemporaryDirectory() as temp:
            p=Path(temp)
            names=("presentations.jsonl","callbacks.jsonl","stages.jsonl")
            records=[]
            for name in names:
                data=b"a"
                (p/name).write_bytes(data)
                records.append(f"{hashlib.sha256(data).hexdigest()}  {p/name}")
            (p/"traces.sha256").write_text("\n".join(records)+"\n")
            with self.assertRaisesRegex(ValueError,"INCOMPLETE_TRACE_MANIFEST"):
                verify_trace_hashes(p,"STARTUP",include_startup_clock=True)

if __name__=="__main__":
    unittest.main()
