# MQ-03 receipt — deterministic scene time and dynamics

Date: 2026-10-06
Status: **RATIFIED**
Quest: **MQ-03**
Implementation commit: `236ed78093f564d28c7b07946ed04256344f61a7`

## Claim

`[in.gnu6]` owns explicit deterministic scene time independently from renderer cadence.

The engine now provides:
- fixed-step semantic integration;
- bounded work after long wall-clock pauses;
- observable dropped time;
- preserved fractional phase;
- continuous spring velocity across retarget/reversal;
- explicit settle criteria;
- direct settle for reduced-motion with unchanged semantic destination;
- deterministic semantic replay fixtures.

## Fixed-step clock

For configured frequency `hz`:

`dt = 1 / hz`

Renderer-observed elapsed time is converted into step units:

`phase += elapsedSeconds * hz`

Whole step units become scene integration steps.

A per-observation budget bounds catch-up:

`executed = min(floor(phase), maxStepsPerAdvance)`

Excess whole steps are not silently replayed. They are recorded as dropped time.

This separates:
- wall-clock observation;
- scene time actually integrated;
- discarded catch-up debt.

## Cadence-independence proof

Two identical scenes were advanced with different renderer cadences:

- 120 observations at 60 Hz;
- 60 observations at 30 Hz.

Both used the same 60 Hz semantic fixed-step clock.

Observed:
- identical scene time;
- identical final entity state;
- identical fixed-step clock observation.

Renderer cadence therefore did not alter the ordered integration path for the tested logical duration.

## Long-pause bound

Clock:
- 60 Hz;
- `maxStepsPerAdvance = 6`.

Input:
- 10 seconds elapsed wall time.

Observed:
- executed steps: **6**;
- dropped steps: greater than 500;
- dropped seconds: greater than 9;
- scene time advanced only `6 / 60` seconds.

A suspended or stalled renderer callback therefore cannot inject unbounded simulation work in one observation.

## Retarget and reversal

A moving entity was integrated toward one target, then retargeted to the opposite side.

Observed immediately after retarget:
- exact current pose preserved;
- exact current velocity preserved;
- target changed;
- no stale-endpoint restart.

The new spring acceleration acts on the preserved dynamic state during subsequent fixed steps.

## Settle semantics

Settled state is not inferred from elapsed time.

An entity is settled only when target error and velocity are all within explicit thresholds:
- position;
- rotation;
- scale;
- linear velocity;
- angular velocity;
- scale velocity.

## Reduced motion

Reduced motion uses direct settle:

`current -> exact target`

The target declaration remains identical to the normal-motion path.

Observed:
- same semantic target in normal and direct traversal;
- direct path ends exactly at target;
- direct path zeroes velocity;
- both paths satisfy the same settled predicate.

Therefore reduced motion changes traversal, not semantic destination.

## Deterministic replay fixture

A replay fixture applies semantic events by fixed simulation tick rather than renderer frame.

Model:

`initial state + ordered events + fixed-step law -> final observation`

Observed:
- identical timeline replay produces identical final scene state;
- reversing semantically contradictory target order produces a different final observation.

Contradictory histories are therefore not averaged or hidden.

## Inherited MQ-02 proofs

MQ-03 relies on already ratified MQ-02 laws for:
- one owner per animated degree of freedom;
- topology authority handoff;
- handoff preserving subject identity, pose and velocity.

These facts were not redefined by MQ-03.

## Causal mutation

Mutation:
the scheduler budget was temporarily removed so:

`executedSteps = availableWholeSteps`

instead of the bounded minimum.

Input:
- 10 second wall-clock pause at 60 Hz;
- configured budget: 6.

Expected RED:
the long-pause gate must detect unbounded catch-up.

Observed:
- mutated executed steps: **600**;
- expected: **6**;
- deterministic-time suite: **1 RED / 3 PASS**.

Restoration:
- original `clock.ts` restored from byte copy;
- before/after SHA-256 identical:
  `75f4c826b52a470bc78887ff8189b2d44458bff963de4ee7a8f3febaf1172503`;
- byte comparison: PASS.

## Fresh restored-state gates

- format: PASS;
- lint: PASS;
- typecheck: PASS;
- full tests: **9 files / 42 tests PASS**;
- `git diff --check`: PASS.

## Known non-claims

MQ-03 does not prove:
- environment collision/admissibility;
- renderer timing correctness;
- browser motion continuity;
- topology deformation quality;
- production scheduling behavior;
- public API stability.

## Exit-gate decision

The MQ-03 exit gate is satisfied:

> identical ordered semantic inputs + identical initial state can produce identical scene observations independently of renderer frame cadence.

Disposition: **RATIFIED**.
