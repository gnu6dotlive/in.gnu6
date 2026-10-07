# MQ-04 receipt — environment / space model

Date: 2026-10-06
Status: **RATIFIED**
Quest: **MQ-04**
Implementation commit: `fd6af22df9b94a17113d32495d568712b494247c`

## Claim

`[in.gnu6]` now has a renderer-neutral world-space environment model for axis-aligned admissibility, exclusion, containment, proximity, occlusion and deterministic placement.

This quest does not claim arbitrary polygonal collision solving.
The current reference geometry is explicitly AABB world-space.
Richer generalized constraints remain MQ-07.

## Implemented world model

The environment can contain typed regions:
- `admissible`;
- `exclusion`;
- `container`.

Each region has stable identity and world-space AABB bounds.

The environment exposes:
- region mutation;
- revisioned observation;
- region lookup;
- region-at-point query;
- nearest-region query.

## Placement

A subject rectangle requests:

- desired position/extent;
- containing admissible region;
- exclusion regions to avoid.

The solver returns:
- `admissible`;
- `adjusted`;
- `rejected`.

For axis-aligned rectangles, candidate placement is constructed from:
- clamped desired coordinates;
- admissible-region edges;
- exclusion-region edges.

Candidates are filtered by containment and non-intersection, then ranked by squared displacement from the desired point.

Tie-breaking is deterministic.

## Spatial relations

Implemented:
- `containsAabb`;
- `intersectsAabb`;
- `aabbDistance`;
- point containment;
- point-to-region distance;
- explicit back-to-front occlusion relation.

Touching bounds have proximity zero while not counting as penetrating overlap.

## Renderer-neutral spatial pick

`pickTopmost` resolves the topmost visible geometric identity from explicit back-to-front ordering.

This primitive does **not** grant interaction authority.

MQ-08 owns the later question of whether a spatial hit receives interaction authority or causes a handoff.

## Environment mutation

The same placement request was observed before and after insertion of a new exclusion region.

Before:
- desired placement admissible.

After:
- same request adjusted by the same solver.

Environment mutation therefore changes spatial truth deterministically without requiring consumer-specific edge logic.

## Multi-consumer pressure

A shared environment model placed three materially different subject classes:
- context-like surface;
- deck-like surface;
- playzone-like surface.

All used the same admissibility/exclusion solver.

Observed:
- context-like subject adjusted;
- deck-like subject adjusted around reserved edge space;
- playzone-like subject remained admissible.

## Nearest region

The same environment can answer nearest matching region with deterministic tie-breaking by region identity.

## Causal mutation

Mutation:
the placement solver was temporarily changed so exclusion regions were ignored.

Expected RED:
obstacle-safe placement and environment-mutation gates must fail.

Observed:
**3 tests RED**:
- obstacle avoidance;
- environment mutation reflow;
- multi-consumer reserved-edge placement.

Example failure:
a placement previously classified `adjusted` became incorrectly `admissible`.

Restoration:
- original `environment.ts` restored from byte copy;
- before/after SHA-256 identical:
  `39976610b6566753b7eff0e3b22d40e54dec3bee2e745c2f8556af6ae948b1cf`;
- byte comparison: PASS.

## Fresh restored-state gates

- format: PASS;
- lint: PASS;
- typecheck: PASS;
- full tests: **12 files / 53 tests PASS**;
- `git diff --check`: PASS.

## Relation to other quests

MQ-02 provides local spatial frames and topology composition.

MQ-04 provides world-space admissibility and environmental facts.

MQ-07 remains responsible for generalized constraints, richer collision/exclusion policy and conflict explanation.

MQ-08 remains responsible for interaction projection and authority handoff. MQ-04 only provides geometric pick truth.

## Known non-claims

MQ-04 does not prove:
- arbitrary polygonal placement;
- oriented-box collision;
- generalized constraint solving;
- renderer behavior;
- interaction authority;
- production browser integration.

## Exit-gate decision

The MQ-04 exit gate is satisfied for the current mandate:

> context-like, deck-like and playzone-like subjects can ask the same engine for deterministic admissible world-space placement without duplicated consumer `near edge` logic.

Disposition: **RATIFIED**.
