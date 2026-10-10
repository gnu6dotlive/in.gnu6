# UPSTREAM-R&D-01 — Bounded host sandbox preflight

Scope: **LAB_HOST_SMOKE**, not an MXC SDK test, security evaluation or production containment certification.

## Environment
- Executed on `gnosisos-x8664` under successful `gnu6-lab-run --check`.
- `bwrap` available at `/usr/bin/bwrap`; `cargo`, `rustc`, `nvidia-smi`, `podman` located.
- `deno` and `hyperfine` executables not found on the runner's PATH. No installs, downloads or builds attempted.

## First attempt: negative fixture receipt
- Mount only `/usr` read-only, `--unshare-user --unshare-pid --unshare-net`, spawn `/usr/bin/true`.
- Exit 1: `bwrap: execvp /usr/bin/true: No such file or directory`.
- Root cause of **fixture**: ELF `/usr/bin/true` requires `/lib64/ld-linux-x86-64.so.2`, absent when only `/usr` is exposed. This does not falsify Bubblewrap's isolation.
- Raw record: [host-smoke-initial.json](../../../evidence/upstream-rd-01/intake/host-smoke-initial.json).

## Corrected fixture
- Use a read-only root filesystem bind `--ro-bind / /`, plus `--unshare-user --unshare-pid --unshare-net`, private `/tmp`, proc and device mounts.
- Spawn `/usr/bin/true`; **exit 0 / PROBE_PASS**. Raw record: [host-smoke.json](../../../evidence/upstream-rd-01/intake/host-smoke.json).
- This only proves the bounded sandbox **can start a trivial program** on this host. A full read-only root exposes host paths for reading, therefore this smoke is **not** a least-privilege boundary, nor proof of denied filesystem/network access.
- MXC permission propagation, non-amplification, network denial, audit logging and real resource confinement: **NOT_RUN**.

Next: design actual policy deny oracle and narrow filesystem mounts; avoid using this bootstrap-only fixture as a security test.
