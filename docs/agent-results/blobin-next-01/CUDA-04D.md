# BLOBIN-CUDA-04D — Device Health & Provider Integrity

**Status:** `MANAGED_GPU_REVOCATION_VERIFIED / PROVIDER_BYTE_IDENTITY_VERIFIED / REAL_DEVICE_LOSS_NOT_EXERCISED / PRODUCTION_NOT_PROVEN`

**Parent:** CUDA-04C `321e18928aa3546ec8daaddf7a3d9a8bf0873cdd`
**Branch:** `feat/blobin-cuda-04d-health`
**Hardware:** RTX 3070 8 GiB, NVIDIA driver 615.71.09, CUDA toolkit 13.4, Rust 1.97.

## Implemented boundaries

- **Managed health:** `ResidentBridge::new_managed` starts `Unverified`; a supervisor must explicitly report recovery before any dispatch or cache hit. Loss reports increment the monotonic generation, destroy the GPU session, discard host samples and block requests. Runtime failures also revoke session health.
- **Native provider approval:** managed CUDA requires an externally approved SHA-256. Every evaluation rehashes the original library pathname before using cached results, so replaced or missing providers revoke the cache.
- **Pinned file:** CUDA-03 loads via `/proc/self/fd/N` while its verified descriptor stays open. Linux `O_NOFOLLOW` rejects leaf symlinks; `O_NONBLOCK` prevents hanging on nonregular input.
- **Observable provenance:** `evaluate_observed` returns the frame together with device health generation and provider fingerprint; `evaluate` remains a legacy compatibility method.
- **Unchanged authorities:** F2 semantic digest and geometry SHA-256 remain separate. A hash is evidence of equal bytes, **not** a signature, a safe binary, or authority to execute.

## Actual GPU proof

The bounded test executes CUDA and Vulkan resident sessions on the RTX 3070. Both managed backends reject requests while unverified, produce results matching the CPU oracle once the operator reports recovery, reject cached frames after a simulated loss signal, and rebuild sessions at a higher generation.

The CUDA test atomically substitutes a copy of the approved binary at the **same pathname** with deliberately modified bytes. The altered hash is rejected *without executing those bytes*; the old session/cache are discarded. Restoring the original bytes at a new inode produces a fresh CPU-parity frame.

No driver reset or actual Vulkan/CUDA hardware device loss was induced. The loss report is a controlled software injection, not a spontaneous GPU health observation.

## Evidence

- `experiments/blobin-next-01/CUDA-04/fixtures/qualification-20261009-d/results.log`
- `experiments/blobin-next-01/CUDA-04/fixtures/qualification-20261009-d/compatibility-04c.log`
- `experiments/blobin-next-01/CUDA-04/fixtures/qualification-20261009-d/cumulative-tests.log`
- `experiments/blobin-next-01/CUDA-04/fixtures/qualification-20261009-d/environment.txt`
- `experiments/blobin-next-01/CUDA-04/fixtures/qualification-20261009-d/SHA256SUMS.txt`
- `experiments/blobin-next-01/CUDA-04/scripts/validate04d.py`

## Incomplete by design

The health state is supplied by its caller. Real asynchronous device loss, device reset, cross-process recovery, GPU hang termination, production telemetry and multi-vendor portability remain unqualified. Hash approval must come from a trusted policy source; the test harness records a hash of its local build, which does **not** establish software origin or safety. In-place file writes during hashing/loading, parent directory symlinks and arbitrary native execution are not fully isolated. Legacy `ResidentBridge::new` bypasses the managed approval policy; it must not be promoted as a production trust boundary.

No integration into the production blob.in/Gnosix runtime, merge, push or deployment was performed.
