#!/usr/bin/env python3
"""A bounded environment smoke: do not infer MXC's enforcement from bwrap alone."""
import json, platform, shutil, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
NAMES=["hyperfine","bwrap","deno","cargo","rustc","nvidia-smi","podman"]
tools={x:shutil.which(x) for x in NAMES}
cmd=[tools["bwrap"],"--unshare-user","--unshare-pid","--unshare-net","--ro-bind","/","/","--proc","/proc","--dev-bind","/dev","/dev","--tmpfs","/tmp","--chdir","/","--","/usr/bin/true"] if tools["bwrap"] else None
smoke={"status":"NOT_RUN","command":cmd,"scope":"host bwrap smoke, NOT MXC SDK"}
if cmd:
    try:
        p=subprocess.run(cmd,capture_output=True,text=True,timeout=8)
        smoke.update({"status":"PROBE_PASS" if p.returncode==0 else "PROBE_FAILED","returncode":p.returncode,"stderr":p.stderr[:300],"stdout":p.stdout[:200]})
    except subprocess.TimeoutExpired:
        smoke.update({"status":"TIMEOUT","timeout_seconds":8})
report={"host":platform.node(),"platform":platform.platform(),"tool_paths":tools,"bwrap_smoke":smoke,"limitations":["does not test MXC policy validation","no GPU/CUDA benchmarks","no Deno/Hyperfine installed"]}
out=ROOT/"evidence/upstream-rd-01/intake/host-smoke.json"
out.parent.mkdir(parents=True,exist_ok=True)
out.write_text(json.dumps(report,indent=2,ensure_ascii=False)+"\n")
print(json.dumps({"tools":tools,"bwrap_smoke":smoke},ensure_ascii=False))
