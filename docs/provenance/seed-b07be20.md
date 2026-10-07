# Seed provenance — b07be20

Status: **OBSERVED / IMPORTED**

Source repository:

`/mnt/workbench/workspaces/gnu.in.labs/.worktrees/gnu6-live-experience-ratify`

Source commit:

`b07be200ac3dbec8e6dd896c84bb043390ea4260`

Source commit subject:

`feat(experience): add headless topology scene kernel`

Source commit time:

`2026-10-06 21:14:15 -0400`

## Import rule

The seed was extracted with `git show <revision>:<path>`.

No consumer/Dock code was imported.
No renderer was imported.
No new engine semantics were introduced during extraction.

The repository layout changed, so relative import paths were rewritten.

A normalization gate reversed only those import-path rewrites and compared each migrated file with the exact source bytes.

Result:

**12/12 PASS**

This establishes that migration changed file location/import addressing, not the seed implementation.

## Source → repository mapping

- `app/experience/topology/types.ts` → `src/algebra/types.ts`
- `app/experience/topology/laws.ts` → `src/algebra/laws.ts`
- `app/experience/topology/math.ts` → `src/dynamics/math.ts`
- `app/experience/topology/field.ts` → `src/dynamics/field.ts`
- `app/experience/topology/scene.ts` → `src/scene/engine.ts`
- `app/experience/topology/hierarchy.ts` → `src/scene/hierarchy.ts`

- `app/experience/topology/deck.ts` → `src/deck/deck.ts`
- `app/experience/topology/contour.ts` → `src/topology/contour.ts`
- `app/experience/topology/morph.ts` → `src/topology/morph.ts`
- `tests/unit/experience-topology-engine.test.ts` → `tests/model/topology-engine.test.ts`
- `tests/unit/experience-topology-contour.test.ts` → `tests/model/topology-contour.test.ts`
- `tests/unit/experience-topology-hierarchy-morph.test.ts` → `tests/model/topology-hierarchy-morph.test.ts`

## Bootstrap law gate

In the autonomous repository:

- test files: **3**
- laws/tests: **18**
- result: **18/18 PASS**
- TypeScript: PASS

This gate proves only that the imported semantic seed behaves as it did in the source repository.

It does not prove:
- generic multi-consumer suitability;
- renderer correctness;
- PixiJS suitability;
- completeness of the spatial/environment model;
- a public API;
- production readiness.

## Post-import repository normalization

After the 12/12 byte-equivalence gate, the new repository formatter was applied.

That formatting change is intentionally downstream of the provenance proof.

Post-format semantic result:
- `npm run check`: PASS
- seed laws/tests: **18/18 PASS**

The repository therefore preserves both:
1. a durable record of exact source provenance;
2. an independently qualified local style/toolchain.
