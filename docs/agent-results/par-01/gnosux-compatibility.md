# A04 — in.gnu6 TS/scenic vs gnos.ux Rust/GPUI: compatibility map

Worker A04, batch IN-GNU6-PAR-01. Status: research note. No interop is postulated, designed for adoption, or authorized.

## Provenance

| Item | Value |
| --- | --- |
| in.gnu6 worktree base | `c9dcdab997f6` (branch `feat/par-01-A04`) |
| Gnosix repo (read-only) | `/mnt/workbench/workspaces/gnu.in.labs/gnosix` @ `33ef40100c055490320f8051db4017b6fc4987d0`, branch `feat/keybindings-01b`; status showed only untracked `docs/`, `jean.json` (not mine, untouched) |
| Rust crates read | `gnos-ux-surface` (`scene.rs`), `gnos-ux-product-intent`, `gnos-ux-sdk` (`lib.rs`), `gnos-ux-gpui` (`scene_runtime.rs`, grep-level only), `Cargo.toml` of all five `gnos-ux-*` |
| TS read | `src/scene/{engine,lifecycle}.ts`, `src/algebra/types.ts`, `src/renderer/contract.ts`, `src/topology/instance.ts`, `src/interaction/projection.ts`, `src/dynamics/clock.ts` (header), `CONTRACT.md` |

Not read in depth: `scene_runtime.rs` (5832 lines), `reactive_envelope.rs`, `connective_hull.rs`, `gnos-ux-envelope-qualify.rs`, `gnosix-desktop-model` beyond id type definitions. Statements about them are `UNKNOWN` unless a line says otherwise.

## Commands actually run

1. `CARGO_TARGET_DIR=<scratchpad>/target cargo test --offline --locked -p gnos-ux-surface --lib scene::` run from the Gnosix repo. Result: 6 passed, 0 failed. The build target was redirected outside the Gnosix tree; Gnosix `git status` was unchanged afterwards.
2. In this worktree, `node_modules` was copied from the sibling `in.gnu6` checkout (HEAD `d160639`, which is not the base SHA; lockfile equality was not checked), then `npx vitest run`: 32 files, 129 tests passed. The targeted run of `renderer-contract-laws`, `topology-ownership-laws`, `topology-engine` and `introspection-frame-laws` gave 25 passed. The copied `node_modules` was deleted afterwards (it is gitignored).
3. Source reading with `cat`, `sed` and `grep`.

Not run: any GPUI/Wayland runtime, the `gnos-ux-gpui` test suite, any cross-language experiment. The cargo test proves the Rust scene kernel passes its own tests at that SHA. It says nothing about interoperability.

## Classification legend

`SAME` means the same concept with compatible semantics, observed in source on both sides. `RELATED` means an overlapping concept with a different owner, shape or semantics. `DIFFERENT` means no shared concept, or a conflicting one. `UNKNOWN` means not established by what was read. Each row also carries a claim status: `VERIFIED_NOW` (source read or test run in this session), `HYPOTHESIS`, `UNKNOWN`.

## Authority map

| Axis | in.gnu6 (TS) | gnos.ux (Rust/GPUI) | Class | Status |
| --- | --- | --- | --- | --- |
| Scope of the engine | Product-agnostic scene. `CONTRACT.md` §2 says it does not know Workspace, Dock or application. | `MonitorScene` hard-codes roles `Bar`, `Body`, `Dock`, `AppStack`, `Notification`, `Overlay`, `Plugin` and a scripted `SceneCycleStep` sequence (rest, dock-reveal, ..., return-rest). | DIFFERENT | VERIFIED_NOW (source) |
| Scene identity | `EntityId`, an open string brand, injective in `SceneEntity` (`types.ts` identity law). Scene identity is the entity set E. | `SceneNodeId(u64)` with four constants (`BAR`=1, `BODY`=2, `DOCK`=3, `APPSTACK`=4). `MonitorSceneId(u64)` names the monitor scene. | RELATED | VERIFIED_NOW |
| Product / desktop identity | None by design. | `WorkspaceId` and `SurfaceId` are `[u8;16]`, wrapped with a `DesktopContextGeneration(u64)` in `Observed*Reference` (product-intent crate). These are desktop-model identities, not scene identities. | DIFFERENT | VERIFIED_NOW |
| Identity mapping scene to product | Not defined (consumers own product state). | `SceneNodeId` is explicitly "not product identity" for `Body`. The mapping from `SceneNodeId` to `SurfaceId`/`WorkspaceId` is not visible in the files read. | UNKNOWN | UNKNOWN |
| Lifecycle | `diffSceneLifecycle`: born, persisted, terminated as set arithmetic over E. Handoff and shape change do not imply birth or death. Mutations are transactional (`mutate` commits only if laws hold). | `SceneEpoch(u64)` increments per `tick`; `scene_id` stable across ticks (test `epoch_is_monotonic_while_scene_identity_remains_stable`). `rebase_viewport` preserves identity (test). Node set was observed fixed at 3 in the first test. No born/terminated delta exists in the files read. | RELATED (both separate identity persistence from time; different mechanisms) | VERIFIED_NOW |
| Scene vs UI intent | `RendererFrame` flows downstream only. Renderer pointer observations are facts, not authority. `InteractionProjector` has no mutation capability. | `ProductIntent` is passive and carries no capability, executor, transport or authorization (doc comment in `gnos-ux-product-intent`). The intent types address desktop workspaces/surfaces, not scene nodes. | RELATED (same discipline: observation and intent are not authority; different targets) | VERIFIED_NOW (source) |
| Geometric projection | `RendererEntityFrame` carries world `Pose2` (position, rotation, 2D scale) plus `Rect` or `Polygon` shape, z, visibility and optional accessibility. | `SceneNodeFrame` carries `envelope: GeometryRect` and `velocity: Vector2`. Relations carry a `CubicJoinPair` connector and a `presence` in [0,1]. `input_regions` and `reservation` are scene-level outputs. | RELATED | VERIFIED_NOW |
| Relations | `SceneRelation` kinds `attached` or `ext:<name>`. Set semantics. | `SceneRelationFrame` with state `Connected`/`Bridged`/`Detached`, presence and a Bezier connector. | RELATED; the vocabularies do not map one-to-one | VERIFIED_NOW |
| Rotation, non-rect shapes | Pose has rotation; polygon shapes exist. | Rect envelopes and bezier connectors seen; rotation and polygons not seen in `scene.rs`. | UNKNOWN for the wider `grammar` module | UNKNOWN |
| Time and dynamics | Fixed-step scheduler `dt = 1/hz`, f64 numbers, records dropped steps explicitly. Rendering cadence cannot change integration order. | `tick(dt: f32)`. In `scene_runtime.rs`, wall-clock `Instant` deltas feed `advance_observed(..., dt)` (grep-level; I did not read the surrounding logic). | DIFFERENT as a determinism contract (fixed-step versus caller-supplied dt); UNKNOWN whether Rust is replay-deterministic | VERIFIED_NOW for signatures; behavior UNKNOWN |
| Threading | JS single thread; no shared mutable state exposed. Renderer frames are deeply frozen. | `Arc<Mutex<..>>` lifecycle/projection state, `mpsc`, `thread::spawn`, GPUI async `cx.spawn` appear in `scene_runtime.rs`. The thread that owns `MonitorScene` was not determined. | DIFFERENT | grep-level only |
| Serialization | No wire format is defined. `rendererSemanticDigest` uses `JSON.stringify` of `sceneTime` and entities, i.e. an in-process digest, not a protocol. | `gnos-ux-surface` and `gnos-ux-product-intent` have no serde dependency. `gnos-ux-gpui` has optional `serde_json` only under `component-lab`. Debug traces are hand-formatted JSON strings in `scene_runtime.rs`. Transport to the activation protocol is zbus (`gnos-ux-activation-client`). | DIFFERENT | VERIFIED_NOW (Cargo.toml and source) |
| FFI | None; `private: true`, public API UNDECIDED (`CONTRACT.md` §8). | None seen; crates are `publish = false`; `gnos-ux-sdk` calls itself "not yet a stable public ABI". | SAME (neither exposes a stable boundary) | VERIFIED_NOW |
| Provider / adapter model | `RendererAdapter<T>`: `render(frame) -> {emittedEntityIds, omitted, output}`. `validateRendererResult` forbids inventing, duplicating or silently dropping identities. | GPUI is the host. `SceneFrame.validate()` checks duplicate nodes, outside-viewport, dangling relations, non-finite scalars. The scene is the producer, with GPUI painting from it. A replaceable-renderer contract was not seen. | RELATED; renderer inversion of control is UNKNOWN | VERIFIED_NOW (source) |
| Latency | PixiJS benchmark harness exists (`test:performance`). No cross-language figure. | Motion trace and perf counters exist in `scene_runtime.rs`. No cross-boundary figure. | UNKNOWN | NOT_RUN |
| Invariant checking style | `assertSceneLaws` on every mutation; 129 tests across 32 files pass. | `SceneFrame::validate` returns a violation list; 6 scene tests pass. | RELATED | VERIFIED_NOW |

### Summary of the map

- The sharpest overlap is discipline, not data: both separate stable identity from time and epoch, and both keep observation and intent from becoming authority.
- The data models are not interchangeable. The TS side is an open entity set with general poses. The Rust side is a fixed, named-role scene with scripted choreography and bezier connectors.
- Identity is the main trap. A TS `EntityId` (string) and a Rust `SceneNodeId` (u64 constants) do not share a namespace, and neither is the Rust desktop identity (`SurfaceId`/`WorkspaceId`, 16 bytes with generation). Treating any pair as equivalent would be an authority amplification by naming (TRONC_COMMUN principles 10, 19).
- Nothing here selects a language, a GPU path or an owner of the scene. `AGENTS.md` and `CONTRACT.md` forbid inferring that from filenames or dependencies, and the Gnosix `AGENTS.md` lists GNOS.UX production process-host and lifecycle ownership as still OPEN.

## Falsifiable boundary hypothesis (gate)

Hypotheses, none run.

**H-A04-1 (identity-lossless read-only projection).** A one-way projection from the Rust `SceneFrame` into a TS `SceneState`-compatible description can preserve node identity and relative order without any change to the Rust types, because `SceneNodeId` is injective within a frame (`SceneFrame::validate` rejects `DuplicateNode`).

- Expected observation if true: for each recorded frame, the mapping `u64 -> EntityId` (for example the decimal string with a fixed prefix) is injective, `assertSceneLaws` accepts the result, and `diffSceneLifecycle` over consecutive frames yields `born=terminated=empty` while the Rust epoch grows.
- Falsifier: any recorded frame where the mapped state fails `assertSceneLaws` (for example a dangling `attached` relation, a vertex-less shape, or non-finite values), or where `diffSceneLifecycle` reports spurious birth or death across ticks.
- Instrument: a recorded-fixture replay. Dump N `SceneFrame`s from a deterministic Rust test (`tick(1/60)` from `MonitorScene::new`, as in the existing tests) to a file, then replay in a vitest. This needs new test code on both sides and is not authorized under this write-set.
- Disposition: true means H-B1 below becomes worth proposing. False means the identity model is not shared and the bridges stay observation-only digests.

**H-A04-2 (geometry is the lossy axis).** Mapping a Rust `envelope: GeometryRect` to a TS `RectShape` plus `Pose2` is representable. The bezier connector and `presence` have no TS counterpart and would be dropped or forced into `ext:` relations.

- Expected observation: connector data cannot round-trip. Falsifier: a TS relation encoding that preserves the `CubicJoinPair` without inventing core vocabulary.
- Instrument: a paper-level or fixture-level comparison, with no code needed in the core.

**H-A04-3 (determinism asymmetry).** The Rust scene is not replay-identical to a fixed-step TS run for the same wall-clock input, because dt is caller-supplied f32.

- Instrument: run the Rust tick N times with a fixed dt in two processes and compare frames bit-for-bit. Unrun. This is a statement about an f32 pipeline, not about the live runtime.

## Minimal reversible bridge proposals

None is authorized. Each is ranked by uncertainty (U: low means well supported by read source) and by authorization needed.

| ID | Proposal | Reversibility | U | Authorization needed |
| --- | --- | --- | --- | --- |
| B0 | Do nothing, and keep this map as the shared vocabulary for future discussion. | Total | none | none |
| B1 | Offline frame-replay fixture (H-A04-1): Rust dumps recorded `SceneFrame`s as JSON, a TS test reads and checks laws. Read-only on both products; the file is the only coupling. | Delete the fixture | medium (H-A04-1 and H-A04-3 pending) | An operator decision to touch a Gnosix test or example (outside A04's write-set), and a decision on where fixtures live. No product code change. |
| B2 | Identity-mapping table document: an explicit, reviewed table of which TS `EntityId` corresponds to which Rust `SceneNodeId` for a given demo scene. No code. | Delete the document | low for the document, high for the mapping (the `SceneNodeId` to product-identity mapping is UNKNOWN) | Operator review of the mapping. It must not imply durable identity; Gnosix lists durable workspace/application identity as OPEN. |
| B3 | A TS `RendererAdapter` that consumes a recorded Rust frame and renders it with PixiJS, with Rust never calling TS. Tests the downstream-only claim. | Delete the adapter | high (needs the lossy geometry decision from H-A04-2) | Operator decision; in.gnu6 `CONTRACT.md` §8 keeps the public API UNDECIDED, so this stays internal. Needs a consumer registration. |
| B4 | Live IPC (for example the existing zbus path) or an FFI binding between runtimes. | Hard to reverse | very high | `BLOCKED_AUTHORITY`. Out of this contract's scope: no interop code, SDK, FFI, GPU migration or language decision. |

Recommended ordering if the operator wants to proceed: B0, then B2, then B1 to settle H-A04-1 and H-A04-3, and only then reconsider B3. B4 is not recommended without new evidence.

## Claim ledger

| Claim | Status |
| --- | --- |
| Gnosix `gnos-ux-surface` scene tests pass (6/6) at `33ef401` | VERIFIED_NOW |
| in.gnu6 test suite passes (129/129) with dependencies copied from the sibling checkout, not a lockfile-verified install | VERIFIED_NOW (qualified) |
| Identity types and lifecycle mechanisms differ as tabulated | VERIFIED_NOW (source) |
| No serde or wire format in `gnos-ux-surface` and `gnos-ux-product-intent` | VERIFIED_NOW (Cargo.toml) |
| Rust `SceneFrame` can be projected into TS scene laws losslessly | HYPOTHESIS (H-A04-1), NOT_RUN |
| Connector geometry cannot round-trip into the TS core vocabulary | HYPOTHESIS (H-A04-2), NOT_RUN |
| Rust scene not replay-deterministic like TS fixed-step | HYPOTHESIS (H-A04-3), NOT_RUN |
| Mapping from `SceneNodeId` to `SurfaceId`/`WorkspaceId` | UNKNOWN |
| Which thread owns `MonitorScene` in the live runtime; latency across any boundary | UNKNOWN / NOT_RUN |
| Any live interop, FFI or IPC | BLOCKED_AUTHORITY |

## Non-claims

This note does not claim that the two systems are, or should become, compatible. It selects no language, no renderer and no GPU path. It does not claim live runtime behavior (no GPUI or Wayland session was run) or anything about `scene_runtime.rs` beyond the quoted grep-level facts. It authorizes no publication or API, and no change to Gnosix or in.gnu6 core. The Gnosix OPEN authority items (profile/config ownership, durable workspace/application identity, production process-host and lifecycle) are untouched.

## Next decision (operator)

Choose between B0 (stop here) and authorizing B1 or B2. B1 needs a decision on where fixture code may live, since it touches the Gnosix tree, which A04 treats as read-only. Separately, someone with Gnosix authority could say whether the `SceneNodeId` to desktop-identity mapping is meant to exist; that settles the largest UNKNOWN.
