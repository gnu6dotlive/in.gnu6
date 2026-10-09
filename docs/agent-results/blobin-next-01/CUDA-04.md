# BLOBIN-CUDA-04 — Geometry Identity & Runtime Bridge (local prototype)

Parent: CUDA-03 `06a5a38`. Isolated worktree: `feat/blobin-cuda-04-bridge`.
Do not merge or publish before qualification.

## Contract
- F2 authority (`SceneAuthority`) remains immutable provenance and is NOT the geometry digest.
- `GeometryId` is SHA-256 of domain tag `blobin/geometry/v1\\0`, little-endian width, height, count, then ordered primitive x/y/radius IEEE-754 bits and op. NaN/inf and invalid primitives rejected by CUDA-02 validation before hashing.
- The digest is content identity, not proof of authorship, authorization, sandboxing, or backend attestation.
- Cache key is (GeometryId, actual backend). Cache entries never span backends. A hit reattaches current F2 provenance; no GPU output can mint F2 authority.
- Explicit strict selection rejects unavailable backends. Preferred selection attempts CUDA→Vulkan→CPU or Vulkan→CPU; output reports actual selected device and hit/miss. No silent claim that CPU was GPU.
- LRU bounds *number of entries*, not memory bytes. Cached output is cloned; no real resident CUDA/Vulkan buffer ownership is integrated here.
- Availability flags are caller-supplied; they are not a health probe or a trust boundary. CUDA native code remains untrusted without separate verification.
- Provider calls use CUDA-02 (nonresident) API. CUDA-03 resident session handoff, lifetime/invalidation, device-loss recovery and live Gnosix runtime integration remain OPEN.

## Reproduction
```sh
cd experiments/blobin-next-01/CUDA-04
cargo test --offline --locked
cargo fmt -- --check
cargo clippy --offline --locked --all-targets -- -D warnings
```

## Qualification still required
Backend parity on real CUDA/Vulkan, device-loss and recovery, reference output validation on cache hits, memory-byte budget, provenance across scene epochs, and native library trust. This slice does NOT imply production readiness.
