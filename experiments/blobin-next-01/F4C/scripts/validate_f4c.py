#!/usr/bin/env python3
"""F4C offline qualification of incremental GPU stage dispatch area and traces."""
import csv,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]/"fixtures"/"qualification-20261008"
MATERIALS=("ceramic","gel","metal","paper")
DIGESTS=("b4fdb097b45a9772","f567b7cdcd2a2e12",
         "9bba8ba166c3bab2","8ede704ee8dd4768",
         "3494761abf321e6e")
def gate(rows,dpr,reduced):
    assert len(rows)==20,len(rows)
    full=256*dpr*192*dpr
    for state in range(5):
        for m,material in enumerate(MATERIALS):
            r=rows[state*4+m]
            assert int(r["state"])==state,r
            assert r["material"]==material,r
            assert int(r["dpr"])==dpr,r
            assert r["reduced"]==str(reduced).lower(),r
            assert r["frame"]=="local",r
            assert r["digest"]==DIGESTS[state],r
            assert int(r["full_px"])==full,r
            assert int(r["stage0_px"])==12*dpr*9*dpr,r
            stage1=int(r["stage1_px"]);stage2=int(r["stage2_px"])
            assert stage1==stage2,r
            assert 0<stage1<full,r
            echo=int(r["stage3_px"])
            assert echo==(stage2 if reduced else full),r
            assert int(r["dispatches"])==4,r
            assert int(r["submissions"])==2,r
            assert int(r["max_channel"])==0,r
            assert int(r["workgroups"])>0,r
            assert float(r["wall_ms"])>0,r
            assert sum(int(r[f]) for f in
                ("stage0_px","stage1_px","stage2_px","stage3_px"))<4*full,r
def verify():
    assert ROOT.is_dir(),ROOT
    for dpr in (1,2):
        for reduced in (True,False):
            filename=f"f4c-dpr{dpr}-reduced{str(reduced).lower()}.tsv"
            with (ROOT/filename).open() as f:rows=list(csv.DictReader(f,delimiter="\t"))
            gate(rows,dpr,reduced)
    sums=(ROOT/"SHA256SUMS.txt").read_text().splitlines()
    assert len(sums)==4
    names=set()
    for line in sums:
        value,name=line.split("  ",1)
        assert name not in names
        names.add(name)
        assert hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==value
    with (ROOT/"f4c-dpr1-reducedfalse.tsv").open() as f:
        altered=list(csv.DictReader(f,delimiter="\t"))
    altered[0]["stage1_px"]="49152"
    try:gate(altered,1,False)
    except AssertionError:pass
    else:raise AssertionError("negative full-frame falsifier failed")
    print("F4C_FIXTURE_GATE_PASS cases=80 dprs=2 reduced_modes=2 "
          "stage_oracles=4 max_rgb=0 sha256=4 falsifier=PASS")
if __name__=="__main__":verify()
