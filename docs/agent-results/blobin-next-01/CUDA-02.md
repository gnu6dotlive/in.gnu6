# BLOBIN-CUDA-02 — Unified Geometry Provider qualification

**Status:** `CUDA02_THREE_BACKEND_NUMERIC_PARITY_VERIFIED / OPTIONAL_CUDA_ABI_PROVEN / PRIVATE_LOCAL / HOST_INTEGRATION_NOT_PROVEN`
**Date:** 2026-10-08 (local workstation)
**Worktree:** `/mnt/workbench/workspaces/gnu.in.labs/.worktrees/blobin-cuda02-provider`
**Branch:** `feat/blobin-cuda-02-unified-provider`
**Source baseline:** F4C qualified commit `98a98f5a1db3790d4e46a1f85f30165a90d9a6a4`; isolated CUDA-01 result `0a64925ec5999359842a14abfd4bc1c3d97b5de9` brought into this worktree by a separate clean cherry-pick `a76819420b6c8a6317ea26867264abff0b4a4477`. Neither originating branch was mutated.

## Qualified implementation

A new original Rust crate `experiments/blobin-next-01/CUDA-02/` defines
one validated input and the same numerical CSG operation grammar across:

- `CPU` pure Rust reference,
- `wgpu/Vulkan` original WGSL shader with numeric GPU readback,
- `CUDA` optional native shared library compiled with `nvcc` and invoked via a bounded C ABI, loaded explicitly through an absolute path.

The request is created from an F2 `Scene` and carries its epoch, scene digest and left/right IDs. These **provenance metadata** remain attached by Rust; compute backends do not accept or return semantic grants. This is **not** a proof of an unforgeable security capability. F2 continues to own its original topology engine and hit targets; numerical CSG is not integrated into F2 authority.

Primitives `repr(C)` 16 bytes: `(x,y,radius,operation)`; output samples `repr(C)` 16 bytes: `(signed_distance,nx,ny,coverage)`. Operations: union, subtraction, intersection, smooth union. Bounded counts 1–256; no more than 1,048,576 pixels; nonfinite/invalid primitives rejected; CPU is the oracle for each of the other two providers.

## Observed GPU qualification

**Hardware:** NVIDIA GeForce RTX 3070 / CUDA compute 8.6 / Vulkan DiscreteGpu.
**Toolchain:** Rust 1.97.0; wgpu 30.0.1; CUDA 13.4 / NVCC V13.4.92; driver as installed, unchanged.
**Configurations:** 16, 64, 256 primitives, each at 256×192 and 512×384, **6 synthetic scenes**. Each scene was evaluated with three actual backends: **18/18 evaluations PASSED**.

| Gate | Result |
|---|---|
| All three provider outputs per scene | **18/18 PASS** |
| Maximum signed field error vs CPU | **0.000000119** |
| Maximum normal component error vs CPU | **0.000694662** |
| Maximum coverage error vs CPU | **0.000014365** |
| Occupancy sign disagreements | **0** |
| Near-boundary sign disagreements | **0** |
| F2 epoch/digest/IDs preserved in every result | **PASS** |
| `auto` with valid CUDA library | **CUDA selected**, PASS |
| `auto` with missing CUDA library | **Vulkan selected**, PASS |
| `auto` without Vulkan adapter or CUDA library | **CPU selected**, PASS |
| Required CUDA without library | **Fail closed**, exit 1 |
| Required Vulkan without adapter | **Fail closed**, exit 1 |
| Required CUDA with missing library file | **Fail closed**, exit 1 |
| Invalid primitive count 257 | **Fail closed**, exit 1 |
| Independent fixtures verifier | **PASS**: 6 scenes, 18 outputs, 3 fallback cases, 4 negative checks, 3 SHA-256, occupancy corruption falsifier |

The maximum observed numerical errors fit the predeclared oracle bounds:
`distance ≤0.0001`, `normal ≤0.006`, `coverage ≤0.007`,
**exact occupancy signs** for these six scenes. Error tolerances are
specific to this lab, not a general topology assurance.

**Illustrative 256-primitive, 512×384 result**: CPU-oracle one-shot 2404.44 ms, Rust CPU-backend one-shot 2135.86 ms, Vulkan-backend call 269.37 ms, CUDA-backend call 223.16 ms. These are **one-run wallclock measurements with different setup/allocation/transfer costs and no rigorous confidence interval**, not reusable speedup ratios, GPU kernel timings, frame latency or p95. Particularly, the CUDA backend is dynamically loaded per call and GPU buffers are recreated.

## Provenance / reproduce

Original files:
`src/lib.rs`, `src/main.rs`, `src/vulkan.rs`, `src/cuda.rs`,
`native/geometry.cu`, `shaders/geometry.wgsl`, `Cargo.lock`.

Fixtures:
`experiments/blobin-next-01/CUDA-02/fixtures/qualification-20261008/`.
The verifier `scripts/validate_cuda02.py` inspects **every stored row**, metadata, numerical error, exact fallback choice, expected negatives, file hashes, plus a deliberately mutated occupancy result.

```sh
cd experiments/blobin-next-01/CUDA-02
export CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target
cargo fmt --check
cargo test --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
python3 scripts/validate_cuda02.py

/opt/cuda/bin/nvcc -O3 -std=c++17 -arch=sm_86 -shared -Xcompiler -fPIC \
  -o /mnt/workbench/build/blobin-cuda02/libblobin_cuda02.so native/geometry.cu
cargo run --offline --locked -- --backend all --count 64 --dpr 2 \
  --cuda-lib /mnt/workbench/build/blobin-cuda02/libblobin_cuda02.so
```

No host package, driver, kernel, container, remote surface or historical
GPL/AGPL-ambiguous blob.in donor was modified. CUDA-01/F4C original worktrees
remain their own qualified branches.

## Invariants not yet qualified

This is **not** an F2 replacement, general polygon topology library,
full interactive editor, stable GPU provider SDK or external plugin system.
No arbitrary nested path winding, multi-device parity, GPU/CPU feature
negotiation beyond bounded local availability, partial source uploads,
zero-copy CUDA↔Vulkan, temporal scheduling, physical in-flight GPU device
loss, security isolation for `dlopen`, trusted shared-object signing,
or throughput/p95 production SLO is proven.

Native libraries loaded with `dlopen` execute arbitrary code with process
privileges; future production would require explicit signed/digest-pinned
operator-trusted libraries or a process boundary. The fallback demo does
not establish that loading arbitrary untrusted code is safe.

**No push, merge to main, publication, release or production deployment.**
