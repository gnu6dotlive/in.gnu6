# blob.in — Render technology radar (2026-10-08)

**Status:** external research; adoption NOT_DECIDED. Historical source and external project documentation are different evidence classes. Cache presence on the developer workstation is **not** proof of build, compatibility or runtime stability.

## Candidates

| Candidate | Evidence today | Suitable role | Boundary / risk |
|---|---|---|---|
| **wgpu 30.x** | Cached locally: `wgpu-30.0.1`; GPU host Vulkan reports RTX 3070 | Primary independent GPU backend, compute/render passes, offscreen textures | Device/adapter feature negotiation, version-specific API, software adapters; build and runtime NOT_RUN for this new lab |
| **Naga 30.x** | Cached `naga-30.0.1`; WGSL frontend and validator documented | Preflight WGSL validation, parse errors and shader target audits | Validation proves syntax/types/capabilities, not final GPU numerical/pixel parity |
| **Lyon 1.x** | Existing historical `blobin-core` uses Lyon; fill+stroke tessellation maintained | Curves, triangulation, CPU geometry source | Overlapping stroke triangles can double alpha at intersections; use coverage union/raster masks |
| **Vello GPU** | 2026 project has `vello_gpu`, CPU preprocessing + GPU raster; wgpu/WebGL2 backends | Candidate for high-quality paths/fills/strokes rather than reimplementing all vector AA | Experimental cross-renderer API and maturity; does not replace blob.in heightfield/material/effect graph |
| **Vello CPU** | `vello_cpu` has mature software rendering track; cached 0.2.0 | Candidate CPU fallback and image tests for vector surface | Not automatically identical to custom WGSL bevel/height/heat; needs a shared material evaluator |
| **Legacy Qt RHI/CXX** | Present in donor snapshot, archived QA | Historical rendering reference during migration/parallel comparison | Not required for standalone Rust GPU/CPU design; backend `cpp` fallback not demonstrated |
| **GPUI integration** | Gnosix has Rust/GPUI surface research, independent identity model | Future host adapter after offscreen prototype | No assumed texture-sharing contract, no authority over Gnosix scene or desktop IDs |

## Decision probes — not premature package selection

1. Compare a **three-layer rounded contour** with 2 holes at DPR 1×/2× using Lyon+custom wgpu vs Vello GPU/CPU. Include edge difference images and mixed-transparent stroke self-overlap.
2. Compare custom `wgpu` compute signed-field and graphics-only path on the same GPU, plus software-only CPU execution. Treat unsupported GPU features as explicit tiers.
3. Run source-pinned Naga validation on every WGSL shader, then at least one real device execution. Type validation does not prove no device loss or correct pixels.
4. Profile SDF atlas construction, GPU upload, height/normal generation, rim light, blur and final composite separately; report actual adapter, driver, texture format and cache strategy.
5. Preserve the historical raster math (sRGB→linear, opacity once, premultiplication) as **design constraints**. No history rewrite or license transfer implied.
6. Decide adoption **after** comparison: use Vello only where it reduces code without obscuring failure modes, independent GPU/CPU capability negotiation and reference oracles.

## References read (external source docs, not imported)

- Vello main README: <https://github.com/linebender/vello> — CPU and GPU variants, maturity and backends.
- Vello GPU source metadata: <https://github.com/linebender/vello/blob/main/vello_gpu/Cargo.toml> — features / backend support.
- wgpu Features: <https://docs.rs/wgpu/latest/wgpu/struct.Features.html> — feature limitations by platform.
- Naga WGSL and validation: <https://docs.rs/naga/latest/naga/> — frontend + validator.
- Lyon tessellation docs: <https://docs.rs/lyon_tessellation/latest/lyon_tessellation/> — fill/stroke tessellation.
- Historical blob.in source: `/mnt/workbench/build/appstack-b3-baseline-4603aed/engine/blob.in`; `README.md`, `Cargo.toml`, `blobin-core/src/{material,raster,tessellate}.rs`, `blobin-qt/shaders/blob.frag`.

**Publication and licensing:** This radar does not determine the legal status of the source snapshot or license the newly-authored experiments. No code from external libraries is copied into the dossier.
