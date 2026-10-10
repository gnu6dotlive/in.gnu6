"""Synthetic falsification only; NEVER qualifies real input-to-photon."""
import copy
import unittest
from gpu05_physical_oracle import analyze

UNIT="reference-dual-channel-capture-01"
CAL="verified-calibration-evidence-01"

def sample(n=30,latency_us=12_000):
    result=[{
        "type":"header",
        "schema":"blobin.gpu05.optical-input.v1",
        "timebase":"shared-hardware-counter",
        "tick_hz":1_000_000,
        "total_calibrated_uncertainty_us":2_000,
        "capture_unit_id":UNIT,
        "calibration_record_id":CAL,
        "input_sensor":"physical-switch-contact",
        "light_sensor":"external-photodiode",
        "display_transition":"dark-to-bright",
        "scene_response":"physical-switch-causes-target-pixel-brightening"
    }]
    for ntrial in range(1,n+1):
        tick=ntrial*200_000
        for channel,time in (("switch",tick),("light",tick+latency_us)):
            result.append({"type":"edge","capture_unit_id":UNIT,
                           "trial":ntrial,"channel":channel,
                           "edge":"rising","tick":time})
    return result

class OpticalHardwareOracleContract(unittest.TestCase):
    def test_numeric_candidate_never_becomes_cp03(self):
        verdict=analyze(sample())
        self.assertEqual(verdict["trial_count"],30)
        self.assertEqual(verdict["latency_p95_ms"],12)
        self.assertEqual(verdict["conservative_p95_upper_bound_ms"],14)
        self.assertTrue(verdict["numeric_candidate"])
        self.assertFalse(verdict["cp03_pass"])
        self.assertFalse(verdict["independent_capture_provenance_qualified"])
        self.assertFalse(verdict["native_input_to_scene_causality_qualified"])

    def test_29_samples_cannot_pass(self):
        self.assertFalse(analyze(sample(n=29))["numeric_candidate"])

    def test_p95_with_calibration_margin_above_budget_cannot_pass(self):
        self.assertFalse(analyze(sample(latency_us=49_000))["numeric_candidate"])

    def test_mixed_monotonic_software_timebase_rejected(self):
        rows=sample()
        rows[0]["timebase"]="clock-monotonic"
        with self.assertRaisesRegex(ValueError,"SOFTWARE_OR_MIXED_CLOCK"):
            analyze(rows)

    def test_camera_software_or_compositor_timing_not_sensor(self):
        rows=sample()
        rows[0]["light_sensor"]="wp_presentation"
        with self.assertRaisesRegex(ValueError,"NOT_INDEPENDENT_PHYSICAL"):
            analyze(rows)

    def test_large_or_missing_calibration_rejected(self):
        for val in (None,5001,True,-1):
            rows=sample()
            rows[0]["total_calibrated_uncertainty_us"]=val
            with self.subTest(val=val),self.assertRaisesRegex(ValueError,"INVALID_CALIBRATED"):
                analyze(rows)

    def test_replay_and_duplicate_light_edge_rejected(self):
        rows=sample()
        clone=copy.deepcopy(rows[2])
        clone["tick"]=rows[2]["tick"]+1
        rows.insert(3,clone)
        with self.assertRaises(ValueError):
            analyze(rows)

    def test_missing_photon_edge_rejected(self):
        rows=sample()
        rows.pop(2)
        with self.assertRaisesRegex(ValueError,"INCOMPLETE_PHYSICAL_TRIAL"):
            analyze(rows)

    def test_unordered_cross_device_ticks_rejected(self):
        rows=sample()
        rows[3]["tick"]=rows[2]["tick"]
        with self.assertRaisesRegex(ValueError,"GLOBAL_CAPTURE_TIME"):
            analyze(rows)

    def test_mismatched_capture_device_rejected(self):
        rows=sample()
        rows[3]["capture_unit_id"]="different-capture"
        with self.assertRaisesRegex(ValueError,"MIXED_CAPTURE"):
            analyze(rows)

    def test_photon_before_input_rejected(self):
        rows=sample()
        rows[1]["channel"]="light"
        rows[2]["channel"]="switch"
        with self.assertRaisesRegex(ValueError,"PHOTON_NOT_AFTER_INPUT"):
            analyze(rows)

    def test_missing_sequence_rejected(self):
        rows=sample()
        for record in rows[1:]:
            if record["trial"]>=15:
                record["trial"]+=1
        with self.assertRaisesRegex(ValueError,"MISSING_TRIAL"):
            analyze(rows)

    def test_no_input_scene_binding_rejected(self):
        rows=sample()
        rows[0]["scene_response"]="free-running-shader"
        with self.assertRaisesRegex(ValueError,"NO_INPUT_RESPONSE_BINDING"):
            analyze(rows)

    def test_noninteger_hardware_ticks_rejected(self):
        rows=sample()
        rows[1]["tick"]=10.0
        with self.assertRaisesRegex(ValueError,"BAD_TICK"):
            analyze(rows)

if __name__=="__main__":
    unittest.main()
