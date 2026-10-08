# 09 — Integrator-only qualification contract

Consume G01–G08 local results from eight unique worktrees. Confirm write-set disjointness, evidence classification, no copying from old blob.in snapshot or external copyrighted code, and no publication. Re-run one fixed E02/E05/E11 trace against CPU and GPU outputs if available. No CPU/GPU parity claim without both actual executions. Record semantic digest (must agree), image differences (declared tolerance), real Vulkan adapter, shader validation and latency split by pass.

Priorities: G01 field/topology → G03 CPU oracle → G02 GPU wgpu → G04 material/bevel → G05 postFX → G08 QA → G06 UX → G07 host adapter. Handoffs blocked by unratified shared FrameIR are tracked as RED, not silently integrated.

Write only integrator-owned `docs/agent-results/blobin-next-01/INTEGRATION.md` in a separate worktree; never merge worker scratch automatically into in.gnu6 product core. Propose a dedicated blob.in crate layout and licensing/provenance decision for operator. Don't push, deploy or publish.
