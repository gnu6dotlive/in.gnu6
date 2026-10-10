#!/usr/bin/env python3
"""Actual MXC SDK network policy support refusal, with spawned positive control."""
import hashlib,json,pathlib,subprocess,tempfile
ROOT=pathlib.Path(__file__).resolve().parents[2]
B=pathlib.Path("/mnt/workbench/workspaces/gnu.in.labs/.probes/upstream-rd-01")
SRC=B/"mxc-c6f301"
PIN="c6f301d53a1430c4c921a05c57af838f7392348f"
assert subprocess.check_output(["git","rev-parse","HEAD"],cwd=SRC,text=True).strip()==PIN
BINARY=B/"mxc-fixture-target/debug/gnu6-mxc-net-refusal"
assert BINARY.is_file()
cases=[]
with tempfile.TemporaryDirectory(prefix="gnu6-mxc-network-only-",dir=B) as t:
    marker=pathlib.Path(t)/"marker.txt"
    for name in ("default-deny","egress-allow","ingress-allow"):
        marker.unlink(missing_ok=True)
        proc=subprocess.run([str(BINARY),name,t],capture_output=True,text=True,timeout=12)
        exists=marker.exists()
        if name=="default-deny":
            good=proc.returncode==0 and exists and marker.read_text()=="EXECUTED"
        elif name=="egress-allow":
            good=proc.returncode==3 and not exists and "slirp4netns" in proc.stderr
        else:
            good=proc.returncode==3 and not exists and "network.ingress.default='allow' is not supported" in proc.stderr
        cases.append({"mode":name,"exit":proc.returncode,"marker_present":exists,
                      "stderr":proc.stderr[:510],"stdout":proc.stdout[:150],"pass":good})
gate="PASS_3_OF_3" if len(cases)==3 and all(c["pass"] for c in cases) else "RED"
receipt={"gate":gate,"mxc_sdk":"1.0.0","source_pin":PIN,
         "test_type":"actual SDK v1 Process/Bubblewrap on local Linux",
         "cases":cases,"egress_supported_here":False,
         "limitations":["slirp4netns absent; no successful MXC egress-allow run",
                        "No positive egress network control", "No production execution",
                        "Refusal to spawn does not certify every host policy"]}
out=ROOT/"evidence/upstream-rd-01/broker-net-02"
out.mkdir(parents=True,exist_ok=True)
(out/"mxc-network-refusal.json").write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+"\n")
print("MXC_NETWORK_POLICY_REFUSAL",gate,[(x["mode"],x["exit"],x["pass"]) for x in cases])
raise SystemExit(0 if gate=="PASS_3_OF_3" else 2)
