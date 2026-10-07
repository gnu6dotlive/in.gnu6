# SQ-009 — gnu6.live Experience consumer provenance backflow

Date: 2026-10-07
State: **CLOSED**
Source project: `gnu6.live`
Consumer branch: `feat/in-gnu6-consumer-sync`
Consumer implementation: `72bf476 feat(experience): bind in.gnu6 consumer provenance`
Consumer base: `5893875d6a3be5194563b8aab05c7f82afd3138c`
Consumer receipt: `docs/evidence/in-gnu6-consumer-provenance-contract-sq009-2026-10-07.md`

## Why this evidence returns to [in.gnu6]

SQ-009 was discovered downstream after MQ-15 and remained NON_CAUSAL to the already-ratified engine
completeness claim.

The downstream work nevertheless creates durable evidence about how a real consumer may consume
`[in.gnu6]` without becoming a semantic-engine fork.

## Result

gnu6.live now consumes `[in.gnu6]` through an explicit revision-bound contract.

Selected engine revision:

`d5b7cd7e074c573634ed7007ca8c79164123c5f1`

Selected source-tree SHA-256:

`7990dd434b1fe0b7d0bbdb5128b9c5cbe59fd087cfabb303d48541e33e704c1d`

The consumer adds:
- a machine-readable per-file SHA-256 manifest;
- deterministic aggregate source-tree digest;
- explicit-revision sync command;
- offline-capable byte-integrity check;
- source-Git revalidation when the autonomous repository is available;
- generated runtime revision identity;
- a bounded adapter-only direct-import rule;
- consumer formatter/linter exclusion for immutable vendored source;
- compatibility tests and browser qualification.

## Causal falsification

A one-line mutation was appended to the vendored `dynamics/math.ts`.

`npm run in-gnu6:check` failed exactly on:

`vendored byte drift: dynamics/math.ts`

An explicit resync from the selected revision restored the exact bytes and the gate returned GREEN.

## Consumer qualification

- consumer provenance gate: PASS;
- offline manifest-only gate: PASS;
- format check: PASS;
- Experience typecheck: PASS;
- targeted consumer/handoff suite: 19 PASS;
- full Vitest: 90 files / 349 PASS, 3 skipped;
- Experience production build: PASS;
- renderer boundary: PASS;
- full Chromium Experience matrix: 68/68 PASS.

Repository-global lint remains RED on 9 pre-existing non-SQ009 findings. Repository-global typecheck
remains RED on the same 8 pre-existing test errors reproduced at the untouched 6.7.4 baseline.
These are recorded as external baseline debt, not hidden or repaired under SQ-009.

## Authority consequences

The downstream contract establishes:

> Consumer policy stays outside the engine; engine source stays immutable inside the consumer;
> revision changes are explicit and byte-verifiable; compatibility must be requalified after a sync.

The transport remains vendoring for now. A future package/workspace/generated-bundle transport may
replace it only if the same provenance and authority laws remain true.

## Main Quest effect

None.

MQ-15 remains `RATIFIED COMPLETE FOR CURRENT MANDATE`.

SQ-009 supplies downstream evidence to MQ-10/MQ-13/MQ-15 history but does not reopen or extend their
mandatory scope.

## Disposition

**CLOSED — DOWNSTREAM CONSUMER PROVENANCE FORMALIZED / NO AUTHORITY TRANSFER**