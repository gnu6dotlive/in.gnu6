#!/usr/bin/env python3
"""Independent, read-only validation of the bounded CUDA-05B evidence."""
from hashlib import sha256
from pathlib import Path
import sys

BASE=Path(__file__).resolve().parents[1]/"fixtures"/"qualification-20261009-b"
PHASES=["F1","F2","F3","F4","F4B","F4C","CUDA-02","CUDA-03","CUDA-04","CUDA-05"]
COMMANDS=["STATUS","RENDER","ACQUIRE","RENDER","RENDER","REVOKE","RENDER","ACQUIRE","RENDER","STATUS"]
TAMPER=["ACQUIRE","RENDER","RENDER","RENDER","ACQUIRE","RENDER"]

def parse(line):
    words=line.split()
    assert words
    return words[0],dict(word.split("=",1) for word in words[1:] if "=" in word)

def verify_socket_line(index, command, response):
    if index in (1,6):
        assert response.startswith("DENIED reason=CUDA05B_NO_ACTIVE_PERMIT")
    elif index in (2,7):
        assert response.startswith("READY generation=")
    elif index in (3,4,8):
        assert response.startswith("RENDER_PASS")
        assert ("cache_hit=true" in response)==(index==4)
        assert "parity=PASS" in response
    elif index==5:
        assert response.startswith("REVOKED")
    else:
        assert response.startswith("STATE generation=")

def verify_log(content):
    grid={"cuda":{}, "vulkan":{}}
    devices=set(); denied=set(); tamper={}
    terminal=0;summary=0
    for line in content.splitlines():
        assert "ERROR" not in line and "CUDA05B_" in line
        kind,attrs=parse(line)
        if kind=="CUDA05B_SOCKET_PASS":
            b=attrs["backend"];i=int(attrs["index"])
            assert b in grid and i in range(10) and i not in grid[b]
            assert attrs["command"]==COMMANDS[i]
            response=line.split(" response=",1)[1]
            verify_socket_line(i,COMMANDS[i],response)
            grid[b][i]=response
        elif kind=="CUDA05B_DEVICE_PASS":
            assert attrs["backend"] in grid and attrs["backend"] not in devices
            assert attrs["probe_challenges"]=="5" and attrs["parity"]=="PASS"
            devices.add(attrs["backend"])
        elif kind=="CUDA05B_DENY_PASS":
            assert attrs["backend"] in grid and attrs["backend"] not in denied
            assert attrs["wrong_uid"]=="REJECTED"
            denied.add(attrs["backend"])
        elif kind=="CUDA05B_TAMPER_STEP_PASS":
            i=int(attrs["index"])
            assert i in range(6) and i not in tamper and attrs["command"]==TAMPER[i]
            response=line.split(" response=",1)[1]
            if i==2: assert response.startswith("DENIED reason=CUDA05B_CHALLENGE_UNAPPROVED_PROVIDER")
            if i==3: assert response.startswith("DENIED reason=CUDA05B_NO_ACTIVE_PERMIT")
            if i in (0,4): assert response.startswith("READY generation=")
            if i in (1,5): assert response.startswith("RENDER_PASS") and "parity=PASS" in response
            tamper[i]=response
        elif kind=="CUDA05B_TAMPER_PASS":
            assert attrs=={"backend":"cuda","same_path":"REJECTED","revoked":"PASS","restored":"PASS"}
            summary+=1
        elif kind=="CUDA05B_ALL_PASS":
            assert attrs=={"devices":"2","socket_operations":"26","wrong_uid":"2","tamper":"1","no_gpu_reset":"PASS"}
            terminal+=1
        else:
            raise AssertionError("unknown evidence type")
    assert all(set(values)==set(range(10)) for values in grid.values())
    assert devices==denied==set(grid) and set(tamper)==set(range(6)) and summary==terminal==1
    for values in grid.values():
        first=int(values[2].split("generation=")[1].split()[0])
        second=int(values[7].split("generation=")[1].split()[0])
        assert first<second

def validate():
    entries=(BASE/"SHA256SUMS.txt").read_text().splitlines()
    assert len(entries)==3
    files={}
    for line in entries:
        expected,name=line.split("  ",1)
        assert name in ("results.log","cumulative-tests.log","environment.txt") and name not in files
        data=(BASE/name).read_bytes()
        assert sha256(data).hexdigest()==expected
        files[name]=data.decode()
    assert len(files)==3
    verify_log(files["results.log"])
    lines=files["cumulative-tests.log"].splitlines()
    phases=[x.split("phase=",1)[1] for x in lines if x.startswith("PHASE_PASS phase=")]
    assert phases==PHASES and lines.count("CUMULATIVE_ALL_PASS phases=10")==1
    n=sum(int(line.split()[3]) for line in lines if line.startswith("test result: ok."))
    assert n==66
    altered=files["results.log"].replace("wrong_uid=REJECTED","wrong_uid=ACCEPTED",1)
    assert altered!=files["results.log"]
    try:verify_log(altered)
    except AssertionError:pass
    else:raise AssertionError("tampered evidence accepted")
    print("CUDA05B_OFFLINE_PASS backends=2 socket_ops=26 wrong_uid=2 tamper=1 rust_tests=66 hashes=3 mutation=REJECTED")
if __name__=="__main__":
    try: validate()
    except (AssertionError,KeyError,ValueError,OSError) as e:sys.exit(f"CUDA05B_OFFLINE_FAIL {e}")
