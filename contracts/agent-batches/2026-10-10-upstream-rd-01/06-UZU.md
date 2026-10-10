# 06 — trymirai/uzu / W06

Contract: `00-BASE.md` and repository `contracts/TRONC_COMMUN.md`. Pinned source: [trymirai/uzu](https://github.com/trymirai/uzu/tree/23f365fc43cd7df0c4e6d6d2415c553a94e37a08) @ `23f365fc43cd7df0c4e6d6d2415c553a94e37a08`.

## Mandate
Research focus: Rust local inference abstraction. Research question: Verify new vs legacy engine boundary and whether CPU/Metal models can be correctness oracles.

## Source targets
Rust/Python/Swift/JS bindings, model format and converter, shared model state, CPU/Metal engine.

## GNU6 beneficiaries (hypotheses, NOT implementations)
gnostral.rs backend contracts; 27B ternary lab; local AI agent serving; unified benchmarks.

## Independent falsification / exit gate
Inspect engine/backend features and actual supported model/quant types; if cheap, run CPU-only semantic fixture.

## Non-claims and license
No CUDA PTQ1 or GGUF inference drop-in claim until source and numerical parity proven; protect 8GB RTX3070.
MIT (GitHub metadata); model licenses must be examined separately.

## Exclusive writes
`docs/agent-results/upstream-rd-01/w06.md` and `evidence/upstream-rd-01/w06/` only; never product code. No network publication, merge, branch rewriting, or required hardware allocation.

## Receipt
Return source URLs (commit-pinned), observed APIs, fit/non-fit, counterevidence, proposed minimal experiment, execution status, license review, and best decision (OBSERVE/EXPERIMENT/ADAPT/REJECT/BLOCKED). A source-read is not a local benchmark.
