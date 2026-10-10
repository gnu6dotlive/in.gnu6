# BLOBIN-CUDA-02 — Unified Geometry Provider (private)

**Local clean-room experiment, not a production backend, public SDK, or a
license determination for the historical blob.in snapshot.**

## Contract

`experiments/blobin-next-01/CUDA-02/` defines one canonical Rust input:

- `Request::from_f2(scene, primitives)` is the only public constructor.
  It validates an F2 `Scene` and freezes its epoch, left/right identity and
  semantic digest as **provenance metadata**. A caller can construct an F2
  scene, so this is not a cryptographic or unforgeable capability.
- `Primitive { x, y, radius, op }` is a 16-byte C-compatible tuple.
  There are four ordered operations: union (0), subtraction (1),
  intersection (2) and smooth union (3), maximum 256 primitives.
- `Sample { distance, nx, ny, coverage }` is a 16-byte C-compatible
  output. A backend returns only computed floats. **The Rust
  orchestrator**, not CUDA or Vulkan, attaches the original F2 metadata.
- Resolution must stay within the lab bounds (16–2048 per dimension,
  no more than 1,048,576 pixels), all scalar inputs must be finite,
  radius strictly positive, first operation union. Invalid requests fail.
- Backends: `cpu`, `vulkan` (`wgpu 30`), `cuda` (optional dynamically
  loaded `nvcc`-built shared library) and `auto` (CUDA if provided
  and working → Vulkan if available → CPU).
- A required backend **fails closed**; `auto` selects another backend
  only after the requested optional compute path is unavailable.
  The result always declares the **selected** backend.

The same ordered CSG primitive buffer is used on all three execution
paths. At the tested resolutions, the GPU results are compared to a
CPU Rust oracle pixel-by-pixel for signed field, finite-difference
normals, coverage and foreground/background occupancy.

**This geometry does not replace F2's authoritative binary mask, hit
testing or semantic transitions.** The CSG scene is an independently
scoped numerical evaluation probe. Neither GPU implementation may
infer or mint new object identities.

## Build without CUDA

```sh
cd experiments/blobin-next-01/CUDA-02
export CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target
cargo fmt --check
cargo test --offline --locked
cargo clippy --offline --locked --all-targets -- -D warnings
cargo run --offline --locked -- --backend cpu --count 16 --dpr 1
cargo run --offline --locked -- --backend vulkan --count 16 --dpr 1
```

Only the `cuda` path requires NVIDIA CUDA and a trusted local
shared library; no CUDA dependency is linked into the Rust binary at
build time. A missing GPU cannot prevent the CPU-only build and tests.

## Optional CUDA library

On the qualified NVIDIA RTX 3070 (compute capability 8.6), CUDA 13.4:

```sh
R=experiments/blobin-next-01/CUDA-02
mkdir -p /mnt/workbench/build/blobin-cuda02
/opt/cuda/bin/nvcc -O3 -std=c++17 -arch=sm_86 -shared -Xcompiler -fPIC \
  -o /mnt/workbench/build/blobin-cuda02/libblobin_cuda02.so "$R/native/geometry.cu"

cd "$R"
cargo run --offline --locked -- --backend all --count 64 --dpr 2 \
  --cuda-lib /mnt/workbench/build/blobin-cuda02/libblobin_cuda02.so

python3 scripts/validate_cuda02.py
```

**Security boundary:** `dlopen` loads native code into the Rust
process without sandbox isolation. The path must be operator-trusted
and, for any future product integration, signature/digest pinned.
The explicit opt-in used here is **not** an appropriate default for
untrusted third-party plugins or arbitrary user-provided paths.

## Qualification artifacts

`fixtures/qualification-20261008/` contains:

- `three-providers.txt` — 6 scenarios × 3 real providers, plus summaries.
- `fallback.txt` — `auto` actually selects CUDA, Vulkan, CPU
  under three controlled configurations.
- `negative.txt` — fail-closed required-CUDA, required-Vulkan,
  missing shared library and invalid input budget.
- `SHA256SUMS.txt` — immutable checksums of those three logs.

The stdlib-only validator `scripts/validate_cuda02.py` checks each
case, provenance metadata, numerical thresholds, all fallback paths
and a deliberately corrupted occupancy result.

The observed maximum absolute error across GPU backends vs CPU is
`distance 0.000000119`, `normal component 0.000694662`,
`coverage 0.000014365`, with **zero occupancy mismatches**.

## Still unproven

No equivalence to F2's complete topology grammar for arbitrary paths;
no CUDA/Vulkan GPU-buffer interop, GPUI/native host adapter, device-loss
recovery, cross-device qualification, multi-scene temporal scheduling,
memory-resident benchmarking, production throughput/energy/p95, or
trusted plugin loader.

This benchmark creates backend resources per evaluation, and CUDA
loads a native library on demand; elapsed host time includes setup,
allocation and copies. Timings must **not** be sold as kernel-only
speedups, frame p95, or proof that CUDA should always be selected.
