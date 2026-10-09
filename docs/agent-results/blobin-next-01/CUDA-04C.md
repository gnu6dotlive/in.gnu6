# BLOBIN-CUDA-04C — GPU Runtime Qualification

**Status:** `CUDA04C_12_OF_12_GPU_PARITY_VERIFIED / RESIDENT_RECONCILIATION_VERIFIED / PRODUCTION_NOT_PROVEN`

**Parent:** CUDA-04B `05154acb11b48d2ae54c53c3ea65ba3caa819921`
**Branch:** `feat/blobin-cuda-04c-qualification`
**Hardware:** GeForce RTX 3070 8 GiB, NVIDIA driver 615.71.09, CUDA compiler 13.4, Rust 1.97.0.

## Qualified observations

The real hardware qualification used CUDA-03's native `sm_86` library rebuilt from source, and resident wgpu/Vulkan sessions. Each sampled output was compared against the CUDA-02 Rust CPU oracle using the *unchanged* distance, normal, coverage and occupancy tolerances.

- **12/12 GPU configurations:** 16/64/256 primitives × DPR 1/2 × CUDA/Vulkan.
- **120/120 host-clock update samples:** 10 independent content mutations per configuration. Every one passed numerical parity.
- **12/12 same-area width/height swaps:** session reconstruction is required even when pixel count and resident byte capacity are unchanged.
- **12/12 changed DPR reconstructions:** recompute with correctly rebuilt buffer capacity.
- **12/12 F2 epoch reattachment checks:** same geometry digest can hit cache while returning *current* scene provenance.
- **12/12 stale content rejection:** edited geometry causes cache miss and produces a CPU-parity result.
- **4/4 controlled CUDA negative gates:** missing native provider, changed path, recovery after invalidation, removal of provider.
- The independent offline validator checks 134 ledger records, three SHA-256-covered evidence files (including cumulative Rust tests) and deliberate corruption rejection.
- **56/56 cumulative Rust unit tests passed**, across nine crates (F1–F4C, CUDA-02, CUDA-03, CUDA-04); strict Clippy and formatting passed.

The native library path change bug identified during this phase is fixed in `resident.rs`: switching or withdrawing the configured CUDA path invalidates the session and any cached numerical frame before further evaluation.
## Measured latency — single qualification run

Values are **host-clock milliseconds** for existing resident sessions: geometry upload, kernel dispatch, synchronization, whole-frame readback, host output copying. They exclude CPU-oracle verification and differ from CUDA GPU device-event timings.

| Primitives | DPR | CUDA p50 / p95 (ms) | Vulkan p50 / p95 (ms) |
|---:|---:|---:|---:|
| 256 | 1 | 1.186 / 5.102 | 0.996 / 5.412 |
| 256 | 2 | 9.613 / 57.095 | 7.813 / 12.484 |

The especially high CUDA 256/DPR2 p95 in this run shows variability. Only ten samples per configuration were collected, on one device in one process; do **not** infer stable percentiles or general backend speedups.

The largest scene tracks **3,149,824 counted resident bytes (CUDA)** and **3,149,840 (Vulkan)**. These numbers exclude staging, driver allocations and other memory overhead. Cached host frames also consume memory.

## Evidence and reproduction

- `experiments/blobin-next-01/CUDA-04/fixtures/qualification-20261009/results.log`
- `experiments/blobin-next-01/CUDA-04/fixtures/qualification-20261009/environment.txt`
- `experiments/blobin-next-01/CUDA-04/fixtures/qualification-20261009/SHA256SUMS.txt`
- `experiments/blobin-next-01/CUDA-04/scripts/validate04c.py`
- `experiments/blobin-next-01/CUDA-04/README.md`

## Unqualified boundaries

No injected loss of CUDA context/Vulkan device has been tested. A cached host sample cannot prove an asynchronously lost GPU is alive. The native path is checked but neither the shared-object bytes at that path nor its signature are checked on each call. `dlopen` is native process execution, not a sandbox or trust boundary. GPU vendor portability, idle energy, multi-session confidence intervals, and production Gnosix/blob.in integration are not established.

**Publish/merge/deploy:** none. This is a private clean-room qualification, not production authorization.
