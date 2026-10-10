#!/usr/bin/env python3
"""Check pinned MXC Linux backend's source-order invariants with causal mutations."""
import hashlib
import json
import subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
SOURCE_ROOT=Path("/mnt/workbench/workspaces/gnu.in.labs/.probes/upstream-rd-01/mxc-c6f301")
SHA="c6f301d53a1430c4c921a05c57af838f7392348f"
R="src/mxc-sdk/src/backends/bubblewrap/common/bwrap_runner.rs"
S=SOURCE_ROOT/R
assert subprocess.check_output(["git","rev-parse","HEAD"],cwd=SOURCE_ROOT,text=True).strip()==SHA
body=S.read_text()
a=body.index("impl SandboxBackend for BubblewrapScriptRunner")
b=body.index("impl BubblewrapScriptRunner {",a)
spawn=body[a:b]
ordered=["validate_common(request)?", "self.validate_prepared(request)?",
         "normalize_object_conflicts(", "check_delegation(&request.policy)",
         "resolve_denied_paths(&request.policy", "check_pin_against_denied_hosts(request)",
         "self.spawn_bwrap("]
def invariant(s):
    positions=[s.find(x) for x in ordered]
    return all(x>=0 for x in positions) and positions==sorted(positions)
assert invariant(spawn),"Pinned MXC runner source ordering diverged"
mutations={}
for key in ordered:
    transformed=spawn.replace(key,"REMOVED_CAUSAL_GUARD",1)
    mutations[key]="RED_CONFIRMED" if not invariant(transformed) else "BAD_ORACLE"
assert all(x=="RED_CONFIRMED" for x in mutations.values())
# Runtime compilation preflight is explicitly offline to avoid unbounded downloads.
cmd=["cargo","tree","-p","mxc-sdk","--offline","--locked","--depth","1"]
env={"PATH":"/usr/bin:/bin","RUSTUP_TOOLCHAIN":"1.97.1"}
proc=subprocess.run(cmd,cwd=SOURCE_ROOT/"src",capture_output=True,text=True,timeout=15,env=env)
out=ROOT/"evidence/upstream-rd-01/mxc-security-01"
out.mkdir(parents=True,exist_ok=True)
report={"source_sha":SHA,"path":R,"source_sha256":hashlib.sha256(S.read_bytes()).hexdigest(),
        "gate_source_order":"PASS","mutation_gates":mutations,
        "source_order":ordered,"sdk_runtime_test":"NOT_RUN",
        "cargo_dependency_resolution":{
            "argv":cmd,"exit_code":proc.returncode,"stderr":proc.stderr[:1100],
            "classification":"OFFLINE_RESOLUTION_BLOCKED" if proc.returncode else "OFFLINE_RESOLUTION_PASS"},
        "limitations":["No MXC SDK execution","No permission audit logs from MXC SDK",
                       "Static ordering does not establish runtime enforcement",
                       "This host is not a proxy for network policy across all Linux versions"]}
(out/"qualification.json").write_text(json.dumps(report,indent=2,ensure_ascii=False)+"\n")
print("MXC_SOURCE_ORDER=PASS","MUTANTS_RED=",len(mutations),
      "SDK_DEPENDENCIES=",report["cargo_dependency_resolution"]["classification"])
