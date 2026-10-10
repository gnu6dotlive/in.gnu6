# [in.gnu6] — Questlog

Status: **ACTIVE PROGRAM AUTHORITY**
Repository: `tension-atoi/in.gnu6` — private as of 2026-10-10; public visibility authorized, disclosure gates pending
Initial questline ratification: **2026-10-06**
Operator authority: **human operator retains final prioritization of Side Quests and all authority-expanding decisions**

## Purpose

This is not a conventional backlog.

The Questlog is the durable map of what must become true before `[in.gnu6]` can honestly be evaluated as complete for its current mandate.

### Main Questline

The Main Questline is the **minimum completeness contract**.

It does **not** mean:
- shortest path;
- easiest implementation;
- fastest release;
- smallest amount of code;
- minimum viable product.

It means:

> the minimum set of invariants, gates, falsifications, consumer proofs and runtime observations that must hold before the current engine mandate may be called complete.

A Main Quest may expand when reality reveals a missing prerequisite. Expansion is not scope failure if the new requirement is causally necessary to the completeness claim.

### Side Quests

Side Quests are optional work discovered along the way.

Examples:
- clean artifacts left outside repositories;
- normalize donor provenance;
- build or update an external toolchain;
- improve documentation;
- repair historical falsehoods;
- clean stale naming;
- create better debug tooling;
- reduce test harness friction;
- benchmark an alternate renderer;
- investigate a donor project.

Side Quests may be valuable, urgent or strategically important, but they are **not automatically promoted into the Main Questline**.

**Side Quest prioritization belongs to the operator.**

Agents may discover, classify, estimate causal value and recommend priority. Agents may not silently convert optional work into mandatory program scope.

**The main quest defines what “complete” means. Side quests preserve what was learned along the way.**

**Optional does not mean unimportant; required does not mean urgent. Priority belongs to the operator.**

## Quest state model

Every quest uses one of these states:

- `UNKNOWN`
- `DISCOVERED`
- `DISCOVERED_BLOCKER_CANDIDATE`
- `HYPOTHESIS`
- `DESIGNED`
- `IMPLEMENTING`
- `RED`
- `GREEN_LOCAL`
- `OBSERVED`
- `INVARIANT_CANDIDATE`
- `RATIFIED`
- `BLOCKED_AUTHORITY`
- `NEEDS_REQUALIFICATION`
- `DEFERRED_BY_OPERATOR`
- `CLOSED`

These are not a linear ladder in every case.

`GREEN_LOCAL` does not imply `OBSERVED`.
`OBSERVED` does not imply `RATIFIED`.
`CLOSED` means the quest's explicit exit gate is satisfied or the operator explicitly terminated it.

# Main Questline — current mandate

Current mandate:

> Build `[in.gnu6]` as one headless, environment/space-aware scenic, spatial and topological runtime that can support multiple concurrent topology systems, multiple products and replaceable renderers without duplicating semantic engines.

The engine owns scene truth and visual/spatial evolution. It does not own consumer business semantics.

## MQ-00 — Establish the autonomous engine project

State: **RATIFIED**

### Required truth
- autonomous Git repository exists;
- repository is private;
- public API remains undecided;
- publication remains unauthorized;
- governing contracts are durable;
- original seed provenance is reconstructible;
- semantic seed passes in the autonomous repository.

### Evidence
- `67f4d27 feat: bootstrap in.gnu6 scenic topology runtime`
- `4042faa chore: harden in.gnu6 bootstrap gates`
- `docs/provenance/seed-b07be20.md`
- `docs/evidence/bootstrap-2026-10-06.md`
- 18/18 seed laws PASS.

### Exit gate
Satisfied.

## MQ-01 — Formal scene ontology

State: **RATIFIED**

### Goal
Define the smallest ontology capable of describing scene truth without assuming Dock, SP Joaillerie, React, PixiJS or any other consumer/provider.

### Required invariants
1. There is exactly one semantic engine.
2. Scene identity is stable and renderer-independent.
3. Scene state is a finite attributed graph `G = (E, R)`.
4. Traversal order is a permutation of the current entity set.
5. Parent hierarchy is acyclic.
6. Product identity and renderer identity remain distinct from scene identity.
7. Relations are explicit instead of hidden in unrelated booleans.
8. Invalid scene states are outside the admissible state set; there is no `ALLOW_PARTIAL`.

### Existing implementation
- entity identity;
- scene relations;
- hierarchy;
- order/permutation laws;
- transactional mutation;
- pose/velocity;
- shapes;
- targets.

### Ratified evidence
- explicit serializable finite-set scene query algebra;
- typed relation vocabulary with core `attached` plus explicit `ext:<name>` namespace;
- relation-set uniqueness law;
- hierarchy acyclicity enforced at scene admission;
- lifecycle set semantics for birth/persistence/termination;
- transactional lifecycle failure;
- independent degree-of-freedom ownership inherited from ratified MQ-02;
- exact deterministic observation frame inherited from ratified MQ-09;
- causal mutation removing cycle rejection produced **1 RED admissibility test**;
- restored-state gates: **29 files / 119 tests PASS**, Pixi build PASS, Playwright **2/2 PASS**.

Implementation:
`cefa6fb feat: close formal scene ontology gaps`

Receipt:
`docs/evidence/mq-01-formal-scene-ontology-2026-10-07.md`

### Exit gate
A deterministic law suite can generate, mutate, query and reject scene graphs without consumer concepts or renderer dependencies.

**Disposition: RATIFIED.**

## MQ-02 — Spatial frames and concurrent topology instances

State: **RATIFIED**

### Ratified principle
Different topology systems may coexist in the same World.

Docklab's edge-relative topology and SP Joaillerie's centered tarot/playzone topology are not alternatives that force one global layout model. They may exist concurrently.

### Core model
Let:
- `W` = World;
- `E` = scene entities;
- `F_i` = spatial frames;
- `T_i` = topology instances.

Each topology instance is conceptually:

`T_i : (E_i, F_i, C_i, t) -> Targets_i`

where:
- `E_i ⊆ E`;
- `F_i` is its local spatial frame;
- `C_i` is its constraint/configuration set;
- `t` is scene time;
- `Targets_i` are proposed scene targets.

### SpatialFrame minimum
Each frame must provide:
- origin `O`;
- basis vectors `u`, `v`;
- `localToWorld`;
- `worldToLocal`;
- parent frame or World attachment;
- deterministic composition.

A frame may represent viewport center, environment edge, playzone, deck, window, card or arbitrary scene region.

### TopologyInstance minimum
A topology instance must declare:
- exact member set;
- spatial frame;
- solver/topology kind;
- constraints;
- owned degrees of freedom;
- current authority state;
- deterministic targets or observations.

### Ownership law
For entity `e` and degree of freedom `q`:

`|Owner(e, q)| <= 1`

Concurrent topologies are allowed when their ownership domains are disjoint.

### Handoff law
A subject may move between topology authorities without changing identity.

Example:

`TarotHand -> Drag -> Playzone`

The subject remains the same entity throughout.

### Composition law
Topologies may be nested.

Example:

`EnvironmentEdgeFrame -> TarotFanTopology`

Therefore a Dock may be edge-attached while internally behaving like a hand of cards.

### Required falsifications
- Docklab edge frame + tarot fan simultaneously;
- SPJ viewport fan + playzone simultaneously;
- nested topology composition;
- two independent topology instances in one World;
- illegal double-ownership must RED;
- legal disjoint ownership must GREEN;
- topology handoff must preserve identity and velocity where applicable.

### Evidence
- implementation: `40edfa1 feat: add composable spatial frames and topology ownership`;
- receipt: `docs/evidence/mq-02-spatial-topology-2026-10-06.md`;
- full restored-state gate: **6 files / 32 tests PASS**;
- causal mutation: disabling double-ownership protection produced **2 RED ownership tests**, then byte-exact restoration PASS.

### Exit gate
The engine can host at least two materially different topology systems concurrently without consumer-specific branching in the core.

**Disposition: RATIFIED.**

## MQ-03 — Deterministic scene time and dynamics

State: **RATIFIED**

### Required invariants
1. Scene time is explicit.
2. Wall-clock pauses cannot inject unbounded simulation steps.
3. Fixed-step or equivalently deterministic integration must be available.
4. Springs preserve continuous velocity.
5. Retargeting starts from current state, not stale endpoints.
6. Reversal preserves meaningful current velocity.
7. One owner per animated degree of freedom.
8. Reduced-motion changes traversal, not semantic destination.

### Existing implementation
- damped spring law;
- semi-implicit Euler integration;
- bounded `dt`;
- pose velocity;
- shape-morph progress spring;
- continuous radial fields.

### Ratified evidence
- fixed-step scheduler with bounded catch-up;
- renderer-cadence independence proof;
- retarget/reversal velocity preservation;
- topology handoff velocity preservation inherited from MQ-02;
- explicit settled predicate;
- reduced-motion destination parity;
- deterministic semantic replay fixture;
- causal mutation: removing the catch-up budget produced **600 executed steps instead of 6** and made the long-pause gate RED;
- restored-state gates: **9 files / 42 tests PASS**, format/lint/typecheck PASS.

Receipt:
`docs/evidence/mq-03-deterministic-time-2026-10-06.md`

Implementation:
`236ed78 feat: add deterministic scene time and dynamics gates`

### Exit gate
Identical ordered inputs + identical initial state produce identical scene observations independent of renderer frame cadence.

**Disposition: RATIFIED.**

## MQ-04 — Environment / space model

State: **RATIFIED**

### Required model
World `W` may contain:
- obstacles `O ⊆ W`;
- safe regions `S ⊆ W`;
- containment `C ⊆ E × E`;
- occlusion `Ω ⊆ E × E`;
- proximity metric `d : E × E -> R+`;
- interaction fields `F`;
- viewport/work-area boundaries;
- local spatial frames.

### Required capabilities
- bounds queries;
- containment;
- exclusion;
- collision or non-penetration where required;
- nearest admissible region;
- occlusion ordering;
- safe placement;
- renderer-neutral hit/pick query;
- environment mutation with deterministic reflow.

### Ratified evidence
- implementation: `fd6af22 feat: add deterministic environment space model`;
- receipt: `docs/evidence/mq-04-environment-space-2026-10-06.md`;
- AABB admissibility/exclusion/containment/proximity/occlusion;
- nearest-region and renderer-neutral geometric pick primitives;
- shared context-like / deck-like / playzone-like placement proof;
- environment mutation reflow proof;
- causal mutation ignoring exclusions produced **3 RED tests**;
- restored-state gates: **12 files / 53 tests PASS**, format/lint/typecheck PASS.

Scope note:
MQ-04 ratifies the current world-space AABB environment model. Arbitrary polygonal/generalized constraint solving remains MQ-07.

### Exit gate
A context surface, deck and playzone can all ask the same engine for admissible spatial solutions without encoding duplicated `near edge` consumer logic.

**Disposition: RATIFIED.**

## MQ-05 — Topology and shape evolution

State: **RATIFIED**

### Required invariants
1. Shape connectivity is represented explicitly.
2. Closed contours have deterministic winding.
3. Morph endpoints are truthful.
4. Different vertex counts can be normalized.
5. Correspondence is deterministic.
6. Morph does not change semantic identity.
7. Shape retargeting does not cut off.
8. Deformation fields can coexist with affine pose evolution when ownership is disjoint.

### Existing implementation
- polygon topology;
- winding normalization;
- arc-length resampling;
- cyclic alignment;
- contour interpolation;
- rounded-rect contour;
- window→card compatible contour generation.

### Headless evidence complete
- per-node contour position + velocity;
- topology retarget/reversal preserving live node state;
- continuous deformation fields;
- shape/environment collision observation;
- nested/local-frame contour projection;
- numerical stability gates;
- causal mutation zeroing velocity produced **2 RED tests**;
- restored-state gates: **14 files / 60 tests PASS**, format/lint/typecheck PASS;
- core donor/product-name scan: NONE.

Implementation:
`4ef5a37 feat: add deformable contour dynamics`

Receipt:
`docs/evidence/mq-05-headless-topology-2026-10-06.md`

### Closure evidence
The renderer/no-cutoff dependency was satisfied by MQ-12 and MQ-13.

Reference browser proof:
- one persistent semantic entity evolves `rounded rect -> polygon/diamond -> card-like surface`;
- same Pixi Graphics handle survives;
- shape evolution uses identity-preserving `SceneEngine.setShape`.

No-cutoff proof:
- continuity trace for the surface→card transition reports `violations = []`;
- every live sample remains mounted, visible, renderable, alpha > 0 and finite.

Fresh requalification:
- headless targeted gate: **2 files / 7 tests PASS**;
- browser no-cutoff targeted gate: **1/1 PASS**.

Closure receipt:
`docs/evidence/mq-05-topology-closure-2026-10-07.md`

### Exit gate
A single entity can continuously evolve between at least three materially different surface topologies under retargeting without identity loss or visual discontinuity in a reference renderer.

**Disposition: RATIFIED.**

## MQ-06 — Deck / card topology family

State: **RATIFIED**

### Goal
Provide generic scene mechanics capable of supporting card/tarot/deck consumers without encoding Gnosix Dock or SPJ product semantics.

### Required capabilities
- ordered persistent identity set;
- fan;
- overlap;
- stack;
- spread;
- z-order;
- hover/focus field;
- neighbor yield;
- lift;
- drag;
- reorder;
- hysteresis;
- insertion/removal;
- transfer between deck/topology regions;
- occlusion-aware stack;
- arbitrary local frame.

### Existing implementation
- tarot fan;
- overlap;
- continuous pointer field;
- hover scale/lift/yield targets;
- reorder preview;
- hysteresis.

### Ratified evidence
- generic stack + spread + tarot fan topology;
- insertion/removal transition planning;
- continuous surviving-card reflow through SceneEngine PoseTargets;
- occlusion-aware stack;
- deck-to-deck transfer preserving one persistent identity;
- arbitrary local-frame center projection;
- optional velocity-derived tilt;
- focus influence independent from pointer;
- Docklab and SPJ consume the same core deck projection primitives;
- consumer harnesses are now included in TypeScript gates;
- causal mutation duplicating transferred identity produced **1 RED test**;
- restored-state full gate: **17 files / 70 tests PASS**, format/lint/typecheck PASS.

Implementation:
`e097d46 feat: complete generic deck topology family`

Receipt:
`docs/evidence/mq-06-deck-topology-2026-10-06.md`

### Post-ratification requalification
MQ-10 exposed a hidden deck-field assumption: lift was implicitly hardcoded to local `-Y`.

The core now requires explicit `liftDirection`, allowing Docklab inward lift and SPJ upward lift with the same solver.

The old behavior was causally falsified by the unified consumer scenario.

MQ-06 remains **RATIFIED** with this assumption repaired.

### Exit gate
Docklab and SPJ deck harnesses both consume the same deck/topology primitives while producing distinct spatial behavior.

**Disposition: RATIFIED.**

## MQ-07 — Constraints, collisions, exclusion and admissibility

State: **RATIFIED**

### Required invariants
- constraints are explicit;
- incompatible constraints RED deterministically;
- solver does not silently drop constraints;
- collision/exclusion ownership is known;
- resolution is stable under small input perturbations;
- no consumer-specific emergency clipping is required for normal operation.

### Minimum capabilities
- positional constraints;
- scale/extent constraints;
- safe-region constraints;
- exclusion zones;
- card/card overlap policy;
- card/environment collision policy;
- topology-specific admissibility;
- deterministic conflict reporting.

### Ratified evidence
- explicit constraint identity + owner + typed result;
- position-range and extent-range constraints;
- safe-region and exclusion constraints;
- peer overlap policy with exact peer identity;
- topology-specific admissibility facts;
- deterministic interval-conflict reporting;
- one evaluation per declared constraint on resolved paths;
- small-perturbation stability gate;
- causal mutation silently dropping topology-admissibility produced **2 RED tests**;
- restored-state gates: **19 files / 78 tests PASS**, format/lint/typecheck PASS.

Implementation:
`c9134bb feat: add explicit spatial constraint solver`

Receipt:
`docs/evidence/mq-07-constraints-admissibility-2026-10-06.md`

### Exit gate
The engine can explain why a target is admissible, adjusted or rejected.

**Disposition: RATIFIED.**

## MQ-08 — Picking, interaction projection and authority handoff

State: **RATIFIED**

### Required capabilities
- scene-space picking;
- local/world coordinate conversion;
- exact identity hit result;
- z/occlusion aware hit ordering;
- pointer trajectory observations;
- transfer requests between topology owners;
- no authority amplification from renderer events.

### Ratified evidence
- provider observation → World projection → exact subject → semantic trajectory;
- World-space and local-SpatialFrame providers resolve the same semantic result;
- exact topmost identity with provider provenance kept separate;
- monotonic pointer sequence + trajectory observations;
- handoff request creation does not mutate ownership;
- explicit handoff route/degree allowlist;
- disallowed routes and degree amplification rejected;
- causal mutation removing degree allowlist produced **1 RED test**;
- restored-state gates: **21 files / 87 tests PASS**, format/lint/typecheck PASS.

Implementation:
`1406c15 feat: add bounded interaction projection and handoff authority`

Receipt:
`docs/evidence/mq-08-interaction-authority-2026-10-06.md`

### Exit gate
At least two renderer/harness environments can submit equivalent pointer observations and resolve the same semantic subject/interaction result.

**Disposition: RATIFIED.**

## MQ-09 — Introspection and explainable scene frames

State: **RATIFIED**

### Minimum observable facts
- scene time;
- exact entity identities;
- world/local pose;
- velocity;
- parentage;
- topology membership;
- ownership by degree of freedom;
- active target;
- constraint result;
- occlusion/containment facts;
- active fields;
- settled/sleeping state.

### Rule
Introspection observes scene truth. It must not become another authority.

### Ratified evidence
- deterministic renderer-neutral `SceneReceipt`;
- local + world pose, velocity, parentage, topology membership, ownership, target, constraints, containment, occlusion, fields and motion-state observation;
- detached ownership observation surface;
- deterministic serialization;
- exact structural receipt diff;
- receipt mutation cannot mutate scene truth;
- causal mutation substituting local pose for world pose produced **1 RED test**;
- restored-state gates: **23 files / 92 tests PASS**, format/lint/typecheck PASS.

Implementation:
`c38c661 feat: add deterministic scene introspection`

Receipt:
`docs/evidence/mq-09-introspection-2026-10-06.md`

### Exit gate
A failing consumer test can produce a compact deterministic scene receipt sufficient to explain the causal difference.

**Disposition: RATIFIED.**
## MQ-10 — Multi-consumer falsification

State: **RATIFIED**

### Required consumers

1. **Docklab harness**
   - edge-relative local frame;
   - nested card topology;
   - neighbor field;
   - reorder;
   - edge mutation.

2. **SP Joaillerie deck harness**
   - viewport-centered fan;
   - responsive consumer metrics;
   - playzone;
   - drag/reorder;
   - tarot-card behavior.

### Core invariant
Neither harness may fork:
- identity law;
- dynamics law;
- topology ownership law;
- scene clock;
- environment model.

### Stronger proof
Both consumers coexist in one synthetic World fixture using one:
- SceneEngine;
- FixedStepClock;
- SpatialFrameGraph;
- SpatialEnvironment;
- TopologyRegistry / OwnershipRegistry;
- SceneReceipt surface.

### Ratified evidence
- Docklab stable edge-frame mutation;
- Docklab local neighbor field + reorder;
- SPJ responsive consumer metrics + reorder;
- SPJ hand→playzone bounded authority transfer;
- one shared semantic clock across both consumer families;
- final deterministic SceneReceipt observes both families;
- no consumer-specific names/branches under `src/`;
- MQ-10 discovered and repaired the prior hardcoded `-Y` lift assumption;
- causal mutation restoring hardcoded `-Y` produced **1 RED end-to-end test**;
- restored-state gates: **24 files / 96 tests PASS**, format/lint/typecheck PASS.

Implementation:
`3d85cd0 feat: prove one-core multi-consumer integration`

Receipt:
`docs/evidence/mq-10-multi-consumer-2026-10-06.md`

### Exit gate
The same `[in.gnu6]` core supports both consumers with adapters/configuration rather than consumer-specific engine branches.

**Disposition: RATIFIED.**

## MQ-11 — Renderer contract

State: **RATIFIED**

### Required invariants
- renderer is replaceable;
- renderer does not own ontology;
- renderer does not invent semantic identity;
- renderer cannot silently mutate scene state;
- rendering cadence does not define simulation truth;
- accessibility strategy is explicit;
- normal-motion and reduced-motion semantics remain equivalent.

### Candidate providers
- DOM/SVG;
- PixiJS;
- Canvas;
- WebGL/WebGPU.

No candidate is ratified by this list.

### Ratified evidence
- deeply frozen renderer frame;
- exact identity partition validation;
- accessibility metadata keyed by existing semantic identity;
- renderer sequence + motion mode excluded from semantic digest;
- deterministic nonvisual CommandRenderer;
- deterministic pure SVG-string adapter;
- renderer mutation cannot alter scene truth;
- causal mutation allowing invented renderer identity produced **1 RED test**;
- restored-state gates: **26 files / 102 tests PASS**, format/lint/typecheck PASS.

Implementation:
`93db694 feat: add bounded replaceable renderer contract`

Receipt:
`docs/evidence/mq-11-renderer-contract-2026-10-06.md`

### Exit gate
The renderer contract is sufficient to implement at least two materially different rendering adapters or one adapter plus one deterministic nonvisual renderer without changing core ontology.

**Disposition: RATIFIED.**

## MQ-12 — Reference browser renderer

State: **RATIFIED — PIXIJS 8.22.0 / WEBGL REFERENCE**

### Provider decision

The operator selected **PixiJS latest** as the first reference browser renderer provider.

This selects a projection provider, not a new semantic engine.

PixiJS remains downstream from the ratified MQ-11 renderer contract and may not acquire:
- semantic identity authority;
- scene clock authority;
- topology ownership;
- interaction authority;
- product/business state.

### Minimum completeness requirement
`[in.gnu6]` ultimately needs at least one browser renderer capable of demonstrating its defining behaviors live.

### Must demonstrate
- deck overlap/occlusion;
- continuous magnification;
- neighbor yield without cut-offs;
- shape topology morph;
- insertion/removal;
- topology handoff;
- nested frames;
- multiple concurrent topology instances;
- reduced-motion semantic parity;
- accessible product integration strategy.

### Ratified evidence
- operator-selected `pixi.js@latest` resolved and pinned to **8.22.0**;
- manual WebGL render path with Pixi ticker stopped;
- stable `EntityId -> Graphics` provider handles;
- Pixi event authority disabled (`eventMode = "none"`);
- sibling semantic DOM accessibility projection;
- real Chrome for Testing browser proof via Playwright;
- live overlap, magnification, neighbor yield, shape morph, insertion/removal, handoff, nested frames and concurrent topologies;
- reduced-motion destination parity;
- shape mutation blocker repaired via identity-preserving `SceneEngine.setShape`;
- causal mutation recreating Pixi handles made browser continuity RED;
- restored-state cycle `check -> build -> browser -> check` PASS;
- Vitest: **26 files / 104 tests PASS**;
- Playwright: **1/1 PASS**;
- npm audit: **0 vulnerabilities**.

Implementation:
`0e40a06 feat: add PixiJS browser reference renderer`

Receipt:
`docs/evidence/mq-12-pixi-browser-2026-10-07.md`

Visual evidence:
`docs/evidence/assets/mq-12-pixi-reference-browser.png`

### Exit gate
A real browser observation demonstrates the above from scene truth without moving semantic authority into the renderer.

**Disposition: RATIFIED.**

## MQ-13 — Motion continuity / no-cutoff gate

State: **RATIFIED**

### Normal-motion invariant
When an entity remains semantically continuous across a state transition, normal-motion rendering must not use disappearance/reappearance as a substitute for transformation.

Cut-off is allowed only when:
- semantic identity actually terminates; or
- reduced-motion policy explicitly chooses direct settle.

### Must falsify
- menu entry/exit;
- card insertion/removal;
- deck reflow;
- hover magnification;
- topology handoff;
- window-like surface → card-like surface;
- context attachment;
- renderer mount/unmount behavior.

### Ratified evidence
- renderer-neutral continuity oracle;
- stable provider-handle law;
- mount/visibility/renderability/alpha/finite-geometry laws;
- real Chrome continuity traces for magnification, reflow, topology morph, insertion/removal, handoff and context/menu lifecycle;
- context attachment observed explicitly;
- semantic birth/termination distinguished from continuous identity;
- causal mutation setting live Pixi alpha to zero produced `PROVIDER_NONZERO_ALPHA_WHILE_SEMANTIC_IDENTITY_LIVES` RED;
- restored-state gates: **27 files / 110 tests PASS**, Vite build PASS, Playwright **2/2 PASS**.

Implementation:
`2a72679 feat: add browser no-cutoff continuity ledger`

Receipt:
`docs/evidence/mq-13-motion-continuity-2026-10-07.md`

### Exit gate
Every required reference transition has an explicit causal continuity proof or an explicit semantic-termination proof.

**Disposition: RATIFIED.**

## MQ-14 — Performance and scale envelope

State: **RATIFIED**

### Required measurements
No fixed targets are ratified yet.

The engine must eventually characterize:
- entity count scaling;
- topology-node count scaling;
- fixed-step cost;
- field evaluation cost;
- contour morph cost;
- picking cost;
- renderer projection cost;
- memory;
- observer overhead.

### Rule
Optimization may not silently weaken invariants.

### Ratified evidence
- reproducible core performance harness;
- reproducible browser/Pixi performance harness;
- fixed-step scaling characterized to 2048 entities;
- field evaluation characterized;
- contour step + retarget scaling characterized to 1024 nodes;
- picking characterized to 5000 occupants;
- representative Docklab 24-card and SPJ 12-card workloads measured;
- core memory observations recorded;
- browser RendererFrame build + Pixi projection measured to 512 entities;
- browser JS heap observed;
- full SceneReceipt observer overhead quantified at 256 entities;
- post-instrumentation functional gates remain GREEN.

Raw evidence:
- `docs/evidence/assets/mq-14-core-envelope.json`
- `docs/evidence/assets/mq-14-browser-envelope.json`

Receipt:
`docs/evidence/mq-14-performance-envelope-2026-10-07.md`

Important limitations:
- browser renderer measurements used headless Chrome SwiftShader, not physical GPU hardware;
- no product SLO was invented;
- SceneReceipt is expensive enough to remain on-demand diagnostic work at the measured scale.

### Exit gate
A documented performance envelope exists for representative Docklab and SPJ workloads, and instrumentation overhead is known.

**Disposition: RATIFIED.**

## MQ-15 — Engine completeness evaluation

State: **RATIFIED COMPLETE FOR CURRENT MANDATE**

This is the final Main Quest. It is not a feature implementation quest. It is a contradiction-oriented evaluation.

### Required question
Can `[in.gnu6]` honestly be called complete for the currently ratified mandate?

### Minimum answer requires
- all mandatory Main Quest exit gates satisfied;
- no unresolved contradiction against the core ontology;
- two distinct consumers proven;
- one real browser rendering path proven;
- environment/space awareness proven;
- concurrent/nested topology proven;
- ownership/handoff proven;
- no-cutoff normal-motion gate proven;
- deterministic/introspectable engine behavior proven;
- performance envelope documented;
- contracts/documentation describe current truth.

### Possible outcomes
- `RATIFIED COMPLETE FOR CURRENT MANDATE`
- `NOT COMPLETE — OPEN GAPS`
- `MANDATE REDEFINED BY OPERATOR`

Completion does not mean frozen forever. It means the current completeness claim is causally supported.

### Ratified evidence
- MQ-00 through MQ-14 individually RATIFIED;
- deterministic completeness gate verifies required states, receipts, evidence surfaces, consumer proofs and provider boundaries;
- completeness gate causal mutation reclassifying MQ-14 produced exactly one RED failure;
- static/core restored-state gate: **29 files / 119 tests PASS**;
- Pixi qualification build: PASS;
- Playwright browser continuity: **2/2 PASS**;
- fresh core and browser performance characterization tied to clean source HEAD `832e0611273862f2ed93d038a7d6feaa9ff1936b`;
- runtime npm audit: **0 vulnerabilities**;
- stale README / CONTRACT / Questlog truth repaired during evaluation;
- no unresolved mandatory contradiction remained after rerun.

Receipt:
`docs/evidence/mq-15-completeness-evaluation-2026-10-07.md`

### Exit-gate decision

**RATIFIED COMPLETE FOR CURRENT MANDATE**

Open Side Quests remain operator-prioritized and do not invalidate Main Quest completion unless future evidence promotes one into mandatory scope.

# Side Quest Registry

Side Quests are operator-prioritized. Agents may append newly discovered Side Quests with evidence and recommendations.

A Side Quest is optional with respect to **whether it is executed now**, but never vague about what completion would mean.

Every Side Quest must distinguish:
- importance;
- urgency;
- necessity to a Main Quest;
- execution order;
- operator priority.

These dimensions must not be collapsed into one field.

## Side Quest categories

### HABITAT
Work that restores or improves the surrounding execution environment.

Examples:
- cleanup artifacts outside repository ownership;
- repair local tooling;
- restore deterministic fixtures;
- remove accidental environmental pollution.

### CAPITAL
Work that creates reusable leverage.

Examples:
- build a reusable external toolchain;
- improve deterministic workflows;
- add reusable diagnostics;
- benchmark alternate providers.

### TRUTH_DEBT
Work that restores epistemic/documentary truth.

Examples:
- update stale documentation;
- remove false historical claims;
- reconcile contradictory receipts;
- recover missing provenance.

A Side Quest may begin as `DISCOVERED_BLOCKER_CANDIDATE` when its causal relation to the Main Quest is unknown.

After inspection, its causal status must become one of:
- `UNKNOWN`;
- `NON_CAUSAL`;
- `CAUSAL_REQUIRED`;
- `CAUSAL_ACCELERATOR`.

Only `CAUSAL_REQUIRED` justifies promotion into Main Quest required scope.

Promotion requires evidence and operator visibility.

## Cross-project quest propagation

Quest knowledge may cross project boundaries without transferring control of either project.

The three independent dimensions are:

### Hierarchy

`MAIN -> SIDE`

This answers where a quest sits inside one project's completeness structure.

### Causality

Allowed typed relations include:

- `BLOCKS`
- `MAY_AFFECT`
- `IMPLEMENTS`
- `PROVIDES_FOR`
- `DUPLICATES`
- `SUPERSEDES`

This answers what causal role one quest may have relative to another.

### Provenance

Allowed typed lineage relations include:

- `DISCOVERED_DURING`
- `DERIVED_FROM`
- `COPIED_FROM`

This answers where the discovery came from.

Avoid generic `related_to` when a typed relation can state the actual meaning.

### Propagation laws

**Discovery is portable.**
A discovery may be projected into any questlog where it becomes relevant.

**Copy, don't silently transfer.**
The source quest remains in its original project until that project's own relation to the discovery is requalified.

**Quest propagation carries knowledge, not authority.**
A copied quest does not grant write, execute, publish, mutate or administrative authority in the destination project.

**Quest propagation carries discovery, not priority.**
The destination receives operator priority `UNSET` unless its operator explicitly assigns one.

**Resolution may propagate evidence back, not automatic closure.**
Closing a derived quest in Project B may add evidence to the source quest in Project A, but the source moves to `NEEDS_REQUALIFICATION` rather than `CLOSED` automatically.

### Copy lineage minimum

A copied quest must preserve at least:

```text
provenance:
  discovered_in: project-a
  derived_from:
    project: project-a
    quest: SQ-A17
    relation: COPIED_FROM
```

The destination then assigns its own:
- local quest ID;
- causal relation to its own Main Quest;
- scope;
- authority;
- completion contract;
- operator priority.

### Resolution-backflow rule

If:

`Project B / SQ-B04 -> CLOSED`

and Project A depends on that work, Project A records:

```text
new_evidence:
  source: Project B / SQ-B04
  state: CLOSED

status:
  NEEDS_REQUALIFICATION
```

Project A must decide whether the new evidence fully, partially or not at all resolves its own quest.

## SQ-001 — Donor provenance: Docklab
State: **OBSERVED**
Suggested priority: medium
Operator priority: **UNSET**

Receipt: `docs/provenance/donor-docklab-b07be20.md`

Value: prevents Docklab implementation details from being mistaken for `[in.gnu6]` ontology.

## SQ-002 — Donor provenance: SP Joaillerie
State: **OBSERVED**
Suggested priority: medium
Operator priority: **UNSET**

Receipt: `docs/provenance/donor-spj-e7595ce.md`

Value: preserves deck/tarot technical capital without granting SPJ engine authority.

## SQ-003 — blob.in donor reconstruction
State: **CLOSED**
Suggested priority: high
Operator priority: **DONE**

Question: Which blob.in mechanics are real reusable topology/dynamics capital, and which displayed Rust/Lyon/FFI claims remain unproven?

Receipt: `docs/provenance/donor-blobin-appstack-b3-4603aed.md`

Extracted fixtures:
- `tests/fixtures/blobin-donor.ts`;
- `tests/model/blobin-donor-laws.test.ts`.

Fresh reconstruction:
- preserved donor snapshot manifest: **95 files**;
- manifest digest: `5a7831ec1293c0dd810cef11c4b953bba6a891c2065b24ac63b43f4f40218b2d`;
- pristine exported workspace: **not self-contained** because `gnosis-gpui-theme` is absent;
- isolated Rust core: **47/47 PASS** under default, sequential and threaded configurations;
- reconstructed workspace excluding only the missing GPUI member: Rust/CXX-Qt/generator/golden tests PASS;
- reconstructed Qt 6 / Corrosion / CXX render-shim build: PASS;
- preserved no-unsafe gate: PASS;
- C++ shim budget: **282/300 PASS**;
- preserved RTX 3070 R8 checksums: PASS;
- R10 14-day dogfood remains explicitly unproven/pending.

Capital retained:
- renderer-neutral ordered contours;
- winding as topological data;
- deterministic damped spring law;
- stable subject identity through shape evolution;
- dirty/cold scheduling separated from semantic state;
- bounded projection/provider frontier.

Not imported as authority:
- Qt/QSG/RHI/CXX provider choices;
- lyon/Rayon provider choices;
- mascot/BubbleBloom semantics;
- backend env naming;
- historical performance/GPU sign-offs as fresh results.

The source suffix `4603aed` could not be independently resolved to a surviving Git object, so the
snapshot path + manifest digest is the reconstruction authority.

Disposition: **CLOSED — DONOR TRUTH RECONSTRUCTED / LAWS HARVESTED WITHOUT CORE AUTHORITY TRANSFER**.

## SQ-004 — PixiJS renderer/vendor study
State: **CLOSED — OPERATOR DECISION**
Suggested priority: medium
Operator priority: **DONE**

Question: Can PixiJS serve as a replaceable browser renderer for high-cardinality cards/topology without acquiring semantic authority?

Operator disposition:
- **PixiJS latest selected for MQ-12 reference browser renderer**;
- selection is provider-level only;
- MQ-11 semantic/authority boundaries remain binding;
- actual suitability remains subject to MQ-12 browser falsification.

Disposition: **CLOSED / PROVIDES_FOR MQ-12**.

## SQ-005 — Repository/toolchain ergonomics
State: **DISCOVERED — PARTIAL CAPITAL HARVESTED**
Suggested priority: low
Operator priority: **UNSET**

Examples:
- deterministic task runner;
- law-test command aliases;
- faster CI;
- contributor/dev shell;
- provenance automation.

Quick wins completed without preempting the Main Quest:
- deterministic `build:pixi-qualification` script;
- deterministic `test:browser` script;
- Playwright/Vite harness separation;
- Biome generated-artifact boundary;
- idempotent `check -> build -> browser -> check` cycle;
- explicit Playwright/Vite graceful shutdown to remove qualification-server teardown races;
- isolated qualification port via `IN_GNU6_QUALIFICATION_PORT`, proven on alternate port 4188;
- Playwright now supervises the Vite process directly rather than an `npx` wrapper; proven by three consecutive 2/2 browser runs, including port 4188.

These completed mechanics do not close SQ-005 as a whole.

Promotion rule: only if repeated mechanics justify deterministic tooling.

## SQ-006 — Historical documentation truth cleanup
State: **DISCOVERED — MQ-15 CAUSAL SUBSET REPAIRED**
Suggested priority: low
Operator priority: **UNSET**

Scope: identify stale/false claims in related docs when they materially confuse `[in.gnu6]` provenance or ontology.

MQ-15 promoted one bounded subset to `CAUSAL_REQUIRED`:
- README still said renderer choice was undecided and PixiJS was merely a candidate;
- README still described Docklab/SPJ as planned rather than proven;
- CONTRACT bootstrap wording could be misread as a present-day ban rather than a repository-creation boundary.

That subset was repaired before MQ-15 completeness evaluation.

Remaining historical/external documentation cleanup stays optional and operator-prioritized.

Not automatically authorized: rewriting unrelated repositories.

## SQ-007 — Future WebGPU renderer/backend path

Kind: side
Category: CAPITAL
State: **HYPOTHESIS**
Suggested priority: high
Operator priority: **LATER**

Intent:
Evaluate a future WebGPU-first rendering path after the PixiJS reference renderer has produced a qualified visual baseline.

Relation to Main Quest:
- quest: MQ-12 / MQ-14
- causal_status: NON_CAUSAL for current MQ-12 closure
- evidence: operator explicitly selected PixiJS latest for the current reference renderer and requested WebGPU as a future quest proposition.

Typed quest relations:
- relation: PROVIDES_FOR
- target_project: in.gnu6
- target_quest: future renderer/backend evolution
- evidence: compare against the qualified PixiJS visual/performance baseline.

Candidate scope:
- PixiJS WebGPU backend qualification;
- native/custom WebGPU renderer prototype;
- shader/deformation pipeline;
- render graph and batching model;
- memory/upload strategy;
- feature parity with the MQ-12/MQ-13 reference cases;
- performance comparison under MQ-14 workloads.

Non-goals:
- do not block current MQ-12;
- do not replace PixiJS by assumption;
- do not move scene/topology/clock/interaction authority into GPU code.

Completion contract:
- same RendererFrame contract or an explicitly ratified successor;
- exact semantic identity preservation;
- reference-case parity;
- browser support evidence;
- measured performance/complexity tradeoff;
- causal proof for any claimed advantage.

Disposition: **OPEN — FUTURE / OPERATOR LATER**.

## SQ-008 — Incremental / lower-cost scene observation

Kind: side
Category: CAPITAL
State: **DISCOVERED**
Suggested priority: medium
Operator priority: **UNSET**

Intent:
Reduce the cost of introspection if future products require high-frequency scene telemetry or continuous receipts.

Relation to Main Quest:
- quest: MQ-09 / MQ-14
- causal_status: NON_CAUSAL for current completeness
- evidence: at 256 entities, full `SceneReceipt` construction measured ~19.8 ms median versus ~0.055 ms for the semantic step alone.

Potential directions:
- incremental/delta receipts;
- dirty-entity tracking;
- topology/ownership observation caching;
- configurable receipt detail levels;
- sampling cadence policy;
- structural sharing.

Non-goals:
- do not weaken MQ-09 truth content silently;
- do not make telemetry another semantic authority;
- do not promote this work to Main Quest without operator-visible causal evidence.

Completion contract:
- same causal truth for equivalent requested detail;
- measured observer overhead improvement;
- explicit tradeoff if any detail is omitted;
- mutation/falsification proof for stale-cache hazards.

Disposition: **OPEN — OPTIONAL / OPERATOR PRIORITY UNSET**.

## SQ-009 — gnu6.live Experience consumer integration / vendored-runtime provenance

Kind: side
Category: CAPITAL
State: **CLOSED**
Suggested priority: high
Operator priority: **DONE**

Intent:
Capture the first substantial downstream `gnu6.live Experience` evidence produced after MQ-15, and
decide how the autonomous `[in.gnu6]` repository should relate to the consumer's current vendored
runtime without transferring consumer authority back into the engine.

Relation to Main Quest:
- quest: MQ-10 / MQ-13 / MQ-15
- causal_status: NON_CAUSAL for the already-ratified current completeness mandate
- evidence: `gnu6.live` Experience 6.7.4 exercises stable identity, topology presentation and
  handoff continuity in a real consumer while MQ-15 is already closed.
- effect on completeness: supporting downstream evidence only; it does not reopen MQ-15 or silently
  extend the ratified mandate.

Typed quest relations:
- relation: PROVIDES_FOR
- target_project: gnu6.live
- target_quest: Experience 6.7.4 causal handoff continuity
- evidence:
  - consumer implementation commit `2f7164d feat(experience): enforce causal handoff continuity`;
  - qualification commit `3e612ed docs(experience): qualify v6.7.4 causal handoff`;
  - runtime tag `experience-v6.7.4-amendment-04`;
  - local live receipt commit `5893875 docs(experience): record v6.7.4 local live receipt`;
  - consumer provenance contract commit `72bf476 feat(experience): bind in.gnu6 consumer provenance`;
  - backflow receipt: `docs/evidence/sq-009-gnu6-live-consumer-provenance-2026-10-07.md`.

Observed downstream proof:
- `gnu6.live` worktree: `feat/experience-6.7.4-handoff-continuity`;
- exact tagged runtime: `3e612ed0f14205589d6e94e9c308739067e6d5ec`;
- local live route: `http://gnu6.local/work/`;
- Vitest: **89/89 files PASS**;
- tests: **346 PASS, 3 skipped**;
- Experience typecheck: PASS;
- Experience production build: PASS;
- renderer boundary: PASS;
- full Chromium Experience matrix: **68/68 PASS**;
- local deployed container: healthy.

Causal consumer result:
- Spawn: new identity materializes at actual RuntimeWindow geometry; no fake Deck provenance;
- Minimize: RuntimeWindow acquires card shape, travels to Deck/helper datum, then commits parked;
- Restore: parked WindowCard/AppCard datum travels as the exact subject, expands at destination, then
  commits mapped;
- Close: existing RuntimeWindow releases in place with no Deck-directed transfer;
- bulk Minimize/Restore serializes exact identities through one transfer owner.

Live observation:
```text
TITLE=Gnosix Experience v6.7.4 · gnu6.live
VERSION=6.7.4
SPAWN_SEQUENCE=materialize
SPAWN_MORPHS=0
CAPTURE_SEQUENCE=shape-then-travel
CAPTURE_MODE_DURING=normal
CAPTURE_MODE_AFTER=minimized
RESTORE_SEQUENCE=travel-then-expand
RESTORE_FROM=stack-slot
RESTORE_TO=managed-window
RESTORE_MODE_DURING=minimized
RESTORE_MODE_AFTER=normal
CLOSE_MORPHS=0
CLOSE_GEOMETRY_DELTA=0.00,0.00,0.00,0.00
```

Why this belongs in `[in.gnu6]` Questlog:
- the consumer imports a vendored `vendor/in.gnu6` snapshot for live WindowStack/topology
  presentation;
- the downstream behavior supplies external evidence that the engine's stable-identity and
  handoff-oriented model can survive product integration;
- the consumer must remain free to own product semantics, RuntimeWindow state and UI policy;
- vendoring creates a provenance/synchronization boundary that should become explicit rather than
  silently drifting.

Boundary repair result:
- `vendor/in.gnu6/manifest.json` now binds the selected revision, exact file set, per-file SHA-256 and
  aggregate source-tree SHA-256;
- `npm run in-gnu6:sync -- --revision <revision>` forbids floating intake and rewrites vendor bytes
  from an explicit autonomous Git object;
- `npm run in-gnu6:check` proves committed bytes against the manifest and, when available, the
  autonomous source repository;
- one-byte drift mutation produced the expected RED result and exact resync restored GREEN;
- direct vendor imports are bounded to `app/experience/engine/**`;
- Biome and ESLint now exclude immutable vendor bytes while the provenance gate owns byte integrity;
- Experience compatibility qualification: **90/90 Vitest files**, **349 PASS + 3 skipped**,
  typecheck/build/renderer PASS, Chromium **68/68 PASS**;
- global format gate is now PASS;
- global lint remains RED on 9 pre-existing non-SQ009 findings;
- global typecheck remains RED on the same 8 pre-existing test errors reproduced at the untouched
  Experience 6.7.4 baseline.

Priority:
- authority: OPERATOR
- suggested_value: high
- operator_priority: DONE
- urgency: medium

Scope:
- reads:
  - autonomous `in.gnu6` source/release state;
  - `gnu6.live/vendor/in.gnu6`;
  - consumer adapters under `gnu6.live/app/experience/engine`;
  - downstream Experience evidence receipts.
- writes:
  - provenance/sync contract;
  - deterministic consumer compatibility gate;
  - optional import/update mechanism if operator authorizes it.
- forbidden:
  - moving RuntimeWindow/business semantics into `[in.gnu6]`;
  - making `gnu6.live` a semantic-engine fork;
  - auto-updating the vendored engine without revision-bound evidence;
  - reopening MQ-15 merely because downstream work continued.

Completion contract:
- exact autonomous-engine revision represented by the consumer is reconstructible;
- vendored bytes or package/source dependency have one explicit source-of-truth rule;
- consumer-specific adapter code is separated from engine code;
- sync/update is deterministic and reviewable;
- downstream compatibility suite proves identity, ownership and no-cutoff invariants after an engine
  update;
- lint/format tooling has an explicit policy for vendored sources rather than accidental project
  inclusion/exclusion;
- any future replacement of vendoring preserves the same bounded authority contract.

Provenance:
- discovered_by: downstream Experience 6.7.4 qualification
- discovered_at: 2026-10-07
- discovered_in: gnu6.live
- source_revision: `3e612ed0f14205589d6e94e9c308739067e6d5ec`
- resolved_in_project: gnu6.live
- resolved_in_revision: `72bf476`
- backflow_receipt: `docs/evidence/sq-009-gnu6-live-consumer-provenance-2026-10-07.md`
- derived_from:
  - project: gnu6.live
  - quest: Experience 6.7.4 causal handoff continuity
  - relation: DISCOVERED_DURING

Disposition:
**CLOSED — DOWNSTREAM CONSUMER PROVENANCE FORMALIZED / NO AUTHORITY TRANSFER**

# Side Quest template

Append new Side Quests using this form:

```text
## SQ-NNN — Short name

Kind: side
Category: HABITAT | CAPITAL | TRUTH_DEBT
State: DISCOVERED | DISCOVERED_BLOCKER_CANDIDATE | ...

Intent:
Why does this quest exist?

Relation to Main Quest:
- quest: MQ-NN
- causal_status: UNKNOWN | NON_CAUSAL | CAUSAL_REQUIRED | CAUSAL_ACCELERATOR
- evidence: ...

Typed quest relations:
- relation: BLOCKS | MAY_AFFECT | IMPLEMENTS | PROVIDES_FOR | DUPLICATES | SUPERSEDES
- target_project: ...
- target_quest: ...
- evidence: ...

Priority:
- authority: OPERATOR
- suggested_value: low | medium | high
- operator_priority: UNSET | NOW | NEXT | LATER | DEFERRED
- urgency: unset | low | medium | high

Scope:
- reads: [...]
- writes: [...]
- forbidden: [...]

Completion contract:
- invariants: [...]
- gates: [...]
- evidence: [...]

Provenance:
- discovered_by: ...
- discovered_at: ...
- discovered_in: ...
- source_revision: ...
- derived_from:
  - project: ...
  - quest: ...
  - relation: DISCOVERED_DURING | DERIVED_FROM | COPIED_FROM

Disposition:
OPEN | CLOSED | PROMOTED_TO_MAIN | DEFERRED_BY_OPERATOR | REJECTED
```

If a Side Quest is discovered to be causally necessary for a Main Quest exit gate, it may be proposed for promotion.

**Promotion requires explicit reasoning and operator visibility.**

# Questlog as causal project state

The Questlog is the bridge between epistemic state and execution state:

```text
reality
  ↓
observation
  ↓
discovery
  ↓
questlog
  ├── MAIN_REQUIRED
  ├── SIDE / OPTIONAL
  ├── BLOCKER_CANDIDATE
  └── CLOSED / REJECTED
        ↓
operator prioritization
        ↓
bounded agent contract
        ↓
evidence
        ↓
quest requalification
```

An agent therefore receives not only a task, but a position inside a durable causal history of work.

# Human continuity checkpoint rule

A Main Quest transition has a human-facing continuity boundary.

After a Main Quest is closed or materially requalified, the agent must return a visible checkpoint to the active human session before entering the next Main Quest.

The checkpoint must state:
- Main Quest disposition;
- implementation/evidence state;
- falsification result;
- known non-claims or remaining dependencies;
- next mandatory Main Quest.

This is an entropy-reduction mechanism for human continuity and persistence.

It does **not** mean:
- every ordinary implementation step requires approval;
- the agent should stop from fear after ordinary RED results;
- Side Quests automatically preempt the Main Quest.

The agent may continue after the checkpoint unless a new authority or operator-reserved decision is required.

# Main Quest extension rule

The Main Questline is extendable.

A new mandatory quest may be inserted when evidence shows that the current completeness definition is missing a necessary invariant.

Required amendment record:

1. contradiction or missing invariant;
2. why existing quests cannot cover it;
3. proposed new Main Quest;
4. dependencies;
5. falsification gate;
6. effect on completeness claim;
7. operator ratification state.

The Questlog must not be shortened merely to make completion easier.

Quest history is append-only in meaning: a quest may be reclassified, promoted, deferred or rejected, but the reason it existed and the evidence that changed its classification must remain reconstructible.

# Quest execution receipt

Every materially completed quest should leave:
- quest ID;
- source HEAD;
- causal inputs;
- implementation commits;
- tests/gates;
- RED mutations where meaningful;
- observations;
- unresolved unknowns;
- truth classification;
- next dependency.

A green checkmark without this binding is insufficient for Main Quest closure.

# Product / Integration Slice ledger

Side Quests remain a backlog for optional capital, habitat and truth-debt work.
They are not the default source of active implementation.

Active cross-repository work should instead be expressed as a bounded Product/Integration Slice with:
- one concrete consumer/product problem;
- one explicit authority boundary;
- the smallest engine change required by that problem;
- RED→GREEN causal proof;
- exact engine revision promotion;
- downstream compatibility qualification;
- local/live observation when a runnable consumer exists;
- a closure invariant and explicit remaining debt.

## EXPERIENCE-IN-01 — Canonical RuntimeWindow ↔ Card handoff

State: **CLOSED — QUALIFIED + LOCAL LIVE**
Category: PRODUCT_INTEGRATION
Operator priority: **DONE**

Problem:
Experience 6.7.4 preserved Minimize/Restore causal semantics but still recomputed scenic handoff
geometry inside the consumer.

Engine change:
- `68371fe feat(topology): plan canonical rect handoffs`;
- generic stable-subject rectangular handoff planner;
- source/transfer/target geometry + radius + endpoint vector;
- no RuntimeWindow/AppCard/Minimize/Restore vocabulary in the engine.

Consumer result:
- `d93a559 feat(experience): canonicalize window card handoff`;
- exact sync to `[in.gnu6]@68371fe`;
- bounded adapter `app/experience/engine/in-gnu6-handoff.ts`;
- consumer no longer computes intermediate card geometry or handoff vector;
- full Chromium Experience matrix: **68/68 PASS**;
- local live container: healthy;
- live capture/restore report `geometryOwner=in.gnu6@68371fe`;
- same semantic identity retained across both directions.

Backflow evidence:
`docs/evidence/experience-in-01-consumer-qualification-2026-10-07.md`

Disposition:
**CLOSED — REAL CONSUMER NEED → MINIMAL ENGINE CAPABILITY → EXACT PROMOTION → LIVE PROOF**

## EXPERIENCE-IN-02 — Live retargeting / unified 6.8.0 baseline

State: **CLOSED — QUALIFIED + LOCAL LIVE**
Category: PRODUCT_INTEGRATION
Operator priority: **DONE**

Problem:
Queued handoffs retained historical geometry, and an active destination could change during transfer.

Engine:
- `2602d30 feat(topology): retarget canonical rect handoffs`, promoted on main;
- stable generic subject/source/sequence, monotone generation, retarget count;
- no product or renderer authority added;
- 32 files / 129 tests PASS; Pixi build PASS; browser 2/2 PASS;
- equal-generation causal mutation RED, restored-state check GREEN.

Consumer:
- `7ae403c feat(experience): consolidate qualified v6.8.0 retarget baseline`;
- exact source tag `experience-v6.8.0`;
- `c7b2a1b docs(experience): ratify v6.8.0 exact-source local live baseline`;
- semantic request queue + activation-time resolution + same-token retarget;
- provider continues an expanding Restore without replaying the Card;
- global quality debts repaired without suppressions;
- 357 Vitest PASS + 3 skips; portal 67 PASS + 5 mobile exclusions;
- Experience Chromium 69/69 PASS; deployed causal scenarios 7/7 PASS;
- immutable Git-archive image healthy at `gnu6.local/work`;
- live capture generation 1→2 and Restore 3→4, same subject/token/node;
- four causal RED findings preserved, including strengthened silhouette oracle.

Backflow:
`docs/evidence/experience-in-02-consumer-qualification-2026-10-07.md`

Disposition:
**EXPERIENCE-6.8.0 RATIFIED AS CURRENT LOCAL PRODUCT BASELINE.**
Shared consumer npm audit has 19 findings, including one critical; advisory remediation remains open.
Initial deployment-swap HTTP 404 is recorded; zero-downtime routing is not qualified.
Neither concern is disguised as a source/functional PASS or an engine completeness failure.
No next slice or public deployment is automatically authorized.

# Current next movement

Current program position:
- MQ-00: RATIFIED
- MQ-01: RATIFIED
- MQ-02: RATIFIED
- MQ-03: RATIFIED
- MQ-04: RATIFIED
- MQ-05: RATIFIED
- MQ-06: RATIFIED
- MQ-07: RATIFIED
- MQ-08: RATIFIED
- MQ-09: RATIFIED
- MQ-10: RATIFIED
- MQ-11: RATIFIED
- MQ-12: RATIFIED — PixiJS 8.22.0 / WebGL reference
- MQ-13: RATIFIED
- MQ-14: RATIFIED
- MQ-15: RATIFIED COMPLETE FOR CURRENT MANDATE

No mandatory Main Quest remains under the current ratified mandate.

Any future mandatory work must come from:
- a newly ratified Main Quest;
- a Side Quest explicitly promoted because evidence makes it causally required;
- or an operator-defined mandate extension.

Operator-approved Side Quest batch:
- SQ-003 — **CLOSED**: blob.in donor truth reconstructed and reusable laws harvested;
- SQ-009 — **CLOSED**: gnu6.live consumer provenance/sync boundary formalized and falsified.

No Side Quest from the SQ-003 / SQ-009 approved batch remains in execution.

Most recent Product/Integration Slice:
- EXPERIENCE-IN-02 — **CLOSED / QUALIFIED + LOCAL LIVE**.
- Current consumer baseline: **EXPERIENCE-6.8.0**, exact source tag `experience-v6.8.0`.

No Product/Integration Slice is currently active. The next slice should originate from a concrete
consumer/product problem, not from automatic backlog traversal.

Other open/deferred Side Quests remain in the registry under their existing priorities. None becomes
mandatory or `NOW` without operator reprioritization.

MQ-15 remains **RATIFIED COMPLETE FOR CURRENT MANDATE**.

Operator may reprioritize Side Quests at any time.

---

**Compress mechanics. Preserve reality.**

**There remains something to understand. Find exactly what.**


## SQ-PUBLIC-01 — Préparation à une source publique et versionnement

**État : AUTHORIZED_STAGED / RELEASE_BLOCKED**

Le 2026-10-10, l'opérateur a explicitement autorisé de rendre
`tension-atoi/in.gnu6` public **progressivement**, tout en conservant
les chantiers techniques (notamment GPU-05) comme priorité. Cette
autorisation remplace seulement l'interdit de visibilité de la
création initiale du dépôt, et **ne remplace pas** les contrats d'API,
de licence ni les gates de confidentialité.

- Nouveau README français/anglais, centré sur les preuves attribuables,
  les frontières de l'ontologie et les limites du prototype Vulkan.
- Versions calendaires justifiées, sans microversions `0.0.x` :
  voir `docs/release/VERSIONING.md`.
- Gates secrets, droits/licences et redaction :
  `docs/public/PUBLICATION-READINESS.md`.
- Aucun `main` push/merge, changement de visibilité ni package
  publié avant fermeture des gates correspondants.
- PACER-07 et ENDURANCE-01 sont qualifiés séparément et rapprochés
  documentairement : CP03 HYBRID-60 reste OPEN.

**Priorité opérateur :** chantier transversal progressif, ne remplace
pas les Main Quests ni la falsification de GPU-05.
