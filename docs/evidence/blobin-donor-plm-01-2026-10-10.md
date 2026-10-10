# BLOB-DONOR-PLM-01 — Pleamar → blob.in / in.gnu6 / Gnosix

**State:** INTERNAL EXPERIMENT / GREEN_LOCAL for EXP-B only. No adoption or public API ratification.
**Branch:** `feat/blobin-donor-plm-01-20261010`, base `in.gnu6/main @ d160639e804c244a84197d8dc4906ffa2df313e8`.
**Companion GPU branch:** `feat/blobin-gpu02-native-20261010 @ a44daf2`, separate worktree rooted at GPU-01 `5d2b5bd`. These branches do not merge or mutate one another.

## Donor interpretation (read-only research)

- `https://github.com/k4ditano/pleamar`: Rust/wgpu renderer, declarative `.plm` language, independently progressing render/logic and GPU-level primitives; reported logic-stall resilience and Marea bar measurements remain donor-reported, not retested here.
- `https://github.com/k4ditano/marea-plm`: example end-user scene/shell and agent UI, not a replacement for Gnosix scene authority or bounded execution grants.
- `https://github.com/k4ditano/pleamar-wm`: distinct Smithay compositor; Gnosix currently runs on Hyprland. No compositor replacement is proposed.
- **No donor code, assets or dependency was vendored.** Any later source reuse needs license/provenance review and separate operator approval.
- The goal is not to transplant Pleamar's ontology. Donor ideas: bounded scene compilation, renderer cadence independent of logic, persistent GPU resources, incremental invalidation, useful materials.

## Capital ledger: owner separation

| Capital | Present code | Invariant preserved | Open question |
| --- | --- | --- | --- |
| Semantic scene | `in.gnu6/src/scene/` | identity/time/topology authority independent of providers | portable versioned native delta encoding |
| Renderer projection | `in.gnu6/src/renderer/contract.ts` | source facts are detached/frozen, exactly-once identity partition | cheap spatial/frame incremental updates |
| Historical blob.in | Rust core, Lyon, springs, oracles (archived source) | historic receipts do not qualify current binary | independent rebuild, license/provenance |
| Native 2D | `blob.in-showcase` / `BLOB-FRAME-v1` | limited to 2D circles; fixed typed IPC | separately versioned hybrid scene input |
| Native hybrid | GPU-01 / GPU-02 branches | real Vulkan mesh/SDF shader, standalone Wayland presentation in GPU-02 | independent live oracle, compositor timing, product bridge |
| GPU effect incrementality | F4C | bounded effect dispatch ROI, CPU oracle | source uploads/readback remain full-frame |
| GPU supervision | CUDA-05F + opt-in resident | permissions/gates/renewals not transferred from renderers | no renewal hitch, distinct process authority |

## EXP-B delivered here

`src/renderer/delta.ts` supplies pure `diffRendererFrames` and `applyRendererDelta`.
Its inputs are existing **MQ-11 authorized immutable RendererFrames**, not user JSON or renderer-origin mutation requests.
It exports no engine mutation method or product intent.

Falsification rules: strictly increasing sequence, immutable ordered `births` / `updates` / `deaths`, reject invented/duplicate/replayed identities, preserve reference identity for unchanged render entities, preserve reduced-motion and clock metadata without synthetic entity updates. Scope: rect/polygon shape ontology only.

`tests/laws/renderer-delta-laws.test.ts`: 5 deterministic checks, including corrupted deltas and duplicate identities. **5/5 GREEN_LOCAL** (plus TypeScript typecheck), not a native-transport qualification, no measured performance gain, and not a Gnosix/GPUI integration.

Fresh full repository gate: `npm run check` PASS on this worktree: biome format/lint, TypeScript, **33 files / 134 tests**.

Causal RED→GREEN: a temporary pose equality mutation (`samePose` -> `true`) produced **1 expected failure** in the 5 focused tests (`card` update lost). The exact original bytes were restored and matched SHA-256 `87f6cce60bd0eb1ce276ee2530f7c1cdc497b523991241c5a13e0552017f6752`; the focused suite then returned **5/5 PASS**. Raw logs: `/mnt/workbench/evidence/runs/blobin-next-01/donor-plm-01-20261010/{causal-red,restored-green}.log`.

The replay helper is a **qualification-only tool**, not an authority to mutate `SceneEngine`. Receivers must not accept attacker-supplied births or updates without an independently authorized semantic source.

## Experiment queue (do not inflate CP00–CP08)

1. **EXP-A logic-stall separation:** stop or block semantic input for 600 ms; measure continuity of renderer presentation in an isolated native host. GPU-02's current single-thread event loop **cannot** prove this property.
2. **EXP-B native scene contract:** convert this prototype into a versioned, bounded Rust DTO and delta replay with independent TS↔Rust test vectors, malformed-message corpus and identity coverage.
3. **EXP-C GPU residency:** independently benchmark and eliminate CUDA05F resident renewal gaps without weakening authorization; GPU-02 standalone wgpu is **not** a substitute for the gated worker.
4. **EXP-D incremental GPU:** promote F4C ROI from effect kernels to upstream generation, transfer and Wayland damage, with independent pixel oracles.
5. **EXP-E material graph:** benchmark render composition, paths, gradients and clips on actual GPU; avoid aesthetic claims as benchmark results.

**Boundaries:** source-only TypeScript experiment, no browser/GUI mutation, no GPU reset, no systemd changes, no merge/push, no public publication. `BLOB-FRAME-v1` remains unchanged. CP02/CP03/CP08 retain their separately ratified gates. Any later merger requires branch-head reconciliation because another ChatGPT worker is concurrently testing GPU-01-related work.
