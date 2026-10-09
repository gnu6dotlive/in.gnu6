#!/usr/bin/env python3
"""Independent structural and semantic audit: do not import the report generator."""
from hashlib import sha256
import json
from pathlib import Path
import subprocess
import sys

BASE=Path(__file__).resolve().parent
EVID=BASE/"evidence"
PUBLIC_KEYS={
    "schema","protocol_commit","observed_utc","baseline_control",
    "namespace_root_probe","namespace_inside_root","namespace_distinct_host_uid",
    "subordinate_mapping","subordinate_peer_distinct_host_uid",
    "distinct_service_identity_gate","production_authorization","raw_sha256",
    "source_visibility","limits"
}
def check(raw,analysis,public):
    assert raw["schema"]=="CUDA05D.IDENTITIES.v1"
    assert analysis["schema"]=="CUDA05D.ANALYSIS.v1"
    assert public["schema"]=="CUDA05D.PUBLIC.v1"
    assert set(public)==PUBLIC_KEYS
    assert set(raw["runs"])=={"baseline","namespace_root","subordinate_auto"}
    assert raw["socket_mode"]=="0o600" and raw["socket_directory_mode"]=="0o700"
    host=raw["host_euid"]
    baseline=raw["runs"]["baseline"]
    userns=raw["runs"]["namespace_root"]
    auto=raw["runs"]["subordinate_auto"]
    assert baseline["process"]["exit_code"]==0
    assert baseline["kernel_peer"]["uid"]==host
    assert baseline["client_claim"]["self_euid"]==host
    assert analysis["baseline_control"]=="PASS"
    assert public["baseline_control"]=="PASS"
    observed=userns["process"]["exit_code"]==0 and userns["kernel_peer"] is not None
    assert (analysis["namespace_root_probe"]=="OBSERVED")==observed
    assert public["namespace_root_probe"]==analysis["namespace_root_probe"]
    if observed:
        assert analysis["namespace_inside_root"] == (userns["client_claim"]["self_euid"]==0)
        assert analysis["namespace_distinct_host_uid"] == (userns["kernel_peer"]["uid"]!=host)
    auto_observed=auto["process"]["exit_code"]==0 and auto["kernel_peer"] is not None
    assert (analysis["subordinate_mapping"]=="observed_peer")==auto_observed
    assert analysis["subordinate_peer_distinct_host_uid"] == (
        auto["kernel_peer"] is not None and auto["kernel_peer"]["uid"] != host)
    assert analysis["distinct_service_identity_gate"]=="NOT_QUALIFIED"
    assert public["distinct_service_identity_gate"]=="NOT_QUALIFIED"
    assert analysis["production_authorization"]==public["production_authorization"]=="DENIED"
    assert analysis["methodological_decision"]=="INCOMPLETE_EVIDENCE_DO_NOT_PROMOTE"
    assert analysis["raw_sha256"]==public["raw_sha256"]
    assert public["protocol_commit"]==analysis["protocol_commit"]
    assert all(public[k]==analysis[k] for k in (
        "namespace_inside_root","namespace_distinct_host_uid","subordinate_mapping",
        "subordinate_peer_distinct_host_uid"))
    # Disallow accidental raw identity, PID, local command details on public surface.
    exported=json.dumps(public,sort_keys=True)
    assert '"kernel_peer"' not in exported and '"pid"' not in exported
    assert '"host_euid"' not in exported and '"self_euid"' not in exported
    assert '"stderr"' not in exported and "tension_atoi" not in exported

def main():
    manifest=(EVID/"SHA256SUMS.txt").read_text().splitlines()
    expected={"identity-observations.json","interpretation.json","public-results.json",
              "../05D-PREREG.md","../probe_identity05d.py","../evaluate05d.py"}
    names=set()
    for entry in manifest:
        value,name=entry.split("  ",1)
        assert name in expected and name not in names
        assert sha256((EVID/name).read_bytes()).hexdigest()==value
        names.add(name)
    assert names==expected
    raw_bytes=(EVID/"identity-observations.json").read_bytes()
    raw=json.loads(raw_bytes)
    analysis=json.loads((EVID/"interpretation.json").read_text())
    public=json.loads((EVID/"public-results.json").read_text())
    assert analysis["raw_sha256"]==sha256(raw_bytes).hexdigest()
    commit=analysis["protocol_commit"]
    registered=subprocess.check_output(["git","-C",str(BASE),"show",f"{commit}:experiments/blobin-next-01/CUDA-05/methodology/05D-PREREG.md"],text=True)
    assert registered.encode()==(BASE/"05D-PREREG.md").read_bytes()
    check(raw,analysis,public)
    changed=json.loads(raw_bytes)
    changed["runs"]["baseline"]["kernel_peer"]["uid"]+=1
    try: check(changed,analysis,public)
    except AssertionError:pass
    else:raise AssertionError("falsifier accepted an incorrect baseline peer UID")
    print("CUDA05D_INDEPENDENT_PASS prereg_commit="+commit[:12]+
          " records=3 tracked_hashes=6 falsifier=REJECTED principal_gate=NOT_QUALIFIED")

if __name__=="__main__":
    try: main()
    except (AssertionError,KeyError,ValueError,OSError) as err:
        sys.exit("CUDA05D_VALIDATION_FAIL "+str(err))
