# GPU05-PACER-02 — bounded adaptive presentation pacing

**State:** PHYSICAL_12S_LOCAL_PASS_FOR_BOUNDED_TWO_FLIGHT / NOT_CP03_PASS.
**Branch:** feat/blobin-gpu05-pacer02-20261010 (isolated worktree).
**Base:** GPU-05 commit 9410e54d68a05efdc0fe3b21debcadc796c093aa.
**Preserved:** all previously ratified GPU01-05 evidence, tests, parallel branches,
Hyprland configuration, live renderd, BLOB-FRAME-v1.

## Why a new pacer

The prior one-in-flight local run produced 419 compositor-presented frames
in 11.001s (about 38Hz) and no discards. The independent compositor receipts
advertised ~75Hz, but observed intervals were commonly two refreshes.
This is a performance RED, even though it removed discarded frames locally.

## Implemented on this separate branch

- src/frame_pacing02.rs: deterministic pure two-in-flight gate, learned
  compositor refresh_ns from actual wp_presentation receipts, conservative
  fallback 13,333,333ns before the first feedback, bounded 250ms watchdog.
- Admission requires fewer than two pending requests AND at least the
  last compositor refresh interval since the preceding submission.
  Invalid/unknown refresh values do not override the fallback.
- Native Wayland GPU host feeds resolved actual feedback and successful
  submitted-frame times back to the pacing gate; submission scheduling
  is distinct from compositor frame confirmation.
- State checks remain bounded: pending cap two, feedback age cutoff 250ms,
  GPU WSI timeout and occlusion reporting from GPU05, bounded shutdown
  drain and explicit CP03_NOT_QUALIFIED.
- New focused deterministic Rust tests: 6/6 PASS for 75Hz, 165Hz,
  pending saturation, delayed feedback, invalid refresh, clock reversal.
  Prior closed test suites were NOT rerun. Strict Clippy PASS.

## Exactly one protected physical experiment

Launcher preflight: GNU6_LAB_READY (memory.high=18GiB, max=22GiB,
swap.max=2GiB). New Rust Release build plus one 12-second physical
Vulkan/Wayland run executed inside gnu6-lab-run --exclusive.
No retries were attempted or needed.

Evidence directory:
/mnt/workbench/evidence/runs/blobin-next-01/gpu05-pacer02-20261010/

Binary SHA-256:
39d2101c790d837fb41fcf5f1d13689ae0b25d5e4df92304ce5b01e37d2f6509
Same-surface wp_presentation JSONL SHA-256:
cad6587769d4ef95185d7fc5f4a3fdaa19480685fb89e980f8bcaedbef8b9738

Physical NVIDIA RTX 3070/Vulkan; Wayland owned surface. The experiment
ended normally and the compositor stayed active.

- Runtime: 12.011 seconds.
- 1,320 submitted / 1,320 compositor-presented frames.
- 0 discarded, 0 unresolved on shutdown.
- max_inflight=2; 0 WSI timeouts, 0 occluded frames, 0 pacing stalls.
- Presentation trace span 12.000020s and average ~109.92 frames/sec.
- Compositor p95 12.160ms, p99 12.210ms, maximum 12.406ms.
- Every presentation receipt advertises refresh_ns=6,063,656
  (about 164.92Hz reference display).
- Raw interval histogram: 659 intervals under 8ms and 660 intervals
  between 11 and 14ms, revealing an approximately alternating
  one-versus-two-vblank cadence.
- No independent input-to-photon oracle, no 120-second dwell, no
  sustained memory attribution; analyzer verdict NOT_QUALIFIED.
- Hyprland PID 2544527 survived; stable blobin_renderd PID 1974196
  survived; no new systemd-oomd kills observed in the latest log.

## Critical comparability warning

The prior 38FPS run was on an ~75Hz monitor and the new 110FPS run
was on a ~165Hz monitor; the native window was also resized.
Do not claim a same-monitor speedup factor or a 165FPS achievement.
The new run provides a real independent proof of >60 presented FPS
on this 12-second sample, with zero discarded, but is NOT a CP03
HYBRID-60 acceptance pass.

## Remaining blockers / next experiments

- GPU05-PACER-03: measured wl_surface.frame callback or a predictive
  refresh scheduler, with proper same-surface feedback attribution,
  to avoid the observed alternating-vblank cadence.
- Run an A/B only after selecting the same monitor, window extent,
  workload, session and compiler build settings.
- Add actual independent input-to-display latency measurement.
- Full CP03 120-second, p95/p99, frame-drop and distribution/license/
  lifecycle gates remain open. CP02/CP07/CP08 remain separately open.
- Direct wgpu Vulkan demonstration does not inherit CUDA-05F
  supervised worker authority.
- Never bypass gnu6-lab-run or disable systemd-oomd.

No push, merge into main, deployed unit update or compositor restart.
