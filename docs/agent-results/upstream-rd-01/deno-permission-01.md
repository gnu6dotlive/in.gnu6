# W03A — Deno permission broker, 10 October 2026

**Verdict: PINNED_SOURCE_GUARDS_PASS / DENO_RUNTIME_NOT_RUN / HOST_OS_CONTAINMENT_NOT_PROVEN.**

## Truth acquired
- Exact [Deno permissions broker](https://github.com/denoland/deno/blob/a18ce33715e30cd2b0d99c7e322ef11a65490e4d/runtime/permissions/broker.rs) and [permission state implementation](https://github.com/denoland/deno/blob/a18ce33715e30cd2b0d99c7e322ef11a65490e4d/runtime/permissions/lib.rs) retrieved through authenticated GitHub Contents API at pinned commit `a18ce33715e30cd2b0d99c7e322ef11a65490e4d`. Blob SHA and SHA-256 recorded in `evidence/upstream-rd-01/deno-permission-01/qualification.json`.
- Broker request includes version, pid, id, timestamp, permission and optional value. Response ID mismatch returns an I/O error; only explicit `allow`/`deny` variants accepted; unknown variant errors; broker connection/check errors exit the process at a dedicated constant code 87.
- The `PermissionState::check` path consults an installed broker **before** checking ordinary permission state. Broker `Allow` returns success; broker `Deny` constructs `PermissionDeniedError`. This ordering creates an **authority-design question**, not a demonstrated vulnerability: external broker grants require an explicit owner and trust boundary.
- Exact pinned source gate detected **10/10** seeded guard removals (and initial faulty oracle was repaired before qualification). No Deno binary found on host; no running Deno JS `Deno.permissions` behavior tested.

## Implications
- The broker is an authorization mechanism; do not treat it as a Linux namespace, filesystem mount or syscall sandbox.
- An external broker might intentionally override a permission state. GNU6 must define whether its Judgment Plane can accept such delegations and how to prevent untrusted callers from inventing grants.
- Deno's `runtime/permissions.rs` is a re-export stub; implementation reviewed at `runtime/permissions/lib.rs` and `broker.rs`.

## Open gate
A bounded Deno runtime fixture needs an exact pinned compatible runtime binary or an independently constructed executable build, before reporting actual broker IPC I/O, unknown result handling and deny of real filesystem/network operations. No automatic Deno installation, product mutation or agent privilege granted.
