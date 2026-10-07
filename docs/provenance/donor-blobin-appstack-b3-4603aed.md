# Donor receipt — blob.in organic-shape engine

Status: **OBSERVED DONOR — PARTIALLY REPRODUCED — NOT AUTHORITY**

Intake date: 2026-10-07

Source snapshot:
- path: `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in`
- snapshot label: `appstack-b3-baseline-4603aed`
- package version declared by the donor: `0.14.0`
- repository metadata declared by the donor: `https://github.com/gnu-in-labs/blob.in`
- license declared by the donor: `GPL-3.0-or-later`
- files in snapshot manifest: **95**
- SHA-256 of sorted per-file SHA-256 manifest:
  `5a7831ec1293c0dd810cef11c4b953bba6a891c2065b24ac63b43f4f40218b2d`

## Revision-boundary qualification

The suffix `4603aed` is part of the preserved baseline directory name and associated build logs.

That short revision could not be resolved to a surviving Git object in the currently available local
repositories. Therefore this receipt does **not** claim that `4603aed` was independently re-proven as
a Git commit.

The durable intake identity for this reconstruction is the snapshot path plus the 95-file manifest
digest above.

No source bytes were modified in the preserved donor snapshot.

## What is actually present

The snapshot contains a substantial Rust/C++/Qt engine, not merely a showcase or design document.

Rust `blobin-core` includes:
- cubic Bézier `Path` / `Contour` primitives;
- rounded `RectBlob`;
- superellipse / squircle `SuperBlob`;
- periodic organic wobble;
- winding inversion through `Inverted<B>`;
- affine 2D group composition via `glam`;
- `lyon` fill tessellation into flat vertex/index buffers;
- damped spring and cubic-Bézier easing solvers;
- mascot state interpolation;
- dirty/cold engine lifecycle;
- multi-engine scheduler with optional Rayon parallel tick;
- CPU material raster oracle and pixel-diff helpers;
- `cxx::bridge` FFI;
- BubbleBloom radial recipe.

The Qt side contains:
- `QQuickItem` render shims;
- shared mesh upload path;
- `QSGGeometry` / `QSGMaterial` plumbing;
- GLSL vertex/fragment shaders;
- shader compilation through Qt RHI tooling.

The workspace also contains:
- token generator;
- CXX-Qt reactive surface;
- golden material tests;
- GPUI consumer adapter;
- Criterion benchmark crate;
- R7/R8/R9/R10 review/sign-off tooling.

## Fresh reproduction performed during SQ-003

### Original snapshot workspace

A direct `cargo test -p blobin-core` from the preserved workspace is currently blocked before
compilation because the `blobin-gpui` workspace member depends on:

`../gnosis-gpui-theme/Cargo.toml`

That dependency is absent from the exported baseline.

The same missing dependency blocks direct Qt CMake configuration because Corrosion requests Cargo
workspace metadata first.

Therefore the claim “the preserved snapshot is a self-contained green workspace” is **false**.

### Isolated blobin-core

A byte copy of `blobin-core` was qualified outside the donor tree. The only qualification edit was
removing the inherited `[lints] workspace = true` stanza so Cargo could treat the crate as standalone.

Fresh results:
- default feature set: **47/47 tests PASS**;
- `--no-default-features` sequential scheduler: **47/47 tests PASS**;
- explicit `--features threaded-solver`: **47/47 tests PASS**.

This freshly proves that the core Rust mechanics compile and execute independently of the missing
consumer package.

### Reconstructed workspace excluding only the missing GPUI member

A temporary copy of the snapshot was created for qualification. The only workspace-structure change
was removing `blobin-gpui` from the member list; donor implementation source was left unchanged.

Fresh results:
- workspace tests excluding benchmark crate: PASS;
- `blobin-core`: 41 unit + 3 lifecycle + 3 scene tests PASS;
- `blobin-cxxqt`: 1 test PASS;
- `blobin-gen`: 7 tests PASS;
- `blobin-golden`: 4 material parity tests PASS;
- Qt 6 / Corrosion configure: PASS;
- Rust release static library build: PASS;
- `cxx` bridge generation/link: PASS;
- Qt shader `.qsb` generation: PASS;
- C++ `blobinqt` module compile/link: PASS.

This proves the Rust→CXX→Qt render shim is buildable from the preserved source once the unrelated
missing GPUI workspace member is excluded.

It does **not** prove that the original unmodified exported workspace was complete.

### Static guards

Fresh against the preserved donor:
- `tools/no-unsafe.sh`: PASS;
- C++ shim LoC guard: **282 / 300**, PASS.

### Historical GPU evidence integrity

The preserved R8 RTX 3070 evidence directory contains a committed checksum manifest.

Fresh `sha256sum -c checksums.sha256`:
- README: PASS;
- golden-vs-hardware sheets: PASS;
- idle/listening/transmit/veille PNGs: PASS.

This proves the evidence files are internally intact. It does not rerun the 2026-06-11 GPU experiment.

## Claim classification

### Reproduced now

**Rust core exists and is executable.**

Fresh Rust tests prove geometry, state, solver, tessellation, scheduler and lifecycle code is real.

**lyon is genuinely used.**

`blobin-core/Cargo.toml` depends on `lyon`, `tessellate.rs` lowers donor paths to
`lyon::path::Path` and uses `FillTessellator`; fresh tests compile and exercise this path.

**The cxx bridge is real.**

`ffi.rs` uses `#[cxx::bridge]`; fresh standalone and reconstructed builds compile the bridge.

**The Qt shim is real and thin.**

Fresh reconstructed Qt build links `libblobinqt.so`; the preserved LoC guard reports 282 lines
against the 300-line budget.

**Sequential/threaded core configurations are both executable.**

The same 47-test core suite passes with and without `threaded-solver`.

**Material parity oracle code is executable.**

The reconstructed workspace passes four `blobin-golden` material parity tests.

### Supported by preserved historical evidence, not freshly rerun

**R7 frame budget.**

The donor records an operator-signed 2026-06-10 result at commit `d07d161`, including
`scheduler_tick_all/16 ≈ 81.6 µs` on an i7-12700F reference box.

This is historical evidence, not a fresh benchmark.

**R8 software/hardware shader parity.**

The donor records:
- llvmpipe software path: 4/4 pixel-perfect;
- RTX 3070 hardware numeric pass;
- later operator qualitative sign-off.

The preserved hardware artifact checksums validate today, but the GPU run itself was not repeated.

**R9 functional/visual fallback review.**

The donor contains signed historical rows after the Qt-5 premise was re-scoped to the Qt-6
reference build.

Those rows are historical receipts. They are not a fresh fallback execution in SQ-003.

### Explicitly not proven / contradicted

**The pristine exported workspace is self-contained.**

Contradicted by the missing `gnosis-gpui-theme` path dependency.

**`BLOBIN_BACKEND=cpp` selects a complete legacy C++ geometry engine in this snapshot.**

Not proven. `backend.rs` implements parsing/reporting, and the donor README itself says:

> wire the `cpp` backend's actual legacy path

is still a later task.

Therefore the backend switch is a contract/selectable label in this source, not a freshly proven dual
semantic engine.

**R10 / production-default dogfood is complete.**

False. `tools/r10-sign-off.json` remains `pending` with no completed 14-day window.

**The snapshot suffix `4603aed` is a currently recoverable Git revision.**

Not proven from the repositories currently available.

## Technical capital extracted into [in.gnu6]

The donor does not become engine authority. Its reusable capital is narrowed to laws that survive
outside mascot, Qt and product semantics.

### Law BLOB-L1 — representation lowers to renderer-neutral geometry

A high-level shape may lower to an ordered closed contour/path before renderer projection.

Reusable statement:

> shape semantics and renderer/provider representation are separate stages.

This is already compatible with `[in.gnu6]`'s `Shape2 → normalized contour → renderer` boundary.

### Law BLOB-L2 — winding is topological data

The donor's `Inverted<B>` reverses winding to express a hole.

Reusable statement:

> ordered contour orientation is meaningful and must be explicit/normalized before correspondence.

This reinforces `signedArea`, `ensureClockwise` and the contour correspondence laws already present
in `[in.gnu6]`.

### Law BLOB-L3 — damped spring evolution uses semi-implicit integration

The donor uses:

`m x'' + c x' + k(x-target) = 0`

and updates velocity before position.

Reusable statement:

> interactive spring dynamics must be deterministic from explicit
> `(position, velocity, target, stiffness, damping, mass, dt)`.

A revision-bound numeric fixture for donor token `SETTLE` is now stored in
`tests/fixtures/blobin-donor.ts` and checked against `[in.gnu6]`'s current scalar integrator.

The donor's additional long-frame substepping is **not** silently imported by this quest.

### Law BLOB-L4 — continuous shape change does not imply identity replacement

The donor evolves one engine/shape through state-driven silhouette changes.

Reusable statement:

> geometry may change continuously while the semantic subject remains stable.

The extracted fixture checks this against `ShapeMorphTrack.subjectId` plus normalized contour
cardinality/winding.

### Law BLOB-L5 — dirty/cold is scheduling metadata, not semantic state

The donor distinguishes:
- semantic mascot state;
- animation activity;
- whether a mesh must be uploaded;
- whether the host may throttle.

Reusable candidate:

> render/update scheduling state must not become product/scene identity state.

This is recorded as technical capital only. It is not added as new mandatory core ontology in SQ-003.

### Law BLOB-L6 — provider work can be bounded at one flat frontier

The donor's architecture keeps geometry/state computation behind a flat vertex/index/material FFI
frontier and measures C++ shim growth.

Reusable candidate:

> a replaceable provider boundary should carry projection data rather than acquire semantic authority.

This agrees with MQ-11 but the exact CXX/Qt implementation is provider-specific and is not imported.

## Provider-specific capital deliberately not imported

The following remain donor/provider choices:
- Qt Quick;
- QSG/RHI;
- CXX / CXX-Qt;
- Rayon;
- lyon;
- GPUI token output;
- mascot states and visual identity;
- BubbleBloom recipe;
- blob-specific shader material/noise;
- the 300-LoC numeric threshold;
- `BLOBIN_BACKEND` environment naming.

## Extracted fixtures

`tests/fixtures/blobin-donor.ts` contains:
- snapshot identity/digest;
- donor `SETTLE` spring coefficients;
- one exact one-substep numeric oracle;
- one neutral source/target shape pair and sample cardinality.

`tests/model/blobin-donor-laws.test.ts` proves the portion of the donor laws that is intentionally
retained by current `[in.gnu6]` abstractions.

No blob.in source was copied into `src/`.

## Disposition

SQ-003 may close when:
- the extracted fixture passes the normal `[in.gnu6]` gate;
- this receipt is committed with the snapshot digest;
- Questlog records the proven/unproven split.

The reconstruction does **not** promote blob.in into a Main Quest and does not alter MQ-15's existing
completeness claim.
