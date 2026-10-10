#!/usr/bin/env python3
"""Independent read-only validation of software Wayland→GPU→wp_presentation.

Never upgrades the result to optical photons or in.gnu6 scene authority.
"""
from __future__ import annotations
import argparse
import json
from pathlib import Path

MAX_BYTES=8_000_000
MAX_EVENTS=40000
SCHEMA="blobin.gpu05.optical.renderer.v1"

def load(path:Path)->list[dict]:
    if path.stat().st_size>MAX_BYTES:
        raise ValueError("OPTICAL_TRACE_TOO_LARGE")
    events=[]
    with path.open(encoding="utf-8") as lines:
        for line in lines:
            if len(events)>=MAX_EVENTS:
                raise ValueError("OPTICAL_TRACE_TOO_MANY_EVENTS")
            row=json.loads(line)
            if not isinstance(row,dict):
                raise ValueError("OPTICAL_EVENT_NOT_OBJECT")
            events.append(row)
    return events

def analyze(events:list[dict])->dict:
    if not events or events[0].get("type")!="header" or events[0].get("schema")!=SCHEMA:
        raise ValueError("NOT_OWNED_OPTICAL_RENDER_TRACE")
    header=events[0]
    if (header.get("physical_capture")!="NOT_RUN" or
        header.get("input_origin")!="wl_pointer_button_scoped_to_own_surface" or
        header.get("semantic_authority")!="unchanged_renderer_local_projection"):
        raise ValueError("INVALID_TRUTH_BOUNDARY")
    active=None
    presses=0
    trials={}
    pending={}
    outcomes={}
    for e in events[1:]:
        kind=e.get("type")
        if kind=="wayland_button":
            if e.get("button")!="BTN_LEFT" or e.get("timing_class")!="unattested_wayland_software_input":
                raise ValueError("FAKED_INPUT_PROVENANCE")
            if type(e.get("pressed")) is not bool:
                raise ValueError("UNKNOWN_BUTTON_STATE")
            if e["pressed"]:
                presses+=1
        elif kind=="probe_bright_requested":
            trial=e.get("trial")
            if type(trial) is not int or trial<=0 or trial in trials:
                raise ValueError("DUPLICATE_OR_INVALID_TRIAL")
            if active is not None:
                raise ValueError("OVERLAPPING_BRIGHT_TRIALS")
            if presses<=len(trials):
                raise ValueError("BRIGHT_WITHOUT_WAYLAND_BUTTON")
            active=trial
            trials[trial]={"bright_requested":True,"bright_presented":False,"dark_reset":False,"dark_presented":False}
        elif kind=="probe_dark_reset":
            trial=e.get("trial")
            if trial!=active or trial not in trials:
                raise ValueError("ORPHAN_DARK_RESET")
            trials[trial]["dark_reset"]=True
            active=None
        elif kind=="roi_frame_submitted":
            seq=e.get("frame_sequence")
            if type(seq) is not int or seq<=0 or seq in pending:
                raise ValueError("INVALID_OR_REPEATED_FRAME")
            trial=e.get("trial")
            bright=e.get("bright")
            if type(bright) is not bool or type(trial) is not int:
                raise ValueError("INVALID_FRAME_ROI_STATE")
            if bright and trial!=active:
                raise ValueError("BRIGHT_FRAME_WITHOUT_ACTIVE_TRIAL")
            if not bright and trial!=0:
                raise ValueError("DARK_FRAME_HAS_TRIAL")
            pending[seq]=(trial,bright)
        elif kind=="compositor_outcome":
            seq=e.get("frame_sequence")
            if seq not in pending or seq in outcomes:
                raise ValueError("UNBOUND_OR_DUPLICATED_OUTCOME")
            trial,bright=pending[seq]
            if e.get("trial")!=trial or e.get("roi_bright_when_submitted")!=bright:
                raise ValueError("FRAME_ROI_OUTCOME_MISMATCH")
            if e.get("time_source")!="wp_presentation_not_photons":
                raise ValueError("FAKED_PHOTON_TIMESTAMP")
            outcome=e.get("outcome")
            if outcome not in ("presented","discarded"):
                raise ValueError("UNKNOWN_COMPOSITOR_OUTCOME")
            outcomes[seq]=outcome
            if outcome=="presented":
                if bright:
                    trials[trial]["bright_presented"]=True
                elif trials:
                    # Dark-frame proof pertains to last completed trial only.
                    completed=[t for t,v in trials.items() if v["dark_reset"]]
                    if completed:trials[max(completed)]["dark_presented"]=True
        else:
            raise ValueError("UNKNOWN_OPTICAL_EVENT")
    if set(outcomes)!=set(pending):
        raise ValueError("UNRESOLVED_OPTICAL_FRAMES")
    if active is not None:
        raise ValueError("OPTICAL_TRIAL_LEFT_BRIGHT")
    complete=sum(t["bright_presented"] and t["dark_reset"] and t["dark_presented"]
                 for t in trials.values())
    return {
        "verdict":"SOFTWARE_WAYLAND_PROJECTION_OBSERVED_NOT_PHOTONS"
                  if complete else "NO_COMPLETE_WAYLAND_OPTICAL_TRIAL",
        "wayland_pointer_presses":presses,
        "trials_requested":len(trials),
        "complete_software_input_to_compositor_trials":complete,
        "frames_with_wp_presentation_outcome":len(outcomes),
        "presented_bright_frames":sum(pending[seq][1] and result=="presented"
                                      for seq,result in outcomes.items()),
        "physical_photodiode_capture_qualified":False,
        "semantic_scene_authority_qualified":False,
        "input_to_photon_qualified":False,
        "cp03_pass":False,
    }

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace",type=Path)
    parser.add_argument("--output",type=Path)
    a=parser.parse_args()
    try:result=analyze(load(a.trace))
    except (OSError,ValueError,json.JSONDecodeError) as exc:
        print(json.dumps({"verdict":"REJECTED","reason":str(exc),"cp03_pass":False}))
        return 2
    data=json.dumps(result,indent=2)+"\n"
    if a.output:a.output.write_text(data)
    print(data,end="")
    return 0

if __name__=="__main__":
    raise SystemExit(main())
