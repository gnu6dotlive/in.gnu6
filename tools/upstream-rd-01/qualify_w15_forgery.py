#!/usr/bin/env python3
"""Compile-fail probes for unforgeable core ExecutionGrant from an external crate."""
import json,os,pathlib,subprocess,tempfile
ROOT=pathlib.Path(__file__).resolve().parents[2]
B=pathlib.Path("/mnt/workbench/workspaces/gnu.in.labs/.probes/upstream-rd-01")
CORE=pathlib.Path("/mnt/workbench/workspaces/gnu.in.labs/gnosix-authority-core")
cases={
 "construct_grant":('use gnosix_authority_core::ExecutionGrant;\nfn main(){let _g=ExecutionGrant{grant_id:String::from("fake")};}\n',("cannot construct","private fields")),
 "deserialize_grant":('use gnosix_authority_core::ExecutionGrant;\nfn main(){let _g:ExecutionGrant=serde_json::from_str("{}").unwrap();}\n',("E0277","Deserialize")),
}
out=[]
with tempfile.TemporaryDirectory(prefix="gnu6-w15-forgery-",dir=B) as td:
    path=pathlib.Path(td);(path/"src").mkdir()
    manifest=f"""[package]
name="gnu6-w15-failed-forgery"
version="0.1.0"
edition="2021"
[dependencies]
gnosix-authority-core={{path="{CORE}"}}
serde_json="1.0"
"""
    (path/"Cargo.toml").write_text(manifest)
    env=os.environ.copy()
    env.update(RUSTUP_TOOLCHAIN="1.97.1",CARGO_HOME=str(B/"mxc-cargo-home"),
               CARGO_TARGET_DIR=str(B/"w15-forgery-target"),CARGO_BUILD_JOBS="2")
    for label,(source,markers) in cases.items():
        (path/"src/main.rs").write_text(source)
        run=subprocess.run(["cargo","check","--offline","-j","2"],cwd=path,env=env,
                           capture_output=True,text=True,timeout=50)
        text=run.stderr
        ok=run.returncode!=0 and all(m in text for m in markers)
        out.append({"case":label,"pass":ok,"cargo_exit":run.returncode,
                    "error_markers":markers,"diagnostic":text[-1700:]})
gate="PASS_2_OF_2" if all(x["pass"] for x in out) else "RED_OR_INCONCLUSIVE"
report={"gate":gate,"cases":out,"source":"gnosix-authority-core@de096640d52c75734813611aca4f858fa7ff47d9",
        "limitations":["Compile-fail only verifies two external access patterns",
                       "Production bootstrap provenance and cross-process serialization not qualified",
                       "The sealed grant remains in-process in W15 adapter"]}
dest=ROOT/"evidence/upstream-rd-01/w15-authority-03/forgery.json"
dest.parent.mkdir(parents=True,exist_ok=True)
dest.write_text(json.dumps(report,indent=2,ensure_ascii=False)+"\n")
print("W15_FORGERY",gate,[(r["case"],r["cargo_exit"],r["pass"]) for r in out])
raise SystemExit(0 if gate=="PASS_2_OF_2" else 2)
