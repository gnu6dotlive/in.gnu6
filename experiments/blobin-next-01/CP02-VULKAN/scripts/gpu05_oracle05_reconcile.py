#!/usr/bin/env python3
"""ORACLE-05: locally correlate native Wayland and TS read-only observations.

This is NOT physical provenance, actual Vulkan 3D pick accuracy, or CP03 PASS.
"""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path

MAX_BYTES=2_000_000
MAX_EVENTS=20_000
def read(path:Path)->list[dict]:
    if path.stat().st_size>MAX_BYTES:
        raise ValueError("ORACLE05_TRACE_TOO_LARGE")
    rows=[]
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line or len(rows)>=MAX_EVENTS:raise ValueError("ORACLE05_BAD_TRACE_LINE_LIMIT")
        row=json.loads(line)
        if not isinstance(row,dict):raise ValueError("ORACLE05_NONOBJECT_TRACE")
        rows.append(row)
    return rows

def digest(path:Path)->str:
    return hashlib.sha256(path.read_bytes()).hexdigest()

def reconcile(native:list[dict],semantic:list[dict])->dict:
    if not native or native[0].get("schema")!="blobin.gpu05.optical.renderer.v1":
        raise ValueError("ORACLE05_NATIVE_TRACE_HEADER_UNSUPPORTED")
    if not semantic or semantic[0].get("schema")!="blobin.gpu05.semantic-observation.v1":
        raise ValueError("ORACLE05_SEMANTIC_TRACE_HEADER_UNSUPPORTED")
    if sum(x.get("type")=="observer_ready" for x in semantic)!=1:
        raise ValueError("ORACLE05_NO_EXACT_HANDSHAKE")
    origin=[x for x in native if x.get("type")=="wayland_button"]
    projected=[x for x in semantic if x.get("type")=="semantic_pointer_observation"]
    if not origin or len(origin)!=len(projected):
        raise ValueError("ORACLE05_MISSING_OR_DUPLICATED_SEMANTIC_EVENTS")
    surface={x.get("wl_surface_token") for x in origin}
    if len(surface)!=1 or surface!={native[0].get("wl_surface_token")}:
        raise ValueError("ORACLE05_CROSS_SURFACE_TRACE")
    identity=[
        (x.get("provider_sequence"),"down" if x.get("pressed") else "up")
        for x in origin
    ]
    observed=[
        (x.get("provider_sequence"),x.get("phase")) for x in projected
    ]
    if identity!=observed or any(type(i[0]) is not int for i in identity):
        raise ValueError("ORACLE05_SEMANTIC_SEQUENCE_OR_PHASE_MISMATCH")
    sequences=[x[0] for x in identity]
    if any(b<=a for a,b in zip(sequences,sequences[1:])):
        raise ValueError("ORACLE05_REPLAYED_OR_REVERSED_EVENT")
    if any(
        x.get("schema")!="blobin.gpu05.semantic-observation.v1" or
        x.get("cp03_pass") is not False or
        x.get("mutation_authority_granted") is not False or
        x.get("scene_render_pose_verified") is not False or
        x.get("physical_chart_qualified") is not False or
        x.get("photodiode_qualified") is not False or
        x.get("semantic_projection_executed") is not True
        for x in projected
    ):
        raise ValueError("ORACLE05_SEMANTIC_AUTHORITY_PROMOTION_OR_MISSING_PROJECTION")
    presses=sum(x["pressed"] is True for x in origin)
    target=sum(x.get("geometric_target_candidate") is True and
               x.get("phase")=="down" for x in projected)
    return {
        "verdict":"SOFTWARE_IPC_SAME_SURFACE_SEQUENCES_MATCHED_TARGET_NOT_QUALIFIED"
                  if target==0 else "SOFTWARE_IPC_SAME_SURFACE_SEQUENCES_MATCHED",
        "native_wayland_event_count":len(origin),
        "observed_semantic_event_count":len(projected),
        "native_button_presses":presses,
        "candidate_control_press_count":target,
        "pressed_semantic_entity_counts":{
            str(k):sum(x.get("semantic_entity_id")==k and x.get("phase")=="down" for x in projected)
            for k in sorted({x.get("semantic_entity_id") for x in projected if x.get("phase")=="down"},key=str)
        },
        "sequence_and_phase_partition_exact":True,
        "independent_human_origin_qualified":False,
        "actual_display_to_scene_calibration_qualified":False,
        "visual_geometry_hit_qualified":False,
        "input_to_photon_qualified":False,
        "cp03_pass":False,
    }

def main()->int:
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument("--native",type=Path,required=True)
    p.add_argument("--semantic",type=Path,required=True)
    p.add_argument("--output",type=Path,required=True)
    args=p.parse_args()
    try:
        result=reconcile(read(args.native),read(args.semantic))
        result["native_sha256"]=digest(args.native)
        result["semantic_sha256"]=digest(args.semantic)
    except (OSError,ValueError,json.JSONDecodeError) as e:
        print(json.dumps({"verdict":"REJECTED","error":str(e),"cp03_pass":False}))
        return 2
    rendered=json.dumps(result,indent=2)+"\n"
    args.output.write_text(rendered)
    print(rendered,end="")
    return 0

if __name__=="__main__":
    raise SystemExit(main())
