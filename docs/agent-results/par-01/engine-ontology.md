# A01 — Engine architecture / ontology atlas

Worker: A01 · batch IN-GNU6-PAR-01 · base `c9dcdab` · package `@gnu6/in 0.0.0-private`
Scope: read-only semantic atlas of `src/**` (22 files, 5703 lines) against MQ-01..MQ-10 in `QUESTLOG.md`. No core TS touched. No public-API qualification.

## 0. Truth classes and instrument limits

| Class | Meaning in this document |
|---|---|
| `VERIFIED_NOW` | Established by reading source at `c9dcdab` in this worktree (a **source** fact, not a behaviour fact). |
| `HISTORICAL_RECEIPT` | Stated by `QUESTLOG.md` / evidence receipts as PASS at an earlier SHA; not re-measured here. |
| `HYPOTHESIS` | Inference from source reading; falsifier named. |
| `NOT_RUN` | Not executed. **No test, typecheck or build was run**: the worktree has no `node_modules` (no `npm ci` was performed; it is a build-environment mutation not needed for a source atlas). Every "oracle" below is therefore *existing and located*, not *re-run*. |

Version convention: `symbol@c9dcdab` = the symbol as it exists in `path` at commit `c9dcdab`. Test cites are `file :: it(...)` titles as they exist at that commit.

## 1. Module graph (import direction = "depends on")

```
algebra/types ─────────────────────────────────────────────┐  (data model: EntityId, Pose2, Shape2, SceneEntity,
   ▲                                                       │   SceneRelation, SceneState, RadialField, PoseTarget)
algebra/laws ──► dynamics/math                             │
   ▲                                                       │
scene/engine ──► algebra/laws, dynamics/math               │  (SceneEngine: sole writer of SceneState)
scene/hierarchy ─► algebra/types   scene/lifecycle ─► algebra/laws
scene/query ─► algebra/types
dynamics/clock ──► scene/engine (type-only)                │  (FixedStepClock: only caller of engine.step)
dynamics/field, dynamics/math ─► algebra/types
spatial/frame ─► algebra/types, dynamics/math              │  (SpatialFrameGraph; FrameId charts, NOT scene entities)
spatial/environment ─► algebra/types                       │  (SpatialEnvironment: regions, AABB, occlusion, pick)
spatial/constraints ─► algebra/types, spatial/environment  │  (pure solver; ConstraintId, ownerId: string)
topology/instance ─► algebra/types, spatial/frame          │  (TopologyRegistry + OwnershipRegistry)
topology/{contour,morph,deformation} ─► dynamics, spatial  │  (shape evolution, pure values)
topology/handoff ─► algebra/types                          │  (rect handoff geometry planning, pure)
deck/deck ─► dynamics/math, algebra, spatial               │  (one topology FAMILY: fan/stack/spread, order, transfer)
interaction/projection ─► spatial/{env,frame}, topology/instance
introspection/frame ─► scene/{engine,hierarchy}, spatial/constraints  (SceneReceipt: read-only, deterministic)
renderer/contract ─► algebra/types, scene/hierarchy        │  (RendererFrame; downstream-only)
renderer/continuity ─► (none)                              │  (law checker over provider traces)

Outside src (consumers of the above, not ontology): adapters/{nonvisual,svg,pixi}.ts, consumers/{docklab,spj-deck}/harness.ts,
qualification/pixi, tests/fixtures/{scene-replay,blobin-donor}.ts.
```

`VERIFIED_NOW`: no `src/**` file imports from `adapters/`, `consumers/`, `tests/`, or any renderer package. A case-insensitive grep of `src/` for `pixi|dock|react|document.|window.|canvas|webgl|joaill|workspace|gnosix|blob` returns exactly one hit: a doc comment in `scene/engine.ts` ("It does not know what a Dock, menu or window is").

Observed structural fact (central to section 5): `scene/engine.ts` does **not** import `topology/instance`. `OwnershipRegistry` is imported only by `interaction/projection.ts` (handoff) and by introspection as plain data (`OwnershipObservation`). Scene truth and authority truth are two separate stores joined only by convention/consumer code.

## 2. Where does each concept live?

| Concept | Home | Notes |
|---|---|---|
| Identity | `EntityId` brand + `asEntityId` (`algebra/laws.ts`); `SceneEngine.mutate` rejects duplicate id | Branded string; the brand is compile-time only. Other id spaces are separate brands: `FrameId`, `TopologyId`, `RegionId`, `ConstraintId`. `ownerId` in constraints and `requestId`/`providerId`/`pointerId` are plain `string`. |
| Hierarchy | `SceneEntity.parent`; `HIERARCHY_IS_ACYCLIC` in `inspectSceneLaws`; `worldPose`/`composePose` in `scene/hierarchy.ts` | Acyclicity is checked in **two** places (laws and `assertAcyclicHierarchy`) plus a third in `SpatialFrameGraph.assertAcyclic` for frames. Scene hierarchy and spatial-frame hierarchy are distinct graphs. |
| Relation | `SceneState.relations` (`attached` \| `ext:<name>`), `RELATIONS_ARE_SET`, `RELATION_ENDPOINTS_EXIST` | Derived facts (occlusion, containment, proximity) are intentionally not stored relations; they are computed in `spatial/environment` and surfaced in `SceneReceipt`. |
| Time | `SceneState.time` advanced only by `SceneEngine.step`; `FixedStepClock` (`dynamics/clock.ts`) | `step` clamps dt to 1/15; clock quantizes and records dropped time. Two clamps with different constants (see H-5). |
| Constraints | `spatial/constraints.ts :: solveSpatialConstraints` (6 kinds); `TopologyConstraint {id,kind}` declared in `topology/instance.ts` | **Declared** topology constraints and **solved** spatial constraints are not linked by type: a `TopologyConstraint` has no reference to a `SpatialConstraint`. |
| Simultaneous topologies | `TopologyRegistry.add` (transactional claims) + `OwnershipRegistry` (`owner : E×Q ⇀ T`) | Membership (`members`) is separated from authority (`claims`). |
| Scenes | `SceneEngine` (one mutable holder of `SceneState`); `SceneState` is the value | There is no scene identity type above `SceneState`; one engine = one scene. "Scene identity is renderer-independent" (MQ-01 inv. 2) is about entity identity. |
| Handoffs | `interaction/projection.ts :: createTopologyHandoffRequest`, `TopologyHandoffAuthority.execute`; `topology/handoff.ts` (rect geometry plan only) | Two different meanings of "handoff": **authority** handoff (ownership transfer) vs **geometry** handoff (reshape/translate plan). They share no types. |
| Deck family | `deck/deck.ts` | A topology *family* (layout + order + occlusion + transfer). It produces targets (`deckPoseTargets`); it does not own anything by itself. |
| Observation | `introspection/frame.ts :: buildSceneReceipt` / `serializeSceneReceipt` / `diffSceneReceipts` | Read-only by documented contract (header lists excluded mutators). |
| Projection to providers | `renderer/contract.ts :: buildRendererFrame`, `validateRendererResult`; `renderer/continuity.ts` | Frame is `deepFreeze`d; identity accounting (emit-or-omit-with-reason) enforced on results. |

## 3. Responsibility boundaries (who may write what)

```
product/consumer ──proposes──► PoseTarget / SceneMutation / TopologyInstance / SpatialConstraint[] / HandoffRequest
                                   │
                  ┌────────────────┴──────────────── engine-owned writers ───────────────┐
                  │ SceneEngine.{mutate,setTarget,clearTarget,setShape,settleTarget,step}│ -> SceneState (laws asserted on every commit)
                  │ SpatialFrameGraph.{add,update,reparent}                              │ -> frames
                  │ SpatialEnvironment.{addRegion,removeRegion}                          │ -> regions (+revision)
                  │ TopologyRegistry.{add,remove} / OwnershipRegistry.{claim,release,transfer} │ -> authority
                  └──────────────────────────────────────────────────────────────────────┘
provider/renderer ──reads──► RendererFrame (frozen) ; ──emits──► RendererPointerObservation (facts, no authority)
                              InteractionProjector.project: provider coords -> world -> exact subjectId (no mutation reach)
```

Boundary claims and status are in section 4. The key structural gap is that the "ownership gate" is **advisory at the engine level** (see H-1).

## 4. Assertion table: `assertion → symbol@version → oracle/test → counter-example → class`

Counter-example column = the concrete mutation or input that would have to be accepted (or rejected) for the assertion to be false, i.e. the falsifier. "causal mutation" entries are `HISTORICAL_RECEIPT` from `QUESTLOG.md` (not reproduced here).

| # | Assertion | Symbol@c9dcdab | Oracle / existing test | Counter-example (falsifier) | Class |
|---|---|---|---|---|---|
| A1 | Entity ids are unique; duplicate add rejected | `SceneEngine.mutate` (`scene/engine.ts`) | Not directly named in `tests/laws`; indirectly exercised by `topology-engine.test.ts`. No dedicated "duplicate identity" test located. | `mutate({add:[e,e]})` or add of an existing id must throw `Duplicate scene identity` | Source: `VERIFIED_NOW`; test: **gap** (see G-1) |
| A2 | `order` is an injective permutation of the entity set | `inspectSceneLaws` laws `ORDER_IS_INJECTIVE`, `ORDER_IS_PERMUTATION_OF_ENTITIES` | `scene-admissibility-laws`, `topology-engine.test.ts` (uses `inspectSceneLaws`) — specific order-violation test not located by title | state with duplicate/missing id in `order` must yield the named violation | `VERIFIED_NOW` (source); behaviour `NOT_RUN` |
| A3 | Parent graph is acyclic, enforced at admission | `inspectSceneLaws`/`HIERARCHY_IS_ACYCLIC`; `assertAcyclicHierarchy` | `scene-admissibility-laws :: rejects a parent cycle at scene admission rather than only during world-pose query` | cyclic parent state accepted by `new SceneEngine` or `mutate`. Causal mutation (remove check) = 1 RED | `HISTORICAL_RECEIPT` (MQ-01); `NOT_RUN` now |
| A4 | Relations form a set; kinds explicit; endpoints exist | `inspectSceneLaws`/`RELATIONS_ARE_SET`, `RELATION_KIND_IS_EXPLICIT`, `RELATION_ENDPOINTS_EXIST` | `scene-admissibility-laws :: treats scene relations as a mathematical set…`, `:: rejects unnamed relation semantics…` | duplicate (kind,from,to) accepted; `""` or `"ext:"` kind accepted. Endpoint-missing law has **no located dedicated test** | A4a/b `HISTORICAL_RECEIPT`; endpoint law: test gap (G-2) |
| A5 | Entity-set queries obey set algebra; relation output canonically ordered | `queryEntityIds`, `queryRelations` (`scene/query.ts`) | `scene-query-laws :: implements set intersection, union and complement…`, `:: queries hierarchy and relation incidence…`, `:: returns relations in canonical deterministic order` | `A∪¬A ≠ E` for some state; order dependent on insertion order | `HISTORICAL_RECEIPT`; `NOT_RUN` |
| A6 | Lifecycle = disjoint birth/persistence/termination; failed mutation is transactional; authority transfer is neither birth nor termination | `diffSceneLifecycle` (`scene/lifecycle.ts`), `SceneEngine.mutate` | `scene-query-laws :: classifies birth, persistence and termination as disjoint set partitions`, `:: treats authority transfer over a persistent subject as neither birth nor termination`, `:: keeps a failed lifecycle mutation fully transactional` | an id in two partitions; engine state changed after a thrown `mutate` | `HISTORICAL_RECEIPT`. Note: `mutate` builds copies and assigns `this.state` only after `assertSceneLaws`, so transactionality holds by construction (`VERIFIED_NOW`, source). |
| A7 | Scene time is renderer-cadence independent; long pause bounded and recorded | `FixedStepClock.advance`, `SceneEngine.step` | `deterministic-time-laws :: produces the same scene state under different renderer cadences`, `:: bounds work after a long wall-clock pause…`, `:: preserves fractional fixed-step phase…`, `:: rejects invalid elapsed-time inputs…` | different cadences summing to the same elapsed produce different `SceneState` | `HISTORICAL_RECEIPT`; `NOT_RUN` |
| A8 | Frames: invertible basis, acyclic, identity-stable under geometry update, rollback on invalid update | `SpatialFrameGraph.add/update`, `localToWorld`, `worldToLocal` | `spatial-frame-laws` (6 tests: round-trip, nested, singular basis, cycle, identity-stable update, rollback) | singular basis accepted; failed update visible in `snapshot()` | `HISTORICAL_RECEIPT`; `NOT_RUN`. **Exception**: un-parenting, see H-2 |
| A9 | `\|owner(e,q)\| ≤ 1`; claims atomic; transfer atomic; membership ≠ authority | `OwnershipRegistry.claim/release/transfer`, `TopologyRegistry.add/observe` | `topology-ownership-laws` (7 tests incl. `rejects silent double ownership…`, `rolls back earlier claims when a later claim conflicts`, `defines topology membership independently from its ownership claims`) | two topologies hold same (e,q); partial claims left after failed `add` | `HISTORICAL_RECEIPT`; `NOT_RUN` |
| A10 | Projection/request creation cannot mutate authority; handoff only on allowlisted route, bounded degrees, current owner = `from` | `InteractionProjector.project`, `createTopologyHandoffRequest`, `TopologyHandoffAuthority.execute` | `interaction-handoff-laws` (5), `interaction-projection-laws` (4: equivalence world/local, topmost identity vs provenance, monotonic sequence) | transfer succeeds with degree outside allowlist; request built from a miss (`subjectId == null`) | `HISTORICAL_RECEIPT`; `NOT_RUN` |
| A11 | Placement/constraint solving is deterministic, evaluates each declared constraint exactly once, rejects incompatible sets instead of dropping one | `SpatialEnvironment.*`, `solveSpatialConstraints` | `environment-placement-laws` (5), `constraint-solver-laws` (6: every constraint once, incompatible rejected, peer overlap, topology-admissibility explicit, perturbation stability, duplicate ids rejected) | `evaluations.length ≠ constraints.length`; one of two conflicting constraints silently dropped | `HISTORICAL_RECEIPT`; `NOT_RUN` |
| A12 | Shape evolution preserves identity and live node state across retarget; fields do not mutate source | `retargetDeformableContour`, `stepDeformableContour`, `deformContourByFields`, `SceneEngine.setShape` | `deformable-contour-laws` (4), `topology-engine.test.ts`/`topology-hierarchy-morph.test.ts` for `setShape` | retarget resets node velocity; `setShape` changes id/order/pose | `HISTORICAL_RECEIPT`; `NOT_RUN` |
| A13 | Deck family: deterministic layout; permutation-preserving order edits; one persistent identity transfers between decks; lift direction is consumer geometry | `layoutTarotFan/CardStack/CardSpread`, `planDeckOrderTransition`, `transferDeckCard`, `deckOcclusionPairs` | `deck-topology-laws` (8) | surviving ids change on insert/remove; hardcoded `-Y` lift (MQ-10 found/repaired, causal mutation = 1 RED) | `HISTORICAL_RECEIPT`; `NOT_RUN` |
| A14 | SceneReceipt is deterministic, insertion-order independent, detached, renderer-concept free | `buildSceneReceipt`, `serializeSceneReceipt`, `diffSceneReceipts` | `introspection-frame-laws` (3 tests) | serialization differs under permuted observation arrays; mutating a receipt mutates the scene | `HISTORICAL_RECEIPT`; `NOT_RUN` |
| A15 | RendererFrame is detached/frozen, carries no authority; adapters cannot invent, duplicate or silently drop identities; cadence/motion mode are not semantic | `buildRendererFrame`, `validateRendererResult`, `rendererSemanticDigest` | `renderer-contract-laws` (4), `renderer-adapter-equivalence.test.ts` (model) | adapter result with unknown id passes validation | `HISTORICAL_RECEIPT`; `NOT_RUN` |
| A16 | Provider handle/visibility/alpha continuity: no cut-off, no resurrection | `inspectRendererContinuity` | `renderer-continuity-laws` (6), `tests/browser/pixi-reference.pw.ts` | handle replacement under surviving identity not reported | `HISTORICAL_RECEIPT` (Playwright 2/2 per MQ-15); `NOT_RUN` |
| A17 | Core contains no consumer/provider names or imports | all of `src/**` | grep run in this session (section 1); also MQ-10 "no consumer-specific names/branches under `src/`" | any `import` from `adapters/` or `consumers/` in `src/` | `VERIFIED_NOW` for the grep and for import direction. This is lexical evidence only, see section 5. |

## 5. Concepts that providers or products could illegally bring in (leak surface)

`VERIFIED_NOW` lexical result: no provider/product *names* in `src/`. The risk is in **vocabulary baked into types**, not names:

| # | Candidate leak | Where | Why it matters | Class |
|---|---|---|---|---|
| L-1 | `SceneRole = "card" \| "surface" \| "control" \| "proxy" \| "anchor" \| "decoration"` is a closed union in the core data model | `algebra/types.ts` | `card` and `control` are product-family vocabulary (deck / UI). A new consumer's role needs a core type change; roles behave as data (no `src/` branches on role were seen in files read, but see H-4). | Source `VERIFIED_NOW`; "is a leak" = `HYPOTHESIS` |
| L-2 | The deck family (`deck/deck.ts`, 545 lines, "card", "tarot", "fan", "playzone"-adjacent) lives in core `src/` | `deck/deck.ts` | CONTRACT section 2 says the engine knows no product notions. The deck is a *topology family* (MQ-06, ratified) and `SpatialFrame`-based; "tarot" is a donor-flavoured name. Whether it is core or consumer-library is an operator question; not decided here. | Source `VERIFIED_NOW`; classification open (`HYPOTHESIS`) |
| L-3 | Pixel-ish / DOM-ish assumptions: `RendererAccessibility.role: string` accepts arbitrary ARIA-like text | `renderer/contract.ts` | Provider-side vocabulary enters the frame as free text; it is projection metadata (test `treats cadence … as projection metadata`), but not enumerated. | `HYPOTHESIS` (low severity) |
| L-4 | `providerId`, `pointerId`, `PointerPhase = down\|move\|up\|cancel` | `interaction/projection.ts` | Pointer-event vocabulary is provider-shaped, kept as provenance. MQ-08 tests assert semantic output equal across providers. | `HISTORICAL_RECEIPT` that it is contained |
| L-5 | `DegreeOfFreedom` is a closed 6-element set (`position.x/y`, `rotation`, `scale.x/y`, `shape`) | `topology/instance.ts` | Ownership cannot express non-pose DoFs (e.g. `z`, `visible`, `role`, `parent`, relations). Those mutate via `SceneEngine.mutate` with no ownership at all (see H-1). | Source `VERIFIED_NOW`; consequence `HYPOTHESIS` |
| L-6 | `TopologyAdmissibilityConstraint.factId: string` + `topologyFacts: Map<string,boolean>` | `spatial/constraints.ts` | Opaque string facts are a consumer-defined side channel into the solver. By design (MQ-07 "explicit instead of hidden in consumer code"), yet the fact's *provenance* is not typed. | `HYPOTHESIS` |
| L-7 | `SettleThresholds` default 0.001 in world units | `scene/engine.ts` | Unit assumption (pixel-ish magnitude) embedded as default; scale-free settle is not defined. | `HYPOTHESIS` (low) |

## 6. Refutable hypotheses (each with a falsifier; all `HYPOTHESIS` / `NOT_RUN`)

H-1 **Ownership is not enforced against scene writes.** `SceneEngine.setTarget/mutate/setShape/settleTarget/step` never consult an `OwnershipRegistry` (no import; grep of `src/` shows registry use only in `interaction/` and as types in `introspection/`). Hence "`|Owner(e,q)| ≤ 1`" (comment on `PoseTarget`) constrains *claims*, not *who may write the pose*. Two topologies can both submit `PoseTarget`s for the same entity; the last `setTarget` wins regardless of ownership. Falsifier: a test that claims `position.x` for topology A, then lets topology B `setTarget` the same entity, and asserts rejection. If it passes at HEAD, H-1 is FALSIFIED. (A mediator may exist in `consumers/*/harness.ts`; not read in depth here. That would make the guarantee consumer-enforced, i.e. exactly the situation CONTRACT section 5 wants to avoid.)

H-2 **`SpatialFrameGraph.reparent(id, null)` cannot detach a frame.** `update` computes `parent: patch.parent ?? current.parent`; `null ?? x` evaluates to `x`, so `{parent: null}` keeps the old parent. The signature `reparent(id, parent: FrameId | null)` advertises un-parenting. The only test (`spatial-frame-laws`) reparents to a non-null id to provoke a cycle. Falsifier: `add(root); add(child parent root); reparent(child, null); expect(snapshot.get(child).parent).toBeNull()`. Failing => confirmed defect (candidate for operator-prioritised Side Quest; this atlas does not fix it).

H-3 **`TopologyRegistry.add` does not verify that members/claim subjects exist in any scene.** It validates frame existence and claim-subject-in-members, but not `members ⊆ E`. `TopologyRegistry` and `SceneEngine` are not linked, so removing an entity from the scene leaves dangling members/claims; `OwnershipRegistry.transfer` also does not require `to` to be a registered topology, nor update `TopologyInstance.claims` (documented: declaration vs authority). Falsifier: add a topology claiming an id absent from scene state and expect `add` to throw; or remove an entity and expect registry cleanup.

H-4 **Role-based branching.** `SceneRole` is data in the files read, but no exhaustive-switch/lint gate prevents core code from branching on `"card"`. Falsifier: a test or grep gate asserting no `role ===` literal comparisons in `src/`.

H-5 **Two time clamps with different semantics.** `SceneEngine.step` clamps dt at 1/15 s and silently discards the excess (no record); `FixedStepClock` bounds by `maxStepsPerAdvance` and records dropped steps. If a consumer calls `step` directly with a large dt, time is lost unobserved, contradicting the "dropped time is observable" claim of the clock. Falsifier: `engine.step(10)` and check whether `state.time` advance is 1/15 and whether any observation records the loss.

H-6 **`SceneState.fields` and `z`/`visible` are outside admissibility laws.** `inspectSceneLaws` checks pose finiteness but not `z`, shape validity (`setShape` accepts any `Shape2` without a finiteness/positivity check in the code read), nor `RadialField` parameters (`radius > 0`, finite). Falsifier: feed `NaN` z, zero-radius field, polygon with < 3 vertices, expect admission rejection.

H-7 **Handoff `subjectId` is not checked against the scene or topology membership.** `TopologyHandoffAuthority.execute` checks route, degrees and current owner, not that `request.from` is a registered topology or that the subject is a current scene entity. Mitigated indirectly: `OwnershipRegistry.transfer` requires current owner = `from`. Residual risk is limited to stale ownership entries (see H-3).

H-8 **Three duplicated acyclicity walkers** (`inspectSceneLaws`, `assertAcyclicHierarchy`, `SpatialFrameGraph.assertAcyclic`) may drift. Falsifier: property test generating random parent maps and comparing both scene checks' verdicts.

## 7. Invariants lacking reproducible falsification (priority-ordered, recommendation only)

Priority belongs to the operator (`QUESTLOG.md` Side Quest rule); these are *discovered* gaps, not promoted scope.

| Gap | Missing oracle | Linked hypothesis |
|---|---|---|
| G-1 | Dedicated duplicate-identity test for `mutate` (A1) | none |
| G-2 | Dedicated `RELATION_ENDPOINTS_EXIST` and `TARGET_HAS_EXACT_ENTITY_OWNER` tests (A4) | none |
| G-3 | Ownership-vs-write gate test (does anything stop a non-owner writing a pose?) | H-1 |
| G-4 | Un-parenting of frames | H-2 |
| G-5 | Registry/scene referential integrity (members, claims, `to` topology) | H-3, H-7 |
| G-6 | Admissibility of `z`, shape and field parameters | H-6 |
| G-7 | Direct `step` large-dt time-loss observation | H-5 |
| G-8 | Static boundary gate: no consumer/provider imports under `src/`, no `role ===` literals (currently only a human-run grep, A17) | H-4, L-1 |
| G-9 | Two meanings of "handoff" (authority vs rect geometry) have no shared type or naming gate | n/a (naming hazard) |

## 8. What this document does not claim

- It does not re-run, and does not re-qualify, any MQ-01..MQ-15 test, build or browser gate (all `NOT_RUN`).
- It does not claim H-1..H-8 are defects; each is a hypothesis with a stated falsifier. H-2 is the strongest because it follows from a single expression, but it is still unexecuted.
- It does not decide whether `deck/` or `SceneRole` belong in core, does not propose a public API, and does not modify `QUESTLOG.md`/`CONTRACT.md`.
- Source-read statements establish what the code says, not what any live consumer does. `consumers/*/harness.ts`, `adapters/*`, `tests/model/*` bodies and `benchmarks/` were not read in depth.
- Test-title lists were obtained from `it(...)` titles by grep, not from execution; counts per file are counts of titles.

## 9. Next decision (for the integrator / operator)

1. Whether to run the qualified targeted gate (`npm ci` then `vitest run tests/laws`) to turn the `HISTORICAL_RECEIPT` rows into `VERIFIED_NOW`; `npm ci` mutates the worktree only, but is a build-environment step I did not take.
2. Whether H-1 (ownership not enforced at the engine) and H-2 (frame un-parenting) warrant falsification tests first, as Side Quests under operator priority.
3. Whether `deck/` and `SceneRole` vocabulary (L-1, L-2) is a deliberate core family or a candidate for relocation (an authority question, not an engineering default).
