# GPU05-PACER-06 — Initial Wayland presentation outcome barrier

**State:** FIRST_FRAME_GATE_SINGLE_MATCHED_AB_POSITIVE / FIRST_FRAME_DISCARD_ZERO_IN_VARIANT / INDEPENDENT_REPLICATION_PENDING / CP03_HYBRID_60_OPEN.

Branch: `feat/blobin-gpu05-pacer06-20261010`. Parent: `efeb0777a3b547b3fe3923ff3b80ff9a931fd440`. No merge/push/production change.

## Why this checkpoint was needed

PACER-05 falsified a 75ms fixed startup delay: first frame discarded in both same-monitor runs. The native window already sends an initial empty wl_surface.commit, waits for an xdg_surface.configure + ack, and configures Vulkan before presenting its first image. However, a wl_surface.frame callback may allow frame #2 before any wp_presentation outcome for frame #1. A callback gives permission to produce more work; it does **not** prove physical presentation.

## New implementation and controls

- `src/first_frame_barrier.rs`: `current` versus `await-first-feedback`; exactly one first-frame submission and one compositor outcome, a 250ms fail-closed timeout, no foreign/duplicate/replayed feedback, no promotion of callbacks to presentation. Both presented and discarded outcomes are recorded; discards are **not exempted**.
- `src/bin/gpu05_native_vulkan.rs`: one CLI switch `--first-feedback-gate` and explicit first frame outcome. Before frame #2, optionally wait for actual resolved wp_presentation sequence #1 (but continue normal callback pacing afterward). Guarded shutdown retains all feedback and callback reconciliation.
- `src/owned_wayland_presentation.rs`: additional first-initial-commit, first XDG configure acknowledgment, post-configuration Vulkan readiness and first-feedback-arm log receipts on the **same owned Wayland connection**.
- `scripts/gpu05_pacer06_compare.py`: rejects mixed output/window configuration, missing XDG lifecycle, unqualified startup mode, nonmatching callback/presentation/stage identity or incomplete run; never claims CP03 PASS.
- **7 new Rust law tests PASS**, plus **3 new Python comparator tests PASS**. Strict Clippy `-D warnings` PASS. Historical closed test suites were not rerun.

## GNU6 lab protection and exact execution

`gnu6-lab-run --check` returned `GNU6_LAB_READY`, with memory.high 18GiB, memory.max 22GiB and swap.max 2GiB. All light edits/checks ran under the launcher. One new **`gnu6-lab-run --exclusive`** scope compiled the exact Release binary once, then ran a short sequential A/B pair. No 73/75 retries, GPU reset, restart of Hyprland, service change, or 120-second run.

Binary SHA-256: `f4984d776bff3a972a2a5da2722149f200a5e11484d6487e76dcef9358f198e8`.

Scene fixture SHA-256: `8837d1e23dc59d4e01386efe7dda775f516262946c1559858d13052f268b6c82`.

Both windows were verified physically on the **same HDMI-A-1** monitor: 2560×1440, 74.964Hz, XDG fullscreen, Hyprland monitor index 0, position [-630,590]. The only intentionally changed runtime parameter was `--first-feedback-gate`, with `--startup-policy immediate` in both cases.

Evidence: `/mnt/workbench/evidence/runs/blobin-next-01/gpu05-pacer06-ab-20261010/` with separate `current/` and `await-first-feedback/` run logs, same-surface `wp_presentation` JSONL, `wl_surface.frame` JSONL, CPU stage JSONL, actual window/monitor inventories, hashes, per-run analysis and `comparison.json`.

## Observed physical comparison

| Metric | current | await-first-feedback |
|---|---:|---:|
| Submitted | 452 | 451 |
| Physically compositor-presented | 451 | 451 |
| Discarded | **1** | **0** |
| First frame outcome | **DISCARDED** | **PRESENTED** |
| Approximate average presented FPS | 74.964 | 74.797 |
| Presentation p95 | 13.405ms | 13.394ms |
| Presentation p99 | 13.466ms | 13.446ms |
| Maximum observed compositor presentation gap | 13.635ms | **26.689ms** |
| Unresolved feedback or callback after shutdown | 0 | 0 |
| WSI timeouts / aborts | 0 / 0 | 0 / 0 |
| First-frame policy wait iterations | 0 | 6 |

Both runs ~6s and completed normally with zero compositor/worker restarts. The corrected variant did not omit the first frame from its oracle; it was genuinely marked **presented** by `wp_presentation`. The later maximum 26.689ms gap in the corrected variant is an explicit startup pacing cost and must be retained in long-run qualification.

Initial event logs confirmed:
`GPU05_PACER06_INITIAL_EMPTY_SURFACE_COMMIT` →
`GPU05_PACER06_XDG_FIRST_CONFIGURE_ACK` →
`GPU05_PACER06_CONFIGURED_BEFORE_VULKAN_SWAPCHAIN` →
`GPU05_PACER06_FIRST_FRAME_FEEDBACK_ARMED` →
`GPU05_PACER06_FIRST_OUTCOME`.

The actual first-feedback outcome in the current policy is `discarded`, and in the gated policy `presented`. **This is one positive matched A/B pair, not independent statistical replication or a proof of root cause inside the compositor**. Compositor load and exact internal frame scheduling were not instrumented.

### Proof identifiers

Current `wp_presentation` JSONL SHA-256:
`f775045d5b591b91bc9bf21970fa164d2479f4c85abc1472bb32ecc919a53023`.

Gated `wp_presentation` JSONL SHA-256:
`0de37a6775b54497341ceaf0725c09ec68bf17edf6fde86320db6249ae7f8bd6`.

Current / gated callback JSONL:
`6f63cd38e37139ca4879fe6bdf51c0820acc0604d42d453812bc110b1d41946a` /
`366f959e0aaf50c958733cecaa8cc87bffbfac18daa3bb86bb19c6b1a8d347e3`.

Current / gated CPU stage JSONL:
`066e8aa0168542d2a0c89f4f3f2bd9dd0fe0865d571d309405c1780f9ec864ea` /
`cc81422276e3d43172deb27747980621cef94382c92fe2fd3ff1c2861b06716f`.

## Next checkpoint — PACER-07

Independently repeat the **matched same-binary startup A/B** at the same pinned monitor and workload, ideally reversing variant order, to test whether elimination of frame-1 discard is reproducible. If validated, inspect alternative monotonic compositor-ready signals / Wayland configure ordering and evaluate a production-admissible startup state machine with explicit bounded authority. Measure startup first-frame latency and 120s steady-state **separately**.

The GPU05 experiment still is standalone direct-wgpu and does not inherit supervised CUDA-05F privileges. CP02 and CP03 HYBRID-60 (120s and input-to-photon oracle), CP07 and CP08 remain OPEN. Never promote this short 6-second pair to CP03 PASS.

Hyprland remained PID 2544527, and `blobin_renderd` PID 1974196 remained running. No parallel worktree, `main`, deployment, service or compositor configuration was changed.
