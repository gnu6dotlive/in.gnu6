# BLOB-02C — Cache-hit incident, 2026-10-10

**State:** reproduced, repaired, locally qualified. The native Wayland viewer E2E still needs independent confirmation.

## Original symptom and cause

The user reported two `RENDER_ERROR WORKER_TIMEOUT` events when moving from CUDA at varying resolutions to Vulkan and back to CUDA. An isolated black-box replay of the exact viewport/backend sequence reproduced the incident. The second identical Vulkan geometry and later repeated CUDA geometry terminated the resident worker with `RESIDENT_UNEXPECTED_DEVICE_OR_CACHE`. The parent misclassified pipe EOF as a timeout and did not expose worker stderr.

CUDA-04 correctly returns `cache_hit=true` when previously computed, validated geometry matches. The new resident child wrongly considered *any cache hit* fatal. This was a bug in the new runtime adapter, not a CUDA/Vulkan driver failure.

## Repair

The worker now rejects an unexpected device but **accepts valid cached samples** and recolorizes them for the requested view. Its internal framed pipe includes an explicit one-byte cache bit, validated by the parent. Public `BLOB-FRAME-v1` still uses the same `FRAME_READY` JSON and sealed RGBA `memfd`, now with truthful `cache_hit`. EOF, invalid cached flag and deadline expiration are differentiated in diagnostics. A failed child is still revoked and replaced.

## Regression gates

- `scripts/test_cache_resize_switch.py`: ten exact reported high-resolution requests, including CUDA → Vulkan → CPU → CUDA, all pass. Same-geometry frames have matching RGBA SHA-256 and `cache_hit=true`.
- `scripts/qualify_resident_bridge.py`: 21 frame requests, both GPU backends, CPU, frame budget rollover, repeat geometry and view-mode recolor from cached SDF data.
- `scripts/qualify_resident_worker.py`: 48 frames per GPU backend, session budget, TTL, and CUDA provider hash.
- Disposable legacy Render Bridge: 12/12 checks PASS, stable user service remains active.

Machine-local raw receipts are under `/mnt/workbench/evidence/runs/blobin-resident-anim-20261010/`. No private viewer screenshots or source published.

## Boundaries

Changes are **only in the opt-in resident worktree**. The live systemd service is unchanged. This fixes the false worker death; it does not guarantee 60 FPS on resizing. CUDA-04 recreates GPU sessions when dimensions change; the reported 642×1024 resizing frames are materially slower than cache hits. The independent native Hyprland E2E is still pending because this remote shell lacks `HYPRLAND_INSTANCE_SIGNATURE`.
