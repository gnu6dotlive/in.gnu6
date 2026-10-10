# BLOBIN-CUDA-01 — GPU geometry extension (optional prototype)

**Status:** `CUDA_STANDALONE_CSG_CPU_PARITY_VERIFIED / RUNTIME_INTEGRATION_NOT_IMPLEMENTED / LOCAL_PRIVATE`
**Branch:** `feat/blobin-cuda-01-geometry`, from F4B `c5f792d1f5e9fb4e25baa9e2be38ed1b8d69a91b`.
**Worktree:** `/mnt/workbench/workspaces/gnu.in.labs/.worktrees/blobin-cuda01-geometry`
**Purpose:** test whether optional NVIDIA CUDA compute can efficiently evaluate more complex implicit geometry without replacing the portable `wgpu/Vulkan` pipeline or F2 scene authority.

## Scope and provenance

Standalone C++/CUDA 13.4 `experiments/blobin-next-01/CUDA-01/csg.cu`, newly authored without importing GPL/AGPL-ambiguous historical blob.in source. The scene contains ordered analytic circles and CSG operations: union, subtraction, intersection and smoothed union. Each pixel computes a signed field approximation, finite-difference 2D normal and analytic coverage.

Both CPU and CUDA versions use exactly the same declared equation. The CPU reference is a distinct execution path and compares **every pixel** in six fixed scenarios. The falsifier mutates one output and requires the bound to reject the sample. This does **not** validate arbitrary polygons, self-intersections, nested multi-component winding, rendering to GPUI, or topology authority outside the F2 grammar.

**Workstation tested:** NVIDIA GeForce RTX 3070, compute capability 8.6, NVIDIA driver 615.71.09, `nvcc V13.4.92`, `-O3 -std=c++17 -arch=sm_86`. No driver, service, OS or kernel modifications.

## Six actual run results

| Primitives | Pixels | CPU single evaluation (ms) | CUDA warmed kernel mean (ms) | CUDA one-shot transfer+kernel+readback (ms) |
|---:|---:|---:|---:|---:|
| 16 | 49,152 | 11.249 | 0.04249 | 0.2785 |
| 16 | 196,608 | 42.180 | 0.14495 | 0.5453 |
| 64 | 49,152 | 46.374 | 0.14520 | 0.3551 |
| 64 | 196,608 | 182.811 | 0.40750 | 0.8156 |
| 256 | 49,152 | 184.924 | 0.50678 | 0.7082 |
| 256 | 196,608 | 738.270 | 4.47125 | 4.2052 |

**All 6/6 numeric/occupancy probes PASS.** Across them:
- Zero CPU/CUDA occupancy disagreements (including zero ambiguous-boundary mismatches in these synthetic fixtures).
- Maximum absolute signed-field delta ≤ `5.96046448e-8`.
- Maximum normal-component delta ≤ `8.90791416e-5`.
- Maximum antialias coverage delta ≤ `2.08616257e-6`.
- Every test passed its deliberately corrupted-result falsifier.

Numeric qualification thresholds, established before testing: `field ≤ 1e-4`,
`normal ≤ 0.006`, `coverage ≤ 0.007`; invalid/non-finite outputs or any non-near-boundary sign disagreement fail closed.

**Timing warning:** each CPU value is a single host evaluation; CUDA kernel values use warmed CUDA-event timings over 20 kernels; one-shot end-to-end values are separate launches and include copy/sync. These distinct measurement scopes cannot be divided or added blindly. In the largest case, the warmed kernel average is greater than the one-shot time, illustrating run-to-run variance rather than a causal performance invariant. No p95, multiple workload distributions, warm device-switching or production speedup is established. These measurements are exploratory only.

## Qualification artifacts

- Source: `experiments/blobin-next-01/CUDA-01/csg.cu`.
- Evidence: `experiments/blobin-next-01/CUDA-01/fixtures/qualification-20261008/results.log`, SHA-256 `29206e6088cfe39f9eeb4f014fd198ff6a6428aa71a9901a5e8fa37fbc4ce3f4`.
- Source SHA-256: `f90e1b37d50d3a7f62079984145705a6640323e4897c71d26057426b30eb7783`.
- Test: `experiments/blobin-next-01/CUDA-01/scripts/verify_cuda01.py` — 6 cases, SHA, numerical bounds, negative mask falsifier.
- CUDA compiler output is **outside Git**: `/mnt/workbench/build/blobin-cuda01-build/csg`.

## Reproduction and next decision

```sh
R=experiments/blobin-next-01/CUDA-01
/opt/cuda/bin/nvcc -O3 -std=c++17 -arch=sm_86 \
  -o /mnt/workbench/build/blobin-cuda01-build/csg "$R/csg.cu"
python3 "$R/scripts/verify_cuda01.py"
```

Optional next experiment: introduce typed F2-compatible batch geometry, explicit memory/latency budgets, CUDA-vs-`wgpu` comparison on the *same* primitives/shading, then design provider negotiation: `GPU(cuda)` only when qualified, `GPU(wgpu)` portable, `CPU` deterministic, with loss/error recovery delegated to the owning engine. None of those integrations are implemented in CUDA-01.

No push, merge, release, public docs publication or production deploy.
