# MQ-15 receipt — engine completeness evaluation

Date: 2026-10-07
Status: **RATIFIED**
Completeness verdict: **COMPLETE FOR CURRENT MANDATE**
Quest: **MQ-15**
Evaluation source HEAD: `832e0611273862f2ed93d038a7d6feaa9ff1936b`

## Required question

Can `[in.gnu6]` honestly be called complete for the currently ratified mandate?

**Answer: YES — for the current mandate, with the explicit non-claims and open Side Quests preserved below.**

This is not a claim that the engine is finished forever, production-complete for every consumer, or frozen against future contradiction.

It means the Main Questline completeness contract currently has no unresolved mandatory gap.

## Main Questline result

MQ-00 through MQ-14 are individually RATIFIED.

MQ-15 evaluated their composition rather than adding new product capability.

The final result is:

`RATIFIED COMPLETE FOR CURRENT MANDATE`

## Completeness matrix

### Ontology
MQ-01 proves:
- one semantic SceneEngine;
- finite attributed scene graph;
- stable EntityId;
- explicit traversal order;
- acyclic hierarchy;
- typed relation set;
- explicit query algebra;
- lifecycle set semantics;
- invalid-state rejection.

### Space and topology
MQ-02 / MQ-04 prove:
- composable SpatialFrames;
- concurrent/nested topology instances;
- world-space environment model;
- admissibility, exclusion, containment, proximity and occlusion.

### Time and dynamics
MQ-03 proves:
- explicit fixed-step scene time;
- bounded catch-up;
- deterministic replay;
- retarget/reversal velocity continuity;
- reduced-motion destination parity.

### Shape evolution
MQ-05 proves:
- node-level contour position + velocity;
- retarget/reversal;
- deformation fields;
- spatial-frame projection;
- environment confrontation;
- live renderer continuity via MQ-12/MQ-13 closure.

### Deck/card topology
MQ-06 proves:
- fan / overlap / stack / spread;
- focus fields;
- neighbor yield;
- reorder + hysteresis;
- insertion/removal;
- transfer;
- arbitrary local frames;
- explicit lift direction rather than hidden screen-coordinate assumptions.

### Constraints
MQ-07 proves:
- explicit constraint identities;
- admissibility/rejection;
- conflict explanation;
- no silent constraint dropping;
- stable deterministic resolution.

### Interaction authority
MQ-08 proves:
- renderer/provider observation projection;
- exact semantic hit identity;
- pointer trajectory observation;
- bounded topology handoff;
- no provider authority amplification.

### Introspection
MQ-09 proves:
- deterministic SceneReceipt;
- local/world pose;
- velocity;
- parentage;
- topology membership;
- degree-of-freedom ownership;
- target;
- constraints;
- containment/occlusion;
- fields;
- motion state;
- structural causal diff;
- no observation authority amplification.

### Multi-consumer falsification
MQ-10 proves one shared semantic core for:
- Docklab;
- SP Joaillerie deck/playzone.

One synthetic World uses the same:
- SceneEngine;
- FixedStepClock;
- SpatialFrameGraph;
- SpatialEnvironment;
- TopologyRegistry / OwnershipRegistry;
- SceneReceipt surface.

### Renderer boundary
MQ-11 proves:
- replaceable RendererFrame contract;
- exact identity partition;
- deep-frozen downstream projection;
- accessibility metadata keyed by semantic identity;
- renderer cadence/motion mode do not redefine semantic truth.

### Real browser reference
MQ-12 proves:
- PixiJS 8.22.0 / WebGL reference projection;
- Pixi ticker stopped;
- stable EntityId -> Graphics handles;
- eventMode disabled as authority source;
- sibling semantic DOM accessibility projection;
- real Chrome browser observation.

### No-cutoff continuity
MQ-13 proves normal-motion continuity for:
- magnification;
- deck reflow;
- topology morph;
- insertion/removal;
- topology handoff;
- context/menu lifecycle;
- semantic birth/termination distinction;
- provider-handle continuity.

### Performance envelope
MQ-14 proves measured current behavior for:
- fixed-step scaling through 2048 entities;
- fields;
- contour stepping and retarget;
- picking through 5000 occupants;
- Docklab 24-card workload;
- SPJ 12-card workload;
- renderer projection through 512 entities;
- memory;
- SceneReceipt observer overhead.

No product SLO was invented.

## MQ-15 deterministic completeness gate

A repository-local deterministic gate now checks:
- MQ-00..14 state is RATIFIED;
- required durable receipts exist and self-identify as ratified;
- required evidence assets exist;
- repository remains private/unpublished;
- PixiJS reference version matches the ratified provider;
- deterministic qualification scripts remain present;
- required core semantic surfaces exist;
- Docklab and SPJ consumer proof surfaces exist;
- browser/reference continuity surfaces exist;
- provider/consumer names do not leak into `src/` ontology;
- exactly one exported semantic Engine class exists;
- README describes current renderer/consumer truth;
- CONTRACT keeps renderer replaceable;
- MQ-14 raw evidence is clean and includes required scale points;
- MQ-13 continuity proof remains addressable;
- Questlog points to the current completeness evaluation state.

Pre-ratification result on source HEAD:

**83 PASS / 0 FAIL**

## Causal mutation of the completeness gate

Mutation:
MQ-14 was temporarily reclassified from `RATIFIED` to `DISCOVERED`.

Expected:
the completeness gate must become RED specifically because one mandatory Main Quest is no longer ratified.

Observed:
- PASS: 43
- FAIL: 1
- exact failure: `MQ-14 ratified`.

Restoration:
- QUESTLOG restored byte-exactly;
- before/after SHA-256 identical:
  `bc44db9c39783820fc093e52b1ee32c036a1d3f7c511a132f16869e2acb8870f`;
- byte comparison: PASS.

Therefore the final completeness gate is causally sensitive to loss of a required Main Quest.

## Contradictions discovered during MQ-15

MQ-15 did not pass immediately.

### 1. Stale completeness state
MQ-00..14 were ratified while MQ-15 still said `BLOCKED BY MQ-01..14`.

Disposition:
repaired before final ratification.

### 2. Stale public repository truth
README still described renderer choice as undecided / Pixi as candidate and consumers as planned.

Disposition:
repaired as a bounded causal subset of SQ-006.

### 3. Bootstrap wording ambiguity
CONTRACT bootstrap wording could be read as a present-day Pixi prohibition rather than the historical repository-creation boundary.

Disposition:
repaired while preserving the original bootstrap provenance.

### 4. Tooling/lint environment mismatch
Node-only qualification/config scripts were being linted as browser code.

Disposition:
Biome Node-only boundaries made explicit; no global safety rule was disabled for product/core code.

### 5. Concurrent repository mutation
During evaluation HEAD advanced by:
`0de6ae4 style: normalize browser qualification config`.

The change was explicitly reconstructed and reviewed:
- author: OpenAI;
- only Playwright/Vite config formatting;
- no semantic/runtime change.

Disposition:
accepted as an observed causal input, then all final gates rerun from a later clean HEAD.

## Fresh final evaluation gates

Source HEAD:
`832e0611273862f2ed93d038a7d6feaa9ff1936b`

Tree before measurement:
**clean**

### Static/core
- format: PASS;
- lint: PASS;
- typecheck: PASS;
- Vitest: **29 files / 119 tests PASS**.

### Browser
- Pixi qualification build: PASS;
- modules transformed: 734;
- Playwright: **2/2 PASS**;
- reference browser: Chrome for Testing 153;
- reference backend: WebGL.

### Completeness
- deterministic completeness gate: **83/83 PASS**.

### Dependency audit
- runtime npm audit: **0 vulnerabilities**.

### Fresh performance observation
Both fresh MQ-15 performance reports were written to ignored temporary paths so the ratified MQ-14 raw evidence was not rewritten.

Both reports record:
- `gitHead = 832e0611273862f2ed93d038a7d6feaa9ff1936b`;
- `treeStatus = ""`.

Representative fresh core observations:
- Docklab 24-card mean: ~0.00466 ms/op;
- SPJ 12-card mean: ~0.00179 ms/op;
- SceneReceipt median observer-overhead ratio at 256 entities: ~234.9x.

Representative fresh browser observations:
- 512 entities RendererFrame build mean: ~6.28 ms;
- 512 entities Pixi render mean: ~1.54 ms;
- Pixi render p95: ~2.40 ms;
- JS heap used: ~39.2 MB.

These are characterization facts, not product SLOs.

## Open Side Quests do not invalidate completion

Main Quest completion does not require the Side Quest registry to be empty.

Known open work includes, among others:
- SQ-003 blob.in donor reconstruction;
- SQ-005 repository/toolchain ergonomics;
- remaining SQ-006 historical/external truth cleanup;
- SQ-007 future WebGPU renderer/backend path.

These remain operator-prioritized unless future evidence promotes one into a new mandatory Main Quest.

## Explicit non-claims

Current completion does not mean:
- public API finalized;
- package publication authorized;
- production deployment completed;
- native Gnosix parity proven;
- physical-GPU performance characterized;
- WebGPU backend selected or qualified;
- every future consumer requirement already modeled;
- ontology can never be challenged.

## Exit-gate decision

All current mandatory Main Quest exit gates are satisfied.

No unresolved contradiction against the ratified core ontology was found after the repaired MQ-15 evaluation.

Two independent consumers are proven.

A real browser renderer path is proven.

Environment/space awareness is proven.

Concurrent/nested topology is proven.

Ownership/handoff is proven.

No-cutoff normal-motion continuity is proven.

Deterministic/introspectable behavior is proven.

A current performance envelope exists.

Governing documentation has been reconciled to current truth.

Disposition: **RATIFIED**

Completeness verdict: **COMPLETE FOR CURRENT MANDATE**.
