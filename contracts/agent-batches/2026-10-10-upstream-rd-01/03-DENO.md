# 03 — denoland/deno / W03

Contract: `00-BASE.md` and repository `contracts/TRONC_COMMUN.md`. Pinned source: [denoland/deno](https://github.com/denoland/deno/tree/a18ce33715e30cd2b0d99c7e322ef11a65490e4d) @ `a18ce33715e30cd2b0d99c7e322ef11a65490e4d`.

## Mandate
Research focus: native runtime and host extensibility. Research question: Test enforceability and ergonomics of a narrow Rust-backed scripting operation with explicit capabilities.

## Source targets
Rust V8/Tokio isolate and native ops; desktop supervision/event paths; permission boundaries.

## GNU6 beneficiaries (hypotheses, NOT implementations)
Local agent tool host; GNOSIX plugin/runtime proposals; reproducible dev tooling and CLI services.

## Independent falsification / exit gate
Run a minimal allow/deny op fixture; verify runtime error and refused filesystem/network access with explicit resource scope.

## Non-claims and license
JS sandbox is not native process sandbox; Deno Desktop WebView does not qualify as native shell renderer.
MIT; evaluate V8 packaging, ABI and runtime footprint before integrating.

## Exclusive writes
`docs/agent-results/upstream-rd-01/w03.md` and `evidence/upstream-rd-01/w03/` only; never product code. No network publication, merge, branch rewriting, or required hardware allocation.

## Receipt
Return source URLs (commit-pinned), observed APIs, fit/non-fit, counterevidence, proposed minimal experiment, execution status, license review, and best decision (OBSERVE/EXPERIMENT/ADAPT/REJECT/BLOCKED). A source-read is not a local benchmark.
