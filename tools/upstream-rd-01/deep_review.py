#!/usr/bin/env python3
"""Exact-revision, read-only source acquisition for isolated UPSTREAM-R&D workers."""
import base64
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BATCH = ROOT / "contracts/agent-batches/2026-10-10-upstream-rd-01"
TARGETS = {
    "W01": (["docs/logging-access-denied.md", "docs/api-reference/README.md", "docs/backends/bwrap/bubblewrap-backend.md"], ["deny", "permission", "audit", "bwrap", "network"]),
    "W02": (["rust-modules/README.md", "docs/perf-damage-tracking-verdict.md", "docs/ui-framework.md"], ["Rust", "ABI", "GPU_ACTIVE", "damage", "deprecated"]),
    "W03": (["runtime/permissions.rs", "runtime/README.md", "doc/desktop-architecture.md"], ["permission", "op", "runtime", "cef", "supervisor"]),
    "W04": (["docs/gpui-shell.md", "docs/ACCESSIBILITY-UI-TESTING.md", "LICENSE-DOCS.md"], ["capabilit", "QuickJS", "accessib", "focus", "license"]),
    "W05": (["src/metric.rs", "src/options.rs", "src/main.rs"], ["rss", "perf", "metric", "shell", "benchmark"]),
    "W06": (["crates/uzu-engine/src/backends/mod.rs", "crates/uzu-engine/Cargo.toml", "crates/uzu-engine/src/backends/cpu/mod.rs"], ["metal", "cpu", "cuda", "grammar", "feature"]),
}
def get(repo, commit, path):
    call = ["gh", "api", f"repos/{repo}/contents/{path}?ref={commit}"]
    p = subprocess.run(call, capture_output=True, text=True, timeout=50)
    if p.returncode: return {"path": path, "status": "BLOCKED_SOURCE", "reason": p.stderr[:250]}
    obj = json.loads(p.stdout)
    if obj.get("encoding") != "base64": return {"path":path,"status":"BLOCKED_SOURCE","reason":"No base64 body"}
    raw=base64.b64decode(obj["content"])
    lines=raw.decode("utf-8", errors="replace").splitlines()
    return {"path":path,"status":"VERIFIED_SOURCE","blob_sha":obj.get("sha"),
            "sha256":hashlib.sha256(raw).hexdigest(),"line_count":len(lines),
            "_lines":lines,"url":f"https://github.com/{repo}/blob/{commit}/{path}"}
def run(wid):
    assert wid in TARGETS, "Unsupported worker"
    branch=subprocess.check_output(["git","branch","--show-current"],cwd=ROOT,text=True).strip()
    assert branch==f"research/upstream-rd-01-{wid.lower()}-20261010", f"Wrong worker branch: {branch}"
    manifest=json.loads((BATCH/"sources.json").read_text())["sources"]
    idx=int(wid[1:])-1
    repo,sha=manifest[idx]["repo"],manifest[idx]["commit_sha"]
    assert len(sha)==40 and not manifest[idx]["errors"]
    paths,patterns=TARGETS[wid]
    out=[]
    for path in paths:
        obj=get(repo,sha,path)
        if obj["status"]=="VERIFIED_SOURCE":
            lines=obj.pop("_lines")
            for signal in patterns:
                hits=[(i,l.strip()) for i,l in enumerate(lines,1) if re.search(signal,l,re.I)]
                obj.setdefault("signals",[]).append(
                    {"query":signal,"status":"PRESENT_IN_FILE" if hits else "NOT_FOUND_IN_FILE",
                     "total_matching_lines":len(hits),
                     "anchors":[{"line":i,"text":v[:190]} for i,v in hits[:3]]})
        out.append(obj)
    receipt={"worker":wid,"repo":repo,"source_commit":sha,
             "method":"GitHub contents at immutable commit; bounded regex inventory, not code execution",
             "files":out,"runtime_status":"NOT_RUN","product_status":"NOT_RUN"}
    evidence=ROOT/f"evidence/upstream-rd-01/{wid.lower()}/source-evidence.json"
    evidence.parent.mkdir(parents=True,exist_ok=True)
    evidence.write_text(json.dumps(receipt,indent=2,ensure_ascii=False)+"\n")
    report=ROOT/f"docs/agent-results/upstream-rd-01/{wid.lower()}.md"
    assert report.exists()
    current=report.read_text()
    assert f"## {wid} — deep source inspection" not in current, "Refusing duplicate append"
    summary=[f"\n## {wid} — deep source inspection", "",
             f"Pinned commit: `{sha}`. **VERIFIED_SOURCE only; runtime tests NOT_RUN.**",""]
    for item in out:
        if item["status"]!="VERIFIED_SOURCE":
            summary.append(f"- **BLOCKED_SOURCE** `{item['path']}`: {item['reason']}")
            continue
        summary.append(f"- **SOURCE VERIFIED** [{item['path']}]({item['url']}), "
                       f"{item['line_count']} lines, SHA-256 `{item['sha256']}`.")
        for s in item["signals"]:
            if s["status"]=="PRESENT_IN_FILE":
                a=s["anchors"][0]
                summary.append(f"  - `{s['query']}` found {s['total_matching_lines']} times; "
                               f"L{a['line']}: {a['text'].replace(chr(96), chr(39))}")
            else: summary.append(f"  - `{s['query']}` **NOT_FOUND_IN_FILE** (not a whole-repository absence claim).")
    summary += ["",f"Evidence: `evidence/upstream-rd-01/{wid.lower()}/source-evidence.json`.",
                "No consumer, kernel, CUDA, licensing or deployment claims follow from textual matching.",
                "Next qualification: validate a narrowly falsifiable hypothesis at the actual target boundary.",""]
    report.write_text(current.rstrip()+"\n"+"\n".join(summary))
    print(json.dumps({"worker":wid,"repo":repo,"verified_files":sum(x["status"]=="VERIFIED_SOURCE" for x in out),
                      "blocked_files":[x["path"] for x in out if x["status"]!="VERIFIED_SOURCE"]}))
    return 0 if all(x["status"]=="VERIFIED_SOURCE" for x in out) else 2
if __name__=="__main__":
    sys.exit(run(sys.argv[1]))
