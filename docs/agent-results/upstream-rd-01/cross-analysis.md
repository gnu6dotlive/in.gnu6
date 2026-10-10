# UPSTREAM-R&D-01 — W07 cross-analysis

**State: SOURCE_QUALIFIED / TRANSFER_PROPOSED / NO PRODUCT INTEGRATION.** Date: 2026-10-10.
Sources are frozen in `contracts/agent-batches/2026-10-10-upstream-rd-01/sources.json`.
Independent evidence: `evidence/upstream-rd-01/w01/…w06/source-evidence.json`, each with SHA-256, exact blob/line and bounded keyword coverage. Three source files per upstream were examined in the second tranche. **Keyword absence in a file is never whole-project absence.**

## Observations that change technical decisions
1. **MXC / OS authority**: the pinned [bwrap contract](https://github.com/microsoft/mxc/blob/c6f301d53a1430c4c921a05c57af838f7392348f/docs/backends/bwrap/bubblewrap-backend.md) describes enforcement and backend limitations. The [access-denied logging doc](https://github.com/microsoft/mxc/blob/c6f301d53a1430c4c921a05c57af838f7392348f/docs/logging-access-denied.md) distinguishes denied-access capture from permissive learning/audit mode. These are different policy behaviors. Independent host bwrap oracle passed but **MXC integration NOT_RUN**.
2. **PlxNative / GPU causality**: [damage tracking verdict](https://github.com/GLinnik21/plx-native/blob/35ca8790072621d6cbad8e75eb70fb1fc3f24915/docs/perf-damage-tracking-verdict.md) reports a specific device-based observation: ≈37.2% quads removed while GPU_ACTIVE moved only ≈−0.81%. This **upstream-only observation** weakens naive draw-count optimization; it does not predict blob.in performance. [Rust module README](https://github.com/GLinnik21/plx-native/blob/35ca8790072621d6cbad8e75eb70fb1fc3f24915/rust-modules/README.md) exposes native Rust/C ABI separation.
3. **Deno / Rust op boundary**: [deno_runtime README](https://github.com/denoland/deno/blob/a18ce33715e30cd2b0d99c7e322ef11a65490e4d/runtime/README.md) describes OS ops; [desktop architecture](https://github.com/denoland/deno/blob/a18ce33715e30cd2b0d99c7e322ef11a65490e4d/doc/desktop-architecture.md) describes native backend and HMR supervision. The selected `runtime/permissions.rs` is a two-line re-export: **insufficient to qualify actual permission enforcement**. Next source is `runtime/permissions/lib.rs`; implementation remains NOT_RUN.
4. **GPUI Kit / authority to render**: [GPUI Shell](https://github.com/longbridge/gpui-kit/blob/ed27c6293b585acc40c10be7138534243bee5841/docs/gpui-shell.md) exposes QuickJS scripting/capabilities but explicitly experimental semantics; [accessibility-driven tests](https://github.com/longbridge/gpui-kit/blob/ed27c6293b585acc40c10be7138534243bee5841/docs/ACCESSIBILITY-UI-TESTING.md) support behavior-oriented oracles. Native GPUI control is not authority to redefine in.gnu6 scene ontology.
5. **Hyperfine / instrument validity**: [metric source](https://github.com/sharkdp/hyperfine/blob/b33755fe1fca8bcc1f182c773ca3ab040bfe517d/src/metric.rs) has availability checks for shell-dependent metrics; [options](https://github.com/sharkdp/hyperfine/blob/b33755fe1fca8bcc1f182c773ca3ab040bfe517d/src/options.rs) exposes execution configuration. The `hyperfine` binary is not installed here; no hyperfine measurement, GPU timer or speedup is claimed.
6. **Uzu / engine portability**: [backend module](https://github.com/trymirai/uzu/blob/23f365fc43cd7df0c4e6d6d2415c553a94e37a08/crates/uzu-engine/src/backends/mod.rs) declares conditional CPU/Metal backend modules; [Cargo features](https://github.com/trymirai/uzu/blob/23f365fc43cd7df0c4e6d6d2415c553a94e37a08/crates/uzu-engine/Cargo.toml) defaults to cpu/metal/grammar. These selected files do **not** support a CUDA/PTQ1 drop-in claim; no 27B model loaded.

## Interaction of research axes
- MXC and Deno: app/runtime permissions must not be mistaken for an enforced OS sandbox. Proposed oracle: request a prohibited file and network operation, verify both host denial and policy-level receipt. Target owner: Judgment Plane / MCP.
- GPUI Kit and in.gnu6: UI state and scene topology must retain distinct owners. Proposed oracle: relocate scene renderer without changing canonical scene identity. Target owner: in.gnu6 and Gnosix separately.
- PlxNative and Hyperfine: performance claims require correct instrumentation. Hyperfine measures process-runtime properties, **not** physical GPU_ACTIVE/VRAM; proposed two-instrument oracle on blob.in hardware.
- Uzu and gnostral.rs: cross-language engine design may transfer while model format/backend/kernel compatibility does not. Proposed oracle: model/tokenizer parity on a small compatible fixture before any RTX3070 benchmark.

## Beneficiary matrix — non-discriminatory intake

| Beneficiary | Candidate research | Owner-local falsification before adoption | Decision |
|---|---|---|---|
| Gnosix shell and panels | GPUI Kit / PlxNative | Native Wayland IME/focus/accessibility; rendering latency evidence | EXPERIMENT |
| Cabinet XDG FileChooser | GPUI Kit / MXC | Portal file chooser D-Bus caller compatibility, correct grant scope | EXPERIMENT |
| Judgment Plane | MXC / Deno | ScopedCapability cannot amplify across runtime and OS backend | EXPERIMENT |
| in.gnu6 | PlxNative / GPUI Kit | Provider swap preserves scene identity, no new product semantics | OBSERVE |
| blob.in | PlxNative / Hyperfine | Frame-time + GPU tracer on same physical test, oracle independent | EXPERIMENT |
| gnostral.rs | Uzu / Hyperfine | Model/quant parity, engine orchestration isolation, baseline provenance | EXPERIMENT |
| Ternary Bonsai 27B lab | Uzu / Hyperfine | CUDA/PTQ1 exact compatibility on RTX3070; no unsupported model conversion | OBSERVE |
| Desktop MCP / local agents | MXC / Deno | Network, filesystem, process controls deny with durable audit | EXPERIMENT |
| gnu6.live | Deno architectural supervision | Deployment watcher, content source SHA and real browser path | OBSERVE |
| docs.gnu6.live | PlxNative / Hyperfine methodology | Bilingual qualified docs + live hash check, no claims-only cards | ADAPT_CANDIDATE |
| VPS/Coolify | MXC / Deno workload lifecycle | Agent tenancy boundaries and disposable sandbox, no prod mutation | OBSERVE |
| DevOps / reproducible OSS | Hyperfine / Deno | Repeatable CLI baselines; instrumentation overhead quantified | EXPERIMENT |

## Negative decisions and work explicitly withheld
- Do not vendor GPL-3.0 PlxNative source into a differently licensed product without formal compatibility review.
- Do not copy GPUI Kit documentation images or prose without satisfying attributed license and third-party exceptions.
- Do not present Bubblewrap with a read-only host root as confidentiality-preserving container; host paths remain readable.
- Do not call JS permission prompts or `ExecutionGrant` an independently proven OS confinement boundary.
- Do not install Deno or Hyperfine on the reference host as an automatic response to a missing binary.
- Do not call source-read qualification `LAB_PASS`, `PROJECT_QUALIFIED` or `PUBLISHED`.

Next gates: close Deno's actual permissions module source trace, run minimal *target-specific* fixtures only after owner review, then document public candidates with per-claim provenance. W07 decision is **qualify experiments**, not authorize product integration.
