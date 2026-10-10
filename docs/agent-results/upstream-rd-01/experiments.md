# UPSTREAM-R&D-01 — W08 bounded experiments

Date 2026-10-10. **Qualified host probes only; no upstream release or consumer integration.**

## E08-01 — Bubblewrap read-only mount and network namespace fixture

- Tool: installed `/usr/bin/bwrap`; execution constrained via `gnu6-lab-run`.
- Mechanism: user/PID/network namespaces; read-only host root for fixture; isolated tmpfs `/tmp`, separate proc and mapped /dev. Python child tries to overwrite a uniquely created worktree sentinel, write `/tmp`, and connect to TEST-NET-1 address 192.0.2.1.
- Oracle positive/negative controls: host sentinel bytes recorded, write must fail with `EROFS`; private temp must succeed; external connection must fail; network namespace inode differs; sentinel must match before/after.
- **Observed: PASS_BOUNDARY_FIXTURE**. Read-only write refusal errno 30 (`EROFS`); temp writing succeeded; attempt to 192.0.2.1 returned errno 101 (`ENETUNREACH`); host sentinel intact; child network namespace distinct.
- Evidence: `evidence/upstream-rd-01/intake/host-denial-oracle.json`; replay tool `tools/upstream-rd-01/probe_bwrap_denial.py`.
- Falsification scope: Linux Bubblewrap on this specific host and this one invocation. Does **not** prove MXC policy propagation, network prevention against every address/path, confidentiality, provenance of kernel enforcement, or production tenant isolation.

## E08-00 — Harness defect preserved
- Earlier bwrap bootstrap with `/usr` read-only only failed to exec dynamically-linked `/usr/bin/true` (loader at `/lib64/ld-linux-x86-64.so.2` absent). Failed fixture was corrected to read-only `/` and trivial command succeeded.
- Evidence: `host-smoke-initial.json`, `host-smoke.json`, `environment-qualification.md`. Negative finding is a test harness issue, not upstream Bubblewrap regression.

## Other experimental gates
| Test | Current state | Minimal next instrument |
|---|---|---|
| MXC policy SDK deny/evaluate with logging | NOT_RUN | compile an exact pinned minimal SDK client, compare declared policy to actual denial |
| Deno Rust native operation / permission broker | NOT_RUN | read exact permissions broker code; sandbox narrow Rust op fixture |
| GPUI Kit accessibility/keyboard on Wayland | NOT_RUN | small UI focus contract and AccessKit tree probe |
| PlxNative on physical LG TV | UNAVAILABLE | treat reference numbers as third-party, design independent GPU tracer on blob.in |
| Hyperfine v2 process benchmark | BLOCKED_TOOL_NOT_INSTALLED | use isolated pinned executable or existing local toolchain only with owner allocation |
| Uzu tiny CPU model numerical oracle | NOT_RUN | model compatibility/type check, no 27B GPU benchmark |
| Cross-project product integration | NOT_AUTHORIZED_AS_AUTOMATIC | target-owner contracts, consumer build and regression oracle |

Next evidence priority: G08-02 compare denial/audit separately at MXC permission policy level; **do not conflate** with E08-01 Linux host oracle.
