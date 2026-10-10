# GPU05-ORACLE-02 — Native Wayland pointer → optical ROI witness

**Checkpoint:** GPU05.ORACLE02 · 2026-10-10
**State:** SOURCE_AND_LAWS_PASS / IDLE_NATIVE_VULKAN_SMOKE_PASS / REAL_POINTER_TRIAL_NOT_OBSERVED / PHOTODIODE_NOT_RUN / CP03_OPEN

## Boundary and intent

ORACLE-01 provided a strict *external* two-channel hardware input-to-photon
schema, but no physical switch/photodiode capture. ORACLE-02 adds a
**renderer-local, explicitly opt-in visual witness**, not semantic authority.

In the private GPU05 Vulkan experiment, the optional command-line argument
`--optical-probe-jsonl <ABSOLUTE_PATH>` activates a dedicated
`wl_seat / wl_pointer` consumer on the **same owned Wayland connection**
used for rendering. Pointer Enter must reference the very same `wl_surface`;
only BTN_LEFT presses while the pointer is inside that surface can generate
one bounded optical trial. Pointer Leave or loss of pointer capability
removes input focus.

The experimental presenter uses `present_optical.wgsl` only when the
feature is enabled. It adds a fixed upper-left UV rectangle
(x=0.10–0.26; y=0.10–0.26), BLACK at rest and WHITE for 350ms following
an eligible press. The rest of the signed tetra MeshSolid + SDF scene
projection remains unchanged. The state machine has a 900ms refractory
window, ignores held-button repeats, and reverts to dark without needing
an inferred keyboard event. This is **not a mutation of the semantic scene**
and never changes the authority of `SceneStream`.

## Causal evidence boundaries

Separate JSONL events classify:
- Actual `wl_pointer.button` events from the owned surface and their
  Wayland event times (software input provenance only).
- Trial bright request, per-frame uniform bright/dark choice, 350ms
  reset and exact frame-sequence projection.
- Matching `wp_presentation` outcome for each submitted frame,
  explicitly **NOT** a physical photon timestamp.

New `gpu05_optical_scene_gate.py` refuses forged hardware claims,
bright transitions without an input event, mismatched frame/ROI outcomes,
duplicate sequences, missing dark resets and bogus photon timestamps.
Even a complete real Wayland click cycle would prove only the software
and compositor path. A physically sensed switch/contact edge and
screen photodiode are still required by ORACLE-01, independently.

## New targeted verification

- 6/6 Rust state-machine tests PASS.
- 9/9 Python causal-ledger tests PASS.
- Strict `cargo clippy --offline --locked ... -- -D warnings` PASS.
- Rust formatting PASS.
- Historical GPU01–PACER07 and 125s endurance suites NOT rerun.

## One safe NVIDIA Wayland/Vulkan smoke

One offline Release build in a **dedicated CARGO_TARGET_DIR**, then
one 10-second opt-in physical Vulkan window test, both executed under
`gnu6-lab-run --exclusive`. Preflight returned GNU6_LAB_READY.
No compositor restart, GPU reset, synthetic input injection or
systemd-oomd change.

Actual host: NVIDIA RTX3070; HDMI-A-1, 2560×1440 fullscreen,
monitor 0, native Wayland client. Both Hyprland PID 2544527 and
stable `blobin_renderd` PID 1974196 survived.

| Observation | Result |
|---|---:|
| Native Run exit | 0 |
| Runtime | 10.018 seconds |
| Compositor-presented frames | 749 |
| Callback outcomes | 749 matching |
| Compositor discarded | 0 |
| Unresolved Wayland objects | 0 |
| WSI timeout / abort | 0 / 0 |
| Real pointer presses observed | **0** |
| White ROI physically observed | **NOT MEASURED** |
| Real Wayland click → bright frame | **NOT EXECUTED** |
| Independent switch + photodiode | **NOT CONNECTED** |
| CP03 complete | **OPEN** |

wp_presentation frame cadence: 74.764 presented FPS, p95 gap 13.400ms,
p99 gap 13.536ms. These are **short smoke** observations, not replacement
for the previously closed 125-second endurance gate.

The optical analyzer correctly emitted
`NO_COMPLETE_WAYLAND_OPTICAL_TRIAL`, with zero observed user presses
and `cp03_pass=false`.

### Exact local evidence (not committed)

Evidence directory name: `gpu05-oracle02-20261010`
under the protected, out-of-repository GNU6 evidence store.

Binary SHA-256:
`4ba51808587698ee3731154bd8c0288aa46ca8c8a2853de7fa13247a8b93111d`

Optical renderer trace SHA-256:
`abf4ce318b2e80f1989d25c1376b582b7768f0bdafde087c50ff0035756d35a3`

Compositor presentation trace SHA-256:
`b6d1cf0ecaf2922d030452e1639b5d4e7eb6f345609cf1b45e85e21ae6f110bc`

The evidence store also contains callback/stage SHA-256 manifests,
source fingerprints, the window inventory, run log and analyzer outputs.

## Next independent gates

1. **Observed real native pointer trial:** user places the pointer on
   the experimental window and clicks, with the probe enabled. Verify
   the button, Bright, corresponding compositor-presented ROI frame,
   Dark reset and subsequent compositor-presented dark frame, all
   under bounded runtime and exact SHA-256. Do not inject automation
   and then claim physical human input.
2. **Actual semantic provenance:** route this user intent through the
   typed, owned scene input boundary, preserving the in.gnu6 ontology.
   The current isolated optical overlay is *only renderer-local*.
3. **Hardware optical witness:** dual-channel physical switch-contact
   and external photodiode with a shared calibrated timer, independent
   capture provenance and >=30 full trials. Do not substitute the C922
   webcam or queue.present timestamps.
4. External CP03 audit and all other acceptance gates remain OPEN.

The main branch, other agents' worktrees, stable renderd service and
product interfaces were not touched. No npm/public release or license
change is implied by this research checkpoint.
