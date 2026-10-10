# CP02 Vulkan GPU-00 — Real compute qualification slice

Truth: REAL Vulkan COMPUTE PROBE locally PASS. CP02 checkpoint NOT QUALIFIED.
Contract: BLOB-NEXT-01 CP02 v0.5, all 7 gates remain unmet.

## Physical source / reference
- Identifies exact wgpu backend Vulkan and a physical NVIDIA GPU (PCI vendor 0x10de).
- Loads a real GLB tetrahedral MeshSurface, explicitly certifies it as a
  conservative convex solid using CP01's validator, then constructs/upload
  four oriented mesh-derived support planes.
- Dispatches an actual WGSL storage-buffer compute shader (1,024 independent
  3-D test positions x 4 channels = 4,096 GPU results).
- Channels: union / intersection / sphere minus tetra / tetra minus sphere.
- Reads GPU output back and compares with independent Rust-side implementation
  of the same float32 field equations; comparison budget <=1e-5 per channel.
- A NON-EUCLIDEAN convex support-plane field is used for this initial probe;
  it does NOT calculate general exact signed mesh distance to triangles.
- No pixels presented, no physical Wayland compositor feedback, no 3-D
  viewport, no arbitrary imported nonconvex mesh, no CUDA comparison.

## Explicit non-claims
- Not CP02-G01..G07 qualification; no G03 GPU mixed booleans as full viewport.
- Not CP03 HYBRID-60 and not even a measured frame rate.
- No multi-vendor or second-compositor claim.
- Error comparisons are against a CPU oracle for this same plane-based
  *approximate* implicit field, not a certified Euclidean mesh oracle.

## Isolated authority
- Separate branch/worktree for the GPU proof at CP01 base 11e30b9.
- No changes to stable service, IPC, Wayland window, deployment, VPS,
  CP01 GEO-05 worktree or shared CUDA experiments.
- A later integration review must reconcile newer GEO-05 worktree,
  source/interface authority and dependency locking before any merge.
