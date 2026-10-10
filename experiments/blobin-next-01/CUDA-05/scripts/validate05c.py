#!/usr/bin/env python3
"""Independent offline CUDA-05C fault containment evidence verifier."""
from hashlib import sha256
from pathlib import Path
import sys
BASE=Path(__file__).resolve().parents[1]/"fixtures"/"qualification-20261009-c"
PHASES=["F1","F2","F3","F4","F4B","F4C","CUDA-02","CUDA-03","CUDA-04","CUDA-05"]
NORM=["STATUS","RENDER","ACQUIRE","RENDER","REVOKE","RENDER","ACQUIRE","RENDER","STATUS"]
FAULT=["STATUS","ACQUIRE","RENDER","RENDER","ACQUIRE","RENDER","STATUS"]
TAMPER=["ACQUIRE","RENDER","RENDER","RENDER","ACQUIRE","RENDER"]
def attrs(line):
    h=line.split()
    return h[0],dict(t.split("=",1) for t in h[1:] if "=" in t)
def check_log(log):
    norm={b:{} for b in ("cuda","vulkan")}
    injected={(b,f):{} for b in ("cuda","vulkan") for f in ("crash","hang")}
    tamper={}; fault_done=set(); denied=set(); tamper_done=0; terminal=0
    for line in log.splitlines():
        assert line.startswith("CUDA05C_")
        kind,v=attrs(line)
        if kind=="CUDA05C_NORMAL_PASS":
            b=v["backend"];i=int(v["index"])
            assert b in norm and i in range(9) and i not in norm[b] and v["command"]==NORM[i]
            result=line.split(" response=",1)[1]
            if i in (1,5):assert result.startswith("DENIED reason=CUDA05C_NO_ACTIVE_GRANT")
            elif i in (2,6):assert result.startswith("READY generation=")
            elif i in (3,7):assert result.startswith("RENDER_PASS") and "parity=PASS" in result
            elif i==4:assert result.startswith("REVOKED")
            else:assert result.startswith("STATE generation=")
            norm[b][i]=result
        elif kind=="CUDA05C_FAULT_STEP_PASS":
            b=v["backend"];f=v["fault"];i=int(v["index"])
            assert (b,f) in injected and i in range(7) and i not in injected[(b,f)]
            assert v["command"]==FAULT[i]
            r=line.split(" response=",1)[1]
            if i==2:
                code="TIMEOUT_KILLED" if f=="hang" else "CRASH_OR_ERROR"
                assert r.startswith("DENIED reason=CUDA05C_WORKER_"+code)
            elif i==3: assert r.startswith("DENIED reason=CUDA05C_NO_ACTIVE_GRANT")
            elif i in (1,4):assert r.startswith("READY generation=")
            elif i==5:assert r.startswith("RENDER_PASS") and "parity=PASS" in r
            else:assert r.startswith("STATE generation=")
            injected[(b,f)][i]=r
        elif kind=="CUDA05C_FAULT_PASS":
            assert v["revoked"]==v["recovered"]=="PASS"
            fault_done.add((v["backend"],v["fault"]))
        elif kind=="CUDA05C_DENY_PASS":
            assert v["configured_wrong_uid"]=="REJECTED"
            denied.add(v["backend"])
        elif kind=="CUDA05C_TAMPER_STEP_PASS":
            i=int(v["index"])
            assert i in range(6) and i not in tamper and v["command"]==TAMPER[i]
            r=line.split(" response=",1)[1]
            if i==2:assert r.startswith("DENIED reason=CUDA05C_WORKER_HASH_NOT_APPROVED")
            if i==3:assert r.startswith("DENIED reason=CUDA05C_NO_ACTIVE_GRANT")
            if i in (0,4):assert r.startswith("READY generation=")
            if i in (1,5):assert r.startswith("RENDER_PASS") and "parity=PASS" in r
            tamper[i]=r
        elif kind=="CUDA05C_TAMPER_PASS":
            assert v=={"backend":"cuda","same_path_worker":"REJECTED","restored":"PASS"}
            tamper_done+=1
        elif kind=="CUDA05C_ALL_PASS":
            assert v=={"real_backends":"2","normal_socket_ops":"18","fault_ops":"28",
                "worker_tamper_ops":"6","injected_crashes":"2","injected_timeouts":"2",
                "wrong_uid":"2","no_gpu_reset":"PASS"}
            terminal+=1
        else:raise AssertionError("unknown record")
    assert all(set(x)==set(range(9)) for x in norm.values())
    assert all(set(x)==set(range(7)) for x in injected.values())
    assert fault_done==set(injected) and denied==set(norm)
    assert set(tamper)==set(range(6)) and tamper_done==terminal==1
    for key,items in injected.items():
        assert int(items[4].split("generation=")[1].split()[0])>int(items[1].split("generation=")[1].split()[0])
def validate():
    files={}
    manifest=(BASE/"SHA256SUMS.txt").read_text().splitlines()
    assert len(manifest)==3
    for entry in manifest:
        digest,name=entry.split("  ",1)
        assert name in ("results.log","cumulative-tests.log","environment.txt") and name not in files
        value=(BASE/name).read_bytes()
        assert sha256(value).hexdigest()==digest
        files[name]=value.decode()
    assert len(files)==3
    check_log(files["results.log"])
    log=files["cumulative-tests.log"].splitlines()
    phases=[l.split("phase=",1)[1] for l in log if l.startswith("PHASE_PASS phase=")]
    assert phases==PHASES and log.count("CUMULATIVE_ALL_PASS phases=10")==1
    total=sum(int(l.split()[3]) for l in log if l.startswith("test result: ok."))
    assert total==68,total
    tampered=files["results.log"].replace("worker_tamper_ops=6","worker_tamper_ops=7",1)
    assert tampered!=files["results.log"]
    try:check_log(tampered)
    except AssertionError:pass
    else:raise AssertionError("forged ledger accepted")
    print("CUDA05C_OFFLINE_PASS backends=2 operations=52 crashes=2 timeouts=2 worker_tamper=1 rust_tests=68 hashes=3 falsifier=REJECTED")
if __name__=="__main__":
    try:validate()
    except (KeyError,ValueError,OSError,AssertionError) as e:sys.exit(f"CUDA05C_OFFLINE_FAIL {e}")
