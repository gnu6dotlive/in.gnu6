# CUDA-04 — Geometry identity and resident runtime qualification

Private clean-room Rust lab. CUDA-04 builds on CUDA-02 numerical CSG, CUDA-03 GPU-resident sessions and F2 provenance. No production blob.in or Gnosix runtime is modified.

## CUDA-04C reproducibility

Use an RTX 3070-class CUDA device supporting `sm_86`; this is the qualified environment, not a portability guarantee. Rust dependencies are installed from local Cargo cache.

```bash
cd experiments/blobin-next-01
mkdir -p /mnt/workbench/build/blobin-cuda04c
/opt/cuda/bin/nvcc -O3 -std=c++17 -arch=sm_86 -shared -Xcompiler -fPIC \
  -o /mnt/workbench/build/blobin-cuda04c/libblobin_cuda03.so CUDA-03/native/resident.cu
cd CUDA-04
cargo test --offline --locked
cargo fmt -- --check
cargo clippy --offline --locked --all-targets -- -D warnings
cargo run --release --offline --locked --bin qualify04c -- \
  --cuda-lib /mnt/workbench/build/blobin-cuda04c/libblobin_cuda03.so --iterations 10 \
  | tee fixtures/qualification-20261009/results.log
python3 scripts/validate04c.py
```
The evidence manifest `fixtures/qualification-20261009/SHA256SUMS.txt` covers the archived ledger, environment record and 56-test cumulative regression ledger. Repeating the run changes measured timings and requires regenerating the manifest; the validator intentionally checks historical evidence, not new runs.

## Observables and limitations

Each case asserts oracle parity for the initial geometry, cache hit, material change, changed F2 epoch, each of 10 further geometry changes, same-area width/height transposition and changed DPR. No stale samples should be returned after edited geometry. Resource counts are from CUDA-03 and exclude driver/allocator/staging overhead.

The recorded host-clock `update_p50_ms` and `update_p95_ms` cover an existing resident session's `update→launch→synchronize→readback` and copying output; CPU oracle verification occurs after the timer. They are **not** CUDA kernel event times, GPU-only measurements, throughput guarantees or independent multi-session performance distributions.

CUDA shared libraries remain native executable code loaded with `dlopen`. The bridge observes changes to the supplied library **path**, not changes to bytes at an identical path. GPU asynchronous loss cannot be detected by an in-memory cache hit; external device-health generation and trusted library attestation remain open prerequisites. There is no live production runtime integration, release, merge or push.
