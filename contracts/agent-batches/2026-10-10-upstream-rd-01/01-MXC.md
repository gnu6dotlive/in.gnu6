# 01 — microsoft/mxc / W01

Contract: `00-BASE.md` and repository `contracts/TRONC_COMMUN.md`. Pinned source: [microsoft/mxc](https://github.com/microsoft/mxc/tree/c6f301d53a1430c4c921a05c57af838f7392348f) @ `c6f301d53a1430c4c921a05c57af838f7392348f`.

## Mandate
Research focus: execution authority and isolation. Research question: Test whether declared permissions are actually enforced, rather than merely validated.

## Source targets
Linux Bubblewrap fail-closed permission enforcement; external process/container lifecycle; UI/network mediation.

## GNU6 beneficiaries (hypotheses, NOT implementations)
Gnosix Judgment Plane, agent runners, desktop MCP, in.gnu6 orchestration, hosted service job control.

## Independent falsification / exit gate
Compare a denied filesystem or network request with actual OS denial; reject unsupported policy rather than downgrade; record host prerequisites.

## Non-claims and license
Container SDK is not equivalent to Gnosix app authorization; Windows/macOS backends cannot prove Linux enforcement.
MIT (GitHub metadata); record runtime dependencies and compatibility before adoption.

## Exclusive writes
`docs/agent-results/upstream-rd-01/w01.md` and `evidence/upstream-rd-01/w01/` only; never product code. No network publication, merge, branch rewriting, or required hardware allocation.

## Receipt
Return source URLs (commit-pinned), observed APIs, fit/non-fit, counterevidence, proposed minimal experiment, execution status, license review, and best decision (OBSERVE/EXPERIMENT/ADAPT/REJECT/BLOCKED). A source-read is not a local benchmark.
