# EXPERIENCE-IN-01 consumer qualification backflow

Date: 2026-10-07
Engine revision: `68371feff38a214d121100cc6d25725a629fdbab`
Consumer: `gnu6.live` Experience
Consumer source commit: `d93a559 feat(experience): canonicalize window card handoff`
Consumer live receipt commit: `f3951c3 docs(experience): record EXPERIENCE-IN-01 local live proof`
Consumer tag: `experience-in-01-canonical-handoff`

## Engine capability consumed

`src/topology/handoff.ts` plans renderer-neutral rectangular handoffs for one stable subject.

The consumer selects one generic sequence:
- `reshape-then-translate`;
- `translate-then-reshape`;
- `direct`.

The engine returns canonical source/transfer/target rectangular geometry, radius and endpoint vector.
It does not own product lifecycle or allocation semantics.
## Downstream proof

gnu6.live synced the exact engine revision through its revision-bound consumer manifest.

Consumer qualification:
- provenance/source check: PASS;
- Experience typecheck: PASS;
- targeted handoff/provenance/lifecycle tests: 22/22 PASS;
- full Vitest: 91/91 files, 352 PASS + 3 skipped;
- production Experience build: PASS;
- renderer boundary: PASS;
- full Chromium Experience matrix: 68/68 PASS.

First Chromium promotion run intentionally exposed one stale hard-coded engine revision assertion.
The runtime correctly reported `in.gnu6@68371fe`; the stale assertion was removed and the full
68-test matrix was rerun successfully.

## Local live proof

`gnu6.local/work` runs consumer source `d93a559` from image
`sha256:b787941bbfa01dab21ed2e7765dc6a5a25a0d4cb1dd0238406fe5885a2a2a248`.

Observed same subject: `workspace-1`.
```text
CAPTURE_OWNER=in.gnu6@68371fe
CAPTURE_SEQUENCE=reshape-then-translate
CAPTURE_RADII=12,17,17
CAPTURE_MODE_DURING=normal
CAPTURE_MODE_AFTER=minimized
RESTORE_OWNER=in.gnu6@68371fe
RESTORE_SEQUENCE=translate-then-reshape
RESTORE_RADII=17,17,12
RESTORE_MODE_DURING=minimized
RESTORE_MODE_AFTER=normal
```

## Authority result

The downstream proof supports the intended boundary:

> product intent selects why a handoff occurs; `[in.gnu6]` owns only the renderer-neutral geometry
> of the stable subject; the consumer projects it and commits product state at its own causal gate.

Spawn and Close remain distinct consumer lifecycle operations and do not acquire false handoff
provenance.

Disposition: **QUALIFIED DOWNSTREAM / PROVIDES_FOR FUTURE ENGINE EVOLUTION**.
