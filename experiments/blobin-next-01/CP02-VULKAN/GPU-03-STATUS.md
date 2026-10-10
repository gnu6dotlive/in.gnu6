# GPU-03 — Scene stream replay into native Vulkan Wayland

**Truth:** LOCAL_RUST_TS_REPLAY_PASS + PHYSICAL_WAYLAND_WINDOW_OBSERVED. NOT realtime semantic IPC, NOT CP02/CP03 qualified.
**Isolation:** feat/blobin-gpu03-scene-bridge-20261010; base b4eb141 plus cherry-picked 44c875a as fb5febc.

## Actual implementation

- Preserves GPU-01 fixture, signed tetra MeshSolid certification, Vulkan/WGSL raymarch, GPU-only copy, and native winit Wayland presentation from GPU-02.
- Defines scene-stream.v1.json as a private, bounded replay fixture of the existing in.gnu6 RendererFrame and RendererFrameDelta structures.
- TypeScript qualification constructs actual SceneEngine snapshots, computes deltas, compares fixture values and replays them. No access to SceneEngine mutation objects is given to the renderer.
- Rust scene_bridge.rs independently parses exact versioned fields, rejects extra keys, stale sequences, duplicate/unknown identities, bad order, invalid geometry, nonfinite values, oversized payloads and bounded-control violations.
- Scene replay supplies a time-interpolated cavity horizontal offset to a GPU uniform. When the fixture is enabled the camera is held fixed; no-script legacy GPU-02 remains animated.
- CLI is opt-in: gpu02_wayland --scene-stream /absolute/path/to/scene-stream.v1.json. No service, BLOB-FRAME-v1, control-plane, or agent authority change.

## Reproducible local observations

- Rust replay suite: 4/4 PASS, including stale/oversized/unknown-identity negative inputs. cargo fmt, cargo clippy --all-targets -D warnings, and cargo build --offline --release: PASS.
- TypeScript scope: 7 relevant tests PASS (5 delta laws + 2 TS/Rust fixture equivalence). Full npm lint, typecheck and Vitest: 34 files / 136 tests PASS.
- Biome scoped format: PASS. Global npm run check returns RED only at root formatting stage for nine pre-existing experimental JSON receipts outside the changed source; historical bytes deliberately unchanged.
- Actual Wayland GPU: NVIDIA RTX 3070 / Vulkan. The app logged GPU03_SEMANTIC_SCENE_ACCEPTED frames=4. The source string in the JSON is untrusted metadata, NOT provenance authentication.
- Static camera live captures with grim from the window itself, distinct original PNG SHA-256:
- fixed-camera-a.png: a30e9b869ba58d48965c46968179b70d82c7ba48a7869b2288461fe8933452eb
- fixed-camera-b.png: e2e8ae893d9a313834b99041119ed5ccef8fd7c6d22c3d6ee0e7bf029bc4a077
- The visible SDF cavity moves within an otherwise stationary tetrahedral scene, establishing a causal visual path from the replayed source coordinate into the GPU shader. This does NOT qualify arbitrary 3D geometry or per-frame GPU oracle parity.
- CLI invalid delta test returned exit=1 and STALE_OR_INVALID_DELTA **before Wayland/GPU window creation**.
- Application logs record about 74.9–76.1 queue-present submissions per second on the chosen HDMI-A-1 monitor (~74.964 Hz). This is not independently measured compositor presentation FPS.
- Evidence directory: /mnt/workbench/evidence/runs/blobin-next-01/gpu03-native-20261010/

## Open boundaries

1. The scene stream is a finite file loaded fully at startup. No live source IPC, no authorized delta ingress, no remote control, no agent identity binding.
2. 2D rectangular entity x motion is a narrowly explicit mapping to one 3D SDF cavity parameter. There is no general 2D→3D scene compiler, mesh identity transfer or in.gnu6 Rust runtime port.
3. Scene source field is only a label; no signed stream integrity, independent trusted signer, or Gnosix ScopedCapability authorization.
4. No wp_presentation feedback timestamps, compositor p95/p99, 120-second dwell, focus/input-to-photon response, GPU reset or multi-vendor QA.
5. The reference 640×432 compute grid is fixed; window scaling changes presentation size, not raymarch resolution. Some intersections reveal aliasing/sampling artifacts; appearance is not quality-approved.
6. Standalone wgpu uses direct Vulkan without CUDA-05F supervision; it is not the production renderd worker and cannot inherit its grants.
7. CP02 seven gates, CP03 HYBRID-60, CP07 alpha and CP08 cross-vendor remain OPEN. No push, merge to main, deployment or user-service mutation.

## Integrator handoff

- First establish compositor presentation feedback through a bounded observer and distinguish queue submissions from compositor presentation.
- Design authenticated bounded scene IPC as a separate versioned interface; keep semantic truth and renderer authority disjoint.
- Reconcile branch GPU03 with parallel CP01 GEO-05 and GPU-01 work only after comparing their exact HEAD, dirty state and test fixtures.
- For future architecture diagrams, global diagram-design skill is installed but style-guide.md is still default and ~/.diagram-design/profiles/ contains no named profiles. Do not publish default-skinned Gnosix diagrams; configure a branded project profile via the tool's onboarding gate first.
