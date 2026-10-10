# GPU05-PACER-05 — Controlled same-output startup A/B

**State:** CONTROLLED_CONFIGURATION_AB_OBSERVED / 75MS_STARTUP_DELAY_NOT_EFFECTIVE_IN_PAIR / FRAME1_DISCARDED_IN_BOTH / CP03_OPEN.

Branch: `feat/blobin-gpu05-pacer05-20261010`.
Parent baseline: `d2a472132c10f702f5e83a6cbba3ca7151065b11`.
All activity local in this separate worktree; no merge, push, compositor configuration change or stable renderd mutation.

## Hypothesis

PACER-04 exposed a 31.475ms CPU call to `queue.present` for startup frame #2 in one eight-second HDMI-A-1 run. Other runs showed first-frame discard. We hypothesized that adding a bounded settle period **after scene import, GPU initialization and surface configuration but before the first frame** could alter the initial WSI handoff without excluding frames from the metric corpus.

**Controlled variable:** `--startup-policy immediate` (0ms) versus `--startup-policy settle75` (75ms). No other source or runtime configuration parameter was deliberately changed.

## New code / law gates

- `src/startup_policy.rs` restricts the control to two exact values and exposes a bounded delay; **2 new Rust tests PASS**.
- `src/bin/gpu05_native_vulkan.rs` adds the opt-in startup policy. The delay is after the same `Live::create` setup and before frame #1; no warmup frame, shader or MeshSolid logic modified.
- `scripts/gpu05_pacer05_compare.py` checks each pair for the same Wayland output, size, monitor mode, refresh, real window geometry, same-surface stage/callback/presentation identities, equal declared fixture and binary fingerprints, and explicit startup policy. It never emits CP03 PASS.
- `scripts/test_gpu05_pacer05_compare.py`: **5 newly introduced comparison law tests PASS** for rejected mismatched controls, origin and data typing.
- Strict Rust Clippy `-D warnings` PASS; no already-closed GPU01–PACER04 test suite repeated.

## Safe execution contract

`gnu6-lab-run --check`: GNU6_LAB_READY, memory.high=18GiB,
memory.max=22GiB, swap.max=2GiB.

One `gnu6-lab-run --exclusive` scope compiled **one identical Release binary** and ran a sequential controlled pair, each seven seconds, with no 73/75 retries. This does not mean completely identical background load: the runs were sequential and have unavoidable time-dependent host/compositor variation.

Binary SHA-256: `063bc4e218247dd18a72d2d85af80836ce383aab33ebdb91881692999050ac91`.

Scene-stream fixture SHA-256: `8837d1e23dc59d4e01386efe7dda775f516262946c1559858d13052f268b6c82`.

Shader hashes: `surface_live.wgsl=5da8efc9e669442ff7aca87c1a05ea7689766a98891673dab94d42f58249749a`; `present.wgsl=e77298b8a2265e76306406b6cda5a19b90169b02bc67494c2a53e15be517803b`.

Both runs were physically observed as native Wayland windows:
`HDMI-A-1`, Hyprland monitor=0, 2560×1440 fullscreen=2, at [-630,590], compositor advertised 74.964Hz; `wp_presentation` receipts reported `refresh_ns=13,339,736` for both.

Evidence:
`/mnt/workbench/evidence/runs/blobin-next-01/gpu05-pacer05-ab-20261010/`.
Within `immediate/` and `settle75/`, preserve `run.log`,
`monitors.json`, `target-window.json`, `presentations.jsonl`,
`callbacks.jsonl`, `stages.jsonl`, `analysis.json`,
`traces.sha256`. Pairwise control result: `comparison.json`.

## Physical observations

| Metric | immediate | settle75 |
|---|---:|---:|
| Submit | 527 | 525 |
| Compositor-presented | 526 | 524 |
| Discarded | **1** | **1** |
| Discarded frame sequence | **1** | **1** |
| Present average FPS | 74.962 | 74.821 |
| Presentation interval p95 | 13.458ms | 13.437ms |
| Presentation interval p99 | 13.579ms | 13.534ms |
| Frame #2 CPU `queue.present` | 0.3501ms | 0.5084ms |
| WSI timeouts / occlusions | 0 / 0 | 0 / 0 |
| Crash / unresolved callback/feedback | 0 / 0 | 0 / 0 |

Direct stage observation:
- A immediate frame #1 `after_queue_present` occurred at 0.4805ms since host ready; it was **discarded**.
- B settle75 frame #1 `after_queue_present` occurred at 75.7611ms since host ready; it was **discarded**.
- Frame #2 was presented in both.
- The original PACER-04 anomalous 31.475ms `queue.present` on frame #2 did **not recur** in either new run; this is a short sample, not proof that startup stalling cannot recur.
- Live Wayland callbacks match feedback sequences in each variant. No GPU-05 process remained open after the pair.
- Hyprland PID `2544527` remained alive; native `blobin_renderd` PID `1974196` remained alive.

## Follow-on PACER-06

A new same-output/same-binary A/B of current startup versus waiting
for the first wp_presentation outcome changed frame #1 from discarded
to presented (451/451 presented and zero discarded in the gated run).
This is one short positive pair, not independently replicated; CP03
remains OPEN. See GPU-05-PACER-06-STATUS.md for complete evidence.

## Falsification verdict and next experiment

**Negative for the naive delay hypothesis in this A/B pair:** a fixed 75ms idle after configuring the GPU surface does **not** prevent the first-frame compositor discard, and no material throughput improvement is evident in the two samples.

Do not conflate a *configuration-controlled sequential A/B* with replicated statistical causal proof: no randomization, independent load controls or multiple matched pairs were performed, intentionally to limit stress.

**PACER-06 proposed (not implemented here):** audit the first-buffer state machine across initial XDG configure/ack, surface mapping, swapchain configure and first Vulkan presentation. Instrument exact Wayland `wl_surface.enter`, `xdg_surface.configure`, output association and pre-first-commit order on the *same native client*. Test an actual protocol-order intervention (not an arbitrary sleep), then one new bounded physical sample. Keep first-frame discard in the acceptance corpus.

CP03 HYBRID-60 remains OPEN: no 120s real compositor experiment, no calibrated input-to-display oracle, CP02/07/08 outstanding and no CUDA-05F supervised admission for this direct Vulkan prototype.
