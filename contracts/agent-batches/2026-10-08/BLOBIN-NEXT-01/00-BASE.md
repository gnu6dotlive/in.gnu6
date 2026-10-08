# 00 — BLOBIN-NEXT-01 worker common contract

1. Verify base SHA, branch, worktree cleanliness, RAM/disk budget, and root private contracts. Never alter `main`, adjacent dirty worktrees or historic blob.in snapshot.
2. Read `FUTURE_ENGINE.md`, `EXPERIENCE_ATLAS.md`, `TECH_RADAR.md`; identify the product authority barrier (license GPL/AGPL mismatch, version 0.14/0.20, backend default conflict, missing GPUI dependency).
3. Allowed writes: your own `docs/agent-results/blobin-next-01/{ID}.md`, plus `experiments/blobin-next-01/{ID}/**`. All source authored clean-room; no historical source copied, no code from Vello/Lyon copied. No external repository writes.
4. Never install GPU drivers or host packages, publish binaries/crates, push, deploy, mutate Gnosix or in.gnu6/src, nor invent a public ABI. Side-effectful profiling limited to your worktree/scratch.
5. Run a falsifier, not just a happy-path screenshot. Record semantic identity, topology epochs, no-GPU feasibility, CPU/GPU effect tiers, numeric tolerances and actual pass timing if tested. Claims: VERIFIED_NOW / HISTORICAL / HYPOTHESIS / DESIGN / NOT_RUN / FALSIFIED.
6. Shader parse ≠ hardware render; old GPU checksum ≠ new GPU proof; CPU fallback ≠ blank image. Keep a substantive no-GPU output path if in scope.
7. Store no secrets or user telemetry. Synthetic heatfields only. In hardware failure, record explicit graceful quality downgrade.
8. Finish with self-contained report + local clean commit; original prototypes under own scratch. The single integrator owns cross-worker merges and decides whether to promote work to standalone blob.in source.
