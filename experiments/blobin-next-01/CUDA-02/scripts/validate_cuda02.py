#!/usr/bin/env python3
"""CUDA-02 offline 3-backend provenance/parity and explicit fallback verifier."""
from collections import defaultdict
from hashlib import sha256
from pathlib import Path
import math

ROOT=Path(__file__).resolve().parents[1]/"fixtures"/"qualification-20261008"
COUNT=(16,64,256)
DPRS=(1,2)
DIGEST="b4fdb097b45a9772"
IDENTITIES=("2970288129","2970288130")
def unpack(line,prefix):
    assert line.startswith(prefix+" "),line
    result={}
    for item in line.strip().split()[1:]:
        key,value=item.split("=",1)
        assert key not in result,key
        result[key]=value
    return result

def bounded(row,chosen,count,dpr,requested):
    assert row["backend"]==chosen,row
    assert row["requested"]==requested,row
    assert int(row["primitives"])==count,row
    assert int(row["dpr"])==dpr,row
    assert int(row["pixels"])==256*dpr*192*dpr,row
    assert row["authority_digest"]==DIGEST,row
    assert row["authority_epoch"]=="0",row
    assert (row["left_id"],row["right_id"])==IDENTITIES,row
    assert row["mismatches"]=="0",row
    assert row["boundary_mismatches"]=="0",row
    thresholds={"max_field":0.0001,"max_normal":0.006,"max_coverage":0.007}
    for key,threshold in thresholds.items():
        value=float(row[key])
        assert math.isfinite(value) and 0.0<=value<=threshold,(key,value)
    for metric in ("cpu_oracle_ms","backend_end_to_end_ms"):
        value=float(row[metric])
        assert math.isfinite(value) and value>0,(metric,value)

def verify():
    lines=(ROOT/"three-providers.txt").read_text().strip().splitlines()
    assert len(lines)==24,len(lines)
    maximal=defaultdict(float)
    for i,(count,dpr) in enumerate((c,d) for c in COUNT for d in DPRS):
        block=lines[i*4:i*4+4]
        for backend,line in zip(("cpu","vulkan","cuda"),block[:3]):
            row=unpack(line,"CUDA02_CASE_PASS")
            bounded(row,backend,count,dpr,"all")
            for key in ("max_field","max_normal","max_coverage"):
                maximal[key]=max(maximal[key],float(row[key]))
        summary=unpack(block[3],"CUDA02_PASS")
        assert summary=={
            "requested":"all",
            "primitives":str(count),
            "dpr":str(dpr),
            "verified_providers":"3",
            "negative_numeric":"PASS",
            "f2_authority":"PASS",
        },summary
    fallback=(ROOT/"fallback.txt").read_text().strip().splitlines()
    assert len(fallback)==6,len(fallback)
    for index,backend in enumerate(("cuda","vulkan","cpu")):
        row=unpack(fallback[index*2],"CUDA02_CASE_PASS")
        bounded(row,backend,16,1,"auto")
        summary=unpack(fallback[index*2+1],"CUDA02_PASS")
        assert summary["requested"]=="auto" and summary["verified_providers"]=="1"
        assert summary["negative_numeric"]=="PASS" and summary["f2_authority"]=="PASS"
    negative=unpack((ROOT/"negative.txt").read_text().strip(),"CUDA02_EXPECTED_NEGATIVE_PASS")
    assert all(negative[k]=="1" for k in (
        "cuda_required","vulkan_required","cuda_library_missing","input_budget"
    )),negative
    manifest=(ROOT/"SHA256SUMS.txt").read_text().strip().splitlines()
    assert len(manifest)==3,len(manifest)
    names=set()
    for line in manifest:
        expected,name=line.split("  ",1)
        assert name not in names,name
        names.add(name)
        assert name in {"three-providers.txt","fallback.txt","negative.txt"}
        assert sha256((ROOT/name).read_bytes()).hexdigest()==expected,name
    counterfeit=lines.copy()
    counterfeit[1]=counterfeit[1].replace("mismatches=0","mismatches=1")
    try:
        bounded(unpack(counterfeit[1],"CUDA02_CASE_PASS"),"vulkan",16,1,"all")
    except AssertionError:
        pass
    else:
        raise AssertionError("negative occupancy corruption was not rejected")
    print("CUDA02_FIXTURE_GATE_PASS scenes=6 cpu_vulkan_cuda_evaluations=18 "
          "auto_fallbacks=3 explicit_failures=4 sha256=3 "
          "authority_provenance=PASS numeric_falsifier=PASS "
          "max_field={:.9g} max_normal={:.9g} max_coverage={:.9g}".format(
          maximal["max_field"],maximal["max_normal"],maximal["max_coverage"]))

if __name__=="__main__":
    verify()
