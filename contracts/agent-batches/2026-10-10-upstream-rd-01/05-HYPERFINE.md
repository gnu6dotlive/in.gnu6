# 05 — sharkdp/hyperfine / W05

Contract: `00-BASE.md` and repository `contracts/TRONC_COMMUN.md`. Pinned source: [sharkdp/hyperfine](https://github.com/sharkdp/hyperfine/tree/b33755fe1fca8bcc1f182c773ca3ab040bfe517d) @ `b33755fe1fca8bcc1f182c773ca3ab040bfe517d`.

## Mandate
Research focus: repeatable benchmarking methodology. Research question: Test direct-exec vs shell measurement differences and metric availability on current Linux.

## Source targets
Repeat/warmup/prepare runs; wall time, peak RSS and optional perf counters; structured exports.

## GNU6 beneficiaries (hypotheses, NOT implementations)
gnostral.rs; Ternary Bonsai lab; Gnosix and blob.in startup; developer CLI; docs performance claims.

## Independent falsification / exit gate
Use deterministic commands with bounded warmups/repetitions and JSON; compare counter availability, process-tree RSS limitation and baseline noise.

## Non-claims and license
CPU execution benchmarks do not measure GPU kernel time, PCIe transfers or VRAM residency; never fabricate GPU gains.
GitHub SPDX says Apache-2.0; inspect LICENSE for any dual licensing and attribution.

## Exclusive writes
`docs/agent-results/upstream-rd-01/w05.md` and `evidence/upstream-rd-01/w05/` only; never product code. No network publication, merge, branch rewriting, or required hardware allocation.

## Receipt
Return source URLs (commit-pinned), observed APIs, fit/non-fit, counterevidence, proposed minimal experiment, execution status, license review, and best decision (OBSERVE/EXPERIMENT/ADAPT/REJECT/BLOCKED). A source-read is not a local benchmark.
