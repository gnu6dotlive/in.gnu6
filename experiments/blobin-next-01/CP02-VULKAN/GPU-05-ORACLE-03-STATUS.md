# GPU05-ORACLE-03 — Typed Wayland pointer provenance into in.gnu6 projection

**Date:** 2026-10-10 (EDT)
**State:** SOURCE_TYPED_PROJECTION_PASS / NEW_TARGETED_LAWS_PASS / PHYSICAL_RUN_BLOCKED_CODE75 / REAL_POINTER_NOT_OBSERVED / CP03_OPEN
**Branch:** `feat/gpu05-semantic-pointer-projection-20261010`
**Parent baseline:** `a88924a8cd507c8835b54031b7ec0ee1a67f6fac` (ORACLE-02)

## Precise question

ORACLE-02 mapped real wl_pointer.button events to a renderer-local 350ms white ROI,
but had no coordinates or scene identity. A mouse press anywhere on that owned
surface could request the ROI. That proves no in.gnu6 semantic targeting.

ORACLE-03 closes the **coordinate/projection source gap** without granting
any scene mutation capability or inventing a physical click.

## Native observer change (compiled, not physically rerun)

- `owned_wayland_presentation.rs`: exact `wl_pointer.enter` and `motion`
  `surface_x`/`surface_y`, with finite bounded coordinates.
- Pointer leave/capability loss clears previous positions. Button events
  without an owned-surface position fail closed; only BTN_LEFT when inside
  our exact wl_surface emits an event.
- A monotonic provider-sequence number is attached to each accepted button.
- Optical JSONL emits the surface token, pointer provider, event sequence,
  logical pixel coordinates, Wayland compositor time and separate host time.
  These remain **software observations**, not physical input capture.
- A new targeted Rust law for finite/bounded coordinates passed; strict
  offline Rust Clippy for the GPU05 binary passed.

## Reuse of in.gnu6 ontology instead of a parallel engine

- `src/interaction/wayland-optical-projection.ts` reuses the existing
  `InteractionProjector`, `SpatialFrameGraph` and topmost semantic pick.
- An explicit affine surface-to-world chart is required; no guessed
  calibration is embedded into the renderer, and a chart's mere presence
  does not prove real physical calibration.
- Scene binding includes exact RendererFrame sequence, visible control
  identity, screen extent and surface token. It rejects stale sequence,
  foreign provider, reused pointer numbers, out-of-bounds screen samples,
  nonfinite/singular charts and unsupported/rotated geometry.
- The pure projection returns provenance, exact entity hit and an
  `exactTargetObserved` boolean. It carries **no authority to mutate**
  scene identity, topology, product state or GPU scene geometry.
- Tests import the genuine `scene-stream.v1.json` initial frame, with a
  **test-declared** mapping of its stage bounds to the 2560x1440 fixture
  viewport. Centre maps to `cavity-control`; misses map to `stage`.
  This mapping is NOT a measured display-to-scene chart.

## Targeted verification

| Check | Result |
|---|---|
| TypeScript compile (`tsc --noEmit`) | PASS |
| Biome formatting/lint of new files | PASS |
| 10 new semantic projection laws | **10/10 PASS** |
| Rust formatting, Clippy -D warnings | PASS |
| New finite/bounded native pointer law | **1/1 PASS** |
| Real native ORACLE-03 Release+window smoke | **BLOCKED, NOT RUN** |
| Human click with correlated surface coordinates | **NOT OBSERVED** |
| Real optical ROI click → compositor-presented bright frame | **NOT OBSERVED** |
| Physical input-to-photon capture with dual sensors | **NOT RUN** |
| CP03 HYBRID-60 complete | **OPEN** |

## GNU6 lab isolation and blocked experiment

`gnu6-lab-run --check` returned GNU6_LAB_READY, memory.high 18GiB,
memory.max 22GiB, swap.max 2GiB. One proposed Release build plus
25-second native window run was submitted via `gnu6-lab-run --exclusive`.
The guard rejected the request with **exit 75: another heavy exclusive job
was active**. The prohibited retry was NOT attempted. No new Release
build, compositor frame measurement, physical click, restart or GPU
benchmark occurred in ORACLE-03.

Read-only inspection found independent concurrent scopes. Their owner was
not conclusively attributed from that process snapshot; no process was
killed and no guard, systemd-oomd or Hyprland configuration altered.
Stable renderd and Hyprland remained active at the last check.

## Reconciliation with historical evidence

- ORACLE-02 at `a88924a`: **749 compositor-presented frames / zero
  discarded / no human button in its 10-second idle smoke**.
- ORACLE-01 at `79eed4e`: strict synthetic dual-channel photodiode
  protocol and verified historical SHA-256 reconciliation, **no hardware**.
- ENDURANCE-01: 9,370 presented over 124.994s, zero discarded,
  one NVIDIA reference sample, not an input-to-photon oracle.
- No historical closed GPU benchmark was repeated.

## Next falsifiable actions, after new lab admission

1. Get a *new* green exclusive grant (without auto-retrying code 75),
   perform an isolated Release build and a short live Wayland window test.
2. With an operator actually present, click/release the real experimental
   window; confirm owned-surface coords, provider sequence and 350ms
   bright/dark cycle have same-surface compositor receipts.
3. Establish and independently verify the real surface-to-world chart;
   connect the typed observation **live** across the source boundary.
   The current TypeScript adapter is deterministic, but **not wired
   into the runtime**: do not claim the click is semantically routed live.
4. Connect/calibrate shared-counter switch and photodiode hardware to
   qualify photons, rather than conflating `wp_presentation` with light.

**Verdict:** ORACLE-03 source and targeted laws qualified; live gate
BLOCKED_BY_LAB_LOCK, all CP03 full acceptance gates remain OPEN.
Repository remains private, package remains private/UNLICENSED.
