#!/usr/bin/env python3
"""Independent offline evidence gate for CUDA-04D; does not probe a GPU."""
from hashlib import sha256
from math import isfinite
from pathlib import Path
import sys

BASE = Path(__file__).resolve().parents[1] / "fixtures" / "qualification-20261009-d"
PHASES = ["F1", "F2", "F3", "F4", "F4B", "F4C", "CUDA-02", "CUDA-03", "CUDA-04"]

def check_ledger(content: str) -> None:
    health, integrity, all_pass = {}, [], []
    for line in content.splitlines():
        parts=line.split()
        assert parts and parts[0].startswith("CUDA04D_") and "FAIL" not in line
        kind=parts[0]
        attrs=dict(p.split("=",1) for p in parts[1:])
        if kind=="CUDA04D_HEALTH_PASS":
            device=attrs["device"]
            assert device in ("Cuda", "Vulkan") and device not in health
            assert attrs["generations"]=="1,2,3"
            assert attrs["parity"]==attrs["loss_blocked"]=="PASS"
            for field in ("cold_ms","hit_ms","restored_ms"):
                v=float(attrs[field])
                assert isfinite(v) and v>0
            health[device]=attrs
        elif kind=="CUDA04D_INTEGRITY_PASS":
            assert attrs == {"same_path_replacement":"REJECTED","original_hash":"RESTORED",
                             "stale_cache":"REJECTED","parity":"PASS"}
            integrity.append(attrs)
        elif kind=="CUDA04D_ALL_PASS":
            assert attrs == {"managed_gpu":"2","simulated_loss":"2",
                             "native_same_path_tamper":"1"}
            all_pass.append(attrs)
        else:
            raise AssertionError("unrecognized evidence record")
    assert set(health)=={"Cuda","Vulkan"} and len(integrity)==len(all_pass)==1

def check_regression(content: str) -> int:
    phases=[line.split("phase=",1)[1] for line in content.splitlines()
            if line.startswith("PHASE_PASS phase=")]
    assert phases==PHASES
    assert content.count("CUMULATIVE_ALL_PASS phases=9")==1
    total=sum(int(line.split()[3]) for line in content.splitlines()
              if line.startswith("test result: ok."))
    assert total==61, total
    return total

def check_compatibility(content: str) -> None:
    cases=set()
    iterations=set()
    neg=final=0
    for line in content.splitlines():
        assert line.startswith("CUDA04C_") and "FAIL" not in line
        parts=line.split()
        kind=parts[0]
        v=dict(x.split("=",1) for x in parts[1:])
        if kind in ("CUDA04C_CASE_PASS","CUDA04C_ITER_PASS"):
            key=(v["backend"],int(v["count"]),int(v["dpr"]))
            assert key[0] in ("cuda","vulkan") and key[1] in (16,64,256) and key[2] in (1,2)
            if kind=="CUDA04C_CASE_PASS":
                assert key not in cases and int(v["iterations"])==8
                assert all(v[x]=="PASS" for x in ("numeric","stale","epoch","transpose","rebuild"))
                cases.add(key)
            else:
                i=int(v["index"])
                assert i in range(8) and (key,i) not in iterations
                assert isfinite(float(v["host_ms"])) and float(v["host_ms"])>0
                iterations.add((key,i))
        elif kind=="CUDA04C_NEGATIVE_PASS":
            assert all(x=="PASS" for x in v.values()) and len(v)==4
            neg+=1
        elif kind=="CUDA04C_ALL_PASS":
            assert v=={"cases":"12","iterations":"96","negatives":"4"}
            final+=1
        else: raise AssertionError("unexpected compatibility record")
    assert len(cases)==12 and len(iterations)==96 and neg==final==1

def main():
    values={}
    entries=(BASE/"SHA256SUMS.txt").read_text().splitlines()
    assert len(entries)==4
    for line in entries:
        digest,name=line.split("  ",1)
        assert name in ("results.log","environment.txt","cumulative-tests.log","compatibility-04c.log")
        assert name not in values
        payload=(BASE/name).read_bytes()
        assert sha256(payload).hexdigest()==digest
        values[name]=payload.decode()
    check_ledger(values["results.log"])
    check_compatibility(values["compatibility-04c.log"])
    n=check_regression(values["cumulative-tests.log"])
    corrupted=values["results.log"].replace("loss_blocked=PASS","loss_blocked=FAIL",1)
    assert corrupted!=values["results.log"]
    try:
        check_ledger(corrupted)
    except AssertionError:
        pass
    else:
        raise AssertionError("tampering not rejected")
    print(f"CUDA04D_OFFLINE_PASS managed_gpu=2 integrity=1 regression_tests={n} hashes=4 compatibility_cases=12 tampering=REJECTED")

if __name__=="__main__":
    try:
        main()
    except (AssertionError, OSError, ValueError, KeyError) as error:
        sys.exit(f"CUDA04D_VALIDATION_FAIL {error}")
