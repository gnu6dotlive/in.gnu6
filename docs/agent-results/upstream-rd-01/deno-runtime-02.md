# Deno runtime qualification · W03B · 2026-10-10

**LAB_PASS: official Deno 2.9.7 process permission fixtures 6/6.**

Official archive `deno-x86_64-unknown-linux-gnu.zip` for tag `v2.9.7` verified against release SHA-256 `c6527f24f4b16031d3ae4fa9f658d5f11534c8d84ce7dc8502420280919c3490`; executable placed only in scratch `.probes/upstream-rd-01/deno-v2.9.7/runtime/`, not installed globally.

Exact executable fixture: `tools/upstream-rd-01/deno_permission_fixture.ts`; replay tool: `qualify_deno_runtime.py`; result `evidence/upstream-rd-01/deno-runtime-02/qualification.json`. Six results: read default deny, read exact allow, other-path denial, explicit deny overriding broad allow, local TCP default network denial, scoped TCP allow — **all PASS**. Negative cases returned `NotCapable`. Disposable files preserved; network tested only against local ephemeral HTTP server.

**Provenance mismatch:** Deno 2.9.7 release predates pinned `denoland/deno@a18ce33715e30cd2b0d99c7e322ef11a65490e4d` from 2026-10-06. Broker Rust source was separately inspected; this runtime test does **not** execute that exact commit or exercise an external permission broker. These Deno permission decisions do not imply an OS kernel sandbox.

Next: exact-revision Deno binary/build or equivalent independent bridge; broker allow/deny, bad response ID and unavailable broker across actual IPC, then MXC-Deno composite host oracle.
