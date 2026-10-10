# BLOB-SHOWCASE-02 — From SDF lab to interactive renderer

Status: development plan, not a claim of production-readiness. Owner boundaries: runtime team owns CUDA-05 and backend/IPC; Claude owns the separate native Wayland viewer project. Any protocol wire change requires compatibility review before deployment.

## Q01 — Always-available local runtime (P0)

Root cause: the bridge was only started inside terminals/E2E and exits when these finish. There was no persistent user service and no socket after shutdown. A reproducible manual launch passes. Source deliverables: `deploy/systemd/blobin-render-bridge.service`, `scripts/recover_stale_socket.py`, `scripts/install_user_service.sh`. **PENDING ACTUAL ACTIVATION**: execution tooling refused installation of a new user service. Operator can explicitly run `./scripts/install_user_service.sh`; confirm active status, reconnect after logout/login, journal logs, recover dead sockets without replacing active listeners. Do not claim fixed until active on user session.

## Q02 — Truthful frame-stage profiler (P0)

A native runtime timer records `gate_ms`, `worker_ms`, `total_ms` for each accepted frame without changing the BLOB-FRAME-v1 wire protocol. `scripts/profile_render_pipeline.py` runs a small, no-network finite series and writes an outside-repo JSON receipt. Local RTX 3070, 320x224, 4 iterations/backend: CPU median roundtrip 14.9 ms; CUDA 401.9 ms (gate 210.3, worker 186.5); Vulkan 361.3 ms (gate 180.2, worker 176.4). These are **end-to-end host times**, not GPU kernel-only timings.

## Q03 — Resident supervised GPU frame worker (P0)

**PROPOSED; NOT YET IMPLEMENTED IN LIVE DAEMON**. CUDA-04 already owns `ResidentBridge` with cache invalidation and managed health. Maintain a long-lived **separate process per backend**, under a typed bounded request protocol and a session-scoped approval lease. Enforce TTL, request-count cap, worker hash, provider re-check, explicit revoke, child death recovery, memory budget, update/rebuild on dimension or primitive-count change, no fallback. Original CUDA-05F fixed-scene gate per frame is currently ~180–210 ms: optimize without relabeling session-level checks as per-frame gates. All changes behind opt-in flag and versioned protocol compatibility. Run fault-injection tests before setting as default.

## Q04 — Animation runtime and frame scheduling (P1)

Separate monotonic clock and scene evaluator from UI. Start with GPU-rendered morphing SDF blobs and CSG cutouts, animated radii/orbits and easing. Drive work by the display's frame-pacing budget with bounded queue (<=1 outstanding frame per viewer), discard obsolete revisions, fixed time sampling for deterministic replay, and freeze the last known-good framebuffer during stalls. No JS, CSS or CPU-imitation of the GPU scene. Test resize/move while animated, and memory/FD stability over 10-minute runs. Target 30 or 60 fps **conditional on actual qualified measurements**, not marketing claims.

## Q05 — Viewer showcase / no engineering-wall UX (Claude-owned, P1)

Make six labs genuinely visual and immediately legible: (a) Morph Studio — two-to-five fluid circles; (b) Boolean Sculpt — animated difference and intersection; (c) Smooth Lab — merge/split with shape trails; (d) Field Explorer — false-color distance isocontours; (e) Surface Lighting — rotate directional light over normals; (f) Live Composer — multi-preset choreography with an expressive hero canvas. Default fullscreen canvas + human-language preset labels; hide developer jargon behind collapsible diagnostics. Expose actual backend used, visible frame latency, resolution scale, and a test-fixture banner only when appropriate. Renders are 2D/2.5D; no unsupported Blender, DLSS, raytracing or PBR claims.

## Q06 — Developer tooling / capture / verification (P1)

Provide CLI doctor for service/socket, latency histogram with process/gate/render/IPC split, timeline recorder/replayer, preview PNG export from real framebuffer, visual regression fixtures and LSB-aware differential report. Capture **only** the viewer rectangle; never the full desktop (an earlier private-window capture incident was disclosed and must not recur). Save raw evidence locally and report provenance. Expose machine-readable JSON and CLI human report.

## Q07 — Qualification and confidence degrees (P1)

Gate readiness locally on explicit functional, numerical, performance, recovery, transport, and privilege tests. Keep external review **optional** and model external confirmations separately as a confidence degree: independent reviewers, hardware/backend diversity, protocol coverage and shared raw evidence count; 2 and 500 confirmations should not collapse to the same label. External adoption is not a prerequisite for local functional qualification, but no evidence of cross-vendor compatibility should be implied without trials.

## Immediate execution order

1. Operator activates service after reviewing the installation script; qualify reconnect, restart and journal.
2. Integrate profiler and run resident proof on fixed and animated scenes; log p50/p95 and GPU fault controls.
3. Turn resident process into revocable session capability; benchmark again before changing viewer frame cadence.
4. Hand Q05 UI storyboard and Q04 scheduling contract to Claude (no edits to their worktree).
5. Release animated native showcase only after actual Wayland E2E on CPU, CUDA and Vulkan and updated runtime security qualification.
