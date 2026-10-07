# Bootstrap evidence — 2026-10-06

Status: **OBSERVED — repository bootstrap**

## Scope

Repository: `in.gnu6`

Purpose: establish an autonomous private Git repository for the semantic seed previously living inside gnu6-live.

This evidence does not claim:
- public API stability;
- renderer readiness;
- PixiJS adoption;
- multi-consumer genericity;
- production readiness.

## Source provenance

Seed source:

`gnu6-live @ b07be200ac3dbec8e6dd896c84bb043390ea4260`

Extraction used `git show <revision>:<path>`.

Before applying the new repository formatter, an import-path normalization gate compared all migrated files against their exact source bytes.

Result:

**12/12 PASS**

After that provenance gate, the local Biome formatter was intentionally applied. This changed formatting only; the semantic law suite was rerun afterwards.

## Fresh autonomous gates

Environment:
- Node: 24 through mise/corepack
- TypeScript: 6.0.3
- Vitest: 4.1.10
- Biome: 2.5.3

Results:
- `npm run format:check`: PASS
- `npm run lint`: PASS
- `npm run typecheck`: PASS
- `npm test`: **3 files / 18 tests PASS**
- `npm audit --omit=dev`: **0 vulnerabilities**

## Authority state

- repository visibility target: **PRIVATE**
- npm package: `private: true`
- public API: **UNDECIDED**
- license decision: **UNDECIDED**
- npm operational marker: `UNLICENSED`
- renderer: **UNDECIDED**
- PixiJS: **candidate provider only**
- publication: **NOT AUTHORIZED**

## Next epistemic gate

The next architectural proof is not another renderer demo.

The engine must be consumed by at least two meaningfully different harnesses without forking its identity/topology/dynamics laws.

Planned falsification consumers:
1. Docklab
2. SP Joaillerie deck harness
