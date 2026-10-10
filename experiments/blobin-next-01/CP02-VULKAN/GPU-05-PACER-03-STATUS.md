# GPU05-PACER-03 — Native Wayland frame callbacks and presentation receipts

State: CALLBACK_ORACLE_AND_TERMINAL_ACCOUNTING_LOCAL_PASS; CP03_HYBRID_60_OPEN.
Branch: feat/blobin-gpu05-pacer03-20261010; base commit b91092d.
Only isolated Rust/wgpu 30 Vulkan on the reference NVIDIA RTX 3070.
Unrelated worker worktrees and stable blobin_renderd remain untouched.

## Changes

- owned_wayland_presentation.rs now attaches wl_surface.frame callbacks to
  the exact wl_surface used by Vulkan WSI and wp_presentation.feedback.
  It uses the same Wayland connection and event queue, carries frame IDs,
  bounds outstanding callbacks to two and rejects duplicate/foreign IDs.
- frame_pacing03.rs separates callback scheduling credit from independent
  compositor-presented evidence; has a single bootstrap credit,
  two outstanding wp_feedback maximum and 250ms fail-closed watchdog.
- gpu05_native_vulkan.rs accepts a distinct --callback-jsonl trace.
  Terminal shutdown drains late callbacks into the pacing policy without
  submitting any new frames and rejects unresolved callback identities.
- Geometry, signed tetra mesh, SDF shader and semantic source authority
  remain unchanged. No modification to CUDA-05F or production service.

## Tests

gnu6-lab-run --check returned GNU6_LAB_READY.
Light commands used the launcher, and each optimized-build / GPU run
used --exclusive. No automated retry on 73/75 or Hyprland restart.
Strict Clippy -D warnings PASS. Seven new pacing law tests PASS,
plus one focused late-terminal-callback test PASS.
Closed GPU-01/02/03/04, GPU05 and PACER-02 tests were not rerun.

## Physical sample A: ~165Hz monitor, 12 seconds

Evidence directory:
/mnt/workbench/evidence/runs/blobin-next-01/gpu05-pacer03-20261010/

Binary SHA-256:
106642de8f8a728d6ead0d2bae14bb840b4c45c26cc292b3bf3259429d536b4f
wp_presentation trace SHA-256:
ed427ec436affaa0047fc02eec5a5a3e2bb55b954bb7e2f65236990f209114ea
wl_surface.frame callback trace SHA-256:
14151c50961f55c5b94d9a5b5f66fa065425cf1a1ca48bd27384bc83ff5b5f8f

- 1,322 submitted and 1,322 compositor-presented frames; zero discarded.
- 1,322 distinct callback identities match 1,322 feedback identities,
  exactly the sequences 1 through 1322, with no extras or duplicates.
- Compositor refresh_ns=6,063,656; presentation span=12.006043s.
- Average 110.028 presented FPS, p95=12.156ms, p99=12.204ms,
  max=12.421ms.
- About 662 ~6ms intervals and 659 ~12ms intervals: the alternating
  one/two-refresh pacing defect REMAINS. No material speedup versus
  PACER-02 ~110FPS may be claimed.
- No WSI timeout, occlusion, watchdog trip or compositor restart.
- One last callback was received in the native-host trace but not
  yet drained into the pacing policy at original shutdown; corrected
  afterward, without altering this historical sample.

## Physical sample B: targeted terminal accounting, ~75Hz, 3 seconds

Evidence directory:
/mnt/workbench/evidence/runs/blobin-next-01/gpu05-pacer03-shutdown-20261010/

Binary SHA-256:
f10d087a2d9b747e23fd720536a91fe94085c728d49ed4a63d8b1056287ca64f
wp_presentation trace SHA-256:
0f354bdcccbfd51c5946c19cf3cbd0dffb7c308e7de39891e7de573ad97ffb8a
wl_surface.frame callback trace SHA-256:
6970ff090f5002e117c09acbe46d8141fe265124fd54ae64fc913ef1d86c6300

- Normal exit; 153 submitted, 153 callbacks, zero callbacks pending
  and zero compositor feedback objects pending at process shutdown.
- 152 compositor-presented and ONE discarded frame, sequence 1.
  This is an explicit RED for zero-discard acceptance, possibly a
  first-frame/cold-window issue but not causally qualified.
- Compositor refresh_ns=13,339,736 (~74.96Hz); 3.00164s trace,
  average 50.306 presented FPS; p95=26.750ms, p99=26.875ms,
  maximum 26.942ms.
- Exact ID partition 1..153 across two independently logged
  native protocol event types; terminal policy counter also agrees.
- No GPU WSI timeout or systemd-oomd killing; Hyprland retained
  PID 2544527.

## Qualification limits and next gates

The two samples used different output refresh rates, window geometries
and durations. They are NOT a controlled A/B and show no proven
pacing throughput improvement. Callback timestamp milliseconds
are not converted to wp_presentation nanoseconds.

Pacer-04 should investigate why FIFO/AutoVsync/WSI-compute/display
still alternate one/two vblank and why frame #1 was discarded at 75Hz.
Pin the output monitor, window extent, source workload and evidence hashes
before any next A/B. Do not silently exempt discarded first frames.

The full 120-second CP03 run, independent input-to-photon oracle,
all CP02/CP07/CP08 gates and CUDA-05F supervised admission remain OPEN.
No merge/push/publication, service edits or compositor restarts.
