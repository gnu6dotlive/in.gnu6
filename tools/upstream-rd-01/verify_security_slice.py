#!/usr/bin/env python3
"""Validate MXC/Deno receipts, then update only UPSTREAM-R&D dispatch status."""
import json
import subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
BATCH=ROOT/"contracts/agent-batches/2026-10-10-upstream-rd-01"
read=lambda p: json.loads((ROOT/p).read_text())
src=read("contracts/agent-batches/2026-10-10-upstream-rd-01/sources.json")["sources"]
mx=read("evidence/upstream-rd-01/mxc-security-01/qualification.json")
net=read("evidence/upstream-rd-01/mxc-security-01/loopback-oracle.json")
deno=read("evidence/upstream-rd-01/deno-permission-01/qualification.json")
assert mx["source_sha"]==src[0]["commit_sha"],"Wrong MXC pin"
assert deno["pinned_commit"]==src[2]["commit_sha"],"Wrong Deno pin"
assert mx["gate_source_order"]=="PASS"
assert len(mx["mutation_gates"])==7 and all(x=="RED_CONFIRMED" for x in mx["mutation_gates"].values())
assert net["gate"]=="PASS_TWO_SIDED_HOST_ORACLE"
assert net["positive_control"]["result"]["connected"] is True
assert net["isolated_test"]["result"]["connected"] is False
assert net["positive_control"]["result"]["net_namespace"]!=net["isolated_test"]["result"]["net_namespace"]
assert deno["source_gate"]=="PASS"
assert len(deno["mutations"])==10 and all(x=="RED_CONFIRMED" for x in deno["mutations"].values())
assert deno["runtime_status"]=="NOT_RUN_DENO_BINARY_UNAVAILABLE"
assert mx["sdk_runtime_test"]=="NOT_RUN"
assert mx["cargo_dependency_resolution"]["classification"]=="OFFLINE_RESOLUTION_BLOCKED"
assert all((BATCH/f).exists() for f in ("12-MXC-SECURITY-01.md","13-DENO-PERMISSION-01.md","14-MXC-DENO-BOUNDARY.md"))
dispatch=json.loads((BATCH/"dispatch.json").read_text())
branch=subprocess.check_output(["git","branch","--show-current"],cwd=ROOT,text=True).strip()
assert branch==dispatch["branch"]
dispatch["mx_deno_security_slice"]={
    "scope":"MXC_W01A_AND_DENO_W03A",
    "mxc_source_guard":"PASS_7_OF_7_MUTATIONS",
    "mxc_host_loopback":"PASS_TWO_SIDED_HOST_ORACLE_NOT_MXC_SDK",
    "mxc_sdk_runtime":"NOT_RUN_OFFLINE_CIDR_DEPENDENCY_MISSING",
    "deno_broker_source":"PASS_10_OF_10_MUTATIONS",
    "deno_runtime":"NOT_RUN_BINARY_ABSENT",
    "mxc_deno_joint_gate":"DESIGNED_NOT_EXECUTED",
    "product_implementations":"UNCHANGED",
    "public_surface":"NOT_DEPLOYED"
}
(BATCH/"dispatch.json").write_text(json.dumps(dispatch,indent=2,ensure_ascii=False)+"\n")
print("W01A_MXC_SOURCE_MUTANTS=7/7 PASS")
print("W01A_HOST_LOOPBACK_TWO_SIDED=PASS / MXC_SDK_NOT_RUN")
print("W03A_DENO_SOURCE_MUTANTS=10/10 PASS / DENO_RUNTIME_NOT_RUN")
print("W14_COMPOSITE=DESIGNED_NOT_EXECUTED; DISPATCH_VALIDATED")
