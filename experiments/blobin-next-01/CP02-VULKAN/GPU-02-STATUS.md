# GPU-02 — native Wayland live Vulkan hybrid surface

**State:** GPU02_NATIVE_WINDOW_OBSERVED / LOCAL_EXPERIMENTAL / NOT_CP02_PASS / NOT_CP03_PASS.
**Source boundary:** forked from GPU-01 `5d2b5bdc65cbd60109060401a6f655714bd165fa`.
**Branch:** `feat/blobin-gpu02-native-20261010`. No merge, push, stable service update, or other worker mutation.

## What was actually implemented

- New standalone `gpu02_wayland` binary: Rust + winit 0.30.13 + wgpu 30.0.1 Vulkan.
- Explicit GLB tetrahedron import and conservative convex MeshSolid certification, using the existing CP01 source at the GPU-01 branch base.
- Derived GPU-01 WGSL triangle-distance + signed hybrid CSG shader. GPU-01's original shader and executable stay untouched.
- Variable camera orbit and moving analytic SDF sphere. Key bindings: Left/Right orbit, Space pause/resume without time jump, Escape/Q close.
- Each redraw: real WGSL compute dispatch (640x432) -> GPU storage buffer -> **GPU-only** buffer-to-texture copy -> fullscreen swapchain draw -> `wgpu::Queue::present`.
- Presentation uses its **own real Wayland window**, not mpv, a stored PNG, the original blob.in-showcase editor, or the stable BLOB-FRAME-v1 daemon.

## Evidence, reference workstation

GPU: physically identified NVIDIA RTX 3070 on Vulkan; compositor: actual Hyprland Wayland session.
Execution logs and two original `grim` screenshots:
`/mnt/workbench/evidence/runs/blobin-next-01/gpu02-native-20261010/`

- `live-run-v2.log`: repeated ~74–76 surface submit operations per second, each measured over a ~2-second interval; **not** independently timestamped compositor-presented FPS.
- `gpu02-live-window.png`: genuine capture of the live GPU02 Wayland window, SHA-256 `76074d2f9f0c39d036762cf0ad1294a5d21020b2d526efda42ff042776b27b64`.
- `motion-a.png` / `motion-b.png`: same window geometry, 2.5 seconds apart, distinct SHA-256 `e2904668...` and `4789fae9...`. Visual inspection: turquoise spherical cavity moves within the brown tetrahedral body. Screenshots prove visible change, not exact display cadence.
- Historical GPU-01 original: 3072 point/channel comparisons with independent CPU signed-mesh oracle, max error `1.938452514061e-7` at fixed scene state. **Those values are inherited GPU-01 evidence, NOT a fresh GPU-02 moving-camera oracle.**
- `cargo check --offline --locked --bin gpu02_wayland`: PASS.
- `cargo build --offline --locked --release --bin gpu02_wayland`: PASS.
- `cargo fmt --check`, `cargo test --offline --locked`, `cargo clippy --offline --locked --all-targets -- -D warnings`: PASS for CP02-VULKAN. CP02 binary test harness has **zero automated unit tests**, so these gates are compile/lint/regression smoke, not GPU behavior proof.
- `CP01-HYBRID cargo test --offline --locked`: 54 tests PASS on this GPU-01-base version. This is not GEO-05 (which exists in a separate worker branch).

## Limits and next falsification

1. **Separate host:** not yet integrated with the original `~/Projects/blob.in-showcase` or semantic `in.gnu6` consumer. `BLOB-FRAME-v1` deliberately remains 2D and untouched; 3D scene transfer needs a separately versioned contract.
2. **Fixed internal 640x432:** window resizes only rescale the GPU image, not the raymarch sampling grid.
3. **Timing:** CPU-side queue submissions are not compositor `wp_presentation` receipts or p95/p99 frame-time, input-latency or 120-second no-stall qualification.
4. **GPU oracle:** live shader animates camera and sphere without a fresh independent oracle over the live time sequence; GPU-01 static spatial parity may not be promoted as GPU-02 dynamic parity.
5. **Missing:** input-to-pixel latency measurements, asynchronous logic-stall isolation, bounded per-frame allocation/VRAM accounting, independent cross-vendor Wayland E2E, GPU device-loss resilience, nonconvex GLB shader path, production safety.
6. **Physical GPU concurrency:** this experiment coexists with the other workers; it does not reset devices, modify Hyprland settings, change systemd, touch the VPS, or claim exclusive benchmark isolation.
7. **No gate inflation:** CP02's seven gates and CP03 HYBRID-60 remain open. CP07/CP08 alpha / Linux cross-vendor remain open. These observations establish a bounded GPU02 slice only.

Next meaningful experiment: wire a **versioned immutable native scene snapshot** and GPU02 real-time viewport to blob.in's product shell via a separate opt-in adapter; measure actual compositor presentation and frame jitter, then reconcile with the other worker's newest CP01/CP02 source rather than merging blindly.

## Reproduce in the isolated worktree

```sh
cd /mnt/workbench/workspaces/gnu.in.labs/.worktrees/blobin-gpu02-native-20261010/experiments/blobin-next-01/CP02-VULKAN
CARGO_TARGET_DIR=/mnt/workbench/build/blobin-gpu02-native-20261010 cargo build --offline --locked --release --bin gpu02_wayland
CARGO_TARGET_DIR=/mnt/workbench/build/blobin-gpu02-native-20261010 cargo clippy --offline --locked --all-targets -- -D warnings
# From an authorized Wayland session only:
CARGO_TARGET_DIR=/mnt/workbench/build/blobin-gpu02-native-20261010 cargo run --offline --locked --release --bin gpu02_wayland
```
