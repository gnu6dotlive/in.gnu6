# CUDA-05 — Bounded GPU Ingress / Supervisor Research

Clean-room, private Rust lab. No live blob.in/Gnosix runtime integration and no production authorization.

## Layout

- `lib.rs`: low-level managed ingress with **crate-private** permits, generation and ingress-instance nonce.
- `supervisor.rs`: real GPU proof by a fresh CUDA/Vulkan resident dispatch plus CPU-oracle parity **before each authorization and subsequent render**. Not a hardware-health daemon.
- `socket.rs`: Linux peer credentials (`SO_PEERCRED`), exact UID allowlist and private socket-directory preconditions.
- `bin/supervise05b.rs`: finite, fixed-workload Unix socket service. Commands: `STATUS`, `ACQUIRE`, `RENDER`, `REVOKE`. No arbitrary source paths or GPU-ready signals from clients.
- `scripts/qualify05b.py`: tests real local GPU dispatch, cache miss/hit, revoked permits, wrong configured UID, and same-path native library substitution; temporary sockets/services are cleaned up.
- `scripts/validate05b.py`: checks archived log structure, ten-crate cumulative regression, three SHA-256-covered files, and deliberate evidence tampering rejection.

## Reproduce

Build the CUDA-03 native `sm_86` library under `/mnt/workbench/build/blobin-cuda04d/libblobin_cuda03.so` as documented by CUDA-04C/04D; then:

```bash
bash scripts/reproduce05b.sh
```

`qualify05b.py` creates short-lived processes and Unix sockets under private temporary directories in `/mnt/workbench/build`. It does not modify services or reset GPUs.

## Trust limits

Peer credentials confirm a Linux UID, **not individual agents**. This lab used the same UID for the service and authorized client; a genuinely distinct service principal and production socket provisioning are future work. The CUDA hash was derived from the locally compiled lab binary and does not attest release origin. Each GPU challenge proves working execution **at that instant only**, not resilience to asynchronous loss. Native `dlopen` is not sandboxed. The challenge allocates a fresh GPU resident session per request and has no optimized hot-path guarantee.

See `docs/agent-results/blobin-next-01/CUDA-05B.md` for evidence and next boundaries.

## CUDA-05C — Disposable worker fault isolation

`isolation.rs` owns a finite parent-side generation gate and launches a **fresh GPU worker process** per authorization or render request. `bin/worker05c.rs` performs the fixed real CUDA/Vulkan oracle-qualified workload. `bin/supervise05c.rs` exposes only the bounded local socket commands; it has no GPU context of its own. A worker crash, timeout, or substituted worker executable SHA-256 revokes the grant. CLI-only crash/hang injection is for laboratory testing; the client protocol cannot request arbitrary commands or faults.

Reproduce all ten Rust crates, release worker compilation, actual GPU child-fault tests and the independent offline ledger check:

```bash
cd experiments/blobin-next-01/CUDA-05
bash scripts/reproduce05c.sh
```

The fixed native CUDA-03 library from CUDA-04D must be available at `/mnt/workbench/build/blobin-cuda04d/libblobin_cuda03.so`. Evidence lives in `fixtures/qualification-20261009-c/`.

**This is not host-UID isolation.** User namespace root still maps to host UID 1000; subordinate UID auto-mapping failed because `newuidmap` lacked required capabilities. The service and client therefore run under the same host account in this qualification. Worker executable and CUDA library hashes are derived from local test builds, not trusted provenance. No actual GPU driver/device reset was attempted. The parent kills and reaps the direct child but has no hardened cgroup/namespace/sandbox boundary. See `docs/agent-results/blobin-next-01/CUDA-05C.md`.
