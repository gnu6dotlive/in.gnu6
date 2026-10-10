# MQ-05 headless receipt — topology and shape evolution

Date: 2026-10-06
Status: **GREEN_LOCAL — AWAITING REFERENCE RENDERER PROOF**
Quest: **MQ-05**
Implementation commit: `4ef5a370e2c084892bad22862716bb2b82fb45e3`

## Headless claim

The semantic engine can evolve one persistent subject through materially different contour targets while preserving node-level dynamic state.

This receipt does **not** ratify MQ-05 completely because its exit gate requires visual continuity in a reference renderer.

## Implemented headless capabilities

### Per-node contour dynamics

Each contour node carries:
- position;
- velocity.

A deformable contour therefore has a real dynamic state rather than only one scalar progress value.

### Retarget / reversal

Retargeting changes the target correspondence while preserving:
- exact subject identity;
- current node positions;
- current node velocities.

Tested sequence:
- large rounded surface;
- diamond topology;
- small card-like surface;
- reversal back toward the original surface.

### Shape velocity

Node velocity is explicit and survives retarget/reversal.

### Deformation fields

Radial spatial fields can deform contour vertices without mutating the source contour.

### Spatial-frame composition

A local contour can be projected through nested `SpatialFrame` chains into World coordinates.

### Environment confrontation

World-projected contours can be checked for:
- admissible-region containment;
- exclusion collisions.

Polygon-vs-AABB intersection checks:
- contour vertex inside region;
- region corner inside polygon;
- edge/edge intersection.

### Disjoint ownership

MQ-02 ownership laws allow affine pose ownership and shape ownership to coexist when their degrees of freedom are disjoint.

### Numerical stability

Repeated integration over thousands of steps, including repeated retargeting and deliberately large elapsed inputs, remained finite under the bounded integration step.

## Causal mutation

Mutation:
retargeting was temporarily changed to replace every live contour-node velocity with zero.

Expected RED:
retarget/reversal continuity gates must detect the reset.

Observed:
**2 of 4 deformable-contour tests became RED**.

The failures showed exact node velocities changing from live non-zero values to zero.

Restoration:
- original `deformation.ts` restored from byte copy;
- before/after SHA-256 identical:
  `9dcb49e0defb56a1d0cb1e173769da0dbedafcffcdc2e04e5a70b4d04729e868`;
- byte comparison: PASS.

## Fresh restored-state gates

- format: PASS;
- lint: PASS;
- typecheck: PASS;
- full tests: **14 files / 60 tests PASS**;
- `git diff --check`: PASS.

## Quick win — core ontology neutrality

A core scan found donor/product names in comments:
- `blob.in`;
- a window→card example.

Those references were removed from `src/`.

Result:
core donor/product-name scan: **NONE**.

The provenance remains in donor receipts and consumer harnesses, where it belongs.

## What remains before MQ-05 ratification

The Questlog exit gate requires:

> a single entity can continuously evolve between at least three materially different surface topologies under retargeting without identity loss or visual discontinuity in a reference renderer.

The headless portion is GREEN.

Still required:
- reference renderer projection;
- visual continuity observation;
- no-cutoff proof under renderer mount/update behavior.

These are causally dependent on MQ-12/MQ-13.

Disposition: **GREEN_LOCAL — NOT YET RATIFIED**.
