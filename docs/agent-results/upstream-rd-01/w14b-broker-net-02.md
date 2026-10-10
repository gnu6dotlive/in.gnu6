# UPSTREAM-R&D-01 · W14B real IPC broker and MXC network refusals
**2026-10-10 · PRIVATE RESEARCH · LAB PASS (bounded)**

## Real Deno v2.9.7 broker IPC — PASS 9/9
Deno `DENO_PERMISSION_BROKER_PATH` connects to a disposable Unix stream socket server outside the Deno process. A newline-framed JSON request contains version=1, PID, ID, datetime, `permission` and `value`. The fixture replies with a correlated ID and `allow` or `deny`.

Cases:
- Broker `allow` overrides absent CLI `--allow-read`: read succeeds
- Broker `deny` overrides an explicitly allowed CLI read: `NotCapable`
- Broker `deny` with no CLI permission: `NotCapable`
- Wrong response ID: process exits 87
- Unknown response variant: exits 87
- Server drops accepted socket: exits 87
- No broker + no read permission: `NotCapable`
- No broker + CLI allow: read succeeds
- Broker path points to nonexistent socket: exits 87

All disposable input files retained their contents; request payloads were inspected and correlated.
Evidence: `evidence/upstream-rd-01/broker-net-02/deno-broker-live.json`.
Fixture: `deno_broker_fixture.ts`, `probe_deno_broker.py`, `qualify_deno_broker_live.py`.

**Provenance:** official Deno Linux v2.9.7 ZIP SHA-256 `c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490`. The source blob `runtime/permissions/broker.rs` at v2.9.7 matches the Oct 6 pinned commit, SHA `a7c70e963ed2a6a7bdc7c14bb8581c39b0ce119c`. The complete Deno runtime build still does not correspond to Oct 6 source.

## MXC SDK network policy refusal — PASS 3/3
Real `mxc-sdk v1.0.0` on the pinned source `microsoft/mxc@c6f301d53a1430c4c921a05c57af838f7392348f`:
- Omitted network section: default isolated namespace, disposable child **started** and wrote its marker — positive spawn control
- Explicit egress allow: SDK returns error because `slirp4netns` not on PATH, process exit 3; **marker absent**, no fallback to a permissive namespace
- Unsupported ingress.default=allow: SDK rejects policy as unenforceable, exit 3; **marker absent**

Evidence: `evidence/upstream-rd-01/broker-net-02/mxc-network-refusal.json`.
Reproduction: `mxc_network_refusal.rs`, `qualify_mxc_network_refusal.py`.
No actual MXC egress allow was qualified: tool dependency missing, and previous host-loopback network control was invalid.

## MXC SDK + real Deno broker — PASS 5/5
A host-side broker Unix socket communicates through a deliberately mounted disposable filesystem path to Deno running **inside MXC `Containment::Process`**. No `--allow-write` flag is passed to Deno. Both layers' results are independently observed.

| Broker response | MXC filesystem | Observed effect |
|---|---|---|
| allow | readwrite | Deno writes `COMPOSITE_ALLOWED` |
| allow | readonly | OS rejects write with `EROFS` (errno 30) |
| deny | readwrite | Deno rejects write with `NotCapable` |
| deny | readonly | Deno rejects write with `NotCapable` |
| mismatched response ID | readwrite | Deno exits with code 87; no marker |

Every case observed one `write` broker request, correct target, intended exit/marker and no mock server error. Evidence `evidence/upstream-rd-01/broker-net-02/mxc-deno-broker.json`, tools `composite_mxc_broker.rs`, `qualify_mxc_deno_broker.py`.

## What this does NOT establish
- An unforgeable, identity-bound Judgment Plane `ExecutionGrant` or signed broker issuer
- Production-ready broker protocol, user-authentication, revocation, TTL, auditable durable receipts, isolation against malicious same-UID host processes
- Actual MXC directional network allow with positive external egress (requires `slirp4netns` and valid test environment)
- Multi-tenant or multi-host isolation, other OS backends, a security certification
- Binary execution of the exact pinned Oct 6 Deno tree

**Recommendation:** engineering handoff can proceed only as a design candidate. Next gate should bind an explicit typed authority and resource scope at the broker/OS seam, qualify issuer identity and cancellation, then retest in a real product context without implicit promotion.
