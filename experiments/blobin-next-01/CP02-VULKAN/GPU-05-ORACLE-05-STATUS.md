# GPU05-ORACLE-05 — Native Wayland → Unix IPC → in.gnu6 semantic observer

**Checkpoint:** GPU05.ORACLE05 · 2026-10-10
**State:** LIVE_CROSS_LANGUAGE_PASS / SEMANTIC_TARGET_GEOMETRY_UNQUALIFIED / PHOTODIODE_NOT_RUN / CP03_OPEN
**Baseline:** 600a0ad3c9dc1746c81a8ddd2c2c9654ceb17c12 (ORACLE-04)

## Scope and implementation

- Native Rust renderer is an **observation producer only**, with opt-in `--semantic-observer-socket`.
- The AF_UNIX client requires a private 0700 socket parent, a 0600 socket, and validates the peer UID with SO_PEERCRED.
- Bounded 2,048-byte packets, a depth-16 queue and a dedicated thread keep writes away from GPU frame scheduling; overflow fails closed.
- The client sends its owned Wayland surface token and actual SceneStream.latest_sequence() during a versioned handshake.
- The Node TypeScript receiver uses the existing in.gnu6 InteractionProjector and a typed WaylandOpticalSemanticProjector, not a second scene engine.
- The server reconstructs fixture scene sequence 13 with applyRendererDelta and checks surface identity, provider sequence and read-only permission.
- The declared 2560x1440 → 400x300 chart is a **test fixture mapping, not a physical or optical calibration**.
- The IPC server is local only, one connection, bounded lifetime and packets; no public service or product mutation API.

## Targeted gates

| Test | Outcome |
|---|---|
| Rust transport/permissions tests | **4/4 PASS** including one synthetic cross-language probe |
| TypeScript observation laws | **7/7 PASS** |
| Python source↔projection reconciliation laws | **6/6 PASS** |
| Strict Rust Clippy; TypeScript; Biome; Node SSR build | PASS |
| Synthetic Rust-to-Node IPC (separate process) | PASS; explicitly synthetic |
| One actual native Vulkan + Node observation run | PASS; 20s; normal exit |

## One physical-window local experiment

NVIDIA RTX 3070 / native Vulkan + Wayland, HDMI-A-1 at 2560x1440 fullscreen.
One newly admitted GNU6 exclusive cgroup built the Release binary separately and ran one 20.025-second sample.
The preceding launch attempt stopped at shell syntax validation **before compiling or opening a window**; it was corrected and not treated as a hardware retry of a refused job.

| Observation | Actual result |
|---|---:|
| Compositor-presented frames | **1,497** |
| Compositor discarded / pending feedback / callbacks | **0 / 0 / 0** |
| FPS (presented) | **74.814** |
| wp_presentation p95 / p99 / max | **13.394 / 13.496 / 26.730 ms** |
| Native wl_pointer button presses and releases | **20 + 20** |
| TypeScript semantic observations | **40** |
| Native/TS provider sequence and phase correspondence | **40/40 exact** |
| Renderer-local software optical cycles | **10 complete** |
| Semantic pressed target: stage | **20** |
| Semantic pressed target: cavity-control | **0 — TARGET GATE OPEN** |
| Independent human origin | Not attested |
| Calibrated display-to-scene chart | **Not qualified** |
| Physical input-to-photon sensor | **NOT RUN** |
| CP03 HYBRID-60 full | **OPEN** |

Independent reconciliation verdict:
`SOFTWARE_IPC_SAME_SURFACE_SEQUENCES_MATCHED_TARGET_NOT_QUALIFIED`.

These are real Wayland button events received by the actual Rust binary, but their independent human origin is not authenticated.
The Node receiver conservatively records sender-claimed input provenance, never an authenticated photodiode or scene-render pose.
Zero exact control hits means the fixture chart is NOT evidence of accurate visual targeting in the current Vulkan camera.
Do not infer production scene mutation authority, object pick validity, user-input-to-light latency, or full CP03.

## Local immutable evidence (not committed)

Root: `/mnt/workbench/evidence/runs/gpu05-oracle05-20261010/`.

- Native Release binary SHA-256: `229dc34fcff8e720d7f94a85226e0633f859f66029585d71bdde916edda0429a`
- Compiled Node receiver SHA-256: `b087896c0b2439c46b00702807af9ed121080df30febac66211a61955aeba7b2`
- Native optical-event JSONL SHA-256: `0d3b169e25cd9ed60e6f14a5e570ff3d1ff1c7b86767f835cc13838a7da88da2`
- Semantic observation JSONL SHA-256: `ce3ede7517b49a5bb10ab36f0a501516161204a209ee716a1bebf33939c00cec`
- Reconciliation JSON SHA-256: `dae3dc673729cbe7b230199eb10846e17852678f2000096c391ca2e8595d7b73`

Private evidence includes cgroup, exact hashes, window/monitor inventory, CPU stage/callback/wp_presentation traces, Node observer logs, and test-only synthetic packets. No raw private evidence is published.

## Remaining falsification

1. Obtain a measured chart from actual window pixels to the *rendered* control under the current Vulkan camera, with frame/pose binding.
2. Test exact visual-control hits and misses in a new deliberate experiment. The present 20 clicks projected onto stage, not cavity-control.
3. Reconcile the rendered interpolation with in.gnu6 frame-time and scene pose; the static scene sequence alone cannot certify visual pick accuracy.
4. Keep the 413.544ms historical gap as an unresolved negative, despite a clean 20-second run.
5. Calibrate the independent shared-clock switch/photodiode oracle; no software timestamp is photons.

**Version stays `2026.10.0-research`. The repository is PUBLIC, package still private/UNLICENSED. No merge, deployment, release, tag or API contract is implied.**
