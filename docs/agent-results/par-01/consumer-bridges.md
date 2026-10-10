# PAR-01 / A03 — Consumer bridges (gnu6.live, Docklab, SPJ)

Worker: A03. Base: `c9dcdab997f63f67caaa182cb265d6cb9b970863`. Branch `feat/par-01-A03`.
Write-set: this file only. Read-only access to `gnu6-live` (no command mutated it).
Status vocabulary: VERIFIED_NOW / HISTORICAL_RECEIPT / HYPOTHESIS / FALSIFIED / NOT_RUN / BLOCKED_AUTHORITY / UNKNOWN.

## 1. Reconstructed revisions

| Item | Fact | Status |
|---|---|---|
| `68371fe` | `68371feff38a214d121100cc6d25725a629fdbab`, parent `00fe599`, 2026-10-07 14:18 -0400, "feat(topology): plan canonical rect handoffs". Reachable in this repo's history. | VERIFIED_NOW (`git log`, `git cat-file`) |
| `2602d30` | `2602d30e3cb10ec40f19718de651b0b34e9c9b23`, 2026-10-07 15:18 -0400, "feat(topology): retarget canonical rect handoffs". `68371fe` is its ancestor (`merge-base --is-ancestor` true). Contained in `main` and the other local branches. | VERIFIED_NOW |
| Engine delta `68371fe..2602d30` | 4 files, +276: `src/topology/handoff.ts` (+63), `tests/laws/rect-handoff-retarget-laws.test.ts` (+93), QUESTLOG, IN-01 evidence. Engine change = retarget only. | VERIFIED_NOW (`git diff --stat`) |
| Consumer tag `experience-v6.8.0` | In `gnu6-live`: annotated tag object `7ae538ae995590b0a0cfa33ab548ccfed3e3fd93` -> commit `7ae403c9423e67a30db5d3d9eee93afb852fb7a5` ("consolidate qualified v6.8.0 retarget baseline"). Consumer branch `feat/experience-in-02-retarget-handoff`. | VERIFIED_NOW |
| Live-receipt commit `c7b2a1b` | Exists in `gnu6-live`, descends from the tag; adds the local-live receipt/assets. It is NOT in the tag's tree (tag tree lacks `experience-v6.8.0-local-live-2026-10-07.md`; it has the source-qualification doc). | VERIFIED_NOW |
| gnu6-live working tree | On `feat/truth-rebuild-v0.7` with unrelated modified files; HEAD `a8da045`. Tag/commit are read through git objects only, so this does not affect the findings. | VERIFIED_NOW |

Pin chain (consumer tag -> engine): `vendor/in.gnu6/manifest.json` at `7ae403c` has
`sourceRevision=2602d30e...`, `sourceTreeSha256=d78894585d14efb637480121240ff1ed76aadb9cf4a38673d30ae575f76ec5fe`;
`app/experience/engine/in-gnu6-revision.ts` exports label `in.gnu6@2602d30`. VERIFIED_NOW (read from git objects).
Byte identity re-checked now: all 22 files under `src/` at `2602d30` have the same SHA-256 as `vendor/in.gnu6/src/**` at `7ae403c`
(22/22, 0 diffs; loop of `git show | sha256sum`). I did NOT recompute the aggregate tree digest nor run `npm run in-gnu6:check`, so the
digest value above is HISTORICAL_RECEIPT; per-file identity is VERIFIED_NOW.
Note: IN-01 (`d93a559`) was synced to `68371fe`; the 6.8.0 tag is synced to `2602d30`. Two distinct pins, not one.

## 2. Contract handoff matrix

| # | Handoff | Product side (owner: consumer) | Engine side (owner: in.gnu6) | Adapter / carrier | Evidence |
|---|---|---|---|---|---|
| H1 | Minimize | Semantic request; queued, resolved at activation; mode flips `normal->minimized` at consumer commit gate. In consumer code the morph kind is named `capture`; the mapping Minimize=capture rests on the IN-01 receipt (CAPTURE_MODE_AFTER=minimized) plus `window-intake.ts`. | `planRectHandoff` with sequence `reshape-then-translate`; returns source/transfer/target rect geometry, radii, vector, generation. | `engine/in-gnu6-handoff.ts` (`resolveInGnu6RectHandoff`), called from `dock/window-intake.ts` `createWindowMorphIntent`. | Source: VERIFIED_NOW. Live: HISTORICAL_RECEIPT (radii 12,17,17; owner `in.gnu6@68371fe`). |
| H2 | Restore | Semantic request, same queue; mode `minimized->normal` at consumer commit. | Same planner, sequence `translate-then-reshape`. | Same adapter. | Source: VERIFIED_NOW. Live: HISTORICAL_RECEIPT (radii 17,17,12). |
| H3 | Retarget (active/queued Minimize/Restore) | Consumer decides when destination changed; keeps same token/node. | `retargetRectHandoff`: keeps subject, original source and sequence; requires strictly newer generation; increments `retargetCount`. | `retargetInGnu6RectHandoff`, `retargetWindowMorphIntent`. | Engine law tests: VERIFIED_NOW (see 3). Live: HISTORICAL_RECEIPT (capture gen 1->2, restore gen 3->4, same subject `workspace-1`). |
| H4 | Detach (window -> card, `direct`) | Consumer-owned allocation change. | Same planner, sequence `direct`, no transfer shape. | Same adapter. | Source: VERIFIED_NOW (`window-intake.ts` maps detach to `direct`). Not separately live-qualified in the receipts read: UNKNOWN. |
| H5 | Deck / window stack presentation | Page, max visible, mode (cascade etc.), card sizes, `WindowInstance` list. | `layoutTarotFan` / `layoutCardSpread` / `layoutCardStack`, `projectDeckCentersToWorld`, `SpatialFrameGraph`. | `engine/in-gnu6-window-stack.ts`, `engine/in-gnu6-deck.ts` (re-exports `previewDeckOrder`, `tiltFromVelocity`). | Source: VERIFIED_NOW. Tests `tests/unit/experience-in-gnu6-window-stack.test.ts`: NOT_RUN by me. |
| H6 | Provenance / sync | Chooses revision explicitly; re-qualifies after each sync. | Immutable `src/` bytes at the chosen commit. | `scripts/in-gnu6-consumer.mjs` (`sync` needs `--revision`; `check`), vendored `vendor/in.gnu6/`, manifest. | SQ-009 receipt: HISTORICAL_RECEIPT (mutation of `dynamics/math.ts` -> RED, resync -> GREEN). Fresh run: NOT_RUN. |
| H7 | Docklab harness (in this repo) | Edge identity (`top/right/bottom/left`) = consumer policy. | `SpatialFrame` with U=edge tangent, V=inward normal; deck fan in frame. | `consumers/docklab/harness.ts` (157 lines). | Exercised by `tests/model/multi-topology-consumers.test.ts`, `multi-consumer-end-to-end.test.ts`: VERIFIED_NOW (see 3). |
| H8 | SPJ deck harness (in this repo) | Catalog, checkout, branding, hooks = SPJ-owned, absent. | Viewport-derived metrics, centered fan, reorder hysteresis, playzone world frames. | `consumers/spj-deck/harness.ts` (116 lines); donor receipt `docs/provenance/donor-spj-e7595ce.md` (rev `e7595cea...`). | Same tests: VERIFIED_NOW. |

## 3. Separation gate: productive intents vs scene geometry

Verified now:
- `src/topology/handoff.ts` contains no occurrence (case-insensitive grep) of minimize, restore, capture, window, dock, deck or workspace. The engine vocabulary is `subjectId`, `sequence`, rect geometry, `generation`.
- In the vendored snapshot at `7ae403c`, grep for minimiz/restore/capture/runtimewindow/dock/workspace in `vendor/in.gnu6/src` hits only a pre-existing doc comment in `scene/engine.ts` ("does not know what a Dock, menu or window is") and an unrelated math comment in `contour.ts`. No product intent leaked into the vendored engine.
- Product-to-sequence mapping lives only in the consumer (`createWindowMorphIntent`: capture -> `reshape-then-translate`, restore -> `translate-then-reshape`, detach -> `direct`). The consumer also owns `outgoingAllocation`/`incomingAllocation`, `visualOwner`, `reducedMotionEndpoint`, the queue and the commit gate.
- Deck presentation goes through a different adapter and different engine module (`deck/deck.ts`, `spatial/frame.ts`), not through `handoff.ts`. Handoff geometry and deck layout are independent paths; neither adapter imports the other (adapter import lists read).

Fresh engine-side run (this worker): exported `git archive HEAD` of the A03 worktree to a scratch dir, linked the existing `in.gnu6/node_modules`, ran
`npx vitest run tests/laws/rect-handoff-geometry-laws.test.ts tests/laws/rect-handoff-retarget-laws.test.ts tests/model/multi-consumer-end-to-end.test.ts tests/model/deck-frame-consumers.test.ts tests/model/multi-topology-consumers.test.ts`
Result: 5 files, 14 tests, all PASS. `src/` at base HEAD is identical to `2602d30` (`git diff --stat 2602d30 HEAD -- src` empty), so this is the same engine the consumer vendors. This is a new run of engine laws; it says nothing about the consumer.

Historical, not re-run: IN-01 (consumer full Vitest 352 PASS + 3 skipped, Chromium 68/68), IN-02 (engine 32 files / 129 tests, consumer 357 PASS + 3 skips, Experience Chromium 69/69, deployed causal 7/7, equal-generation mutation RED then restored GREEN, four consumer RED findings preserved). All HISTORICAL_RECEIPT; the consumer suites were not executed here.

## 4. Gaps (exact)

G1. No fresh consumer-side run. `npm run in-gnu6:check`, consumer unit tests and browser matrix were not run (consumer worktree dirty on another branch; running there would risk mutating it). Status NOT_RUN.
G2. Live evidence is local-only and historical: `gnu6.local/work` image `gnu6-live-experience:local-7ae403c9423e` was not probed now. Whether it still runs: UNKNOWN. The receipt notes a transient portal HTTP 404 on router swap, cause NOT_PROVEN; zero-downtime routing not qualified.
G3. Consumer npm audit 19 findings (6 moderate, 12 high, 1 critical), per IN-02 receipt. Out of scope; no security claim.
G4. Minimize naming: engine receipts call the operations Capture/Restore; "Minimize" is the product name. Only the IN-01 live output ties them together (mode after capture = minimized). Fine, but the engine repo holds no machine-readable mapping, by design (consumer policy).
G5. Detach/`direct` and Deck stack pagination have no live receipt in the evidence read. UNKNOWN.
G6. SPJ has no real consumer: no vendored snapshot, no sync contract, no app integration in this repo's evidence; only a donor receipt and a harness. SPJ product status in `spjoaillerie-grimoire`: not inspected, UNKNOWN. Any statement that SPJ "consumes" the engine would be unsupported beyond the harness.
G7. Docklab: same - the in-repo harness is a donor translation (the gnu6.live `dock-lab` code is the real product-side Dock, vendored via `gnu6-live`, but its relation to the original Docklab donor repo was not inspected). The tag `experience-v6.8.0` covers gnu6.live Experience only.
G8. Tag target vs receipts: the tag tree lacks the local-live receipt (added at `c7b2a1b`). Anyone citing "tag contains the live receipt" would be wrong.
G9. Aggregate digest `d78894...` and consumer in-repo tests not recomputed (see section 1).

## 5. Optional Product/Integration Slice proposal (not started, not authorized)

No new consumer need was observed, so no slice is required. If the operator wants one, the only need grounded in the evidence is G6:

PROPOSAL (HYPOTHESIS): `SPJ-IN-01` - give SPJ a revision-bound consumption contract equal to SQ-009 (explicit revision sync, byte manifest, adapter-only imports), triggered only by a concrete SPJ product problem. Exit gate: SPJ-side check goes RED on a one-line mutation of vendored source and GREEN after explicit resync; engine unchanged unless a generic gap is shown. It would mutate another repo and so needs operator authorization; it is not automatically mandatory and does not alter MQ-15.

## 6. Non-claims

- No claim that gnu6.live, Docklab or SPJ currently pass anything; consumer suites were not run.
- No claim about deployment health, security, or production readiness.
- No vendored update, deployment, or edit in gnu6.live/SPJ occurred. No side quest is made mandatory. Public API stays undecided; no publication.
- Passing harness/engine tests does not prove product behavior; `exit 0` is not product success.

## 7. Commands run

`git status -sb`, `git log`, `git cat-file -t`, `git merge-base --is-ancestor`, `git diff --stat 68371fe 2602d30`, `git branch --contains`;
read-only `git -C gnu6-live` (`rev-parse`, `tag -l`, `show <rev>:<path>`, `grep <rev>`, `ls-tree`, `branch --contains`, `log`);
loop of `git show | sha256sum` comparing 22 engine vs vendored files; `git archive HEAD | tar -x` into the session scratchpad plus `npx vitest run` on 5 test files (14/14 PASS).
