# CUDA-05C — Dedicated Supervisor Identity & Fault Isolation

**Status:** `WORKER_PROCESS_FAULT_ISOLATION_QUALIFIED / DEDICATED_HOST_UID_UNAVAILABLE / HARDWARE_DEVICE_LOSS_NOT_QUALIFIED / PRODUCTION_NOT_PROVEN`

**Parent:** CUDA-05B `5332dcb7f2c8ec63795f20ccd5e4d655a00f834e`
**Branch:** `feat/blobin-cuda-05c-fault-isolation`.

## Qualified scope

The Linux lab introduces `src/isolation.rs`, `src/bin/worker05c.rs`, and `src/bin/supervise05c.rs`. The supervisor process **does not own a GPU context**. For every acquire/render operation it executes a fresh, disposable GPU child process, which performs a real 16-primitive CUDA/Vulkan computation with CPU oracle parity. Child exit status and fixed result marker are required; a child failure revokes the supervisor's current generation. Child execution has a bounded host timeout, explicit kill and wait/reap.

- **Two GPU backends** (RTX 3070 CUDA, Vulkan): real worker execution PASS.
- **18 normal socket operations**, **28 fault-scenario operations**, and **six worker-integrity substitution operations** PASS.
- **Two intentionally killed workers**, **two intentionally blocked workers terminated at deadline**. In each case the parent supervisor remains operational, rejects the old grant, then reacquires and renders through a new process.
- **Same-path worker executable corruption:** the configured SHA-256 is rechecked before every spawn. Corrupted bytes are rejected without execution, the grant is revoked, and a restored binary permits new acquisition.
- **Two configured mismatched-UID socket checks** reject requests. Socket paths live in temporary 0700 directories and have 0600 file permissions.
- **68/68 cumulative Rust tests PASS** across F1→F4C, CUDA-02, CUDA-03, CUDA-04, CUDA-05. Cargo formatting and Clippy strict PASS.

## Scope of the fault simulation

These are real operating-system child processes. The injected crash uses `SIGKILL` **only on the worker**, and the hang is a child sleeping beyond its deadline until the supervisor kills and reaps it. This is stronger than a purely synthetic state-machine transition but **does not simulate device loss, driver reset, or a hanging GPU kernel inside an uninterruptible driver call**. No physical GPU reset or permanent service change was made.
## Distinct Linux identity — NOT QUALIFIED

The workstation user is UID 1000. Rootless user namespaces work with `unshare --user --map-root-user`, but this maps namespace root back to host UID 1000. An attempted subordinate-host-UID mapping via `unshare --user --map-auto` returned `newuidmap: Could not set caps` despite subordinate ranges being configured. Therefore the supervisor and approved clients ran under **the same host UID**.

The two wrong-UID tests use a deliberately mismatched allowlist and a real `SO_PEERCRED` check. They do **not** demonstrate an independently running unprivileged process from a second host account.

No host account was created, no groups were changed, and no permanent service was installed. A distinct service UID with controlled client group/ACL, signed release policy and independent operating-system identity are still required before the *dedicated identity* part can be considered closed.

## Additional production blockers

- The worker SHA-256 and CUDA library SHA-256 are from local builds, not verified signed release policy. Files may be changed in place between hash and `exec`/`dlopen`; pathname re-binding is not fully sealed.
- Each worker inherits normal process privileges and may access other resources available to its UID; no seccomp, cgroup resource/CPU/GPU quota, mount namespace, supervisor privileges separation, or dedicated worker UID is yet enforced.
- The worker has no child processes by contract and the supervisor kills/reaps the direct child, but it is not hardened against malicious worker forks or uninterruptible GPU-driver wait.
- The IPC protocol provides only a fixed 16-primitive demonstration. It does not integrate the full blob.in/Gnosix scene topology or production policy.
- Real driver/device faults, cross-vendor portability, production latency, crash-loop throttling and independent agent authorization remain open.

## Reproduction

```bash
cd experiments/blobin-next-01/CUDA-05
bash scripts/reproduce05c.sh
```

The original native CUDA-03 shared library must already exist under `/mnt/workbench/build/blobin-cuda04d/libblobin_cuda03.so`, as produced during CUDA-04D. The script regenerates the finite local-service GPU trials and all ten-crate tests, plus `fixtures/qualification-20261009-c/results.log`, `cumulative-tests.log`, `environment.txt`, `SHA256SUMS.txt`. `scripts/validate05c.py` checks the integrity and test matrix and rejects a deliberately modified summary record.

**Decision:** Close the *process fault isolation* subgate only. CUDA-05C overall remains **PARTIAL** until distinct OS principals and real device-loss handling can be qualified in an appropriate disposable environment. No merge, push or deployment.
