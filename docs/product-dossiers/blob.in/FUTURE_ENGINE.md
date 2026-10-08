# BLOBIN-NEXT-01 — Rust GPU-first / CPU-complete rendering contract

**Status:** DESIGN_PROPOSAL / NO_PRODUCTION_IMPLEMENTATION / PRIVATE. 2026-10-08.
**Authority:** operator's latest product direction: Rust engine, GPU-accelerated primary, CPU-compatible secondary. This **supersedes the proposed future C++ fallback as a target architecture**; it does not retroactively change the historical Qt/CXX source or R9. **No release or license/publication decision.**

## Current truth vs future design

Historical snapshot `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in`, 95 files, SHA-256 manifest `5a7831ec1293c0dd810cef11c4b953bba6a891c2065b24ac63b43f4f40218b2d`:
- Present: Rust `blobin-core` geometry / Bezier / superellipse / inversion / tessellation Lyon / springs / materials / CPU material patch reference; Qt/CXX shaders; golden/pixel comparison; GPUI adapter blocked by a missing sibling crate.
- Not qualified: native standalone `wgpu` renderer, GPU compute signed-field pipeline, heightmaps, multi-pass bevel/post-effects, GPU/CPU runtime switching, CPU feature parity of future materials.
- Historical QA: B01 isolated Rust tests PASS, Qt/GPU historical only; R10 pending; C1–C4 version/license/default backend contradictions **open**.

No core product source may be imported from the historical snapshot into another project until provenance and licensing authority is settled. A fresh independent technical spike may use **newly authored code** in an isolated lab.

## Non-negotiable invariants

1. **Rust owns semantic state:** topology, identity, geometry, hit-test and time are independent of the renderer. GPU output is projection, not the source of truth.
2. **Two functional backends:** `GpuWgpu` (primary), `CpuReference` (secondary); no dependency on C++, Qt, GPUI or a physical GPU to evaluate the same scene. CPU cannot silently skip selection, geometry, hit-testing or accessibility. Optional expensive light-scattering/temporal passes may downgrade visibly and explicitly.
3. **Stable identity across topology:** topology epochs track merge/split/cutout; a component disappearing from a mesh does **not** silently delete an interactive entity. State migrations have owner, continuity and accessibility tests.
4. **Frame determinism:** fixed semantic timestep, seeded procedural fields, serializable events, stable ordering and a `frame_semantic_digest`. GPU/CPU pixels may differ within declared tolerances; *semantic* outcomes must agree.
5. **Color correctness:** sRGB input converted to linear working space; alpha premultiplied once; compositing rules tested with translucent nested bevels, gradients and noise. Internal HDR/wide gamut are optional capabilities, not baseline.
6. **Fail-closed capability handling:** `BackendCapabilities` names storage textures, compute, MSAA, max texture dimensions, readback and device loss. Unsupported shaders/effects report explicit degraded mode, not invisible partial paints.
7. **Low-power by construction:** no frame when scene and effects are cold. Dirty rect + effect halo tracking controls re-rasterization. Animated time and heatfield decay are explicit reasons to wake.
8. **Privacy and operability:** heatmaps visualize synthetic/local user-approved signals, never undisclosed behavioral collection. Lost GPU, resize, DPI shift, nonfinite geometry, and OOM have deterministic bounded fallback and diagnostics.
9. **Accessibility is not a raster effect:** UI semantics, contrast, keyboard and reduced motion live above pixels; glow/bevel never communicate state alone.
10. **No authority amplification:** blob.in renderer can display an authorized surface, not mint GNOSIX desktop/window capabilities. GPUI integration must be an adapter contract, not ownership migration.

## Intended decomposition

```text
Product Interaction Intent (consumer-owned)
     │ typed event; no renderer privileges
     ▼
Rust Scene / Topology Grammar  ← stable IDs, topology epochs, parent graph
     │ compute shape IR + selection/hit regions
     ▼
Canonical Field Model  (signed distance + coverage + height + material ids)
     │ deterministic frames and source events
     ├──── CPU oracle: path tessellation / tile raster / material evaluator
     │                       │
     │                       └── CpuFrame_RGBA_linearPremult
     ▼
GPU rendering adapter (wgpu + WGSL)
     │  capability probe and shaders validated with Naga
     ├─ field generation [vector/mesh/SDF atlas] + clipping/masks
     ├─ height/normal/curvature [bounded resolution]
     ├─ materials [base, metal/ceramic/paper/gel parameters]
     ├─ edge/bevel lighting [multi-source directional, energy bounded]
     ├─ layered effect graph [shadow, rim, heat, bloom, blur, trails]
     └─ final tone-map/composite [linear premult → output color space]
     ▼
Host adapter: offscreen texture, standalone winit, or future GPUI
     └─ CPU fallback surface on init failure / device loss
```

**Avoid a 'one huge shader' architecture.** Render graph passes declare inputs, outputs, scale, coordinate system, lifetime, dirty region, time dependency, capability requirements and fallback. Effect nodes are bounded and reviewed; no arbitrary executable shaders accepted from untrusted clients.

## Geometry & topology

- `ShapeGraph`: closed curves and contours; intersections / union / subtraction / smooth-union; multiple disconnected components; holes; winding even-odd/nonzero; clipping and nested transforms.
- `Field2D`: signed distance `d(x,y)`, per-pixel edge coverage and gradient. Support analytic primitives when possible; GPU sampled-field atlas for arbitrary curves, bounded CPU distance transform as fallback; field fidelity measured.
- `TopologyEvent`: `Connect`, `Disconnect`, `Split`, `Merge`, `PunchHole`, `SealHole`, `Reparent` with before/after components and continuity proof. A blend between two SDFs is not by itself a semantic topology event.
- `ElevationField`: height from unsigned or signed distance, local curvature and interaction impulses. `normal = normalize(-∂h/∂x,-∂h/∂y,1)`. GPU computes normals from heightmap; CPU reference uses matched finite-difference convention.
- `BorderProfile`: inside/outside/center alignment, variable width, curvature-aware gradient, bevel width, corner treatment, dash/rhythm. Self-intersecting transparent stroke needs a *single coverage accumulation* rule: simple triangle-stroke overlap is insufficient.
- `MaterialGraph`: reusable evaluators (solid, gradient, stratified, ceramic, soft translucent, brushed, noise), not per-consumer ad hoc fragment scripts.
- `HeatField`: explicitly sourced local scalar field with spatial kernels, bounded decay, normalization and privacy. Heat is visual information, not a data collection mandate.

## GPU/CPU tier contract (candidate, not benchmarks)

| Domain | GPU primary | CPU compatible |
|---|---|---|
| Flat/curved fills, holes, borders | wgpu meshes/coverage/SDF | Rust path/tiles/coverage |
| Heightmap, bevel, rim light | WGSL field + normal/light passes | Rust finite differences + scalar light |
| Heatmap, local impacts | GPU field accumulation/blur | Rust tile field + separable blur |
| Shadows/blur/bloom | Multi-resolution post passes | Reduced-resolution or fewer taps with clear quality grade |
| Animated temporal trails | Ping-pong textures / explicit feedback | Limited history or disabled visual trail, preserving interaction and scene |
| Device loss / unsupported compute | Recreate device or drop to graphics pass/CPU | Full semantic scene remains available |
| Text/a11y/hit testing | Host semantics; engine hit regions | Same host semantics; never shader-only |

**Candidate dependencies (benchmark, don't lock prematurely):**
- `wgpu` + WGSL/Naga for native GPU/custom compute; `lyon` for reliable fill and stroke geometry.
- Compare `vello_gpu` for vector rendering and `vello_cpu` for software rasterization against custom code. Vello GPU and CPU have differing maturity; do not assume matching custom shader/effects or a stable cross-renderer public API.
- Existing raster.rs serves as a **historical material** oracle, not as complete CPU renderer for future effects.
- GPU hardware currently observable: NVIDIA RTX 3070 / Vulkan 1.4; targets should also be tested without GPU and on a software adapter.

## Performance/evidence gates

Targets are **budgets to falsify**, not current performance claims:
- 1080p 60 Hz: entire active interaction frame <=16.67 ms p95 on declared reference device; stretch 120 Hz =8.33 ms; separately report simulation vs upload vs field passes vs composite vs presentation. No averages alone.
- Idle: zero renderer submits after cold transition; quantify CPU utilization, memory, GPU wake and power with real instrument.
- CPU: UI actions/hit-tests remain usable under a chosen 30 Hz quality tier on a declared reference CPU; no scene loss. CPU quality differences exposed by quality profile.
- Visual gates: edge RMS/SSIM with masks, signed-distance gradient error, contour Hausdorff distance, color delta in linear space, premultiplied transparency fringes, topology-component count, topology-epoch/ID continuity, screenshot evidence at 1×/2× DPR and dark/light colors.
- Failure injection: GPU missing/device lost, zero-sized viewport, extreme border width, huge blur radius, nonfinite inputs, nested clipping and 50 simultaneous organic shapes. All have bounded memory/time/error handling.
- Accessibility and motion: keyboard-only traversal, high contrast, reduced-motion static outcome, zoom 200%, and visual state not conveyed exclusively by color/heat.
- Each new effect ships **(a)** Rust reference maths, **(b)** WGSL validation, **(c)** deterministic fixtures, **(d)** comparisons, **(e)** profiling, **(f)** explicit degradation behavior.

## Immediate milestone path

**F0 / identity & license gate:** determine where standalone blob.in code may live, source provenance and authoritative license. Block copying historic source, not new independent experiments.

**F1 / renderer contract & 1st vertical slice:** newly authored `FrameIR` + `CpuReference` and `GpuWgpu` both render the **same rounded/organic contour and beveled border** to an offscreen image; headless and software-only passes required.

**F2 / topology challenge:** two connected blobs merge/split with stable product IDs, holes and occlusion; output deterministic topology-event log and border geometry.

**F3 / material & height:** gradient, heightfield, ceramic/burnished bevel, heat field, single correct premultiplied composite. Compare GPU vs CPU with tolerances.

**F4 / effect graph + resilience:** shadows, bilateral blur, bloom, material layering, temporal echoes, device loss, CPU fallback; add dirty-region QA and reduced-motion variants.

**F5 / host + public authority:** only after GPU/CPU proofs and operator authorization, prototype a host adapter for Gnosix/GPUI; independent UI semantics. Public docs remain gated by license/confidentiality reviews.

## Explicitly unresolved

- Backend product authority `rust` vs `cpp` across historic docs, license GPL/AGPL, Cargo 0.14 vs tokens 0.20, identity of live upstream Git object; see `VERIFICATION.md`.
- Direct GPU/CPU API design, Vello adoption, GPUI texture exchange, and capability tier selection must be decided by tests; this document only states candidate boundaries.
- This spec is not permission to push, distribute, import GPL code into differently licensed repositories, or modify production Gnosix.
