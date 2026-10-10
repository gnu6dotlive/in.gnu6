# UPSTREAM-R&D-01 — W15 authority-gated live Deno broker (2026-10-10)

**Status: LAB_PASS_11_OF_11 × 2; COMPILE_FAIL_2_OF_2; PRODUCTION_ISSUER_NOT_RUN.**

## Acquired source truth
- `gnosix-authority-core@de096640d52c75734813611aca4f858fa7ff47d9` (read-only); core `src/scope.rs`, `src/grant.rs`, `src/clock.rs`, `src/bootstrap.rs`
- `system-one-rust@745e962bb3628d52f0c98386ace232a35003db69` (read-only); `src/authority_compat.rs`
- Real core APIs exercised: `ScopedCapability`/typed `ResourceScope::Filesystem`, test authority factory, `ExecutionGrant::consume_atomic`, epoch revoker, manual monotonic clock
- Real official Deno v2.9.7 broker IPC over Unix socket, `DENO_PERMISSION_BROKER_PATH`, one newline-delimited JSON request and correlated allow/deny
- Scratch Rust adapter compiled against **live local core** in an isolated Cargo project; both original source trees remain clean

## Causal outcomes
| Case | Broker result | Target file |
|---|---|---|
| Valid scoped `write`, matching process UID/PID/resource/action, unconsumed grant | allow | Written |
| Grant request escapes authority scope | deny | Absent |
| Grant issuance TTL exceeds capability | deny | Absent |
| Expired monotonic grant | deny | Absent |
| Authority epoch revoked | deny | Absent |
| Already consumed grant replay | deny | Absent |
| Action digest mismatch | deny | Absent |
| Resource identity mismatch | deny | Absent |
| Requested `write` not in grant operations | deny | Absent |
| Deliberately mismatched expected peer PID | deny | Absent |
| Symlink alias of authorized target path | deny | Absent |

**All 11 cases passed twice (22/22 expected outcomes)** in fresh isolated directories. `SO_PEERCRED` was checked for UID and PID, the Deno request PID matched its process, the exact requested path and canonicalized parent were checked, and only then was the real sealed grant consumed. Deno wrote `COMPOSITE_ALLOWED` only in the positive case; negative cases returned `NotCapable`.

Evidence: `evidence/upstream-rd-01/w15-authority-03/qualification-first.json`, `qualification.json`. Reproduction tools: `w15_grant_broker.rs`, `qualify_w15.py`.

## External forgery compile-fail cases
- External crate attempts `ExecutionGrant{grant_id: ...}`: Rust compile fails with *cannot construct ExecutionGrant with struct literal syntax due to private fields*
- External crate attempts `serde_json::from_str::<ExecutionGrant>("{}")`: Rust compile fails because `Deserialize` is not implemented
- Both failures are expected and specifically classified; `evidence/upstream-rd-01/w15-authority-03/forgery.json`, `qualify_w15_forgery.py`

## Boundaries — still OPEN
1. **Test feature is not a production issuer.** This lab deliberately enables `gnosix-authority-core/test-support` to mint synthetic authority. Its inclusion in deployed builds would be unsafe, even though default consumer access to a grant's private fields is denied.
2. **No production identity source.** Kernel `SO_PEERCRED` UID/PID ties the local socket peer to the spawned Deno child only; it does not bind a trusted user principal, remote host, persisted credential, signed token or unique workload attestation.
3. **Filesystem race/symlink hardening is incomplete.** The fixture rejects a synthetic alias and uses a canonical parent plus exact value, but does not use `openat2`, a resolved O_PATH inode, or a secured dirfd for execution; adversarial filesystem races remain untested.
4. **No three-layer execution yet.** This W15 tests Deno + real authority core; W14B separately tested Deno + MXC SDK. There is *no* simultaneous authority-core → live broker → Deno → MXC invocation on one process chain.
5. **Deno v2.9.7 != Oct 6 full-tree binary.** The broker component was blob-identical in W14B, but the entire runtime build wasn't reproduced from the pinned revision.
6. **No public deployment or claim.** Core, desktop MCP, Gnosix, in.gnu6/main, VPS and production access policies untouched.

## Proposed next falsification
Build W16 using a *single* process chain that transports an in-process synthetic `ExecutionGrant` through broker IPC while MXC enforces a read-only filesystem policy. Separate non-production tests must probe peer process substitution, live epoch revocation, TTL between broker decision and syscall, resolved resource inode and request cancellation. Only after owner review of the production issuer and endpoint authority model may this graduate toward a product worktree.
