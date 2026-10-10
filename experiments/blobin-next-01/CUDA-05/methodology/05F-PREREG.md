# CUDA-05F-P01 — Real Rust GPU supervisor under a distinct host principal

**Status:** preregistered before qualification. **Source base:** CUDA-05E `f458c47dae8ffe67272d2a947d3ac022a2a79b74`. Contract: `GPU-EVIDENCE-CONTRACT-v1.json` (copied byte-for-byte to docs-hub-01g).

## Question and falsifiable hypotheses

Does the actual Rust `supervise05f` service execute as a different **host UID** than its authorized IPC caller, enforce `SO_PEERCRED` and directory/socket DAC, and launch the existing CUDA-05C GPU child with CPU oracle parity and fail-closed recovery from injected **child-process** crashes/timeouts?

- **H0:** any of these independently required properties fails, is missing, or was verified only in another program; reject full integration gate.
- **H1 (bounded):** same finite Rust service run demonstrates real host UID boundary, peer allow/deny, DAC refusal, real CUDA/Vulkan workload, worker crash & timeout recovery and both worker/native digest refusal without leaving the service running.
- A one-run service with CUDA doesn't prove Vulkan in that *same* service; use a separate equivalent service run for Vulkan, retain backends distinctly.
- Real device loss, driver reset, interprocess authority beyond UID, malicious Docker root, kernel races and production deployment are **not** covered.

## Fixed scope and environment

RTX 3070 workstation is an active display device; **no reset/destructive GPU mutation**. Rootful Docker uses existing locally cached Debian image, --network none, --read-only rootfs, --cap-drop ALL, no-new-privileges, bounded PIDs/memory, NVIDIA GPU passthrough, read-only host runtime libraries and driver ICD mounts, minimal temporary writable socket directory. Debian glibc is older than the binary built on the host: *preflight* found host `/hostlib` loader/rpath needed. Stage copied **patchelf-interpreter-rewritten** release binaries in the temporary lab directory (immutable for the run except declared substitution). Do not call this a sealed trusted build or sandbox against daemon administrators.

Service configured as host UID65534 / GID of authorized host user; group-traversable directory 0710, socket 0660; primary client host UID1000; connected unauthorized UID65533 same group; DAC outsider UID65532:GID65532. Confirm actual host UID through /proc of Docker-inspected server pid; no mere `id` textual assertion.

## Fixed operations / oracle

For **each backend** (CUDA, Vulkan), use the *actual* Rust service binary and existing CUDA-05C GPU worker:
1. Baseline `STATUS`, then `RENDER` before grant -> deny.
2. Authorized `ACQUIRE` -> launches real GPU child + CPU parity.
3. Authorized `RENDER` -> fresh real GPU child + parity.
4. Same-group unauthorized client sends `STATUS`: reaches `SO_PEERCRED` and is explicitly denied; service remains alive.
5. Distinct-group DAC outsider attempts `STATUS`: connect must be denied before server accept, and does not consume one of the finite request budget.
6. Authorized `RENDER` succeeds after denial.
7. One **crash-injected** child and one **timeout-injected** child are attempted in separate finite service runs (only via operator CLI `--fault-index`), must revoke state and deny stale render, then successful reacquisition and CPU parity.
8. Manipulate a **copy** of approved worker/native bytes at the same path and verify fail-closed refusal before executing replaced bytes; restore exact bytes and reacquire. Never tamper with active base binary on host.
9. Socket/service cleanup, no leaked named Docker containers, no live host service modifications.

## Predetermined gate decisions

Each of the 11 contract gates returns PASS, FAIL, BLOCKED, NOT_RUN or NOT_QUALIFIED with raw supporting observation ID. *host_service_uid*, *authorized_host_peer*, *rejected_connected_peer*, *dac_rejected_peer*, *cuda_gpu_parity*, *vulkan_gpu_parity*, *worker_crash_recovery*, *worker_timeout_recovery*, *worker_binary_integrity*, *provider_integrity* are PASS only if actual scoped Rust process/worker interaction meets the oracle. *release_authority* always NOT_QUALIFIED unless there is separate independently authorized production approval (not granted here). Overall laboratory status PARTIAL if one gate missing; do not redefine a gate during acquisition.

Raw protocol: immutable source commit, UTC time, platform, userns mode, PID/UID observed from /proc, socket/file modes, per-request commands and exit codes, worker and library hashes, bounded stderr, cleanup results. No credentials. Derive a **sanitized** public result by strict allowlist. Independent validator recomputes claims from raw measurements, verifies source hash/commit and rejects an intentional falsification.

**STOP** on failed Docker resource controls, inability to clean temporary directory, timeouts with unreaped workers, any GPU/display disruption or any proposal to change host accounts/persistent services. Unavailable is a valid result.

## Docs-HUB-01G contract

The docs pipeline may ingest only the allowlisted `public-results` JSON after its SHA and schema validation; it must preserve negative gates and cannot promote a `PASS` service test to production. A human review and explicit publication grant remain separate.
