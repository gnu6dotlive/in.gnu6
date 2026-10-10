# BLOB-SHOWCASE-02C — Native animation / GPU-resident preview handoff
**Status:** locally qualified experimental runtime path. Do **not** replace the existing user service until qualified in Claude's actual Wayland window and reviewed for session authority.
**Ownership:** Runtime team owns this worktree; Claude owns `~/Projects/blob.in-showcase`. No Claude-owned files are modified here.

## Architecture
`viewer → BLOB-FRAME-v1 AF_UNIX SOCK_SEQPACKET → renderd → CUDA-05F session gate → separate resident GPU child → native CUDA/Vulkan SDF → RGBA8 → sealed memfd → viewer`.

The historical production-disabled **disposable** worker path stays the default. Opt-in mode requires `--resident-worker`, `--resident-sha256`, `--resident-ttl-ms`, and `--resident-frame-budget` on the preview daemon. There is no new user-facing wire protocol, nor a new 3D engine. The user can change SDF primitives every frame; all pixels still come from actual CPU/CUDA/Vulkan backend computation.

The session gate is evaluated on acquisition/renewal, **not every warm frame**. The worker is pinned to approved executable and CUDA provider digests, revocable by process termination, and bounded by TTL and frame budget. Linux `pidfd` ties TTL cancellation to the exact child, avoiding PID reuse. This is still a **same-UID desktop preview**, not an untrusted-host/hostile-worker containment proof.

## Start (separate terminal, does NOT stop or modify the stable service)
```bash
cd /mnt/workbench/workspaces/gnu.in.labs/.worktrees/blobin-showcase-resident-02/experiments/blobin-next-01/CUDA-05
./scripts/start_resident_preview.sh
```
The preview listens on `$XDG_RUNTIME_DIR/blobin-showcase-resident-02/render.sock`; the existing service continues listening on `$XDG_RUNTIME_DIR/blobin-showcase/render.sock`.
## Attach Claude's real native Wayland viewer, without changing its code
```bash
cd ~/Projects/blob.in-showcase
cargo run --offline --release -p blobin-showcase -- \
  --socket "$XDG_RUNTIME_DIR/blobin-showcase-resident-02/render.sock" \
  --backend cuda
```
Close only this viewer, then press Ctrl+C in the preview terminal to stop the experimental daemon. The stable systemd service remains active for normal launch.

## Measurement scope
E2E test used a separate temporary socket and 7 geometry-modified frames per backend, with a deliberately small **three-frame** lease to force renewals:

| Stage | CUDA (RTX 3070) | Vulkan (RTX 3070) |
|---|---:|---:|
| First frame, acquisition and worker initialization | ~783 ms | ~352 ms |
| **Warm frame** round-trip, median | **~7.4 ms** | **~6.2 ms** |
| Budget renewal (frame 4, 7) | ~513–573 ms | ~305–311 ms |

Real content digests changed on all seven frames. The worker enforces lease TTL and budget and rejects a mismatched CUDA provider hash. Historical disposable path still passes **12/12** black-box checks; active systemd CUDA preview was rechecked PASS. These are host round-trip measurements in a test client, **not** verified presentation FPS inside Hyprland.

Preview launcher currently chooses TTL **30 s**, budget **120 frames**. The renewal spike is a known product gap, so continuous 60 FPS is **not** yet qualified. Do not claim a guaranteed frame rate; UI must preserve last complete frame during renewal and show measured latency.

## Viewer presentation request (Claude-owned; a separate task)
- Default to a **large expressive canvas** with an animated *Morph Studio* preset; make controls obvious and keep low-level diagnostic details collapsible.
- Demonstrate actual evolving geometry: smooth unions, subtractive holes, animated intersections, field contour patterns, rotating derived-normal lighting, and a choreographed multi-blob scene.
- A deterministic time-based animation loop should mutate allowed primitive coordinates/radii and increment `scene_revision`, sending one request only when none is outstanding. Coalesce missed ticks rather than queue frames.
- Offer **Play/Pause**, speed, scene preset, backend selector, and field/normal/lit/coverage views. Preserve CPU/CUDA/Vulkan source truth: never synthesize a mock geometry frame as if from a GPU.
- Maintain bounded dimensions (16–1024 per axis), primitive count (1–64), and no unsupported 3D/PBR/DLSS claims.
- Respect Claude's prior safety rule: screenshot **only** the viewer window, never private desktop windows.
## Qualification gate before stable default
1. Rebuild in dedicated `CARGO_TARGET_DIR`; never share the active service's binary directory (an accidental shared build previously changed pinned executable bytes and was immediately restored).
2. Run `python3 scripts/qualify_resident_worker.py` and `python3 scripts/qualify_resident_bridge.py` against the isolated binaries. Verify TTL, digest pinning, frame-budget turnover, CPU identity, memfd seals, exact dimensions, frame revisions and no backend fallback.
3. Repeat Claude's window/resize/interactivity E2E with the **resident preview socket**. Measure actual presented FPS and p50/p95 latency, including session renewal spikes.
4. Add a session-level fault-injection matrix: worker crash/hang, provider bytes replacement, supervisor rejection, GPU device loss and interrupted readback. Do not conflate passing output checks with containment or production authorization.
5. Only after green gates and explicit operator approval consider installing/updating the stable service. No automatic merge, public push, domain deployment or user-service restart.

**Qualification degrees:** Local operational evidence may qualify a bounded runtime; independent external reviewers are optional, and their count, independence and hardware diversity augment confidence without acting as a blanket prerequisite.
