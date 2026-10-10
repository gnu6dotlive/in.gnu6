# UPSTREAM-R&D-01 · W14 Composite MXC × Deno · 2026-10-10

**Disposition: FILESYSTEM_LAB_PASS_4_OF_4; NETWORK_INCONCLUSIVE; BROKER_NOT_RUN; JUDGMENT_PLANE_NOT_RUN.**

## Tested authorities and provenance
- MXC `microsoft/mxc@c6f301d53a1430c4c921a05c57af838f7392348f` / SDK 1.0.0, built from exact detached upstream checkout in a scratch Cargo manifest with Rust 1.97.1; no upstream edits
- Deno binary: official `v2.9.7` Linux x86_64 release; distribution ZIP SHA-256 `c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490` verified
- **Deno executable is not from pinned source `denoland/deno@a18ce33715e30cd2b0d99c7e322ef11a65490e4d`** (later 2026-10-06 source)
- Actual invocation path: host Python launches Rust MXC SDK wrapper, which spawns Deno inside MXC `Containment::Process` on Linux
- All write targets and positive controls belong to one unique disposable temp directory under `.probes/upstream-rd-01`

## Filesystem 2-layer test — PASS
| Deno write permission | MXC filesystem policy | Observed Deno result | Marker state |
|---|---|---|---|
| Explicit scoped allow | `readwrite_paths` | `ok:true` | `COMPOSITE_ALLOWED` present |
| Explicit scoped allow | `readonly_paths` | `ok:false`, `Read-only file system (os error 30)` | Absent |
| Default Deno deny | `readwrite_paths` | `ok:false`, `NotCapable` | Absent |
| Default Deno deny | `readonly_paths` | `ok:false`, `NotCapable` | Absent |

All four independent mode checks passed **twice**, in two separate disposable directories: 8/8 case outcomes repeated, with both host sentinels preserved. Automated receipt verifier `tools/upstream-rd-01/verify_composite.py` checks both runs.
**Bounded inference:** on this machine and these exact tests, Deno `--allow-write` did not bypass MXC's OS-enforced read-only mount. Likewise, an MXC writable grant did not bypass Deno's own default permission denial.
Evidence: `evidence/upstream-rd-01/composite-01/qualification.json`. Reproduction: `tools/upstream-rd-01/composite_mxc_deno.rs`, `composite_deno_fixture.ts`, `qualify_composite.py`.

## Network cross-test — INCONCLUSIVE, not a security FAIL

The intended positive control was a disposable TCP server bound to **host** `127.0.0.1:<ephemeral>`. Deno `--allow-net` inside MXC couldn't reach it *even when the MXC request omitted the network section*. Three other denial cases produced the expected errors, but **there was no positive MXC network control**.

The pinned MXC [Bubblewrap backend documentation](https://github.com/microsoft/mxc/blob/c6f301d53a1430c4c921a05c57af838f7392348f/docs/backends/bwrap/bubblewrap-backend.md) explicitly states that no-network policy synthesizes a default deny, always runs in a **private network namespace**, and host-loopback is denied. This invalidates the assumption that `network:None` means host-loopback access. It is *not* evidence that an explicit MXC egress allow policy is broken.

Receipt: `evidence/upstream-rd-01/composite-01/network-qualification.json` (**INCONCLUSIVE_HOST_LOOPBACK_POSITIVE_CONTROL**); original first fixture retained as `network-qualification-initial.json`. Reproduction: `composite_mxc_deno_net.rs`, `composite_deno_net.ts`, `qualify_composite_network.py`.

## Explicitly unproven boundaries
- Deno external **permission broker IPC**: not configured, so broker grant override and broken/unknown response fail-closed paths remain `NOT_RUN`
- Exact pinned Deno source runtime: `NOT_RUN` — Deno 2.9.7 and Oct 6 source SHA differ
- MXC explicit egress allow and proxy-only paths: `NOT_RUN` — test requires a supported network-positive oracle, not forbidden host loopback
- Judgment Plane `ScopedCapability`/`ExecutionGrant`, issuer identity, TTL and unforgeability: `NOT_RUN`
- Physical multihost/tenant isolation, seccomp and cross-platform enforcement: `NOT_RUN`

## Gate outcome
`W14_FILESYSTEM_COMPOSITE=LAB_PASS` for one executable configuration; `W14_NETWORK_COMPOSITE=INCONCLUSIVE`; `W14_END_TO_END=OPEN`.
No GNU6 product code modified; no production service, secrets or Git default branch touched. Results are research evidence, not a public product/security certification.
