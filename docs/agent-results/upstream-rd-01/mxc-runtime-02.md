# MXC runtime qualification · W01B · 2026-10-10

**LAB_PASS: SDK filesystem gates 2/2, isolated Linux host.**

Source: `microsoft/mxc@c6f301d53a1430c4c921a05c57af838f7392348f`, MIT; checkout unchanged. MXC SDK 1.0.0 compiled in a scratch Rust project using cargo cache and target directory under `.probes/upstream-rd-01`, Rust 1.97.1, offline locked SDK dependencies. `cargo check --offline --locked -p mxc-sdk --lib` succeeded.

Executable proof: `tools/upstream-rd-01/qualify_mxc_runtime.py`, fixture source `mxc_runtime_fixture.rs`; receipt `evidence/upstream-rd-01/mxc-runtime-02/qualification.json`. Two disposable modes: `readonly_paths` permits reading sentinel, rejects marker creation with read-only filesystem, and leaves marker absent; `readwrite_paths` permits marker creation; sentinel unchanged. Two mode checks **PASS_2_OF_2**. No production files touched.

Limitations: actual MXC SDK backend Process/Bubblewrap on one Linux workstation, not every policy combination; no external broker, no composite Deno integration, no tenancy or security certification. Rust 1.97.1 differs from upstream's preferred 1.93 toolchain. The separate scratch Cargo manifest points to the pinned SDK by absolute local path; no donor code was vendored into GNU6.

Next: policy support refusal, denied_paths, network egress allow/deny through the MXC SDK, and cross-vendor Linux qualification before declaring any GNU6 product gate.
