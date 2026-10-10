#!/usr/bin/env python3
"""ORACLE-06 local compositor-framebuffer control-marker witness.

A screenshot can demonstrate colored pixels in a compositor-provided
framebuffer, NOT actual photons emitted by a display or 3D cavity picking.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path
from PIL import Image

WIDTH,HEIGHT=2560,1440
MAX_PNG_BYTES=32_000_000
POINTS={
 "control_center":(1280,720),
 "control_left_border":(1206,720),
 "control_top_border":(1280,666),
 "outside_control":(1150,720),
}
def validate(im:Image.Image,window:list[dict],observer:list[dict])->dict:
    if im.size!=(WIDTH,HEIGHT):
        raise ValueError("ORACLE06_WRONG_SCREENSHOT_GEOMETRY")
    if len(window)!=1 or window[0].get("size")!=[WIDTH,HEIGHT] or (
        window[0].get("monitor")!=0 or window[0].get("fullscreen")!=2
    ):
        raise ValueError("ORACLE06_WINDOW_NOT_PROVEN_OWNED_FULLSCREEN")
    ready=[r for r in observer if r.get("type")=="observer_ready"]
    if len(ready)!=1 or ready[0].get("scene_visual_marker_contract_consistent") is not True:
        raise ValueError("ORACLE06_UNVERIFIED_SCENE_SHADER_RECTANGLE")
    rgb=im.convert("RGB")
    colors={k:rgb.getpixel(xy) for k,xy in POINTS.items()}
    center=colors["control_center"]
    left=colors["control_left_border"]
    top=colors["control_top_border"]
    delta=sum(abs(a-b) for a,b in zip(center,left))
    top_delta=sum(abs(a-b) for a,b in zip(center,top))
    outside_delta=sum(abs(a-b) for a,b in zip(center,colors["outside_control"]))
    # Shader border is vivid cyan-green; centre darker. Compare relative
    # channels rather than asserting a display color-profile transform.
    border_signature=all(
      v[1]>v[0]+45 and v[1]>v[2]+8 for v in (left,top)
    )
    valid=border_signature and delta>90 and top_delta>90 and outside_delta>25
    return {
      "verdict":"COMPOSITOR_BUFFER_MARKER_CANDIDATE_PASS" if valid
                else "COMPOSITOR_BUFFER_MARKER_NOT_CONFIRMED",
      "window_viewport":[WIDTH,HEIGHT],
      "marker_bounds_uv":[0.47,0.46,0.53,0.54],
      "control_marker_border_colour_observed":border_signature,
      "centre_border_delta_rgb_l1":delta,
      "centre_top_delta_rgb_l1":top_delta,
      "centre_outside_delta_rgb_l1":outside_delta,
      "independent_compositor_screenshot_origin":False,
      "actual_screen_photons_captured":False,
      "actual_pointer_visual_hit_qualified":False,
      "semantic_mutation_authority":False,
      "cp03_pass":False,
    }

def main()->int:
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--screenshot",type=Path,required=True)
    p.add_argument("--window",type=Path,required=True)
    p.add_argument("--semantic",type=Path,required=True)
    p.add_argument("--output",type=Path,required=True)
    args=p.parse_args()
    try:
        if args.screenshot.stat().st_size>MAX_PNG_BYTES:
            raise ValueError("ORACLE06_PNG_TOO_LARGE")
        with Image.open(args.screenshot) as im:
            window=json.loads(args.window.read_text())
            obs=[json.loads(x) for x in args.semantic.read_text().splitlines()]
            result=validate(im,window,obs)
    except (OSError,ValueError,json.JSONDecodeError) as e:
        print(json.dumps({"verdict":"REJECTED","error":str(e),"cp03_pass":False}))
        return 2
    args.output.write_text(json.dumps(result,indent=2)+"\n")
    print(json.dumps(result,indent=2))
    return 0 if result["verdict"]=="COMPOSITOR_BUFFER_MARKER_CANDIDATE_PASS" else 3

if __name__=="__main__":
    raise SystemExit(main())
