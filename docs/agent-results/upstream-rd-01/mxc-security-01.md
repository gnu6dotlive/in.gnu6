# W01A — MXC security boundary, 10 October 2026

**Verdict: SOURCE_GUARD_ORDER_PASS / HOST_NETWORK_ORACLE_PASS / MXC_SDK_RUNTIME_NOT_RUN.**

## Truth acquired
- Actual immutable clone `microsoft/mxc@c6f301d53a1430c4c921a05c57af838f7392348f`, local path `/mnt/workbench/workspaces/gnu.in.labs/.probes/upstream-rd-01/mxc-c6f301`; checkout SHA matched pin, source tree unmodified.
- [Bubblewrap runner source](https://github.com/microsoft/mxc/blob/c6f301d53a1430c4c921a05c57af838f7392348f/src/mxc-sdk/src/backends/bubblewrap/common/bwrap_runner.rs): validation of common request, network support, filesystem object conflict normalization, caller-access delegation, denied-path resolution, pinned host conflict, and `spawn_bwrap` appear in that order in the relevant `SandboxBackend` implementation.
- Source-order gate: `tools/upstream-rd-01/qualify_mxc_security.py` detects all **7/7** explicit guard-removal mutations; exact evidence `evidence/upstream-rd-01/mxc-security-01/qualification.json`. This does NOT show actual MXC runtime enforcement or audit logs.
- Host environment: Bubblewrap `/usr/bin/bwrap` can create an unprivileged private network namespace under `gnu6-lab-run`.
- Two-sided causal loopback probe: a disposable listener on host `127.0.0.1:<ephemeral port>` is reachable from the control bwrap child WITHOUT `--unshare-net`, but not reachable from the child WITH `--unshare-net`; network namespace IDs differ. Evidence `evidence/upstream-rd-01/mxc-security-01/loopback-oracle.json`. This proves only a **bounded host namespace fixture**.
- Exact pinned SDK Cargo metadata `cargo metadata --offline --locked --no-deps` parsed; offline dependency resolution `cargo tree --offline --locked -p mxc-sdk` **BLOCKED** at missing `cidr` crate. Rust 1.93 pin not installed; host 1.97/1.97.1 available. No dependencies fetched, no compile attempted.

## Transfer contracts (proposals only)
- Judgment Plane: `ScopedCapability` / `ExecutionGrant` must be checked independently from physical sandbox; ask for missing capability denial, impossible host policy refusal, and no implicit fallback.
- Desktop MCP & in.gnu6 workload hosts: reject unsupported combinations before spawning the worker; identity/namespace/source trace part of receipt.
- VPS workloads: no default adoption, deployment or tenant-security claim.

## Open gate
Build a tiny MXC 1.0 SDK sample in isolated debug target dir with explicitly bounded dependency acquisition; then test requests that must be rejected *before* spawn versus errors from a launched sandbox. Build real MXC audit/denial receipt. Until then MXC_SDK_RUNTIME=NOT_RUN.
