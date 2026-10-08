#!/usr/bin/env python3
"""Offline F4B qualification: checks every measured row, tampering and SHA-256."""
from __future__ import annotations
from pathlib import Path
import csv
import hashlib
import math

ROOT=Path(__file__).resolve().parents[1] / "fixtures" / "qualification-20261008"
SEMANTIC="b4fdb097b45a9772"
CASES={
    "dpr1-reducedfalse-modecompare-destroytrue":(20,1,False,True),
    "dpr2-reducedfalse-modecompare-destroytrue":(24,2,False,True),
    "dpr1-reducedtrue-modecompare-destroyfalse":(24,1,True,True),
    "dpr1-reducedfalse-modeauto-destroytrue":(8,1,False,False),
}
def verify_rows(rows: list[dict], samples:int,dpr:int,reduced:bool,with_gpu:bool):
    assert len(rows)==samples,(samples,len(rows))
    for i,row in enumerate(rows):
        assert int(row["sample"])==i,row
        assert int(row["dpr"])==dpr,row
        assert row["reduced"]==str(reduced).lower(),row
        assert row["semantic_digest"]==SEMANTIC,row
        assert row["backend"]==("gpu" if with_gpu else "cpu"),row
        assert (int(row["damage_x"]),int(row["damage_y"]))==(23*dpr,19*dpr),row
        assert (int(row["damage_w"]),int(row["damage_h"]))==(12*dpr,10*dpr),row
        assert int(row["changed_pixels"])==120*dpr*dpr,row
        count=i+1 if with_gpu else 0
        assert int(row["explicit_submits"])==count,row
        per=(12*dpr+7)//8 * ((10*dpr+7)//8)
        assert int(row["workgroups"])==per*count,row
        assert math.isfinite(float(row["wall_ms"])),row
        assert float(row["wall_ms"])>=0.0,row

def validate():
    assert ROOT.is_dir()
    for name,expected in CASES.items():
        with (ROOT/f"{name}.tsv").open() as fd:
            rows=list(csv.DictReader(fd,delimiter="\t"))
        verify_rows(rows,*expected)
        summary=(ROOT/f"{name}.summary.txt").read_text()
        assert summary.startswith("F4B_PASS "),name
        assert f"samples={expected[0]}" in summary,name
        assert f"idle_checks={expected[0]}" in summary,name
        assert f"dpr={expected[1]}" in summary,name
        assert f"gpu_patches={expected[0] if expected[3] else 0}" in summary,name
        assert "semantic="+SEMANTIC in summary,name
        if expected[3]:
            assert "p95_cpu_observed_gpu_roi_end_to_end_ms=Some(" in summary,name
        else:
            assert "p95_cpu_observed_gpu_roi_end_to_end_ms=None" in summary,name
        if name.endswith("destroytrue") and expected[3]:
            assert "destruction=CONTROLLED_DESTROY_CPU_CONTINUITY_GPU_REACQUIRED" in summary,name
        if not expected[3]:
            assert "destruction=NO_GPU_FOR_CONTROLLED_DESTRUCTION" in summary,name
    sums=(ROOT/"SHA256SUMS.txt").read_text().splitlines()
    assert len(sums)==8
    seen=set()
    for entry in sums:
        expected,path=entry.split("  ",1)
        assert path not in seen,path
        seen.add(path)
        assert hashlib.sha256((ROOT/path).read_bytes()).hexdigest()==expected,path
    # Verify the oracle would turn RED if a real semantic frame were lost.
    with (ROOT/"dpr1-reducedfalse-modecompare-destroytrue.tsv").open() as fd:
        counterfeit=list(csv.DictReader(fd,delimiter="\t"))
    counterfeit[7]["semantic_digest"]="0000000000000000"
    try:
        verify_rows(counterfeit,20,1,False,True)
    except AssertionError:
        pass
    else:
        raise AssertionError("F4B fixture falsifier failed")
    print("F4B_FIXTURE_GATE_PASS rows=76 sources=8 hashes=8 "
          "gpu_region_patches=68 cpu_fallback_patches=8 "
          "idle_checks=76 controlled_destroy_reacquire=2 falsifier=PASS")

if __name__=="__main__":
    validate()
