#!/usr/bin/env python3
"""F4 bounded postprocessing fixture gate. Stdlib-only; no GPU required to audit."""
import csv
import hashlib
import struct
from pathlib import Path

ROOT=Path(__file__).resolve().parents[1]/"fixtures"/"qualification-20261008"
STATES=((2,0),(1,0),(1,1),(1,0),(2,0))
DIGESTS=("b4fdb097b45a9772","f567b7cdcd2a2e12","9bba8ba166c3bab2",
         "8ede704ee8dd4768","3494761abf321e6e")
MATERIALS=("ceramic","gel","metal","paper")
def check_png(path,w,h):
    raw=path.read_bytes()
    assert raw[:8]==b"\x89PNG\r\n\x1a\n",path
    assert raw[12:16]==b"IHDR",path
    assert struct.unpack(">II",raw[16:24])==(w,h),path
    assert raw[24:26]==b"\x08\x06",path
def verify_rows(rows,dpr,reduced):
    assert len(rows)==40, len(rows)
    for step in range(5):
        for j,material in enumerate(MATERIALS):
            for frame in range(2):
                row=rows[step*8+j*2+frame]
                assert int(row["state"])==step,row
                assert row["material"]==material,row
                assert int(row["frame"])==frame,row
                assert int(row["dpr"])==dpr,row
                assert int(row["epoch"])==step,row
                assert row["digest"]==DIGESTS[step],row
                assert (int(row["components"]),int(row["holes"]))==STATES[step],row
                assert row["backend"]=="gpu",row
                assert int(row["dispatches"])==4,row
                assert int(row["max_rgb"])<=12,row
                assert float(row["mean_rgb"])<=0.5,row
                expected_reset=(frame+1 if reduced else 1)
                assert int(row["reset_cpu"])==expected_reset,row
                assert int(row["reset_gpu"])==expected_reset,row
def main():
    items=((ROOT/"standard",1,False),(ROOT/"standard",2,False),
           (ROOT/"reduced",1,True))
    worst=0;avg=0.0
    for directory,dpr,reduced in items:
        file=directory/f"ledger-dpr{dpr}-reduced{str(reduced).lower()}.tsv"
        with file.open() as f: rows=list(csv.DictReader(f,delimiter="\t"))
        verify_rows(rows,dpr,reduced)
        worst=max(worst,max(int(r["max_rgb"]) for r in rows))
        avg=max(avg,max(float(r["mean_rgb"]) for r in rows))
        for step in range(5):
            for material in MATERIALS:
                for frame in range(2):
                    name=f"state{step}-{material}-frame{frame}-dpr{dpr}"
                    for mode in ("cpu","gpu","diff"):
                        check_png(directory/f"{name}-{mode}.png",256*dpr,192*dpr)
    digest_lines=(ROOT/"SHA256SUMS.txt").read_text().splitlines()
    assert len(digest_lines)==363
    seen=set()
    for line in digest_lines:
        sha,rel=line.split("  ",1)
        assert rel not in seen
        seen.add(rel)
        assert hashlib.sha256((ROOT/rel).read_bytes()).hexdigest()==sha
    assert len(list((ROOT/"standard").glob("*.png")))==240
    assert len(list((ROOT/"reduced").glob("*.png")))==120
    with (ROOT/"standard"/"ledger-dpr1-reducedfalse.tsv").open() as f:
        rows=list(csv.DictReader(f,delimiter="\t"))
    rows[16]["holes"]="0" # intentionally falsify punched-hole topology
    try:
        verify_rows(rows,1,False)
    except AssertionError:pass
    else:raise AssertionError("falsifier failed to reject wrong hole count")
    print(f"F4_FIXTURE_GATE_PASS scenes=120 png=360 sha256=363 "
          f"worst_rgb={worst} max_mean={avg:.6f} falsifier=PASS")
if __name__=="__main__":main()
