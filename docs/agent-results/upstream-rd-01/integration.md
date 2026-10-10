# UPSTREAM-R&D-01 — W09 integration verdict

**Verdict: RESEARCH_BATCH_02_QUALIFIED_FOR_PRIVATE_BRANCH / PRODUCT_TRANSFER_PENDING.** 2026-10-10.

## Reconciliation receipts
- W01–W06 originated from six separately checked-out Git worktrees/branches off `db794a6c01646cb553104c002a836f6ff801054c`, itself descended from private remote main `4042faa`.
- Each worker operated only on `docs/agent-results/upstream-rd-01/wXX.md` plus `evidence/upstream-rd-01/wXX/source-evidence.json`; each produced a single local commit.
- Integrator verified each worker commit's immediate parent and two-file diff, then cherry-picked each onto `research/upstream-rd-01-20261010`. No cross-worker file collisions.
- 6 upstream revisions, 12 intake files, 18 new deep-inspection files: **30 retrieved pinned file receipts total**. Some are the same files intentionally read at two stages; not 30 unique upstream paths.
- W07 cross-project analysis complete as a candidate/research decision; W08 host Bubblewrap oracle passed for this test fixture only.
- A worker in this tranche means **deterministic local source worker**, not independently reasoning or autonomous remote LLM agent.

## Boundaries and counterexamples
- Deno `runtime/permissions.rs` turned out to be a re-export stub; no implementation security claim was made from it. Follow `runtime/permissions/lib.rs` and brokers before capability decisions.
- Uzu engine Cargo/backends show CPU/Metal conditional implementation in selected source files, not working CUDA PTQ1.
- GPU_ACTIVE upstream PlxNative result must never be renamed to blob.in physical measurement.
- The first bwrap failed command was a broken mount fixture (absent dynamic loader); preserved alongside corrected positive test.
- The host negative/positive bwrap fixture never calls MXC's SDK; **MXC policy and audit semantics remain untested**.
- Source licenses: GPUI Kit Apache-2.0 source/selected CC BY 4.0 docs; Hyperfine Apache/MIT grant files. No license approval for transitive content, embedded fonts/icons, or model weights.

## Publication and authority
Only the **private Git research branch** may be pushed. No production, core engine, main-branch merge, submodule, vendor, partner system, public repo visibility, API contract, npm package or hosted site changed in this tranche.

## Open gates
- W01: MXC SDK exact policy deny/audit fixture — NOT_RUN.
- W02: physical GPU oracle independent of reference platform — NOT_RUN.
- W03: Deno Rust permission broker runtime test — NOT_RUN.
- W04: Wayland GPUI accessible focus/IME surface — NOT_RUN.
- W05: Hyperfine v2 binary and calibrated controlled runs — BLOCKED_TOOL_NOT_INSTALLED.
- W06: small Uzu numerical/CPU fixture; CUDA/PTQ1 compatibility — NOT_RUN.
- W10: public website docs build/serve qualification — NOT_RUN.
- W11: Git remote equality verified after push (must be checked separately).
