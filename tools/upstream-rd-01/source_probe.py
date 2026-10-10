#!/usr/bin/env python3
"""Read selected exact-revision upstream files; store fingerprints and bounded source anchors."""
import base64, concurrent.futures, hashlib, json, re, subprocess, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
BATCH = ROOT / "contracts/agent-batches/2026-10-10-upstream-rd-01"
TARGETS = {
"microsoft/mxc": ["docs/backends/bwrap/bubblewrap-backend.md", "docs/container-lifecycle.md"],
"GLinnik21/plx-native": ["docs/perf-damage-tracking-verdict.md", "docs/module-layers.md"],
"denoland/deno": ["doc/architecture.md", "doc/desktop-architecture.md"],
"longbridge/gpui-kit": ["docs/ARCHITECTURE.md", "docs/ACCESSIBILITY-UI-TESTING.md"],
"sharkdp/hyperfine": ["README.md", "Cargo.toml"],
"trymirai/uzu": ["crates/uzu-engine/src/backends/mod.rs", "crates/uzu-engine/Cargo.toml"]}
KEYWORDS = re.compile(r"bubblewrap|den(y|ied)|permission|damage|gpu_active|render|rust|v8|rust|host|accessib|focus|rss|warmup|metrics|metal|cuda|backend|quant", re.I)
def fetch(repo, sha, path):
    req=f"repos/{repo}/contents/{path}?ref={sha}"
    p=subprocess.run(["gh","api",req],capture_output=True,text=True,timeout=50)
    if p.returncode: return {"path":path,"error":p.stderr[:160]}
    obj=json.loads(p.stdout)
    if obj.get("encoding")!="base64": return {"path":path,"error":"Non-base64 API content"}
    data=base64.b64decode(obj["content"])
    lines=data.decode("utf8",errors="replace").splitlines()
    hits=[{"line":i,"snippet":line[:155].strip()} for i,line in enumerate(lines,1) if KEYWORDS.search(line) and line.strip() and not line.strip().startswith("<!--")][:8]
    return {"path":path,"sha256":hashlib.sha256(data).hexdigest(),"github_blob_sha":obj.get("sha"),"line_count":len(lines),"anchors":hits,"url":f"https://github.com/{repo}/blob/{sha}/{path}"}
def inspect(item):
    repo, sha = item["repo"], item["commit_sha"]
    return {"repo":repo,"pin":sha,"files":[fetch(repo,sha,p) for p in TARGETS[repo]]}
def main():
    manifest=json.loads((BATCH/"sources.json").read_text())["sources"]
    with concurrent.futures.ThreadPoolExecutor(max_workers=6) as pool:
        rows=list(pool.map(inspect,manifest))
    dest=ROOT/"evidence/upstream-rd-01/intake/source-probe.json"
    dest.parent.mkdir(parents=True,exist_ok=True)
    dest.write_text(json.dumps(rows,indent=2,ensure_ascii=False)+"\n")
    for idx,r in enumerate(rows,1):
        lines=[f"# W{idx:02d} — {r['repo']} / exact-source prequalification",
               "",f"Source pin: `{r['pin']}`.", "Classification: VERIFIED_SOURCE for retrievable files only; executable behavior NOT_RUN.",
               "", "## Source receipts"]
        for f in r["files"]:
            if "error" in f: lines += [f"- **BLOCKED** `{f['path']}`: {f['error']}"]
            else:
                lines += [f"- **VERIFIED_SOURCE** [{f['path']}]({f['url']}) — {f['line_count']} lines; SHA-256 `{f['sha256']}`."]
                lines += [f"  - Line {a['line']}: {a['snippet'].replace(chr(96), chr(39))}" for a in f["anchors"][:4]]
        lines += ["","## Qualification boundary",
                  "Local compilation, runtime integration, hardware compatibility and benefits: **NOT_RUN**.",
                  "Candidate interpretations require independent owner gates; upstream project existence is not GNU6 adoption.",
                  "Licenses: consult pinned source files and transitive dependencies before vendoring or redistributing.",
                  ""]
        file=ROOT/f"docs/agent-results/upstream-rd-01/w{idx:02d}.md"
        file.parent.mkdir(parents=True,exist_ok=True)
        file.write_text("\n".join(lines))
        print(r["repo"], [(f["path"],"error" if "error" in f else "VERIFIED") for f in r["files"]],flush=True)
    return 0 if all("error" not in f for r in rows for f in r["files"]) else 2
if __name__=="__main__": sys.exit(main())
