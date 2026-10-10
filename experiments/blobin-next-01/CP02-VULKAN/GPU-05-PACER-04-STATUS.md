# GPU05-PACER-04 — Wayland output pinning and WSI stage attribution

State: HDMI_75HZ_SHORT_SAMPLE_PASS / CP03_HYBRID_60_OPEN
Branch: feat/blobin-gpu05-pacer04-20261010
Base commit: 4d4d13710909e488092a84da98238d957d8764f4

## Scope / implemented

- Bind wl_output v4 and exact wl_output.name on the native Wayland connection.
- Optional CLI --pin-output HDMI-A-1 or DP-2, implemented through
  xdg_toplevel.set_fullscreen(selected wl_output) BEFORE initial commit.
  No Hyprland config mutation; selection fails closed when output absent.
- Optional --stage-jsonl records CPU Instant observations of swapchain
  acquisition, command preparation, queue.submit, wp_feedback arm,
  queue.present and asynchronous callback/feedback receipt.
- Stage clocks are NOT GPU execution timestamps or the wp_presentation clock.
  Frame sequence and the owned wl_surface token are the join keys.
- New analyzer and eight targeted synthetic positive/negative tests PASS.
  It rejects missing/duplicate events, cross-surface identity mismatches,
  invalid timestamps and attempts to promote the record to CP03.
- Rustfmt and strict Rust Clippy PASS. Previously closed suites not rerun.

## Safety / deployment boundaries

- gnu6-lab-run --check returned GNU6_LAB_READY; memory.high 18GiB,
  memory.max 22GiB and swap.max 2GiB.
- All light work under gnu6-lab-run. Exactly one new optimized-build
  plus eight-second physical sample under gnu6-lab-run --exclusive.
- No retry on 73/75, no Hyprland restart, no systemd-oomd changes,
  no merge, push, remote deploy, CUDA-05F change or live renderd mutation.
- Hyprland PID 2544527 and native renderd PID 1974196 remained active.

## Physical sample: same native Vulkan wl_surface on pinned HDMI-A-1

Evidence directory:
  /mnt/workbench/evidence/runs/blobin-next-01/gpu05-pacer04-hdmi-20261010/

- Wayland wl_output.name HDMI-A-1 selected via XDG fullscreen protocol.
- Hyprland reported reference monitor ID=0, 74.964Hz, 2560x1440,
  and the experimental client was actually on monitor ID=0 with the
  exact 2560x1440 fullscreen size and coordinates [-630, 590].
- NVIDIA RTX 3070 / Vulkan / signed tetra MeshSolid + SDF shader.
- Runtime 8.025s; 599 submitted; 599 compositor PRESENTED;
  599 matching wl_surface.frame callbacks, zero discarded and
  zero unresolved objects on shutdown.
- Actual compositor refresh_ns=13,339,736; presented FPS=74.839.
- Physical interarrival p95=13.425ms, p99=13.572ms,
  maximum=26.647ms at frame sequence 2.
- No reported WSI timeout or pacing watchdog trip.
- Frame 1 was actually PRESENTED. Do NOT discard warmup frames.

## CPU stage metrics (measured call durations, not GPU timings)

| Stage | Median ms | p95 ms | p99 ms | Max ms |
|---|---:|---:|---:|---:|
| Surface acquisition | 0.0203 | 0.0643 | 0.0864 | 0.135 |
| CPU command preparation | 0.0225 | 0.0568 | 0.0744 | 0.1361 |
| queue.submit | 0.0846 | 0.1754 | 0.2340 | 0.4954 |
| feedback arm | 0.0072 | 0.0142 | 0.0412 | 0.1201 |
| queue.present | 0.0414 | 0.0836 | 0.1582 | 31.475 |
| total CPU path | 0.1930 | 0.3756 | 0.4703 | 31.6956 |

The sole queue.present call exceeding 0.3ms was frame 2:
31.475ms CPU call duration. That exact frame corresponds to the
26.647ms maximum compositor-presented gap. The second-largest
queue.present call was only 0.2352ms (sequence 138).

This locates an unusual startup WSI/presentation call bottleneck;
it does not prove GPU kernel saturation or assign a driver root cause.
No causal comparison is possible with earlier unpinned 75Hz runs,
which used different window geometry, mode, session and duration.

## Evidence identifiers

Release binary SHA-256:
a8f5eaadd14eb11ae9e4d50e523d3b38a03f0791eb9af6f5f8d410231cea89bd

Compositor feedback JSONL SHA-256:
033a64706cb70d0b2186635ea8c80287d6e71008d2e22e25739ead4bb76a8b20

Wayland callback JSONL SHA-256:
e1370cb84dc9ae7c4cd0fe1525c326bac46fa00b8b957729a1ff61cf5e624861

CPU frame-stage JSONL SHA-256:
1b288e8d1e07bbe17ec556bfc2ae2032ffb1fede72f5981a7f1a1b9b92198fa2

Detailed samples and attribution:
  analysis.json
  pacer04-outlier-analysis.json
  target-window.json
  monitors-before.json
  run.log
  traces.sha256
  binary.sha256

## Verdict and next falsification

PACER-04 locally proves 74.839 real compositor-presented FPS,
zero discarded and exact callback identity on HDMI-A-1 for eight
seconds, with stage-attributed first-vblank startup anomaly.

CP03 HYBRID-60 remains OPEN: no 120s dwell, independent calibrated
input-to-photon oracle, independent technical audit or full fault gates.
CP02, CP07 and CP08 remain separately OPEN.

PACER-05 should control output and geometry for BOTH A and B,
vary exactly one startup/pacing variable, retain physical wp feedback
and precise source/binary SHA256. Then investigate the frame-2
queue.present call with a WSI trace and GPU timestamp instrumentation.
Do not infer GPU execution duration from CPU Instant stage timing.
