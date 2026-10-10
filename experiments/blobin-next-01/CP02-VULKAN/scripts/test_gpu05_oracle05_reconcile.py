"""Synthetic oracle law fixtures, not input from a human."""
import copy
import unittest
from gpu05_oracle05_reconcile import reconcile

def sample():
    n=[{"type":"header","schema":"blobin.gpu05.optical.renderer.v1","wl_surface_token":"t"}]
    s=[{"type":"observer_header","schema":"blobin.gpu05.semantic-observation.v1"},
       {"type":"observer_ready"}]
    for sequence,pressed in ((1,True),(2,False)):
        n.append({"type":"wayland_button","wl_surface_token":"t",
                  "provider_sequence":sequence,"pressed":pressed})
        s.append({"type":"semantic_pointer_observation","schema":"blobin.gpu05.semantic-observation.v1",
                  "provider_sequence":sequence,"phase":"down" if pressed else "up",
                  "semantic_entity_id":"stage","geometric_target_candidate":False,
                  "semantic_projection_executed":True,
                  "cp03_pass":False,"mutation_authority_granted":False,
                  "scene_render_pose_verified":False,"physical_chart_qualified":False,
                  "photodiode_qualified":False})
    return n,s
class WireIdentityContracts(unittest.TestCase):
    def test_exact_matching_never_closes_cp03(self):
        n,s=sample()
        result=reconcile(n,s)
        self.assertEqual(result["native_wayland_event_count"],2)
        self.assertEqual(result["candidate_control_press_count"],0)
        self.assertFalse(result["cp03_pass"])
    def test_dropped_semantic_packet_rejected(self):
        n,s=sample()
        with self.assertRaisesRegex(ValueError,"MISSING_OR_DUPLICATED"):
            reconcile(n,s[:-1])
    def test_replayed_provider_sequence_rejected(self):
        n,s=sample()
        n[2]["provider_sequence"]=1
        s[3]["provider_sequence"]=1
        with self.assertRaisesRegex(ValueError,"REPLAYED"):
            reconcile(n,s)
    def test_cross_surface_native_rejected(self):
        n,s=sample()
        n[2]["wl_surface_token"]="foreign"
        with self.assertRaisesRegex(ValueError,"CROSS_SURFACE"):
            reconcile(n,s)
    def test_false_scene_authority_promotion_rejected(self):
        n,s=sample()
        s[2]["mutation_authority_granted"]=True
        with self.assertRaisesRegex(ValueError,"AUTHORITY_PROMOTION"):
            reconcile(n,s)
    def test_output_photodiode_claim_rejected(self):
        n,s=sample()
        s[2]["photodiode_qualified"]=True
        with self.assertRaisesRegex(ValueError,"AUTHORITY_PROMOTION"):
            reconcile(n,s)
if __name__=="__main__":
    unittest.main()
