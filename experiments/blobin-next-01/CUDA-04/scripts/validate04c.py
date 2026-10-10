#!/usr/bin/env python3
"""Independent offline CUDA-04C ledger validation, including tamper rejection."""
from collections import defaultdict
from hashlib import sha256
from math import ceil, isfinite
from pathlib import Path
import sys

FIXTURE = Path(__file__).resolve().parents[1] / "fixtures" / "qualification-20261009"
PAIRS = {(b, c, d) for b in ("cuda", "vulkan") for c in (16, 64, 256) for d in (1, 2)}

def inspect(data):
    cases, iterations = {}, defaultdict(dict)
    negatives, final = [], []
    for line in data.splitlines():
        if "FAIL" in line or not line.startswith("CUDA04C_"):
            raise AssertionError("unknown or failed ledger record")
        tokens = line.split()
        kind = tokens[0]
        fields = dict(t.split("=", 1) for t in tokens[1:])
        if kind == "CUDA04C_NEGATIVE_PASS":
            assert set(fields) == {"absent", "switched", "recovered", "revoked"}
            assert set(fields.values()) == {"PASS"}
            negatives.append(fields)
        elif kind == "CUDA04C_ALL_PASS":
            final.append(fields)
        elif kind in ("CUDA04C_CASE_PASS", "CUDA04C_ITER_PASS"):
            pair = (fields["backend"], int(fields["count"]), int(fields["dpr"]))
            assert pair in PAIRS
            if kind == "CUDA04C_CASE_PASS":
                assert pair not in cases and int(fields["iterations"]) == 10
                for attr in ("numeric", "stale", "epoch", "transpose", "rebuild"):
                    assert fields[attr] == "PASS"
                assert 0 < int(fields["resident_bytes"]) <= 4_000_000
                for attr in ("cold_ms", "cached_ms", "edited_ms", "update_p50_ms", "update_p95_ms", "rebuild_ms"):
                    assert isfinite(float(fields[attr])) and float(fields[attr]) > 0
                assert float(fields["update_p95_ms"]) >= float(fields["update_p50_ms"])
                cases[pair] = fields
            else:
                i, ms = int(fields["index"]), float(fields["host_ms"])
                assert 0 <= i < 10 and isfinite(ms) and 0 < ms < 1_000_000
                assert i not in iterations[pair]
                iterations[pair][i] = ms
        else:
            raise AssertionError("unexpected ledger record")
    assert set(cases) == PAIRS and set(iterations) == PAIRS
    assert len(negatives) == len(final) == 1
    assert final[0] == {"cases": "12", "iterations": "120", "negatives": "4"}
    for pair in PAIRS:
        assert set(iterations[pair]) == set(range(10))
        ranked = sorted(iterations[pair].values())
        for q, attr in ((0.5, "update_p50_ms"), (0.95, "update_p95_ms")):
            assert abs(ranked[ceil(10*q)-1] - float(cases[pair][attr])) <= 0.000002
    return len(cases), sum(len(v) for v in iterations.values())

def main():
    ledger = (FIXTURE / "results.log").read_text()
    assert inspect(ledger) == (12, 120)
    hashes = (FIXTURE / "SHA256SUMS.txt").read_text().splitlines()
    assert len(hashes) == 3
    checked = set()
    for entry in hashes:
        digest, name = entry.split("  ", 1)
        assert name in ("results.log", "environment.txt", "cumulative-tests.log") and name not in checked
        assert digest == sha256((FIXTURE / name).read_bytes()).hexdigest()
        checked.add(name)
    regression = (FIXTURE / "cumulative-tests.log").read_text()
    phases = [line.split("phase=", 1)[1] for line in regression.splitlines()
              if line.startswith("CUMULATIVE_PHASE_PASS phase=")]
    assert phases == ["F1", "F2", "F3", "F4", "F4B", "F4C", "CUDA-02", "CUDA-03", "CUDA-04"]
    assert regression.count("CUMULATIVE_ALL_PASS phases=9") == 1
    assert sum(int(line.split()[3]) for line in regression.splitlines()
               if line.startswith("test result: ok.")) == 56
    tampered = ledger.replace("numeric=PASS", "numeric=FAIL", 1)
    assert tampered != ledger
    try:
        inspect(tampered)
    except AssertionError:
        pass
    else:
        raise AssertionError("corruption accepted")
    print("CUDA04C_OFFLINE_VALIDATION_PASS cases=12 iterations=120 files=3 regressions=56 corruption=REJECTED")

if __name__ == "__main__":
    try:
        main()
    except (AssertionError, OSError, ValueError, KeyError) as error:
        sys.exit(f"CUDA04C_VALIDATION_FAIL {error}")
