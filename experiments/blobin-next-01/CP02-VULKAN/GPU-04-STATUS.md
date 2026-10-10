# GPU-04 — bounded live scene ingress and Wayland presentation boundary

**Truth:** LIVE_SCENE_IPC_OBSERVED + GPU_ANIMATION_OBSERVED + PRODUCER_STALL_LOCAL_PASS / WP_PRESENTATION_NOT_OBSERVED / NOT_CP03_PASS.

Base: GPU-03 local commit `19f86d6460966f2bf2ddeb12dbc83929d4e5b6b1`.
All work private in `feat/blobin-gpu04-live-20261010`; no merge, push, release, compositor change, device reset or stable-service change.

## Live runtime: real scope

- Explicit CLI: `gpu02_wayland --scene-stream <qualified-v1-fixture> --live-socket <private-absolute-path>`.
- One local AF_UNIX stream listener for opt-in control. Socket parent must exist and have mode 0700; socket is 0600. Linux SO_PEERCRED checks effective UID, but **same UID is not trustworthy product/agent authentication**.
- A separate bounded reader thread accepts newline-delimited packets, max 8192 bytes per packet, and forwards them through a bounded 16-item queue. Renderer drains at most 16 queued messages per redraw. No socket read in the GPU render path.
- Packet is `{"schema":"blobin.gpu04.delta.v1","delta":<RendererFrameDelta-v1>}`; exactly typed data, no program code, shell commands, arbitrary material expression, GPU shader injection or engine mutation capability.
- Rust checks version, identity partition, strict sequence progression, finite bounds and a narrow `cavity-control` projection, and keeps only the latest frame. Invalid packets leave the last valid scene projection intact.
- With live socket enabled, shader uses the last accepted offset; the camera stays fixed. Legacy GPU-02 behavior without live socket is preserved.
- `scripts/gpu04_send_delta.py` is a deterministic local sender. Its SEND message is expressly **not an acceptance acknowledgment**. Verify acceptance from the receiver log.
- The listener stops with the process; on ordinary Rust teardown the socket path is removed. Abrupt SIGTERM may leave a stale socket file; reusing that path fails closed. The isolated test harness cleaned its own remaining socket paths.

## Verified local checks

1. `cargo fmt --check`, `cargo clippy --offline --all-targets -- -D warnings`, `cargo test --offline --bin gpu02_wayland`, and `cargo build --offline --release --bin gpu02_wayland` passed.
2. **6/6 Rust tests** in the GPU-02 executable (GPU-03 tests plus GPU-04 stale/replay/oversized/invalid-boundary tests); previously validated in.gnu6 TS bridge unchanged. No source file in other workers changed.
3. Real local NVIDIA RTX 3070/Vulkan window: sequence 14 accepted, stale sequence 14 rejected (`STALE_OR_INVALID_DELTA`), sequence 15 accepted. `grim` recorded three distinct captures, visually inspected as different cavity positions with the same tetrahedron/camera.
4. Explicit 0755 socket-parent test fails before creating a listener: `GPU04_PARENT_MUST_BE_PRIVATE_0700`.
5. Logic-producer stall with the **same Unix connection held open**: sleep measured 600.155 ms; the renderer kept queue-present operations going, and both valid deltas 14 and 15 were accepted. Across four approximately 2-second windows from this run, worst reported queue-present interarrival p99 was **7.706 ms** and maximum **12.158 ms**.
6. Independent raw Wayland registry synchronization observed `wp_presentation` **version 2** and `wl_compositor` **version 6** on `/run/user/1000/wayland-1`. This proves advertised protocol availability only.
7. All captures, negative logs, timings and registry observation: `/mnt/workbench/evidence/runs/blobin-next-01/gpu04-live-20261010/`.

## Nonclaims and limits

- Queue.present count/interarrival is **not** `wp_presentation` feedback, actual scanned-out/displayed FPS, or input-to-photon latency. We did not bind feedback to GPU-04's specific `wl_surface` because the current winit 0.30 surface owner does not expose a safe integration handle for independent protocol listeners. Registering `wp_presentation` from a **second Wayland client** would not qualify this surface.
- **No CP03 HYBRID-60**: no verified 7200/120s compositor frames, compositor p95/p99, all-stall budget, or input latency; no independent E2E or stress/fault suite. CP02 seven gates also remain open.
- Same-UID writes and user-supplied `source` metadata are not trusted Gnosix identities, signed grants, or model authority. The demo control contract accepts no foreign remote users; production requires authenticated scoped ingress, peer process lifetime binding, and audit receipts.
- No generic 3D scene graph, no arbitrary MeshSolid import from IPC, no full Rust in.gnu6 runtime; only the bounded 2D control x coordinate projects to the existing GPU signed sphere.
- No `BLOB-FRAME-v1` change, no CUDA-05F supervised worker admission: GPU-04 remains a standalone direct-wgpu proof. CP07 and CP08 not qualified.
- The display loop remains an opt-in experimental Wayland window, with 640×432 fixed compute buffer and rescaled window output. Some raymarch surfaces show aliasing.

## Reproduction on the authorized desktop

```sh
W=/mnt/workbench/workspaces/gnu.in.labs/.worktrees/blobin-gpu04-live-20261010
P="$W/experiments/blobin-next-01/CP02-VULKAN"
export CARGO_TARGET_DIR=/mnt/workbench/build/blobin-gpu04-live-20261010
cargo test --manifest-path "$P/Cargo.toml" --offline --bin gpu02_wayland
cargo build --manifest-path "$P/Cargo.toml" --offline --release --bin gpu02_wayland

# Execute only in your authorized Wayland desktop session.
D=$(mktemp -d "$XDG_RUNTIME_DIR/blobin-gpu04-XXXXXX")
SOCKET="$D/scene.sock"
"$CARGO_TARGET_DIR/release/gpu02_wayland" --scene-stream "$P/fixtures/scene-stream.v1.json" --live-socket "$SOCKET"
# In another shell, after GPU04_PRIVATE_DELTA_SOCKET_READY:
python3 "$P/scripts/gpu04_send_delta.py" --socket "$SOCKET" --fixture "$P/fixtures/scene-stream.v1.json" --from-sequence 13 --to-sequence 14 --scene-time 4 --control-x 18
```

Do not advertise this local test socket publicly, connect it to the VPS or create a privileged systemd service.

## Integration next

1. Implement a **per-surface** Wayland `wp_presentation.feedback` observer with explicit ownership shared with the Wayland event loop or a controlled host adapter. Evidence must correlate the compositor's presentation feedback with a particular GPU-04 frame sequence and classify discarded feedback. Merely discovering v2 is insufficient.
2. Reconcile source contracts with parallel CP01/GPU01 worker HEADs; compare clean trees and exact hashes before any merge.
3. Upgrade the local same-UID fixture channel to an authenticated typed scene-origin channel with bounded grants, timeouts and lifecycle. Keep `in.gnu6` semantic authority out of the renderer.
4. Run independent 120-second CP03 HYBRID-60 once genuine presentation timestamp collection is operational.
