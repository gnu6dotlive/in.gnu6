# EXPERIENCE-IN-02 — consumer qualification and 6.8.0 baseline
Date: 2026-10-07
State: **CLOSED — QUALIFIED + LOCAL LIVE**
Engine source: `2602d30e3cb10ec40f19718de651b0b34e9c9b23`, promoted on main.
Consumer source: `7ae403c9423e67a30db5d3d9eee93afb852fb7a5`
Consumer live receipt commit: `c7b2a1b`
Consumer tag: `experience-v6.8.0`
Consumer branch: `feat/experience-in-02-retarget-handoff`

## Concrete problem and bounded change
Previously resolved queued intents could execute against old geometry; active destination changes
could replay a completed presentation stage.
The engine now retargets a generic rectangular plan for the same subject, preserving original source
and sequence, requiring a strictly newer generation and incrementing retarget count.
No RuntimeWindow, Minimize, Restore, queue, Deck allocation, DOM or Motion policy enters the engine.
The consumer queues semantic Minimize/Restore requests, resolves them at activation, preserves
coordinator state when geometry functions are reconstructed, and retargets one active token/node.
A presentation phase guard prevents a Restore already expanding from shrinking back to Card.

## Fresh qualification
Engine: format/lint/typecheck PASS, 32 files / 129 tests PASS, Pixi build PASS, browser 2/2 PASS.
Consumer: full verify and Experience typecheck/build PASS, 91 files / 357 PASS + 3 explicit skips.
Aggregate Chromium: portal 67 PASS + 5 mobile exclusions; Experience 69/69 PASS.
Independent read-only review found late Restore silhouette replay; its fix was re-reviewed clean.
Equal-generation mutation produced RED, then bytes restored and final engine check GREEN.

## Causal consumer evidence
WindowStack modality mutation: RED.
Historical Capture source replay initially escaped the center-only oracle; strengthened dimension
continuity made it RED (65.359375 px regrowth).
Late Restore viewport growth reproduced 545.5468 px shrink before the phase-aware fix.
WindowStack five repeats, active/queued retarget ten repeats and late Restore five repeats PASS.
No assertion or boundary was weakened to obtain this qualification.

## Exact local deployment
Immutable Git archive of consumer 7ae403c was built and labeled with the exact source SHA.
Image: `gnu6-live-experience:local-7ae403c9423e`
Id: `sha256:7ede430e124e521fe16d471bc8c7546cebc216bdfc40bd5a6f75714c375b9fbc`
Route: `http://gnu6.local/work/`; running / healthy.
Direct deployed suite: 7/7 PASS; independent FR boots: 3/3 PASS.
Observed subject workspace-1, geometry owner in.gnu6@2602d30:
Capture token 1, generation 1→2, same node, mapped until commit.
Restore token 2, generation 3→4, same node, minimized until commit.
Viewport growth during actual Restore expansion produced zero width/height decreases in eight samples.
Served entry asset SHA-256 equals the container file hash.

Authoritative consumer receipts:
- `docs/evidence/experience-v6.8.0-source-qualification-2026-10-07.md`
- `docs/evidence/experience-v6.8.0-local-live-2026-10-07.md`
- `docs/evidence/assets/experience-v6.8.0-local-live.json`
- `docs/evidence/assets/experience-v6.8.0-falsification.json`

## Remaining explicit limits
First router-swap observation had a portal HTTP 404; later three boots and complete live suite passed.
Exact transient cause is NOT_PROVEN; zero-downtime router replacement is not qualified.
Consumer online npm audit retains 19 findings (6 moderate, 12 high, 1 critical).
No security clearance follows from functional tests or absence of node_modules in Experience.
Advisory remediation and Experience chunk-size warning remain documented consumer concerns.
MQ-15 stays RATIFIED COMPLETE FOR CURRENT MANDATE. No public publication or next slice follows.
