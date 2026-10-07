# MQ-10 receipt — one-core multi-consumer falsification

Date: 2026-10-06
Status: **RATIFIED**
Quest: **MQ-10**
Implementation commit: `3d85cd0f21eb636f8473cbe6bc43f21ea1b9afdb`

## Claim

Docklab and SP Joaillerie can coexist inside one synthetic World while sharing the same semantic engine primitives.

The proof uses one:
- SceneEngine;
- FixedStepClock;
- SpatialFrameGraph;
- SpatialEnvironment;
- TopologyRegistry / OwnershipRegistry;
- introspection surface.

No consumer-specific branch exists under `src/`.

## Docklab pressure

The Docklab harness now proves:
- stable spatial-frame identity across edge mutation;
- edge-relative tangent/inward basis;
- local neighbor field before World projection;
- local reorder on tangent coordinate;
- same generic tarot-fan solver after edge mutation.

## SP Joaillerie pressure

The SPJ harness proves:
- viewport-derived responsive deck metrics remain consumer policy;
- same generic tarot-fan solver;
- local reorder through the same permutation/hysteresis law;
- hand + playzone topology coexistence;
- bounded handoff from hand to playzone through the same interaction authority model.

## One-world scenario

A single scenario creates:
- Docklab deck topology;
- SPJ hand topology;
- SPJ playzone topology;
- six persistent card identities;
- one shared environment;
- one shared scene clock.

The Dock frame mutates from bottom to left without changing frame identity.

The SPJ hand transfers one card's pose authority to playzone without changing card identity.

One fixed-step clock advances targets from both consumer families.

A final SceneReceipt observes both families together and reports the SPJ playzone as the effective owner of the transferred card's pose degrees.

## Contradiction discovered during MQ-10

The previous generic deck influence model encoded:

`lift = -Y`

This was a hidden screen-coordinate assumption.

It worked for a screen-aligned hand but was incorrect for an arbitrary edge frame whose local V axis means inward.

Correction:

`DeckInfluenceSpec` now requires explicit `liftDirection`.

The core normalizes that direction.

Current consumer policies:
- Docklab: local +V = inward;
- SPJ: local -Y = upward.

This is a genericity repair discovered by multi-consumer falsification, not a consumer fork.

## Stable frame mutation

`SpatialFrameGraph.update` now mutates origin/basis/parent transactionally while preserving FrameId.

Invalid singular updates roll back before becoming observable.

## Causal mutation

Mutation:
the core was temporarily changed back to a hardcoded lift direction:

`{ x: 0, y: -1 }`

Expected RED:
Docklab on a left-edge frame should move the focused card outward instead of inward.

Observed:
the unified MQ-10 scenario became RED.

Observed x:
- expected inward relative to base: greater than 44;
- mutated: 26.

Restoration:
- original deck core restored from byte copy;
- before/after SHA-256 identical:
  `1a97562c2a85c58f812595f7af5b15643de666064e3e03f124071d23e5aca8a8`;
- byte comparison: PASS.

## Fresh restored-state gates

- format: PASS;
- lint: PASS;
- typecheck: PASS;
- full tests: **24 files / 96 tests PASS**;
- `git diff --check`: PASS;
- core consumer-name scan: **NONE**.

## Exit-gate decision

The MQ-10 exit gate is satisfied:

> the same `[in.gnu6]` core supports Docklab and SP Joaillerie with adapters/configuration rather than consumer-specific engine branches.

Disposition: **RATIFIED**.
