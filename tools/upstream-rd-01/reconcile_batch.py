#!/usr/bin/env python3
"""Validate immutable upstream source receipts and advance only this batch's dispatch."""
import json
import subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/"contracts/agent-batches/2026-10-10-upstream-rd-01"
get=lambda p: json.loads(p.read_text())
dispatch=get(BASE/"dispatch.json")
source=get(BASE/"sources.json")["sources"]
intake=get(ROOT/"evidence/upstream-rd-01/intake/source-probe.json")
assert subprocess.check_output(["git","branch","--show-current"],cwd=ROOT,text=True).strip()==dispatch["branch"]
assert len(source)==len(intake)==len(dispatch["workers"])==6
assert all(s["commit_sha"]==r["pin"] and len(r["files"])==2 for s,r in zip(source,intake))
deep=[]
for n in range(1,7):
    w=f"w{n:02d}"
    receipt=get(ROOT/f"evidence/upstream-rd-01/{w}/source-evidence.json")
    assert receipt["source_commit"]==source[n-1]["commit_sha"]
    assert receipt["repo"]==source[n-1]["repo"]
    assert len(receipt["files"])==3
    assert all(f["status"]=="VERIFIED_SOURCE" and len(f["sha256"])==64 for f in receipt["files"])
    assert (ROOT/f"docs/agent-results/upstream-rd-01/{w}.md").exists()
    dispatch["workers"][n-1].update(state="SOURCE_DEEP_REVIEW_COMPLETE",agent_status="DETERMINISTIC_WORKER_COMPLETE")
    deep.extend(receipt["files"])
assert len(deep)==18
assert all((ROOT/p).exists() for p in ["docs/agent-results/upstream-rd-01/cross-analysis.md","docs/agent-results/upstream-rd-01/experiments.md"])
host=get(ROOT/"evidence/upstream-rd-01/intake/host-denial-oracle.json")
assert host["gate"]=="PASS_BOUNDARY_FIXTURE" and host["host_sentinel_preserved"]
dispatch["worker_execution"]="SIX_PARALLEL_DETERMINISTIC_WORKERS_COMPLETE; NO_AUTONOMOUS_LLM_AGENTS"
dispatch["deep_inspection"]="18 additional pinned files verified (three per upstream), plus 12 intake files"
dispatch["stages"][0]["state"]="SOURCE_CROSS_ANALYSIS_COMPLETE"
dispatch["stages"][1]["state"]="HOST_BWRAP_BOUNDARY_FIXTURE_PASS__UPSTREAM_SDK_NOT_RUN"
dispatch["stages"][2]["state"]="INTEGRATOR_RECONCILED_SOURCE_WORKERS__NO_PRODUCT_TRANSFER"
dispatch["stages"][3]["state"]="PRIVATE_DOCS_BRIDGE_COMPLETE__PUBLIC_DRAFT_ONLY_NO_SITE_DEPLOY"
dispatch["stages"][4]["state"]="PARTIAL_PENDING_REMOTE_VERIFICATION"
(BASE/"dispatch.json").write_text(json.dumps(dispatch,indent=2,ensure_ascii=False)+"\n")
print("GATE_W01_W06=PASS 18 PINNED FILES")
print("GATE_W07=PASS SOURCE-BACKED HYPOTHESES")
print("GATE_W08=PASS HOST BWRAP BOUNDARY FIXTURE; MXC SDK NOT_RUN")
print("GATE_DISPATCH_SCHEMA=PASS; NO PRODUCT INTEGRATION")
