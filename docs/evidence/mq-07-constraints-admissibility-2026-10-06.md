# MQ-07 receipt — explicit constraints, collision policy and admissibility

Date: 2026-10-06
Status: **RATIFIED**
Quest: **MQ-07**
Implementation commit: `c9134bb25e3cc018f0002fa6f88d2a7ad2bccd59`

## Claim

`[in.gnu6]` now evaluates explicit spatial/topological constraints without silently dropping them and can explain why a target is admissible, adjusted or rejected.

## Constraint ontology

Every constraint has:
- stable constraint identity;
- explicit owner identity;
- typed kind;
- deterministic evaluation result.

Implemented kinds:
- `position-range`;
- `extent-range`;
- `within-region`;
- `avoid-region`;
- `overlap-policy`;
- `topology-admissibility`.

The solver returns one `ConstraintEvaluation` per declared constraint on resolved paths.

## Interval composition

Position and region bounds are combined as admissible interval intersections.

If:

`X = intersection_i X_i`

or:

`Y = intersection_i Y_i`

is empty, the solver reports an explicit conflict with the responsible constraint IDs.

Example proven conflict:
- one constraint requires x >= 500;
- another requires x <= 300.

Observed:
- target rejected;
- both constraints reported `conflict`;
- conflict reason: `incompatible-position-x-range`.

No constraint wins silently.

## Extent and environment adjustment

Extent constraints can clamp width/height before placement.

Safe-region and exclusion constraints then operate over the resulting extent.

A target may therefore become `adjusted` with per-constraint explanations such as:
- `extent-clamped`;
- `moved-inside-region`;
- `moved-outside-exclusion`.

## Peer overlap policy

Card/entity overlap policy references an exact peer identity and a maximum subject-overlap ratio.

Violation is explicit:
`peer-overlap-ratio-exceeded`.

The policy currently rejects rather than inventing hidden movement.

## Topology-specific admissibility

Consumers/topologies may provide explicit deterministic facts.

A `topology-admissibility` constraint declares:
- fact ID;
- expected value.

Mismatch is reported as:
`topology-admissibility-mismatch`.

Missing fact is reported as:
`topology-fact-unavailable`.

This moves admissibility truth into the constraint report instead of hidden consumer branches.

## Stability

Small input perturbations that do not cross a constraint boundary preserve the same admissibility classification and constraint statuses.

## Causal mutation — silent drop

Mutation:
the solver was temporarily changed to remove every `topology-admissibility` constraint from the evaluation sequence.

Expected RED:
- declared-constraint cardinality gate fails;
- topology-specific rejection becomes incorrectly admissible.

Observed:
**2 tests RED**.

One result contained only 1 evaluation for 2 declared constraints.
Another changed from expected `rejected` to incorrect `admissible`.

Restoration:
- original `constraints.ts` restored byte-exactly;
- before/after SHA-256 identical:
  `1ed317ebb22a137cd1990a94e8390b26560ce0a0a2f8cc0e11be6f3b87252737`;
- byte comparison: PASS.

## Fresh restored-state gates

- format: PASS;
- lint: PASS;
- typecheck: PASS;
- tests: **19 files / 78 tests PASS**;
- `git diff --check`: PASS.

## Known boundaries

MQ-07 does not claim:
- arbitrary nonlinear optimization;
- continuous collision response;
- interaction authority;
- renderer clipping correctness;
- browser performance.

Those belong to later quests.

## Exit-gate decision

The engine can explain why a target is admissible, adjusted or rejected, while preserving explicit constraint ownership and conflict provenance.

Disposition: **RATIFIED**.
