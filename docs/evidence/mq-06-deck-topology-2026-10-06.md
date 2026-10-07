# MQ-06 receipt — generic deck/card topology family

Date: 2026-10-06
Status: **RATIFIED**
Quest: **MQ-06**
Implementation commit: `e097d4697c25d62154426537466060753b6396fe`

## Claim

`[in.gnu6]` now exposes one generic deck/card topology family sufficient for the current Docklab and SP Joaillerie harnesses without consumer-specific branches in the semantic core.

## Implemented deck topology

The core now provides:
- persistent ordered identity sets;
- tarot fan;
- overlap;
- linear spread;
- ordered stack;
- deterministic z-order;
- continuous pointer influence;
- focus influence independent of pointer;
- neighbor yield;
- lift;
- reorder preview with hysteresis;
- insertion/removal order planning;
- deck-to-deck transfer;
- stack occlusion observation;
- optional velocity-derived tilt;
- projection of deck centers through arbitrary SpatialFrames;
- deck pose → SceneEngine PoseTarget conversion for continuous reflow.

## Insertion/removal dynamics

Order mutation and motion remain separate.

Deck topology determines the new target pose.
Scene dynamics owns convergence.

A tested insertion changed order:

`[a,b,c] -> [a,x,b,c]`

For surviving identity `b`:
- identity remained `b`;
- current x remained 0 immediately after retarget;
- target x became 40;
- after one fixed step, `0 < x < 40`;
- linear x velocity became positive.

Surviving cards therefore reflow through SceneEngine dynamics instead of teleporting.

## Deck transfer

Transfer moves one persistent identity from source deck to destination deck.

The transferred identity must exist exactly once across the resulting pair.

### Causal mutation

Mutation:
the transfer function was temporarily changed to copy the subject into the destination without removing it from the source.

Expected RED:
the deck-transfer identity gate must fail.

Observed:
**1 of 7 deck topology tests became RED**.

The source incorrectly remained:

`[a,b,c]`

instead of:

`[a,c]`.

Restoration:
- original `deck.ts` restored byte-exactly;
- before/after SHA-256 identical:
  `276c628b772da0f00bdd3117e75d015ad057b3eae27988c7b1c5d2f71492c56c`;
- byte comparison: PASS.

## Occlusion

Stack occlusion is derived from:
- explicit z-order;
- card pose;
- card extent;
- shared environment occlusion logic.

It does not introduce a second occlusion ontology.

## Arbitrary local frame boundary

Deck topology produces local poses.

`projectDeckCentersToWorld` projects only card centers through SpatialFrame.

This is intentionally narrower than flattening complete `Pose2`:
arbitrary frames may include reflection/shear that cannot be truthfully represented by only one scalar rotation plus component scale.

Rotation/scale remain local until a later renderer/affine layer can represent the full frame transform.

## Multi-consumer proof

Docklab:
- edge-relative SpatialFrame;
- tarot/fan deck behavior.

SP Joaillerie:
- viewport/hand SpatialFrame;
- tarot/fan deck behavior.

Both now consume the same core deck projection helper.

Their spatial behaviors remain different because the frames differ, not because the engine forks.

## Quick wins

### Consumer typecheck coverage

The repository TypeScript gate previously covered only:
- `src/**/*.ts`;
- `tests/**/*.ts`.

Consumer harnesses were executed by Vitest but not independently typechecked.

`tsconfig.json` now includes:

`consumers/**/*.ts`

This immediately exposed stale duplicate imports in both harnesses, which were corrected.

### Shared center projection

Duplicated local→World mapping in Docklab/SPJ harnesses was replaced by one core helper.

This reduces repeated mechanics without hiding the affine limitation described above.

## Fresh gates

Latest full restored-state gate:
- format: PASS;
- lint: PASS;
- typecheck: PASS, including consumers;
- tests: **17 files / 70 tests PASS**;
- `git diff --check`: PASS;
- core product/donor-name scan: NONE.

Fresh MQ-06 targeted gate before commit:
- **3 files / 10 tests PASS**;
- typecheck PASS.

## Known non-claims

MQ-06 does not prove:
- renderer card appearance;
- visual insertion/removal continuity;
- Balatro-quality motion;
- generalized constraints;
- arbitrary affine pose flattening;
- browser performance envelope.

Those belong to later quests.

## Exit-gate decision

The MQ-06 exit gate is satisfied:

> Docklab and SP Joaillerie harnesses consume the same deck/topology primitives while producing distinct spatial behavior.

Disposition: **RATIFIED**.
