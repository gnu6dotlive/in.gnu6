#!/usr/bin/env python3
"""Offline CUDA-01 numerical qualification, run-log hash, negative falsifier."""
import hashlib
import math
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]/"fixtures"/"qualification-20261008"
EXPECTED={(count,pixels) for count in (16,64,256) for pixels in (49152,196608)}
def check(lines):
    assert len(lines)==6,len(lines)
    seen=set()
    for line in lines:
        assert line.startswith("CUDA01_PASS "),line
        assert "device=NVIDIA GeForce RTX 3070 cc=8.6" in line,line
        d=dict(part.split("=",1) for part in line.split() if part.count("=")==1)
        key=(int(d["primitives"]),int(d["pixels"]))
        assert key in EXPECTED and key not in seen,key
        seen.add(key)
        assert int(d["repeat"])==20,d
        assert int(d["bad_mask"])==0,d
        assert int(d["ambiguous_boundary"])==0,d
        assert d["falsifier"]=="PASS",d
        assert 0<=float(d["max_field"])<=1e-4,d
        assert 0<=float(d["max_normal"])<=0.006,d
        assert 0<=float(d["max_coverage"])<=0.007,d
        for field in ("cpu_single_ms","gpu_kernel_mean_ms","gpu_one_shot_transfer_included_ms"):
            value=float(d[field])
            assert math.isfinite(value) and value>0,(field,value)
    assert seen==EXPECTED,seen
def main():
    path=ROOT/"results.log"
    data=path.read_bytes()
    lines=data.decode().strip().splitlines()
    check(lines)
    sums=(ROOT/"SHA256SUMS.txt").read_text().strip().splitlines()
    assert len(sums)==1
    digest,name=sums[0].split("  ",1)
    assert name=="results.log"
    assert hashlib.sha256(data).hexdigest()==digest
    altered=lines.copy()
    altered[5]=altered[5].replace("bad_mask=0","bad_mask=1")
    try:check(altered)
    except AssertionError:pass
    else:raise AssertionError("negative CUDA mask mutation was not rejected")
    print("CUDA01_FIXTURE_PASS scenarios=6 primitives=16,64,256 "
          "resolutions=2 mask_disagreements=0 float_oracles=PASS "
          "sha256=PASS falsifier=PASS")
if __name__=="__main__":
    main()
