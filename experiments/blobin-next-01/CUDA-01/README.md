# BLOBIN-CUDA-01 — Optional CUDA geometry acceleration probe

**Private research only; not a blob.in runtime backend or release.** A
standalone, newly written C++/CUDA 13.4 kernel benchmarks synthetic
constructive-solid-geometry fields. It does not import the historical
GPL/AGPL-ambiguous blob.in snapshot or vendor F2/F3 Rust code.

## Evaluated workload

The kernel evaluates between 16 and 256 ordered analytic circle primitives per
pixel with four operations:

- union (`min`),
- subtraction (`max(a,-b)`),
- intersection (`max`),
- smooth union (bounded polynomial blend).

Each evaluation returns approximate signed field, normalized finite-difference
normal, and analytic antialias coverage. This is **not a general arbitrary
path/tessellation system**. The smooth blend does not preserve exact Euclidean
signed distance, and the experiment does not prove manifold topology for
unrestricted shapes.

A separate host C++ implementation evaluates the same contract. All pixels
are numerically checked. Sign/occupancy mismatches are tracked with an explicit
near-boundary tolerance; no disagreements were observed in the fixtures.
The harness also deliberately corrupts a result to test that the numeric gate
can turn red.

## Qualified workstation

- NVIDIA GeForce RTX 3070, compute capability **8.6**.
- CUDA **13.4**, NVCC **V13.4.92**, NVIDIA driver **615.71.09**.
- CUDA binaries live under `/mnt/workbench/build/blobin-cuda01-build/`
  and are **not** checked in.
- No driver reset, package installation, GPU resource reconfiguration,
  host-runtime integration or external publication.

Build and reproduce:

```sh
R=experiments/blobin-next-01/CUDA-01
mkdir -p /mnt/workbench/build/blobin-cuda01-build
/opt/cuda/bin/nvcc -O3 -std=c++17 -arch=sm_86 \
  -o /mnt/workbench/build/blobin-cuda01-build/csg "$R/csg.cu"

for count in 16 64 256; do
  for dpr in 1 2; do
    /mnt/workbench/build/blobin-cuda01-build/csg \
      --primitives "$count" --width "$((256*dpr))" \
      --height "$((192*dpr))" --repeat 20
  done
done
python3 "$R/scripts/verify_cuda01.py"
```

The event-based mean uses warmed repeated kernel launches, and one-shot
end-to-end time separately includes H2D transfer, kernel, D2H transfer and
synchronization. **Different-run time measurements are not directly additive**;
their crossing order is not evidence of incorrect calculation.

These are exploratory numbers on **one GPU** and synthetic primitive layouts,
not a full CPU-vs-GPU shipping performance comparison. A future Rust provider
would need a typed C ABI, capability negotiation, CPU fallback, memory budget,
security sandbox and explicit non-amplification before integration.

## Research boundaries

The baseline F4C GPU effect pipeline remains based on portable `wgpu/Vulkan`.
CUDA-01 does **not** supersede it. F2 continues to own topology authority,
scene identities and epochs. A future CUDA integration should implement the
**same typed scene contract**, not mint independent authority.

No merge, push, public docs update, release or production deployment.
