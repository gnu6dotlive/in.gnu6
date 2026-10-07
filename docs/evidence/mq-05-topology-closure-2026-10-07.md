# MQ-05 closure receipt — topology and shape evolution

Date: 2026-10-07
Status: **RATIFIED**
Quest: **MQ-05**
Headless implementation: `4ef5a370e2c084892bad22862716bb2b82fb45e3`
Reference renderer evidence: MQ-12
No-cutoff evidence: MQ-13

## Closure question

Can one semantic entity continuously evolve between materially different surface topologies under retargeting without identity loss or visual discontinuity in the reference renderer?

Answer: **YES, for the current ratified mandate and Pixi/WebGL reference path.**

## Headless evidence inherited

MQ-05 already proved:
- explicit polygon topology;
- deterministic contour winding;
- arc-length resampling;
- deterministic cyclic correspondence;
- normalized vertex counts;
- per-node position + velocity;
- retarget/reversal preserving live node state;
- continuous deformation fields;
- nested SpatialFrame projection;
- environment collision/admissibility observation;
- numerical stability under repeated integration.

Causal mutation:
zeroing contour-node velocities during retarget made **2 tests RED**.

## Reference renderer evidence inherited from MQ-12

The same semantic subject evolves:

`rounded rect -> polygon/diamond -> card-like surface`

Observed in Chrome for Testing through PixiJS 8.22.0 / WebGL:
- same semantic EntityId;
- same Pixi Graphics provider handle;
- no remove/add recreation;
- shape updates are applied through `SceneEngine.setShape`;
- provider remains downstream from RendererFrame.

MQ-12 causal mutation:
recreating Pixi handles during render made handle continuity RED.

## No-cutoff evidence inherited from MQ-13

The window-like/surface -> card-like transition is collected as a continuity trace.

Observed for every live sample:
- semantic identity exists;
- provider handle present and stable;
- mounted;
- visible;
- renderable;
- alpha > 0;
- finite geometry.

Continuity violations:
`[]`

MQ-13 causal mutation:
forcing alpha=0 while semantic identity remained alive produced
`PROVIDER_NONZERO_ALPHA_WHILE_SEMANTIC_IDENTITY_LIVES` RED.

## Fresh requalification gate

Headless targeted gate:
- deformable-contour laws: PASS;
- topology/environment/frame integration: PASS;
- **2 files / 7 tests PASS**.

Browser targeted gate:
- MQ-13 no-cutoff ledger: **1/1 PASS**.

`git diff --check`: PASS.

## Exit-gate decision

The MQ-05 exit gate is now satisfied:

> a single entity can continuously evolve between at least three materially different surface topologies under retargeting without identity loss or visual discontinuity in a reference renderer.

Disposition: **RATIFIED**.

## Known non-claims

MQ-05 does not prove:
- every arbitrary topology family;
- every renderer backend;
- future WebGPU behavior;
- unbounded contour complexity;
- performance envelope.

Those remain outside this closure or under MQ-14 / SQ-007.
