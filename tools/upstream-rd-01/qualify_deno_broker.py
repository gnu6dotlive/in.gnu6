#!/usr/bin/env python3
"""Pinned Deno broker source contract with independent red mutations (no Deno runtime)."""
import base64
import hashlib
import json
import subprocess
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
REPO="denoland/deno"
PIN="a18ce33715e30cd2b0d99c7e322ef11a65490e4d"
paths=["runtime/permissions/broker.rs","runtime/permissions/lib.rs"]
def source(path):
    p=subprocess.run(["gh","api",f"repos/{REPO}/contents/{path}?ref={PIN}"],
                     capture_output=True,text=True,timeout=50)
    assert p.returncode==0,p.stderr[:500]
    obj=json.loads(p.stdout)
    raw=base64.b64decode(obj["content"])
    return {"text":raw.decode("utf8"),"sha256":hashlib.sha256(raw).hexdigest(),
            "blob_sha":obj["sha"],"path":path,
            "url":f"https://github.com/{REPO}/blob/{PIN}/{path}"}
broker,lib=map(source,paths)
required_broker=[
    "if response.id != id", "Permission broker response ID mismatch",
    '"allow" => BrokerResponse::Allow', '"deny" => BrokerResponse::Deny',
    "Permission broker unknown result variant", "std::process::exit(BROKER_EXIT_CODE)",
]
required_lib=["if let Some(resp) = maybe_check_with_broker",
              "BrokerResponse::Allow =>", "BrokerResponse::Deny { message } =>",
              "match self {\n      PermissionState::Granted => {"]
def gate(b,l):
    return all(x in b for x in required_broker) and all(x in l for x in required_lib) and b.count("std::process::exit(BROKER_EXIT_CODE)") >= 2 \
       and l.index("if let Some(resp) = maybe_check_with_broker") < l.index("match self {",l.index("if let Some(resp) = maybe_check_with_broker"))
assert gate(broker["text"],lib["text"])
mutations={}
for key in required_broker:
    mutate=broker["text"].replace(key,"RED_GUARD_REMOVED",1)
    mutations["broker:"+key]="RED_CONFIRMED" if not gate(mutate,lib["text"]) else "BAD_ORACLE"
for key in required_lib:
    mutate=lib["text"].replace(key,"RED_GUARD_REMOVED",1)
    mutations["lib:"+key]="RED_CONFIRMED" if not gate(broker["text"],mutate) else "BAD_ORACLE"
print("MUTATION_STATUS", {k:v for k,v in mutations.items() if v!="RED_CONFIRMED"})
assert all(value=="RED_CONFIRMED" for value in mutations.values())
anchors={}
for entry in (broker,lib):
    textlines=entry["text"].splitlines()
    keywords=required_broker if entry["path"].endswith("broker.rs") else required_lib
    anchors[entry["path"]]=[{"keyword":needle,"line":entry["text"][:entry["text"].index(needle)].count("\n")+1}
                              for needle in keywords]
out=ROOT/"evidence/upstream-rd-01/deno-permission-01"
out.mkdir(parents=True,exist_ok=True)
report={"repo":REPO,"pinned_commit":PIN,"source_gate":"PASS",
        "mutations":mutations,"source_files":[{k:v for k,v in e.items() if k!="text"} for e in (broker,lib)],
        "anchors":anchors,"runtime_status":"NOT_RUN_DENO_BINARY_UNAVAILABLE",
        "limitations":["Textual source invariants only; not proof of actual Deno permissions at runtime",
                       "Broker presence is optional and application permission state still has separate semantics",
                       "Unknown result and response-ID guard coverage inspected, not executed"]}
(out/"qualification.json").write_text(json.dumps(report,indent=2,ensure_ascii=False)+"\n")
print("DENO_PINNED_SOURCE=PASS; GUARDED_PATHS=",len(required_broker)+len(required_lib),
      "MUTANTS_RED=",len(mutations),"RUNTIME=NOT_RUN")
