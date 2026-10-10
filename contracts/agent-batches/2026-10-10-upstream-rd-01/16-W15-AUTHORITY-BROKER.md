# W15 — Judgment Plane authority-to-broker seam

Extends 00-BASE, 14-MXC-DENO-BOUNDARY and 15-W14B-BROKER-NET-02. Date 2026-10-10.
**Authority:** isolated lab program only. No write permissions over production Judgment Plane, gnosix-authority-core, Gnosix desktop, Desktop MCP, in.gnu6/main, agents, Coolify, GitHub visibility or other beneficiary repositories.

## Sources of truth
- `gnosix-authority-core@de096640d52c75734813611aca4f858fa7ff47d9`, read-only repository
- `system-one-rust@745e962bb3628d52f0c98386ace232a35003db69`, read-only working tree
- Official Deno 2.9.7 binary with separately verified release SHA-256; the Deno full-tree source remains different from pinned Oct 6 source
- MXC SDK v1.0.0 pinned in prior batch; **not exercised inside W15**

## W15-a lab contract
- Implement a non-production Rust Unix socket broker using real `ScopedCapability`, `ResourceScope::Filesystem`, `AuthorityEpochSource`, `TrustedMonotonicClock`, `ExecutionGrant::consume_atomic`, and `SystemAuthorityFactory::create_engine_for_testing`.
- Gate Deno's real permission request on kernel `SO_PEERCRED` UID/PID + request PID, exact action/resource binding, scoped delegated capability and one-time grant consumption.
- Cases: valid, escape, excessive TTL, expiry, revocation, replay, changed action digest, changed resource identity, wrong operation, wrong expected peer PID and symlink alias. Reject without writing any file.
- Compile-fail probes from an outside crate must reject struct construction and JSON deserialization of the sealed grant.
- Run two independent matrices with fresh disposable directories and save both receipts.

## Binding limitations
`test-support` is enabled *only in the scratch Cargo crate*. The factory is not a production issuer. **Do not ship a broker or authority binary with this feature or treat file-system peer credentials as cryptographic authorization.**
File path resolution currently checks a canonicalized temporary parent and exact lexical target; it is **not** an openat2/dirfd race-free path resolver. No real issuer credential, TTL across processes, signed grant transport, resource-inode binding, cancellation propagation, production deployment or MXC-in-the-same-process test is authorized by a laboratory PASS.

Exclusive W15 worker write-set: `tools/upstream-rd-01/w15_grant_broker.rs`, `qualify_w15.py`, `qualify_w15_forgery.py`, `docs/agent-results/upstream-rd-01/w15-authority-03.md`, `evidence/upstream-rd-01/w15-authority-03/` and this contract. Integration by reviewed cherry-pick only.
