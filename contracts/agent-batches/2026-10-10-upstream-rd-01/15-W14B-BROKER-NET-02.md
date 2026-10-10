# W14B — Broker IPC Deno × MXC network refusals

Scope: extension of 00-BASE, 12-MXC-SECURITY-01, 13-DENO-PERMISSION-01 and 14-MXC-DENO-BOUNDARY.
**Authority:** isolated experiment on one Linux workstation. No changes to Gnosix Judgment Plane, production MCP, Coolify, upstream source checkout, user filesystem policies, `main`, public sites or credentials.

## Source/binary pins
- MXC: `microsoft/mxc@c6f301d53a1430c4c921a05c57af838f7392348f`, SDK 1.0.0, source-built in scratch target with Rust 1.97.1
- Deno: official Linux v2.9.7 release, archive SHA-256 `c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490`
- Deno `runtime/permissions/broker.rs` Git blob `a7c70e963ed2a6a7bdc7c14bb8581c39b0ce119c` matches in v2.9.7 and pinned Oct 6 upstream commit. The **entire binary is not built from the latter commit**.

## Gates
- B01: live Deno broker Unix stream: allow/deny before ordinary permission state, response-ID mismatch, unknown response, broken socket, missing broker path, baseline without broker
- B02: actual MXC SDK network policy: supported ruleless-deny positive spawn; slirp4netns-dependent allow and unsupported ingress-allow both reject **before child spawn**, with durable errors
- B03: actual Deno process inside MXC: broker socket roundtrip allows/denies disposable file writes, independently from OS read-only/writeable policy; mismatched response ID exits 87
- B04: receipts and exact provenance. Failed/negative gates remain visible. No claim for successful external egress or security certification.

Exclusive writes for this isolated worktree: new files `tools/upstream-rd-01/*broker*.py`, `tools/upstream-rd-01/deno_broker_fixture.ts`, `tools/upstream-rd-01/mxc_network_refusal.rs`, `tools/upstream-rd-01/qualify_mxc_network_refusal.py`, `docs/agent-results/upstream-rd-01/w14b-broker-net-02.md` and `evidence/upstream-rd-01/broker-net-02/*`, plus this contract. Integration occurs via reviewed cherry-pick.
