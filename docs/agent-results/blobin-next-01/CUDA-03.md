# BLOBIN-CUDA-03 — Resident Geometry & Performance

**Status:** `CUDA03_RESIDENT_CUDA_VULKAN_NUMERIC_PARITY_VERIFIED / BOUNDED_BENCHMARK_VERIFIED / PRODUCTION_NOT_PROVEN`

**Baseline:** CUDA-02 commit `b8c30c39a90b8effd01614839e6ae0425c79e143`.
**Branch:** `feat/blobin-cuda-03-resident`.
**Private worktree:** `/mnt/workbench/workspaces/gnu.in.labs/.worktrees/blobin-cuda03-resident`.
**Hardware:** NVIDIA RTX 3070 8 GiB, compute capability 8.6, CUDA 13.4, wgpu 30.0.1 / Vulkan.

## Implementation and authority boundary

An original standalone Rust/CUDA laboratory at `experiments/blobin-next-01/CUDA-03/` retains device-side buffers between kernel invocations. CUDA maintains primitives, outputs, CUDA timing events and owned session resources. Vulkan maintains a wgpu device, shader pipeline, bind group and input/output storage. Both use the exact numerical CSG grammar from qualified CUDA-02, without modifying its code.

The Rust CPU evaluator remains the oracle. Each input is constructed from the F2 scene and preserves its F2 epoch/digest/IDs; neither GPU backend can mint scene identity or modify F2 topology. **The F2 digest does not cover independently edited CSG primitives.** Geometry edits therefore need their own future typed content digest for caching and reconciliation.

CUDA is loaded only from an explicitly supplied absolute shared-library path. `dlopen` is process-native code execution and **not** a sandbox or verified plugin boundary.

## Observed matrix (20 warm-state dispatches per backend)

The CUDA values below are CUDA **device-event kernel p95**. The Vulkan values are **host-observed synchronized submit+wait p95**. They are different instruments; no direct GPU speedup ratio is claimed.

| CSG primitives | Resolution | CUDA kernel event p95 (ms) | Vulkan host synchronized p95 (ms) |
|---:|---|---:|---:|
| 16 | 256×192 | 0.028672 | 0.110268 |
| 16 | 512×384 | 0.080640 | 0.338612 |
| 64 | 256×192 | 0.097280 | 0.284073 |
| 64 | 512×384 | 0.296000 | 0.519834 |
| 256 | 256×192 | 0.396352 | 0.597734 |
| 256 | 512×384 | 1.233920 | 1.637402 |

Resident memory is bounded to one 16-byte tuple per primitive and one 16-byte tuple per pixel; Vulkan also maintains a 16-byte uniform. The largest scene allocates **3,149,824 bytes CUDA** and **3,149,840 bytes Vulkan** in counted buffers. These numbers exclude drivers, pipelines, staging memory and program overhead.

For the largest scene, CUDA complete readback was 1.853 ms, whereas Vulkan readback was 28.621 ms in this single run. These are **one-shot end-to-end readbacks**, not robust latency distributions or guaranteed platform characteristics.

## Verification

- **6/6 scenario pairs passed:** 16, 64, 256 primitives × DPR 1/2.
- **12/12 resident GPU result comparisons passed** against Rust CPU on all pixels, with zero occupancy disagreements.
- **6/6 geometry edits passed** on both resident backends. An 0.080 increase in root-circle radius changes the numerical field and requires recalculation. The old output is rejected; resident buffer byte capacity remains unchanged.
- **5/5 negative gates passed:** CUDA library not explicitly provided, CUDA library absent, too few benchmark iterations, excessive primitive count, Vulkan device unavailable.
- Independent offline evidence gate: **24 records**, two SHA-256-covered files and a deliberately corrupted mismatch counter rejected.
- Rust unit tests **5/5**, formatter and strict Clippy **PASS**.

The maximum numerical errors remain inside CUDA-02's predeclared limits: signed field ≤0.0001, normal component ≤0.006, coverage ≤0.007; exact foreground/background occupancy.

## Reproduction

Follow `experiments/blobin-next-01/CUDA-03/README.md`: build the native
provider with `nvcc`, run `cargo test --offline --locked`, then execute
`scripts/check_negative.sh` and `scripts/validate_cuda03.py`.

Private evidence lives in `fixtures/qualification-20261008/`. The six-case
results SHA-256 is `cbc9a8b0239eda17943e085826d006f739e45f500d6b295860e210af640a7c54`.
No binaries are tracked.

**Open:** device-loss resilience, interop, kernel parity on other GPUs,
native-library trust boundary, GPU/host end-to-end p95 across multiple
sessions, idle energy, and integration into the Gnosix runtime.
This slice stays local and unpublished.
