# MQ-13 receipt — motion continuity / no-cutoff

Date: 2026-10-07
Status: **RATIFIED**
Quest: **MQ-13**
Implementation commit: `2a726795f6b08e09d37c1864ddb5faa1c3292308`

## Claim

Normal-motion transitions that preserve semantic identity no longer rely on disappearance/reappearance as a substitute for transformation.

The reference browser path now produces explicit continuity traces that distinguish:
- continuous semantic identity;
- semantic birth;
- semantic termination.

## Continuity oracle

A pure renderer-neutral oracle inspects ordered samples containing:
- semantic existence;
- provider handle identity;
- mount state;
- visibility;
- renderability;
- alpha;
- finite geometry;
- scene time.

For a transition declared `continuous`:

`semanticExists = true`

must remain true for the live interval, and the provider handle must remain:
- present;
- stable;
- mounted;
- visible;
- renderable;
- alpha > 0;
- finite.

For `semantic-termination`:
- identity may disappear only after the live phase;
- resurrection inside the same lifecycle trace is illegal;
- provider projection may not outlive semantic termination.

## Pixi provider observation

The Pixi adapter exposes a read-only provider observation surface:
- handle presence;
- mounted state;
- visible;
- renderable;
- alpha;
- finite geometry.

This observation does not mutate scene or provider state.

## Browser reference transitions

A real Chrome-for-Testing browser was used to collect continuity traces for:

1. hover magnification;
2. deck reflow;
3. window-like/surface topology -> card-like topology;
4. card insertion;
5. card removal;
6. hand -> playzone topology handoff;
7. context/menu surface entry and exit;
8. context attachment to an anchor;
9. renderer mount/unmount behavior across birth/termination.

The context/menu lifecycle covers both:
- menu-like entry/exit continuity;
- explicit context attachment relation.

## Observed results

All restored-state continuity traces produced:

`violations = []`

For every live semantic sample:
- handle present;
- handle stable;
- mounted = true;
- visible = true;
- renderable = true;
- alpha > 0;
- finite geometry = true.

Insertion begins with semantic absence, then birth, then uninterrupted continuity.

Removal and context/menu exit end in semantic termination; provider handle is absent after termination.

## Causal mutation

Mutation:
Pixi rendering was temporarily changed so live entity `dock-c` received:

`alpha = 0`

while semantic identity remained alive.

Expected RED:
the no-cutoff oracle must detect provider disappearance-by-opacity.

Observed:
the browser continuity test became RED with repeated:

`PROVIDER_NONZERO_ALPHA_WHILE_SEMANTIC_IDENTITY_LIVES`

Restoration:
- original `adapters/pixi.ts` restored from byte copy;
- before/after SHA-256 identical:
  `069134efbe58a78fc50ebb1d1d47f3d677558e3dff5ac2c4c79b5107f6a07a72`;
- byte comparison: PASS.

## Harness contradiction and repair

One restored-state browser run observed an execution-context loss during MQ-12 replay.

Immediate rerun first encountered a transient leftover Vite listener, then the listener disappeared.

Classification:
**HARNESS / ENVIRONMENT — teardown race observed, not reproduced as product behavior.**

Repair:
Playwright webServer teardown now uses explicit graceful shutdown:
- signal: `SIGTERM`;
- timeout: 2000 ms.

Qualification:
two consecutive complete browser runs passed without port collision.

This quick win is recorded under SQ-005 CAPITAL.

## Fresh restored-state gates

- format: PASS;
- lint: PASS;
- typecheck: PASS;
- Vitest: **27 files / 110 tests PASS**;
- Vite Pixi qualification build: PASS;
- Playwright: **2/2 PASS**;
- `git diff --check`: PASS.

## Known non-claims

MQ-13 proves continuity for the required reference transitions in the current Pixi/WebGL qualification surface.

It does not prove:
- every future product transition;
- all browsers/backends;
- arbitrary third-party consumer transitions;
- performance envelope.

Performance remains MQ-14.

## Exit-gate decision

The MQ-13 exit gate is satisfied:

> every required reference transition now has either an explicit causal continuity proof or an explicit semantic-termination proof.

Disposition: **RATIFIED**.
