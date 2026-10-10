import unittest
from PIL import Image
from gpu05_oracle06_pixel_gate import validate,WIDTH,HEIGHT,POINTS

def case():
    im=Image.new("RGB",(WIDTH,HEIGHT),(90,163,170))
    im.putpixel(POINTS["control_center"],(89,118,139))
    im.putpixel(POINTS["control_left_border"],(111,254,229))
    im.putpixel(POINTS["control_top_border"],(111,254,229))
    return im,[{"monitor":0,"size":[WIDTH,HEIGHT],"fullscreen":2}],[{"type":"observer_ready","scene_visual_marker_contract_consistent":True}]

class PixelContracts(unittest.TestCase):
    def test_local_buffer_candidate_is_not_cp03(self):
        im,w,o=case();r=validate(im,w,o)
        self.assertEqual(r["verdict"],"COMPOSITOR_BUFFER_MARKER_CANDIDATE_PASS")
        self.assertFalse(r["actual_screen_photons_captured"])
        self.assertFalse(r["actual_pointer_visual_hit_qualified"])
        self.assertFalse(r["cp03_pass"])
    def test_black_frame_wont_pass_marker(self):
        im,w,o=case();im.paste((0,0,0),(0,0,WIDTH,HEIGHT))
        self.assertEqual(validate(im,w,o)["verdict"],"COMPOSITOR_BUFFER_MARKER_NOT_CONFIRMED")
    def test_unverified_xdg_geometry_rejected(self):
        im,w,o=case();w[0]["size"]=[640,432]
        with self.assertRaisesRegex(ValueError,"WINDOW_NOT_PROVEN"):
            validate(im,w,o)
    def test_wrong_monitor_rejected(self):
        im,w,o=case();w[0]["monitor"]=1
        with self.assertRaisesRegex(ValueError,"WINDOW_NOT_PROVEN"):
            validate(im,w,o)
    def test_semantic_chart_disagreement_rejected(self):
        im,w,o=case();o[0]["scene_visual_marker_contract_consistent"]=False
        with self.assertRaisesRegex(ValueError,"UNVERIFIED_SCENE_SHADER"):
            validate(im,w,o)
    def test_different_dimensions_rejected(self):
        im,w,o=case();small=im.crop((0,0,400,300))
        with self.assertRaisesRegex(ValueError,"WRONG_SCREENSHOT_GEOMETRY"):
            validate(small,w,o)

if __name__=="__main__":
    unittest.main()
