#!/usr/bin/env python3
"""Reproducible bounded MXC SDK filesystem allow/deny fixture."""
import hashlib,json,pathlib,subprocess,tempfile
ROOT=pathlib.Path(__file__).resolve().parents[2]
BASE=pathlib.Path("/mnt/workbench/workspaces/gnu.in.labs/.probes/upstream-rd-01")
SRC=BASE/"mxc-c6f301"
SDK_SHA="c6f301d53a1430c4c921a05c57af838f7392348f"
assert subprocess.check_output(["git","rev-parse","HEAD"],cwd=SRC,text=True).strip()==SDK_SHA
binary=BASE/"mxc-fixture-target/debug/gnu6-mxc-fixture"
assert binary.is_file()
out=[]
with tempfile.TemporaryDirectory(prefix="gnu6-mxc-only-",dir=BASE) as t:
    root=pathlib.Path(t); sentinel=root/"sentinel.txt"; marker=root/"marker.txt"
    sentinel.write_text("GNU6-MXC-TEMP-FIXTURE\n")
    for mode in ("ro","rw"):
        marker.unlink(missing_ok=True)
        cmd=[str(binary),mode,t]
        p=subprocess.run(cmd,capture_output=True,text=True,timeout=12)
        marker_present=marker.exists()
        passed=(p.returncode==0 and "GNU6-MXC-TEMP-FIXTURE" in p.stdout and
                ("WRITE_DENIED" if mode=="ro" else "WRITE_ALLOWED") in p.stdout and
                marker_present==(mode=="rw") and
                (not marker_present or marker.read_text()=="allowed"))
        out.append({"mode":mode,"pass":passed,"exit_code":p.returncode,
                    "stdout":p.stdout[:500],"stderr":p.stderr[:500],
                    "marker_present":marker_present})
    intact=sentinel.read_text()=="GNU6-MXC-TEMP-FIXTURE\n"
result={"gate":"PASS_2_OF_2" if all(x["pass"] for x in out) and intact else "FAIL",
        "source_commit":SDK_SHA,"sdk_version":"1.0.0",
        "runner_sha256":hashlib.sha256(binary.read_bytes()).hexdigest(),
        "results":out,"sentinel_intact":intact,
        "limitations":["Local Linux host, Process/Bubblewrap backend only",
                       "No external broker, service tenancy or production authority",
                       "Probe binary is a separate local Rust fixture; source repo untouched"]}
path=ROOT/"evidence/upstream-rd-01/mxc-runtime-02/qualification.json"
path.parent.mkdir(parents=True,exist_ok=True)
path.write_text(json.dumps(result,indent=2)+"\n")
print("MXC_SDK_RUNTIME",result["gate"], [(x["mode"],x["pass"],x["marker_present"]) for x in out])
raise SystemExit(0 if result["gate"]=="PASS_2_OF_2" else 2)
