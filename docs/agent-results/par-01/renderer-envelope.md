# A02 — Renderer providers and visual envelope

Worker: A02 / batch IN-GNU6-PAR-01
Base: `c9dcdab997f63f67caaa182cb265d6cb9b970863` (branch `feat/par-01-A02`)
Date: 2026-10-08
Scope: read-only study of `src/renderer/**`, `adapters/**`, `qualification/**`, `tests/browser/**`, `tests/model/renderer-*.test.ts`, `tests/laws/renderer-*.test.ts`, `docs/evidence/mq-11..14*`, QUESTLOG SQ-007. No source or benchmark tool was modified.

Status vocabulary: VERIFIED_NOW, HISTORICAL_RECEIPT, HYPOTHESIS, FALSIFIED, NOT_RUN, BLOCKED_AUTHORITY, UNKNOWN.

## 1. Boundary: RendererFrame (source: `src/renderer/contract.ts`)

`scene truth -> buildRendererFrame(SceneState, options) -> RendererFrame -> adapter.render(frame) -> RendererResult<T>`. Downstream only.

RendererFrame (schemaVersion 1) = `{ sequence, sceneTime, motionMode: "normal"|"reduced", entities[] }`.
Per entity: `entityId, role, visible, z, worldPose (resolved through hierarchy), shape (rect | closed polygon), accessibility | null`.
Properties read from source: detached clones, deeply frozen, traversal follows `scene.order`, no engine/clock/mutation handle, no product state.

Contract checks:
- `validateRendererResult`: sequence match; each frame identity emitted xor omitted exactly once; no invented identity; no duplicate.
- `rendererSemanticDigest`: JSON of `sceneTime` + `entities`; excludes `sequence` and `motionMode`.
- Omission reasons declared: `not-visible | culled | unsupported`. Only `not-visible` is produced by any adapter in the repo (VERIFIED_NOW by grep). `culled` and `unsupported` are declared but never exercised.

Continuity oracle (`src/renderer/continuity.ts`): pure, renderer-neutral. Samples `{semanticExists, handleId, mounted, visible, renderable, alpha, finiteGeometry}`; 13 named laws (stable handle, mounted, visible, renderable, alpha > 0, finite geometry, one-way termination, no provider before birth or after termination).

## 2. Matrix: provider | source | capacités | preuve | inconnu

| Provider | Source | Capabilities (observed in source) | Proof | Unknown |
|---|---|---|---|---|
| Nonvisual `CommandRenderer` (id `nonvisual-command`) | `adapters/nonvisual.ts` | Pure frame -> `RenderCommand[]` (`entityId, role, worldPose, shape`). Omits invisible entities with `not-visible`. No DOM, no GPU. Role: deterministic oracle. Drops `z`, accessibility and motion mode from output. | VERIFIED_NOW: `renderer-adapter-equivalence.test.ts` (2 tests) pass. Receipt MQ-11 (HISTORICAL_RECEIPT). Causal mutation on the validator turned the contract test RED (HISTORICAL_RECEIPT, SHA-restored). | Whether command order is a valid paint order (no z sort, relies on `scene.order`). Not tested against Pixi output. |
| SVG string `SvgStringRenderer` (id `svg-string`) | `adapters/svg.ts` | Pure string; one `<g data-entity-id transform>` per visible entity; rect (with `rx`) and polygon; projects accessibility to `role`, `aria-label`, `aria-hidden`; XML escaping; `data-motion-mode`; throws on non-finite numbers. No DOM mutation. Described in source as "contract-qualification adapter, not the MQ-12 reference". | VERIFIED_NOW: equivalence tests assert identity attributes and aria output, plus determinism. | Never parsed or rasterized by a browser in any test. No `z` handling: output order is frame order, not sorted by `z`. Rotation or scale composition vs Pixi untested (inference: SVG `translate rotate scale` order should match Pixi, unverified). |
| PixiJS 8.22.0 / WebGL `PixiBrowserRenderer` (id `pixi-browser`) | `adapters/pixi.ts`, `qualification/pixi/**` | Reference provider (ratified, MQ-12). `EntityId -> Graphics` map (stable handle); ticker stopped, one manual `app.render()` per submitted frame; `eventMode = "none"`; fill by role; `zIndex = z`; hidden entities keep a hidden handle, removed entities destroy the handle; sibling semantic DOM (button/div, role, aria-label) keyed by `data-entity-id`; `observeProvider()` read-only probe; `diagnostics()`. | VERIFIED_NOW: Playwright 2/2 PASS (see section 4). HISTORICAL_RECEIPT: MQ-12, MQ-13 mutation REDs. | Physical-GPU behavior (receipts used SwiftShader). Browsers other than Chrome for Testing. Pixels (no pixel or screenshot assertion; the screenshot is evidence only). Mobile. Context loss. |
| Pixi WebGPU backend | `adapters/pixi.ts` accepts `preference?: "webgpu"` | The option exists in the options type; defaults to `"webgl"`, and the qualification surface hard-codes `"webgl"` at `qualification/pixi/main.ts:225,812`. | NOT_RUN. No test, receipt or benchmark selects it. | Everything: backend availability in the instrument, parity, perf. HYPOTHESIS only. |
| Native/custom WebGPU | none | QUESTLOG SQ-007 (state HYPOTHESIS, operator priority LATER, non-causal, "do not replace PixiJS by assumption"). | No code. | Everything. Not a decided migration. |
| DOM, Canvas2D | none | `adapters/README.md` lists them as candidates not to be created speculatively. | None. | Hypothetical only. |

Distinction kept: only `nonvisual`, `svg`, `pixi` exist as code. Everything else in the last three rows is a hypothetical or an exposed-but-unexercised option.

## 3. Identity and continuity proofs

Identity:
- Stable `EntityId -> Graphics` handle under magnification, neighbor yield, shape morph (via `SceneEngine.setShape`, added in MQ-12), unrelated insertion/removal, and hand -> playzone handoff. Asserted by handle tokens in `tests/browser/pixi-reference.pw.ts` (VERIFIED_NOW, 2026-10-08).
- No adapter may invent identities (`validateRendererResult`; law tests VERIFIED_NOW).
- Semantic DOM nodes are keyed by the same `EntityId`; asserted for `hand-b` (role, label).

Continuity (MQ-13): browser traces for hover magnification, deck reflow, window-to-card morph, insertion, removal, hand -> playzone handoff, context menu entry/exit plus attachment. All traces return `violations = []` and are re-run by the second Playwright test (VERIFIED_NOW). Every live sample asserts handle present, mounted, visible, renderable, alpha > 0, finite geometry. Insertion starts from semantic absence; removal ends with `handleId === null`.

Causal mutations (HISTORICAL_RECEIPT, not re-run now): MQ-11 validator weakened -> RED; MQ-12 handle recreated each render (`handle-3` -> `handle-13`) -> RED; MQ-13 `alpha = 0` on live `dock-c` -> RED with `PROVIDER_NONZERO_ALPHA_WHILE_SEMANTIC_IDENTITY_LIVES`. Each restored byte-identical (SHA-256 recorded in the receipts).

Reduced motion: `normal` and `reduced` give the same semantic destination; reduced settles directly. Asserted in Playwright (`semanticDestinationEqual`, equal targets). The digest excludes motion mode by construction.

## 4. Fresh runs (this worker, this worktree)

Environment: worktree `c9dcdab`, `mise exec node@24`, `npm ci` (repo-local `node_modules`, gitignored; no host package change), existing cached Playwright Chromium.

| Command | Result |
|---|---|
| `npx vitest run tests/laws/renderer-contract-laws.test.ts tests/laws/renderer-continuity-laws.test.ts tests/model/renderer-adapter-equivalence.test.ts` | 3 files, 12 tests PASS |
| `npx tsc --noEmit` | exit 0 |
| `IN_GNU6_QUALIFICATION_PORT=4392 npx playwright test` | 2 passed (8.2 s) |

Not re-run: full `npm run check`, `npm run build:pixi-qualification` as a separate step, MQ-14 performance (`test:performance*`), causal mutations. The Playwright run used the repo's headless Chromium; I did not capture which GL backend/ANGLE adapter it picked, so SwiftShader vs hardware in this run is UNKNOWN (the assertion `rendererName` contains "webgl" passed).

## 5. Performance envelope (HISTORICAL_RECEIPT, MQ-14, not re-measured)

Instrument: Chrome for Testing 153.0.8010.12, Pixi 8.22.0, WebGL via ANGLE/Vulkan/SwiftShader (software), ticker stopped, resolution 1, antialias off. Mean frame build / Pixi render: 16 entities 0.033 / 0.130 ms; 64: 0.142 / 0.197; 256: 1.60 / 0.775; 512: 6.02 / 1.47 (p95 1.80). At 512 entities, `buildRendererFrame` costs more than Pixi projection. JS heap about 20.6 MB (16) to 51.9 MB (512). These characterize the qualification environment, not a product threshold and not physical GPU throughput.

## 6. Limits of the tests (what a green result does not establish)

1. Headless/SwiftShader results do not establish quality, throughput, precision or memory on real GPUs (receipt's own limitation; this run did not check the backend).
2. Single browser (Chrome for Testing), single OS; Playwright reported the host as officially unsupported and used a fallback build in MQ-12.
3. No pixel-level or screenshot-diff assertions anywhere. "Rendered correctly" is inferred from scene-side and provider-handle observations (`visible`, `alpha`, bounds), not from composited pixels.
4. `observeProvider` reads Pixi object flags. It cannot detect a draw that is occluded, clipped, off-canvas or visually degenerate while flags look healthy.
5. Parity is only proven across Command vs SVG at the identity-partition level and by string/`toContain` checks. There is no cross-provider geometric or visual parity test (SVG vs Pixi vs Command). No `z` ordering parity test: Command and SVG ignore `z`, Pixi uses `zIndex` (source observation; effect on equal-`z` or mismatched order UNKNOWN).
6. `culled` and `unsupported` omission paths are unexercised.
7. Entities that are `visible: false` in the scene keep a hidden Pixi handle. The continuity oracle would flag such a sample (`PROVIDER_VISIBLE_WHILE...`); the reference traces avoid that case. Inference, not tested.
8. Accessibility: semantic DOM nodes are positioned with `opacity: 0.001` and `pointer-events: none`. No assistive-technology run (screen reader, keyboard traversal order, focus) exists for them.
9. `FALSIFIED`/negative results found: none in this study. No contradiction between receipts and the fresh run.
10. Long-duration leaks, GPU memory, context loss, resize/DPR changes (`resize()` exists, untested), devicePixelRatio > 1 are all NOT_RUN.

## 7. Most discriminating qualification tests for any future alternative (WebGPU or other)

Ordered by discriminating power per cost:

1. Same-frame cross-provider parity harness: feed one `RendererFrame` fixture set (rect, polygon, nested transforms, mixed `z`, hidden entities, reduced mode) to the Command oracle and the candidate; compare emitted/omitted partitions and per-entity transforms. Cheapest test that exposes `z`/transform divergence. Candidate must pass `validateRendererResult` unchanged.
2. Re-run the unchanged MQ-12/MQ-13 Playwright scenarios with only the backend changed (for Pixi: `preference: "webgpu"`); require identical handle-token and `violations = []` outcomes. Any need to edit the scenarios signals contract drift.
3. Pixel oracle: a deterministic raster comparison (fixed viewport, DPR 1 and 2, antialias off) of Pixi/WebGL versus candidate for a handful of static frames, with a stated tolerance. This closes limits 3 and 4 and is the only way to catch "flags healthy, pixels wrong".
4. Reuse the three causal mutations (recreate handle per render, force alpha 0, accept ghost identity) against the candidate; each must turn its gate RED. A candidate that cannot be made RED by these is not yet evidenced.
5. Backend identity gate: record the actual backend and adapter string (hardware vs software) in the receipt, and fail the run if it differs from the declared instrument. Required before any performance comparison.
6. MQ-14 workloads (16/64/256/512 entities) on a physical GPU, same entity counts, reporting frame build vs projection separately, since build dominates at 512 on the current reference.
7. Failure modes: WebGPU unavailable (fallback or explicit unsupported), device/context loss, resize and DPR change, tab hidden; each must produce a declared omission (`unsupported`) or an error, never a silent partial frame.
8. Accessibility parity: semantic layer attributes identical to the reference for the same accessibility inputs; one real keyboard/screen-reader pass is a separate operator-authority item (host tooling changes need authorization).

## 8. Claim statuses

| Claim | Status |
|---|---|
| RendererFrame is a detached, frozen, downstream-only projection with identity partition law | VERIFIED_NOW (source read + law tests PASS) |
| Command and SVG adapters preserve identity and determinism | VERIFIED_NOW |
| Pixi 8.22.0/WebGL reference preserves handle identity across the reference scenarios | VERIFIED_NOW (Playwright 2/2) |
| No-cutoff continuity holds for the nine reference transitions | VERIFIED_NOW (Playwright test 2) for the instrument used |
| Causal mutations turn the gates RED | HISTORICAL_RECEIPT (not re-run) |
| MQ-14 numbers | HISTORICAL_RECEIPT (SwiftShader, not physical GPU) |
| Pixi WebGPU preference works | NOT_RUN |
| Native/custom WebGPU renderer is better or required | UNKNOWN; SQ-007 is LATER, hypothesis |
| Cross-provider visual or z-order parity | NOT_RUN (z asymmetry observed in source) |
| Real-GPU, multi-browser, mobile, a11y-tool behavior | NOT_RUN / UNKNOWN |

## 9. Non-claims

This document does not ratify, select or deprecate any provider, does not decide a WebGPU migration, does not authorize public API or publication, does not promote SQ-007 or any side quest, and does not alter MQ-11 to MQ-14 dispositions. A green headless run is not evidence of GPU quality or product readiness.

## 10. Next decision (operator)

Whether to promote a cross-provider parity and pixel-oracle gate (tests 1 and 3 above) as a prerequisite SQ-007 work item, and when a physical-GPU instrument run is authorized. Until then SQ-007 stays LATER.
