# CUDA-05B — Supervised GPU Ingress / Hardware Challenge

**Status:** `LOCAL_SUPERVISOR_PROTOCOL_QUALIFIED / LIVE_GPU_CHALLENGE_QUALIFIED / DISTINCT_OS_PRINCIPAL_NOT_QUALIFIED / REAL_DEVICE_LOSS_NOT_QUALIFIED`

**Parent:** CUDA-05A `bfebf176512950af29745b219d4b579cdac27c3d`
**Worktree:** `feat/blobin-cuda-05b-supervisor`

The scope is a **private, bounded and non-deployed** Linux proof. It does not claim a production trust perimeter.

## Implemented

- `Supervisor` owns the managed CUDA-05 ingress and grants. The previously injectable `Ingress`, `Probe`, and `Permit` types are now crate-private; clients can no longer link against them as part of this crate's public Rust API.
- Before each acquire or render (including a cache hit), the supervisor creates a fresh managed resident GPU session, dispatches a real 16-primitive CUDA or Vulkan workload, synchronizes and compares the resulting pixels against the CPU oracle. Failure revokes the old session and grant.
- Process-local permits bind **device + generation + 128-bit OS-random ingress-instance identifier**, so matching epochs across ingress instances cannot authorize foreign permits.
- CUDA requires an absolute native-library path and an operator-provided approved SHA-256 digest. The challenge and resident implementation re-inspect the provider, rejecting replaced native bytes at the same pathname.
- A finite Unix-domain service owns the supervisor and permits. The only client commands are `STATUS`, `ACQUIRE`, `RENDER`, `REVOKE`; the client cannot supply a library, request geometry or "Ready" event. The socket is created in a private directory with mode 0600. Linux `SO_PEERCRED` verifies the connecting UID; unsolicited UIDs are denied.
## Real hardware qualification (2026-10-09)

**Device:** NVIDIA GeForce RTX 3070, driver 615.71.09; native CUDA resident provider compiled for `sm_86`; Vulkan backend using the existing wgpu resident runtime.

- **2/2 real backends PASS:** CUDA and Vulkan each passed five live GPU proof challenges.
- **20/20 primary socket operations PASS:** initial denial, successful acquire, real render, cached render, explicit revoke, denial, reacquire and fresh render for each backend.
- **2/2 configured wrong-UID checks REJECTED** using the real Linux peer credential interface.
- **6 additional CUDA substitution transactions:** same-path atomic replacement with corrupted bytes caused denial and revocation. No corrupted binary was executed. Restoration required a fresh grant; subsequent output passed CPU parity.
- **26 socket operations total**, with deterministic, bounded, self-cleaning service processes and no GPU reset.
- **66/66 cumulative Rust unit tests PASS** across F1, F2, F3, F4, F4B, F4C, CUDA-02, CUDA-03, CUDA-04, CUDA-05.
- Strict Clippy, Cargo formatting, evidence checksums and independent tamper-sensitive validation all PASS.

## Explicitly unqualified

1. **Separate identities:** the test supervisor and clients run as Linux UID 1000; the wrong-UID gate was qualified by configuring a deliberately mismatched allowed UID, **not by running a real second user principal**. Mode-0600 sockets are for the same owner. A production deployment requires a genuinely separate service UID, distribution of a restricted client socket, hardening and a credential/role model.
2. **Actual hardware faults:** no GPU reset, driver loss, Vulkan device-lost callback, PCIe error, watchdog timeout or cross-process recovery was induced. The live challenge proves device operation only at the instant of a dispatch.
3. **Provider trust:** the approved digest comes from the locally built CUDA binary for this lab. Digest equality is not signed origin or safe native execution; existing same-file race and `dlopen` risks remain.
4. **Performance:** each request builds a new proving GPU session, deliberately prioritizing correctness rather than hot-path latency. No production throughput, energy usage or p95 target is qualified.
5. **Runtime:** no Gnosix/blob.in production process, agent IPC authority, lifecycle daemon, systemd unit, merge, push or deployment was changed.
## Reproduction and evidence

Source: `experiments/blobin-next-01/CUDA-05/src/{lib.rs,supervisor.rs,socket.rs,bin/supervise05b.rs}`.

Commands:

```bash
cd experiments/blobin-next-01
mkdir -p /mnt/workbench/build/blobin-cuda04d
/opt/cuda/bin/nvcc -O3 -std=c++17 -arch=sm_86 -shared -Xcompiler -fPIC \
  -o /mnt/workbench/build/blobin-cuda04d/libblobin_cuda03.so CUDA-03/native/resident.cu
cd CUDA-05
bash scripts/reproduce05b.sh
```

Archived proof: `experiments/blobin-next-01/CUDA-05/fixtures/qualification-20261009-b/`, containing `results.log`, `cumulative-tests.log`, `environment.txt` and their `SHA256SUMS.txt`.

**Next boundary:** CUDA-05C — dedicated supervisor OS identity, restricted client-group/socket authorization, trusted policy provenance, bounded health-monitor recovery and fault injection in a sacrificial GPU worker. No production authorization follows automatically from CUDA-05B.
