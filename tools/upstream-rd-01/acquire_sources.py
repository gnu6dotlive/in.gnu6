#!/usr/bin/env python3
"""Acquire pinned, bounded upstream metadata through the authenticated GitHub CLI."""
import concurrent.futures
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

REPOS = ["microsoft/mxc", "GLinnik21/plx-native", "denoland/deno", "longbridge/gpui-kit", "sharkdp/hyperfine", "trymirai/uzu"]
def api(repo, suffix):
    p = subprocess.run(["gh", "api", f"repos/{repo}" + (f"/{suffix}" if suffix else "")], capture_output=True, text=True, timeout=40)
    if p.returncode: return {"error": p.stderr[:240]}
    return json.loads(p.stdout)
def acquire(repo):
    meta, commit, readme = api(repo, ""), api(repo, "commits/HEAD"), api(repo, "readme")
    return {"repo": repo, "commit_sha": commit.get("sha"), "default_branch": meta.get("default_branch"), "license_spdx": (meta.get("license") or {}).get("spdx_id"), "archived": meta.get("archived"), "pushed_at": meta.get("pushed_at"), "readme_blob_sha": readme.get("sha"), "commit_summary": commit.get("commit", {}).get("message", "").splitlines()[:1], "url": meta.get("html_url"), "errors": [item["error"] for item in (meta, commit, readme) if "error" in item]}
def main():
    with concurrent.futures.ThreadPoolExecutor(max_workers=6) as executor:
        rows = list(executor.map(acquire, REPOS))
    out = Path(__file__).resolve().parents[2] / "contracts/agent-batches/2026-10-10-upstream-rd-01/sources.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps({"captured_at_utc": datetime.now(timezone.utc).isoformat(), "sources": rows}, indent=2, ensure_ascii=False) + "\n")
    for x in rows: print(x["repo"], x["commit_sha"] or "UNAVAILABLE", x["license_spdx"], x["errors"])
    return 0 if all(row["commit_sha"] and not row["errors"] for row in rows) else 2
if __name__ == "__main__": sys.exit(main())
