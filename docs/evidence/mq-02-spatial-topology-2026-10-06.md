# MQ-02 receipt — spatial frames and concurrent topology instances

Date: 2026-10-06
Status: **RATIFIED**
Quest: **MQ-02**
Implementation commit: `40edfa1d8c9d80cac77f63c2b6adb979f9d75238`

## Claim

`[in.gnu6]` can host materially different topology systems concurrently in one World without requiring consumer-specific branching in the semantic core.

The proof surface covers:
- composable spatial frames;
- arbitrary invertible local bases;
- nested frame composition;
- topology instances with explicit members, constraints and claims;
- one-owner-per-degree-of-freedom law;
- disjoint concurrent ownership;
- atomic topology authority handoff;
- deterministic topology observations;
- transactional rollback on conflicting claims;
- Docklab and SP Joaillerie harnesses coexisting against the same core.

## Implemented semantics

### Spatial frames

A frame is an affine chart:

`world(p) = O + u * basisU + v * basisV`

A frame is admissible only when its basis matrix is invertible.

Implemented:
- `SpatialFrameGraph`;
- `localToWorld`;
- `worldToLocal`;
- nested frame composition;
- singular-basis rejection;
- cycle rejection.

### Topology instances

A topology declares:
- exact member set;
- frame;
- solver kind;
- explicit constraints;
- ownership claims.

Membership and authority are distinct.

### Ownership

For entity `e` and degree of freedom `q`:

`|Owner(e,q)| <= 1`

The engine allows:
- multiple topology instances;
- multiple owners for the same entity only when their degree sets are disjoint;
- atomic authority transfer from one topology to another.

Authority transfer does not rewrite scene identity, pose or velocity.

## Consumer falsification

### Docklab

The Docklab harness translates an edge-relative environment surface into a generic spatial frame:
- tangent axis;
- inward axis;
- local tarot/fan topology;
- projection into World coordinates.

No Dock-specific branch was added under `src/`.

### SP Joaillerie

The SPJ harness translates:
- viewport;
- centered tarot hand;
- playzone;

into sibling/nested spatial frames and uses the same generic deck solver.

No SPJ-specific branch was added under `src/`.

### Coexistence

A single synthetic World hosts:
- Docklab edge-relative topology;
- SPJ viewport-centered topology;
- SPJ hand + playzone;
- ownership handoff between SPJ hand, drag and playzone.

## Required falsifications

Observed GREEN:
- Docklab edge frame + tarot fan simultaneously;
- SPJ viewport fan + playzone simultaneously;
- nested frame composition;
- two independent topology instances in one World;
- legal disjoint ownership;
- handoff preserving subject identity, pose and velocity.

Observed RED:
- illegal double ownership;
- conflicting later topology claim.

## Causal mutation

Mutation:
the double-ownership guard was temporarily changed so conflicting ownership could be accepted.

Expected:
ownership law tests become RED.

Observed:
**2 ownership tests became RED**.

Restoration:
- original file restored from a byte copy;
- before/after SHA-256 matched;
- byte comparison PASS.

This establishes that the ownership gate is causally sensitive to the invariant it claims to prove.

## Fresh gates

On the restored state:
- format: PASS;
- lint: PASS;
- typecheck: PASS;
- full tests: **6 files / 32 tests PASS**;
- `git diff --check`: PASS;
- core consumer-name scan: no Docklab/SPJ/product branch references remained in `src/`.

Fresh test rerun before ratification:
- **6 files / 32 tests PASS**.

## Known non-claims

MQ-02 does not prove:
- deterministic fixed-step time;
- collision/admissibility solving;
- renderer behavior;
- browser motion continuity;
- public API stability;
- production readiness.

Those remain separate Main Quests.

## Exit-gate decision

The MQ-02 exit gate is satisfied:

> the engine can host at least two materially different topology systems concurrently without consumer-specific branching in the core.

Disposition: **RATIFIED**.
