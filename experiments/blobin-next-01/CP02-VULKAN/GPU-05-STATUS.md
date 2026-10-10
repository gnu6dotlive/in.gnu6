# GPU-05 — physical compositor presentation qualification

> **Updated 2026-10-10:** This is the original source-only proposal. The current code/evidence state is in [GPU-05-RESUME-2026-10-10.md](GPU-05-RESUME-2026-10-10.md); the SOURCE_ONLY label below is historical.


**State (2026-10-10): SOURCE_ONLY / WAYLAND_PHYSICAL_TEST_PENDING / NOT_CP03_PASS.**

**Base:** branch `feat/blobin-gpu05-presentation-20261010`, created exclusively from GPU-04 `b15ad6b14043579dc3eef4378caa52549da4eb88`. Other worker worktrees, native renderd service, and BLOB-FRAME-v1 must remain unchanged. No publication/merge/push.

**Access note:** the workstation's command-execution connector returned an explicit refusal while inspecting the presentation integration. Do not retry that denied path. Remote Desktop Commander listed the alternative device as offline. Read/write access to this worktree still worked; therefore the analyzer and negative-test source files could be prepared, but **they have not been run, formatted, linted or committed under this mandate**. No runtime or protocol qualification is claimed.

## Established premises — inherited, not promoted

- GPU-04 opt-in local AF_UNIX stream, same-UID checks, bounded validated deltas, Rust/wgpu 30 real Vulkan frame production and animated Wayland window observed.
- The independent Wayland registry probe of `/run/user/1000/wayland-1` announced `wp_presentation` version 2. This was **not feedback from blob.in's surface**.
- Queue.present submissions and CPU interarrival p99 at GPU-04 were not compositor presentation receipts. The 600.155-ms producer stall was tested at the application boundary only.
- Source branch retained strict GPU-01/02/03/04 provenance boundaries. CP03 60-FPS and CP08 remain OPEN.

## Critical architecture decision

**Own the compositor connection and `wl_surface` in the future GPU-05 native Wayland host.**

The present GPU-04 window uses `winit 0.30.13`, which internally owns the Wayland connection and event queue. `winit::platform::wayland::WindowExtWayland` exposes an `xdg_toplevel` pointer, not a supported `wl_surface` proxy/event queue API for an external observer. Wrapping its raw Wayland display in a second independent event-reader risks racing/stealing events from winit. Binding `wp_presentation` on a **different Wayland connection** also cannot request feedback on another client's `wl_surface`.

Neither workaround counts as same-surface feedback.

### Proposed GPU-05 host ownership

1. Create `wayland_client::Connection::connect_to_env()` and **one sole owner event queue**.
2. Bind `wl_compositor`, `xdg_wm_base` and `wp_presentation` on that connection. Record `clock_id`; fail closed if presentation protocol unavailable.
3. Create own `wl_surface`, `xdg_surface` and `xdg_toplevel`. Commit initial empty surface, await `xdg_surface.configure`, acknowledge configuration before swapchain presentation.
4. Create a Vulkan `wgpu::Surface` from the **same owned raw Wayland display/surface handles** using the documented unsafe raw-handle contract. Keep display, proxy, GPU surface and native window lifetimes joined in the host; never free while the swapchain is live.
5. Reuse the verified GPU-04 signed tetrahedron/SDF compute and GPU-only copy/presenter logic. The resulting window **must render blob.in's own shader**; a separate colored wl_shm or generic test window does not qualify.
6. Request `wp_presentation.feedback(wl_surface)` for the commit corresponding to **each identified GPU frame**; register an immutable mapping `feedback_object_id → render_frame_sequence → surface_token`. Avoid attaching multiple feedback IDs to the same frame.
7. Dispatch `presented` / `discarded` through that *same* Wayland event loop. Correlate delivered timestamps with the presentation clock domain. Detect missing/unresolved feedback objects, surface recreation, output changes and throttling.
8. Write append-only JSONL receipts with explicit schema and SHA-256 evidence artifact identifiers, then separately archive the actual host executable hash, source commit and session/monitor facts.
9. Keep the GPU-04 delta reader segregated from the compositor event thread. Enforce bounded queues and fail closed on producer failure; only typed scene facts may cross.
10. Run an independent 120-second GPU05 qualification with frame feedback, perf distribution, input-to-photon oracle and shutdown/recreation stress. No CP03 uplift without all gates.

### Non-goals of this preparatory slice

Do not patch winit internals, create foreign Wayland proxies on another client, wrap a foreign display in a competing event reader, adopt Smithay as compositor, replace Hyprland, weaken CUDA-05F, or claim NVIDIA results generalize to AMD/Intel.

## Prepared Rust feedback correlation module (NOT YET COMPILED)

`src/presentation_tracker.rs` is a new pure-Rust state machine intended to be imported **only by the future native-owned Wayland event loop**. It holds strict frame-sequence to feedback-ID mappings, a declared clock domain, surface generation, pending feedback, presented versus discarded outcomes, monotonic timestamps, fail-closed invalid nanoseconds, and a bounded receipt buffer. It rejects protocol event duplication and surface regeneration with unresolved feedback. Its unit tests are source-only, **not executed**. The current GPU-04 winit host is unchanged and does not use the new module.

The state machine is not the protocol listener; it remains to be compiled and connected to physical events. Never promote synthetic tracker tests into a compositor timing qualification.

### GPU-05 native owned-surface adapter — source prepared

`src/owned_wayland_presentation.rs` now sketches the owned Wayland client boundary: `Connection` + one `EventQueue`, `wl_compositor`, `xdg_wm_base`, `wp_presentation`, owned `wl_surface` and XDG toplevel, configure acknowledgement, presentation clock, feedback creation *before the corresponding Vulkan commit*, and `presented` / `discarded` dispatch to the Rust tracker. Its receipt drain is bounded and a blocking diagnostic roundtrip is explicitly separated from the render hot path.

**Not yet integrated or compiled:** it is not referenced by the current Cargo binary. The eventual native host requires `wayland-client` 0.31.15 with `system` enabled (libwayland FFI), `wayland-protocols` 0.32.13 client, raw-window-handle 0.6.2, an audited `wgpu::Surface` handle/owner lifetime, and a single FD-backed Wayland event pump. No raw `wl_surface` pointer has been passed to Vulkan yet. Winit's GPU-04 window is still untouched; **no physical GPU-05 feedback was acquired**.

The adapter is therefore a testable implementation candidate, **not a release-ready Wayland host**. It needs rustfmt/Clippy, compile-time API fixes, exact lifetime audit, actual Vulkan GPU-04 shader integration, then physical feedback collection.

## Trace protocol for the future owned native host

The source-only analysis tooling in this branch defines **`blobin.gpu05.presentation.v1`** JSON Lines (one object per line). This is *not yet a captured data format from a running observer*.

First record:

```json
{"type":"header","schema":"blobin.gpu05.presentation.v1","feedback_origin":"wp_presentation","feedback_surface_owner":"gpu05-owned-wayland-surface","wl_surface_token":"opaque-session-surface-01","clock_id":1}
```

A feedback after compositor outcome:

```json
{"type":"feedback","feedback_id":123,"wl_surface_token":"opaque-session-surface-01","frame_sequence":122,"outcome":"presented","tv_sec_hi":0,"tv_sec_lo":5823,"tv_nsec":123456789,"refresh_ns":16666667}
```

A discarded feedback has `outcome: "discarded"` and no guaranteed timestamp. Each feedback ID and sequence must be unique; differing surfaces cannot be aggregated.

An independent input-to-display oracle record (NOT derived by subtracting arbitrary Wayland input timestamps from `wp_presentation`):

```json
{"type":"input_latency","oracle":"independent-same-clock-calibrated","latency_ms":23.8}
```

**Important:** these are trace claims, not cryptographic attestations. The analyzer cannot independently prove that the data came from Hyprland. A separate operator-controlled capture/attribution review is mandatory.

## Metric analyzer and local tests

`scripts/gpu05_presentation_gate.py` statically enforces:

- Correct schema, owned surface token and announced presentation clock.
- Uniqueness and contiguity of frame sequences plus unique feedback IDs; receipt source distinction. Missing outcomes fail closed.
- Explicitly separate `presented` and `discarded`; unresolved feedback is rejected from the trace corpus.
- Monotonic compositor timestamps (after ordering by frame sequence), valid nanoseconds.
- At least **7,200 compositor-presented** frames across **≥120 seconds**.
- Frame-to-frame p95 ≤17 ms, p99 ≤20 ms, maximal gap ≤33.4 ms, **zero discarded**.
- Independently calibrated input-to-display p95 ≤50 ms, with at least 30 latency samples (never inferred from `queue.present`).
- Even with all metrics green, its output remains **METRICS_CANDIDATE_PROVENANCE_PENDING** and `cp03_pass=false`.

`scripts/test_gpu05_presentation_gate.py` includes synthetic scenarios for nominal cadence, mixed surfaces, missing or duplicated feedback, one discarded frame, missing independent input measurements, uncalibrated input clock, `queue.present` impersonation, nonmonotonic timestamps, large stutter, invalid nanoseconds, insufficient 120-second dwell, and input latency exceedance. **These tests are not yet executed on the workstation.**

Proposed commands after reauthorization:

```sh
P=/mnt/workbench/workspaces/gnu.in.labs/.worktrees/blobin-gpu05-presentation-20261010/experiments/blobin-next-01/CP02-VULKAN
python3 -m unittest discover -s "$P/scripts" -p 'test_gpu05_*.py' -v
python3 -m py_compile "$P/scripts/gpu05_presentation_gate.py" "$P/scripts/test_gpu05_presentation_gate.py"
# Standalone pure Rust state-machine unit tests (do NOT imply Wayland execution):
RUST_TEST_BIN=$(mktemp /tmp/blobin-gpu05-tracker-test.XXXXXX)
rustc --edition 2021 --test "$P/src/presentation_tracker.rs" -o "$RUST_TEST_BIN"
"$RUST_TEST_BIN"
rm -f "$RUST_TEST_BIN"
# Only after the owned Wayland host emits real attributable wp_presentation receipts:
python3 "$P/scripts/gpu05_presentation_gate.py" /path/to/physical-surface-feedback.jsonl
```

## Closing criteria for GPU-05 (future work)

- Documented Rust event dispatch owner and exact same-surface `wp_presentation` feedback acquisition.
- Executable `cargo fmt`, `cargo clippy --offline --all-targets -- -D warnings`, Rust unit/negative tests and physical NVIDIA compositor trace.
- Real `wl_surface` feedback `presented` and `discarded` handling with correlated frame sequences, no independent-client substitution.
- Independent timestamp oracle, fail-closed unexpected surface/clock; full captures and process/build hashes.
- Explicit connection back to CUDA-05F supervision and CP03 gate evidence; no automatic PASS.
