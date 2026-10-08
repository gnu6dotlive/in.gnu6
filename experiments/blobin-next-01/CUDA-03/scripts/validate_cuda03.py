#!/usr/bin/env python3
"""Offline CUDA-03 resident benchmark and parity evidence gate."""
from pathlib import Path
from hashlib import sha256
import math
ROOT=Path(__file__).resolve().parents[1]/"fixtures"/"qualification-20261008"
CONFIGS=[(p,d) for p in (16,64,256) for d in (1,2)]
DIGEST="b4fdb097b45a9772"
def parse(line,prefix):
    assert line.startswith(prefix+" "),line
    pairs=[x.split("=",1) for x in line.split()[1:]]
    result=dict(pairs)
    assert len(result)==len(pairs),line
    return result
def positive(row,key):
    value=float(row[key])
    assert math.isfinite(value) and value>0,(key,value)
def check_case(row,backend,count,dpr):
    assert row["backend"]==backend,row
    assert int(row["count"])==count,row
    assert int(row["dpr"])==dpr,row
    assert int(row["iterations"])==20,row
    pixels=256*dpr*192*dpr
    assert int(row["pixels"])==pixels,row
    assert row["digest"]==DIGEST,row
    expected=16*(count+pixels)+(16 if backend=="vulkan" else 0)
    assert int(row["resident_bytes"])==expected,row
    for key in ("cpu_once_ms","init_ms","host_p50_ms","host_p95_ms","readback_ms"):
        positive(row,key)
    assert float(row["host_p95_ms"])>=float(row["host_p50_ms"]),row
    if backend=="cuda":
        for k in ("gpu_event_mean_ms","gpu_event_p50_ms","gpu_event_p95_ms"):
            positive(row,k)
        assert float(row["gpu_event_p95_ms"])>=float(row["gpu_event_p50_ms"]),row
    else:
        for k in ("gpu_event_mean_ms","gpu_event_p50_ms","gpu_event_p95_ms"):
            assert row[k]=="NA",row
    assert int(row["mismatches"])==0,row
    for k,threshold in (("max_field",1e-4),("max_normal",0.006),("max_coverage",0.007)):
        v=float(row[k])
        assert math.isfinite(v) and 0<=v<=threshold,(k,v)
def verify():
    rows=(ROOT/"results.txt").read_text().strip().splitlines()
    assert len(rows)==24,len(rows)
    for i,(count,dpr) in enumerate(CONFIGS):
        block=rows[4*i:4*i+4]
        for j,backend in enumerate(("cuda","vulkan")):
            check_case(parse(block[j],"CUDA03_CASE_PASS"),backend,count,dpr)
        updated=parse(block[2],"CUDA03_UPDATE_PASS")
        assert (int(updated["count"]),int(updated["dpr"]))==(count,dpr)
        assert updated["digest"]==DIGEST
        assert updated["allocation_bytes_unchanged"]=="PASS"
        assert updated["stale_falsifier"]=="PASS"
        assert 0<=float(updated["cuda_field"])<=1e-4
        assert 0<=float(updated["vulkan_field"])<=1e-4
        closure=parse(block[3],"CUDA03_PASS")
        assert closure=={"count":str(count),"dpr":str(dpr),"iterations":"20",
            "providers":"3","gpu_buffers_resident":"PASS","numeric_parity":"PASS"},closure
    neg=(ROOT/"negative.txt").read_text().strip().splitlines()
    assert len(neg)==5,neg
    names=("required_lib","absent_lib","bad_iterations","bad_count","missing_vulkan")
    for line,name in zip(neg,names):
        r=parse(line,"CUDA03_EXPECTED_FAIL")
        assert r=={"case":name,"exit":"1"},r
    manifest=(ROOT/"SHA256SUMS.txt").read_text().strip().splitlines()
    assert len(manifest)==2,manifest
    found=set()
    for line in manifest:
        expected,name=line.split("  ",1)
        assert name in ("results.txt","negative.txt") and name not in found
        found.add(name)
        assert sha256((ROOT/name).read_bytes()).hexdigest()==expected,name
    altered=rows.copy()
    altered[0]=altered[0].replace("mismatches=0","mismatches=1")
    try:
        check_case(parse(altered[0],"CUDA03_CASE_PASS"),"cuda",16,1)
    except AssertionError:pass
    else:raise AssertionError("negative mismatch tamper was not detected")
    print("CUDA03_FIXTURE_GATE_PASS scenes=6 gpu_benchmarks=12 "
          "geometry_updates=6 native_paths=2 cpu_oracle=PASS "
          "required_failures=5 hashes=2 tamper_falsifier=PASS")
if __name__=="__main__":
    verify()
