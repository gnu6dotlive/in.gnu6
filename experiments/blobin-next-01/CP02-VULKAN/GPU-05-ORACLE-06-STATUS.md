# GPU05-ORACLE-06 — Scene-owned 2D visual marker and compositor-buffer witness

**Date:** 2026-10-10 (EDT)
**State:** RENDERED_CONTROL_MARKER_LOCAL_BUFFER_PASS / SOURCE_GEOMETRY_CONTRACT_PASS / REAL_POINTER_HIT_MISS_NOT_OBSERVED / CP03_OPEN
**Baseline:** ORACLE-05 `a51cd429d5f850512cbb12463ddef88f35294cdb`
**Branch:** `feat/gpu05-oracle06-visual-hit-calibration-20261010`

## What changed

The previous native scene projected a 3D SDF/cavity and an independent upper-left optical ROI, but never visibly drew the 2D scene entity `cavity-control`. ORACLE-05's real 20 clicks landing on `stage` were therefore insufficient evidence of visual pick mismatch or cursor error.

ORACLE-06 adds an *explicitly opt-in* 2D overlay marker for `cavity-control`, computed from the qualified scene frame (stage rectangle + control position, shape and scale), using the same shader that presents the Vulkan image. This is a **2D scene-owned control marker**, not a claim that the marker bounds correspond to the independently raymarched 3D cavity or the animated camera.

- Rust `SceneStream.latest_visual_control_uv()` checks scene entity IDs, visibility, roles, rectangular geometry, zero rotation and bounded UV extents. The reference scene sequence 13 yields **[0.47, 0.46, 0.53, 0.54]**.
- `present_optical_control.wgsl` draws this marker as a visible teal rectangle with cyan border on the owned surface. Existing upper-left 350ms optical region and 3D raymarch content remain functionally separate.
- The `--visual-control-overlay` flag requires reference `HDMI-A-1`, qualified scene stream, opt-in optical witness and private semantic observer. The renderer transmits the marker rectangle/viewport/scene identity through the existing local IPC handshake.
- TypeScript independently recomputes the rectangle from the exact scene frame; rejects altered bounds, foreign IDs, stale frame, nonfinite points, changed pixels/viewport and any sender claim of independent display-capture authority.
- The renderer gates pointer processing until it has seen its fullscreen 2560×1440 render viewport. Resizing after verification fails closed. Scene and product mutation authority remain absent.

## New source-level qualification

| Test | Result |
|---|---|
| New Rust scene geometry tests | **2/2 PASS** |
| New TypeScript marker geometry + falsification tests | **8/8 PASS** |
| Historical ORACLE-05 TypeScript laws, narrowly rerun for compatibility | **7/7 PASS** |
| New Python compositor-buffer pixel-gate tests | **6/6 PASS** |
| Targeted Rust Clippy strict, TS typecheck and Biome checks | PASS |
| New Rust Release + Vulkan shader creation | PASS in second run |

## Two physical attempts, separately preserved

1. **BOOT_RACE_REPRODUCED:** First guarded attempt aborted before presentation with `ORACLE06_EXPECTED_REFERENCE_FULLSCREEN_SIZE`. The initial XDG configure was not yet the final fullscreen geometry. No actual Vulkan frame gate could be claimed. Evidence: `/mnt/workbench/evidence/runs/gpu05-oracle06-20261010/physical/`.
2. **MAPPED_FULLSCREEN_PASS:** Corrected source defers the expected reference viewport qualification until the renderer observes its *actual* configured fullscreen size. A second separately admitted, protected run succeeded with the same exact HDMI-A-1 2560×1440 geometry. Evidence: `/mnt/workbench/evidence/runs/gpu05-oracle06-20261010/physical-b/`.

### Successful 28-second reference run

| Observation | Measured |
|---|---:|
| Native Vulkan compositor-presented frames | **2,098** |
| Compositor discards | **0** |
| Pending presentation feedback / callbacks | **0 / 0** |
| WSI timeouts / occluded | **0 / 0** |
| Presented FPS | **74.892** |
| wp_presentation gap p95 / p99 / max | **13.440 / 13.551 / 26.698 ms** |
| Actual monitor/window | **HDMI-A-1 / 2560×1440 / fullscreen=2** |
| TypeScript marker handshake contract | **MATCHES scene sequence 13** |
| Native Wayland clicks received | **0** |
| Actual operator hit / miss | **NOT OBSERVED** |
| Physical photodiode or switch timing | **NOT RUN** |
| CP03 full | **OPEN** |

The compositor screenshot was obtained using `grim -o HDMI-A-1` *after verifying the experimental window fills that monitor*. The image is stored out of Git and has not been treated as a hardware or photographic light observation.

At local reference pixels, its centre RGB was **[89,118,139]** and left/top control border RGB **[111,254,229]**; centre/border L1 distance **248**. The independent read-only pixel gate returned `COMPOSITOR_BUFFER_MARKER_CANDIDATE_PASS` under a verified Wayland window inventory and a matching TypeScript scene marker declaration. This qualifies visible pixels in the compositor framebuffer only; it does not prove physical photons, actual cursor hit accuracy, human input or selection of 3D cavity geometry.

### SHA-256 preserved out of Git

- Native GPU-05 Release binary: `0935357cb9e59bce534836d255ff51783d0e469ec66d3c5a0e69cd97815b37bc`
- Compiled Node receiver: `aa89a25fcabf1a613830478eb53b1997241ad6b03293702afe6a44210b05e88f`
- Private compositor screenshot: `717bdad806a49152cd3fd9c4a8ff7ec387f3f15134b42347f290b1ae53a5ad0c`
- Pixel gate JSON: `9fcf7a1cb707b6096ab75167d913aa242456ca5f32dccde030e30527ae6200d1`

Private evidence directory also contains native callback, stage and compositor presentation JSONL traces, binary/source hashes, window inventories and cgroup/health before-and-after snapshots. Both Hyprland PID 2544527 and `blobin_renderd` PID 1974196 remained alive.

## Remaining ORACLE-06 gate

The **physical-on-screen cursor hit/miss** is not qualified because this run had no button events. One operator-present short trial should click exactly within the visibly bounded 2D control and then outside it; the typed observer must produce the same-surface hit→miss sequence and no false authorization. A true hit is not proof that the separate 3D cavity itself is visually picked.

Next milestones and the independent hardware-critical path are in `GPU-05-CP03-E2E-CLOSURE-PATH.md`.

**No merge, deployment, release, API grant or claim of CP03 closure.**
