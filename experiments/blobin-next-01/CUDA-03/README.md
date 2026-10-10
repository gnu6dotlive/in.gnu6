# BLOBIN-CUDA-03 — Resident Geometry & Performance

**Private clean-room laboratory.** This experiment extends the qualified
CUDA-02 numerical CSG contract. No historical blob.in donor files are
imported and no Gnosix/in.gnu6 runtime is modified or deployed.

## What is resident

- **CUDA:** native owned context holding the ordered primitive buffer,
  geometry output buffer and CUDA events. It reuses CUDA-02's actual
  kernel code without changing that source. Warm launches reuse allocations.
  Device events measure the kernel path, excluding host copies.
- **Vulkan:** a persistent wgpu device, pipeline, bind group, primitive
  buffer and output buffer. Repeated calls create command encoders but do not
  recreate the GPU resources. A host-side synchronized duration measures
  submission plus completion; no Vulkan hardware GPU timestamp was used.
- **CPU:** the original Rust CUDA-02 evaluator serves as the full numerical
  oracle for every pixel and each edited scene.

Each session performs four warm-up dispatches, then 20 timed dispatches,
followed by a separately timed full-frame GPU readback. Resource creation,
kernel calculation and readback are **distinct measurement scopes**.
The test uses a single RTX 3070 with CUDA 13.4 and wgpu/Vulkan 30.0.1.

## Reproduce

```sh
cd experiments/blobin-next-01/CUDA-03
export CARGO_TARGET_DIR=/mnt/workbench/build/blobin-f1-target
mkdir -p /mnt/workbench/build/blobin-cuda03
/opt/cuda/bin/nvcc -O3 -std=c++17 -arch=sm_86 -shared -Xcompiler -fPIC \
  -o /mnt/workbench/build/blobin-cuda03/libblobin_cuda03.so native/resident.cu

cargo test --offline --locked
cargo fmt --check
cargo clippy --offline --locked --all-targets -- -D warnings
cargo run --offline --locked -- --count 64 --dpr 2 --iterations 20 \
  --cuda-lib /mnt/workbench/build/blobin-cuda03/libblobin_cuda03.so
bash scripts/check_negative.sh
python3 scripts/validate_cuda03.py
```

All six configurations (16, 64 and 256 primitives at DPR 1 and 2)
passed CPU parity for both resident GPU implementations. Editing the root
circle requires a recalculation in existing buffers; stale numerical
results are deliberately rejected by an independent oracle. Five negative
gates check missing libraries, unavailable Vulkan, and request bounds.

## Exact limits

The CUDA kernel time is measured with CUDA device events; Vulkan uses
host-synchronized submission-to-completion durations. **Do not divide
these values to claim a cross-API GPU speedup.** Initialization remains
hundreds of milliseconds, and the readback is still whole-frame.

A change in CSG primitive values deliberately leaves the original F2
scene digest unchanged. That digest identifies the F2 scene, **not** the
independent CSG buffer. A future unified cache must separately bind
its geometry digest to each request, and may not infer authorization
from these provenance fields.

A loaded CUDA shared object executes native code in the caller's process:
the operator-provided absolute path in this test is not a sandbox boundary.
No trusted loader, signed provider, Vulkan/CUDA interop, full F4C scene
integration, device-loss recovery, GPU power measurement, cold-idle
verification, repeatable cross-device p95 or production SLO is proven.
No push, merge, public release or deployment.
