#!/usr/bin/env python3
"""Offline F3 fixture qualification; stdlib only. Not a performance benchmark."""
from pathlib import Path
from collections import Counter
from hashlib import sha256
import csv
import struct
import sys

ROOT = Path(__file__).resolve().parents[1]
DEFAULT = ROOT / "fixtures" / "qualification-20261008"
MATERIALS = ("ceramic", "gel", "metal", "paper")
TOPOLOGY = ((2, 0), (1, 0), (1, 1), (1, 0), (2, 0))
DIGESTS = ("b4fdb097b45a9772", "f567b7cdcd2a2e12",
           "9bba8ba166c3bab2", "8ede704ee8dd4768", "3494761abf321e6e")

def verify_records(rows, dpr):
    assert len(rows) == 20, f"dpr {dpr}: expected 20 rows, got {len(rows)}"
    for step in range(5):
        for index, name in enumerate(MATERIALS):
            item = rows[step * 4 + index]
            assert int(item["step"]) == step
            assert item["material"] == name
            assert int(item["dpr"]) == dpr
            assert int(item["epoch"]) == step
            assert item["digest"] == DIGESTS[step]
            assert (int(item["components"]), int(item["holes"])) == TOPOLOGY[step]
            assert item["backend"] == "gpu"
            assert int(item["max_rgb"]) <= 8
            assert float(item["mean_rgb"]) <= 0.35
            for field in ("max_height", "max_heat", "max_normal_z"):
                assert 0.0 <= float(item[field]) <= 0.003
    return (
        max(int(r["max_rgb"]) for r in rows),
        max(float(r[f]) for r in rows
            for f in ("max_height", "max_heat", "max_normal_z")),
    )

def verify_png(path, width, height):
    data = path.read_bytes()
    assert data.startswith(b"\x89PNG\r\n\x1a\n"), path
    assert data[12:16] == b"IHDR", path
    assert struct.unpack(">II", data[16:24]) == (width, height), path
    assert data[24:26] == b"\x08\x06", f"expected 8-bit RGBA {path}"

def main():
    root = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT
    assert root.is_dir(), root
    max_rgb = 0
    max_field = 0.0
    for dpr in (1, 2):
        with (root / f"material-ledger-dpr{dpr}.tsv").open() as fd:
            rows = list(csv.DictReader(fd, delimiter="\t"))
        rgb, field = verify_records(rows, dpr)
        max_rgb = max(max_rgb, rgb)
        max_field = max(max_field, field)
        width, height = 256 * dpr, 192 * dpr
        for step in range(5):
            for material in MATERIALS:
                name = f"s{step}-{material}-dpr{dpr}"
                for kind in ("cpu", "gpu", "diff"):
                    verify_png(root / f"{name}-{kind}.png", width, height)
        keys = [sha256((root / f"s0-{m}-dpr{dpr}-cpu.png").read_bytes()).digest()
                for m in MATERIALS]
        assert len(set(keys)) == 4, f"material outputs indistinguishable, dpr={dpr}"
    manifest = (root / "SHA256SUMS.txt").read_text().splitlines()
    assert len(manifest) == 122, "expected 120 PNG + 2 ledgers"
    names = set()
    for line in manifest:
        expected, name = line.split("  ", 1)
        assert name not in names, f"duplicate digest entry: {name}"
        names.add(name)
        assert sha256((root / name).read_bytes()).hexdigest() == expected, name
    assert len(list(root.glob("*.png"))) == 120
    # Deterministic causal mutation: change the observed hole count and demand RED.
    with (root / "material-ledger-dpr1.tsv").open() as fd:
        invalid = list(csv.DictReader(fd, delimiter="\t"))
    invalid[8]["holes"] = "0"
    try:
        verify_records(invalid, 1)
    except AssertionError:
        pass
    else:
        raise AssertionError("negative topology mutation did not turn validator RED")
    print(f"F3_FIXTURE_GATE_PASS dprs=2 states=5 materials=4 scenes=40 pngs=120 "
          f"manifests=122 max_rgb={max_rgb} max_field={max_field:.7f} "
          f"negative_topology_falsifier=PASS")

if __name__ == "__main__":
    main()
