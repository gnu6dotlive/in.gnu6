# GPU05-ORACLE-03-PHYSICAL-01 — Owned Wayland 24s replay and max-gap falsification

**Date:** 2026-10-10 · **Verdict:** NATIVE_PIPELINE_OBSERVED / ONE_MAX_GAP_VIOLATION / NO_REAL_POINTER_INPUT / CP03_OPEN
**Branch:** `feat/gpu05-semantic-pointer-projection-20261010`
**Source baseline:** `498c18b619f747d13d6132474dd86719035c9c0d`

## Authority and safety

A first ORACLE-03 GPU test was refused by GNU6 isolation with code 75.
Following a NEW explicit operator `go`, a fresh single `gnu6-lab-run --exclusive`
admission succeeded. There was no automated retry of the prior rejected
operation and no guard bypass. A new cold offline Rust Release build was
constructed in its own CARGO_TARGET_DIR; existing PACER and endurance
binaries, other worktrees, Hyprland and production renderd were untouched.

Reference: NVIDIA RTX 3070, native Wayland Vulkan, HDMI-A-1,
2560×1440 fullscreen, `--startup-policy immediate`,
`--first-feedback-gate await-first-feedback`, optional optical witness.
Running time ~24.024s. The actual compositor window geometry was observed
and its PID logged before measurements. Both Hyprland PID 2544527 and
stable renderd PID 1974196 survived.

## Experimental result

| Dimension | Measured outcome |
|---|---:|
| Submitted / compositor-presented | 1,769 / **1,769** |
| Compositor discards | **0** |
| Outstanding compositor feedback / callbacks at shutdown | 0 / 0 |
| Timeouts / watchdog / abort | 0 / 0 / false |
| Mean actual compositor-presented FPS | **73.671** |
| wp_presentation interarrival p95 / p99 | 13.398 / 13.476 ms |
| **Longest real compositor interval** | **413.544 ms — RED** |
| Compositor gaps >33.4ms | **1** |
| First frame | Presented |
| Wayland BTN_LEFT events in trace | **0** |
| Bright optical trials | **0** |
| Physical input-to-photon | NOT RUN |
| Scene projection of real pointer | NOT OBSERVED / NOT LIVE WIRED |
| CP03 HYBRID-60 full | **OPEN** |

Native optical witness output correctly reported
`NO_COMPLETE_WAYLAND_OPTICAL_TRIAL`; no claim of human interaction or
physical photon observation is admissible. Neither the 125s endurance
checkpoint nor PACER-07 was rerun.

## Causal diagnostic — one long gap, origin still unknown

The new source-only read-only `gpu05_oracle03_gap_audit.py` correlates
frame sequences across two independent **within-clock** oracles, never
subtracting Wayland compositor timestamps from host CPU timestamps.
Seven new negative law tests PASS, including cross-surface, rollback,
missing-stage, fake-header and exact max-gap falsification.

- Compositor presentation interval **frame 1501 → 1502:** 413.544 ms.
- Host queue-present **frame 1501 → 1502:** 435.418 ms.
- Host `after_queue_present → frame_callback_observed` for
  **frame 1501:** 434.994 ms.
- Host `after_queue_present → feedback_observed` for
  **frame 1501:** 434.992 ms.
- Largest host submit-to-feedback observation in this region:
  448.042 ms for frame 1500.
- Second-largest compositor gap: 26.700 ms (startup).
- All frame identities and callbacks reconciled at graceful shutdown.

These observations are **consistent with a delayed compositor-event
delivery and a pause in the callback-paced submission loop**.
They do NOT determine whether the underlying cause was scheduler pressure,
the Wayland compositor, the display/GPU driver, or the client's event loop.
No systemd-oomd kill was found for the relevant run window.
Record this as a REAL NEGATIVE and do not average it away with the p95.

## Durable evidence and source

Private out-of-Git evidence directory:
`/mnt/workbench/evidence/runs/gpu05-oracle03-physical-20261010/`

Release binary SHA-256:
`95f4effd5d6fca107bbd030ad6855bdec6427db73568afe34c4e0b83ce2f78da`

Gap audit JSON SHA-256:
`e6e81b949ad519afb380e394d72fcfb0788203df52556a94ae45124327ee0310`

Evidence archive: `build.log`, `run.log`, `binary.sha256`, `native-sources.sha256`,
`source-head.txt`, `monitors.json`, `window.json`, `optical-events.jsonl`,
`presentations.jsonl`, `callbacks.jsonl`, `stages.jsonl`,
`compositor-analysis.json`, `optical-gate.json`, `gap-audit.json`,
`traces.sha256`, `hypr-before.txt`, `hypr-after.txt`,
`renderd-before.txt`, `renderd-after.txt`.

## Remaining gates and next falsification

1. Study the 435ms event-loop/compositor feedback pause under a
   controlled, new and specifically approved hypothesis; compare
   scheduler/preemption/compositor telemetry without affecting Hyprland.
2. Schedule a **human-present** short native click/hold/release test
   only when the operator can actually interact with the window.
3. Correlate owned-surface coordinates to an independently verified
   surface-to-world calibration. The existing TS projection is
   deterministic, **not connected live to Rust**.
4. Independent button-contact and photodiode sensors with one
   calibrated timebase are needed to test true input-to-photon.
5. CP03 full, cross-vendor, package/public API, supervised CUDA-05F
   remain separate unqualified contracts.

**Explicit result:** physical compositor presentation path works for
the recorded sample and has no discarded feedback; the max-gap timing
gate is RED, no input click was observed and CP03 is OPEN.
