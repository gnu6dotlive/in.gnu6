"""ORACLE02: synthetic software-channel falsification, not a photon sensor."""
import copy
import unittest
from gpu05_optical_scene_gate import analyze

def header():
    return {"type":"header","schema":"blobin.gpu05.optical.renderer.v1",
            "physical_capture":"NOT_RUN",
            "input_origin":"wl_pointer_button_scoped_to_own_surface",
            "semantic_authority":"unchanged_renderer_local_projection"}

def button(pressed):
    return {"type":"wayland_button","button":"BTN_LEFT","pressed":pressed,
            "timing_class":"unattested_wayland_software_input"}

def frame(seq,trial,bright,outcome="presented"):
    return [
        {"type":"roi_frame_submitted","frame_sequence":seq,"trial":trial,"bright":bright},
        {"type":"compositor_outcome","frame_sequence":seq,"trial":trial,
         "roi_bright_when_submitted":bright,"outcome":outcome,
         "time_source":"wp_presentation_not_photons"}]

def valid():
    return [header(),*frame(1,0,False),button(True),
            {"type":"probe_bright_requested","trial":1},
            *frame(2,1,True),button(False),
            {"type":"probe_dark_reset","trial":1},
            *frame(3,0,False)]

class OpticalSoftwareWitnessLawTests(unittest.TestCase):
    def test_bright_and_dark_presented_but_photon_gate_stays_open(self):
        result=analyze(valid())
        self.assertEqual(result["complete_software_input_to_compositor_trials"],1)
        self.assertEqual(result["presented_bright_frames"],1)
        self.assertFalse(result["input_to_photon_qualified"])
        self.assertFalse(result["semantic_scene_authority_qualified"])
        self.assertFalse(result["cp03_pass"])
    def test_no_input_remains_not_qualified(self):
        result=analyze([header(),*frame(1,0,False)])
        self.assertEqual(result["complete_software_input_to_compositor_trials"],0)
        self.assertFalse(result["cp03_pass"])
    def test_brighter_without_wayland_button_rejected(self):
        rows=valid()
        rows.remove(button(True))
        with self.assertRaisesRegex(ValueError,"BRIGHT_WITHOUT_WAYLAND_BUTTON"):
            analyze(rows)
    def test_software_timestamp_disguised_as_photon_rejected(self):
        rows=valid()
        rows[6]["time_source"]="physical_sensor"
        with self.assertRaisesRegex(ValueError,"FAKED_PHOTON_TIMESTAMP"):
            analyze(rows)
    def test_discarded_bright_is_not_physical_success(self):
        rows=valid()
        rows[6]["outcome"]="discarded"
        result=analyze(rows)
        self.assertEqual(result["complete_software_input_to_compositor_trials"],0)
        self.assertFalse(result["cp03_pass"])
    def test_duplicate_or_unresolved_frame_rejected(self):
        rows=valid()
        with self.assertRaisesRegex(ValueError,"UNBOUND_OR_DUPLICATED_OUTCOME"):
            analyze(rows+copy.deepcopy(rows[6:7]))
        rows=valid()
        rows.pop()
        with self.assertRaisesRegex(ValueError,"UNRESOLVED_OPTICAL_FRAMES"):
            analyze(rows)
    def test_mixed_hardware_claim_rejected(self):
        rows=valid()
        rows[0]["physical_capture"]="INDEPENDENT_PASS"
        with self.assertRaisesRegex(ValueError,"INVALID_TRUTH_BOUNDARY"):
            analyze(rows)
    def test_bright_still_lit_at_stop_rejected(self):
        rows=valid()[:7]
        with self.assertRaisesRegex(ValueError,"OPTICAL_TRIAL_LEFT_BRIGHT"):
            analyze(rows)
    def test_roi_submitted_without_active_trial_rejected(self):
        rows=[header(),*frame(1,1,True)]
        with self.assertRaisesRegex(ValueError,"BRIGHT_FRAME_WITHOUT_ACTIVE_TRIAL"):
            analyze(rows)
if __name__=="__main__":
    unittest.main()
