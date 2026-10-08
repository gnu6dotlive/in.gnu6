# BLOBIN-NEXT-01 — Program of independent Rust GPU/CPU development spikes

Status: **CONTRACTS_READY / WORKERS_NOT_STARTED / PRIVATE**. The operator's product direction is Rust-owned, GPU-first, CPU-compatible. These are **new clean-room R&D proofs**, not copying ambiguous-license snapshot code, not production features, not new main quests of in.gnu6.

Read: root `AGENTS.md`, `contracts/TRONC_COMMUN.md`, `CONTRACT.md`, `QUESTLOG.md`, then the blob.in dossier `FUTURE_ENGINE.md`, `EXPERIENCE_ATLAS.md`, `TECH_RADAR.md`, BLOBIN-NEXT-01 `00-BASE.md` and your specialist contract. Snapshot `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in` is READ ONLY; use it for evidence, never copy its code while GPL/AGPL authority is unresolved.

All eight workers run in separate branches/worktrees based on a **frozen SHA containing this contract batch**, in the private repository. Writes are restricted to independent `docs/agent-results/blobin-next-01/<ID>.md` and `experiments/blobin-next-01/<ID>/**` paths. The latter are private clean-room prototypes, not blob.in product source; the standalone blob.in upstream repo and final license must be qualified separately. All new code remains non-distributed.

| Worker | Responsibility |
|---|---|
| G01 | Topology & signed fields |
| G02 | wgpu/WGSL GPU-native prototype |
| G03 | CPU-complete reference renderer |
| G04 | Material graph / bevel / heat math |
| G05 | Post-process and resilience graph |
| G06 | Topology-first UX experiments |
| G07 | Gnosix GPUI / host adapter research |
| G08 | Qualification / benchmark / fidelity harness |

No `git push`, external package publication, VPS or public docs release, production Gnosix changes or upstream snapshot mutation. Dependence resolution `cargo --offline` is preferred; build target uses a bounded per-worker folder under /mnt/workbench, **never /tmp**, which was nearly full at intake. Never install OS/driver packages from a worker.

**Cross-worker handoff:** a shared typed proposal is written as a fixture spec or test manifest; no worker imports sibling scratch code directly. The integrator compares actual implementations and settles contradictions. Initial preferred challenge = **E02 + E05 + E11**: topology bloom with real holes, focus relief height/bevel, and GPU-loss CPU continuity.

Every worker reports exact SHA, source refs, run commands, tests, traces, hardware/driver when applicable, observed vs designed assertions, adversarial failures and next gate. A good still image is not a topology proof. `dispatch.json` prepares tasks; it does not execute them.
