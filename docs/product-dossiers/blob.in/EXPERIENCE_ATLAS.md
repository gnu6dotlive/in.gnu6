# BLOBIN-NEXT-01 — Challenging UX / topology experience atlas

**Status:** IDEATION → TESTABLE_EXPERIMENTS. A concept is not an implemented product. Each scene must reveal *why a spatial operation is better UX* than static CSS/blur. Screen recordings or screenshots without causal input evidence **do not pass**.

**Common experiment harness:** preset seed + viewport + DPR + backend + fixed input trace; capture semantic scene digest, topology event log, hit-test state, CPU/GPU frame images, pass GPU timings, degraded-mode classification, motion preference. All demos have keyboard navigation and reduced-motion final states.

## Gallery (four tiers)

### Tier 1 — Prove the core language

| ID | Experience, behavior and topology challenge | Material/effect challenge | Oracle |
|---|---|---|---|
| E01 **Breathing Membrane Dock** | A dock border is a deformable elastic boundary; incoming/outgoing applications change local pressure. Neighboring anchors yield but distant controls keep identity/hit areas; releasing pressure returns to the exact resting shape. | Dynamic inside/outside edge, curvature-sensitive bevel, soft contact shadow | Stable item IDs, bounded displacement, no self-intersection, zero wake after settle; mouse/keyboard same destination |
| E02 **Topology Bloom** | Radial launcher sprouts satellites that join and detach; one donut shape can form and close a hole without destroying launcher identity. | SDF union/subtraction, hole-aware bevel, cutout shadows, inner stroke | Component/hole count matches declared events, winding preserved, hit regions never attach to a negative space |
| E03 **Liquid Tabs** | Tabs elastically coalesce when related then detach as relationships change; focus always belongs to exactly one tab despite merged silhouette. | Matcap-like soft ceramic skin, seam reveal, constrained tension | Tab ownership unique; focus ring still distinct; selection accessible in CPU/reduced motion |
| E04 **Cutout Aperture** | A floating surface opens a hole through itself to reveal a bounded preview of another surface; clipping remains correct under rotation, zoom and parent reflow. | Nested clipping, variable rim thickness, bevel inside and outside | No hit through the hole unless preview authorized; preserve occlusion/depth ordering and identity |

### Tier 2 — Heightmaps / heatmaps / lighting

| ID | Experience & interaction | Rendering challenge | Oracle |
|---|---|---|---|
| E05 **Relief Focus** | Pointer, keyboard focus or programmatic selection creates a local height impulse on a control; feedback follows focus ownership, not cursor accident. | Signed-distance-derived heightmap, normals, rim lighting, self-shadow, relief/bisseau | Height bounded and finite, no shimmering across DPR, keyboard contrast ≥ declared target |
| E06 **Heatfield Atlas** | User-controlled local heat impulses visualize accumulated spatial activity over an application *without passive telemetry*. Heat decays smoothly and reveals frequent interaction zones. | Gaussian/anisotropic field, masked diffusion, heatmap palette, threshold isolines | Synthetic trace deterministic; no logging/network; same peaks CPU/GPU with declared numerical tolerance |
| E07 **Glass Geological Layers** | Multiple overlapping card surfaces reveal relief bands and strata when pulled apart; layer order conveys containment/relationship. | Layered material graph, edge normals, refraction approximation, roughness, depth cue | Occlusion consistent; not dependent on transparency for meaning; CPU uses simpler legible stratification |
| E08 **Reactive Boundary Flow** | Border carries travelling pulses from a focused node to its semantic neighbours along path geodesics; no random glow. | Arclength parametrization, dashed flowing strokes, SDF edge masks, emissive passes | Pulse speed in world units not frame count; no crossing unrelated graph edges; reduced motion shows destination |

### Tier 3 — Post-process and hard failures

| ID | Experience & interaction | Rendering challenge | Oracle |
|---|---|---|---|
| E09 **Notification Metamorph** | Notification enters as an orbiting bubble, expands into a readable card, then collapses to an inbox token. Morph never deletes underlying notification identity. | Bevel changing sign, directional bloom, controlled motion trails, text-safe backdrop | Content/pointer/keyboard continuity during morph; 200% zoom; no unreadable glare |
| E10 **Topology Canvas Lab** | A live playground exposes boolean operators union/intersect/cut, connected components, genus/holes and graph ownership; each operation changes a debug topology ledger. | Analytic/SDF vs tessellated pixels side-by-side, normals/heightfield toggle | Each edit emits an event with expected Euler characteristic; stale GPU pixels never become topology authority |
| E11 **GPU Failure Theatre** | Lose GPU device mid-animation, recover to CPU preserving shapes, focus and current spring time. Explicit quality grade remains visible for debugging. | Resource lifetime, surface handoff, CPU renderer, restart of effect graph | Same semantic digest pre/post; no content loss; ≤ one deliberately documented presentation discontinuity; no infinite retry |
| E12 **Temporal Echo Without Ghosts** | Reflowing window surfaces leave an optional short-lived field echo, but click/hit targets always correspond to *current* geometry. | Temporal framebuffer history, alpha decay, mask, motion-compensated velocity | Temporal ghosts noninteractive; history reset on topology epoch, color-space/resize change; reduced motion disables trail |

### Tier 4 — Extreme stress test scenes

| ID | Experience | Boundary challenged | Oracle |
|---|---|---|---|
| E13 **Archipelago of 50** | 50 organic windows and connectors continuously merge, split, reparent, focus and reflow in 4K. | Field atlas tiling, atlas eviction, dirty bounds, blurs and clipping; GPU budget | No invented entities or memory leak; 5 min trace with stable frame p95, bounded readbacks; explicit CPU tier |
| E14 **Curvature Forge** | A controllable material lab for paper, ceramic, machined metal, gel, rubber and chalk. Border widths become a spatially variable function of curvature, focus and material. | Bevel, anisotropic highlights, emboss, noise, roughness, outlines, shadows, heat overlay | Same contour across all styles; no opacity halo; per-node shader cost measurable and comparable |
| E15 **Cross-surface Handoff** | A visible blob deforms around two separate host surfaces and moves ownership when the user drags content across a boundary; it never directly manipulates authority. | Offscreen texture, DPI migration, clipping, seamless reanchor | Host-approved handoff event required; ID, focus and interaction semantics remain consumer-owned; no unauthorized cross-window drag |
| E16 **Shadow Without Shape** | Switch high-contrast/no-effects mode mid-scene: lighting, gloss and heat vanish but semantic hierarchy remains absolutely readable. | Flat CPU/GPU material downgrade, stroke/labels/hit regions | Same navigation and focus order; meets selected contrast criteria; no shader-only state |

## Topology law pack — hard cases

1. **Union topology:** A∪B at tangency and self-intersection. Components and holes reported after robust cleanup, not guessed from a shader color.
2. **Subtraction:** A\B when B touches outer boundary, crosses two separate components, or is infinitesimally close; winding and hit-tests must agree.
3. **Morphing genus:** one hole appears/disappears. IDs survive but topology epoch changes exactly once per event transaction.
4. **Self-intersection and transparency:** stroke triangles overlap; coverage composited once per path, no dark seams/double alpha. Lyon stroke tessellation alone does not solve this for arbitrary translucent self-overlap.
5. **Thin slivers:** bevel width greater than region width, tiny 1px features at 2× DPR; avoid negative heights and NaNs.
6. **Occlusion:** cutouts / blur halo must not allow hit-testing through protected surfaces.
7. **Scale & time:** resize, DPR 1→2, abrupt target change, time freeze/resume; physical motion and focus not dependent on rendered FPS.
8. **Numerical:** enormous world coordinates, finite but near-limit values, degenerate segments, zero-length edges, flat curvature, large heat impulses.
9. **Color:** premultiplied alpha, linear-sRGB conversion, transparent blurred black/white on dark/light backgrounds; no colored fringes.
10. **Life cycle:** entity remains while mesh is rebuilt; GPU device replacement cannot mint/deactivate semantic IDs.

## Suggested first showcase

**E02 + E05 + E11** is an aggressive but precise vertical slice:

- **Topology Bloom** demonstrates union/cut holes and real geometry.
- **Relief Focus** demonstrates heightmap/normals/bevel without losing legibility.
- **GPU Failure Theatre** demonstrates that CPU is a genuine compatible renderer rather than a mock fallback.

Show each simultaneously with **GPU**, **CPU**, **difference image**, **topology ledger** and **pass timings**, with a toggle that works mid-transition.

**Rationale:** A visually attractive screenshot is not enough. The same scene must survive topology changes, input, reduced motion, CPU execution and GPU loss to count as a blob.in product capability.

## Anti-patterns

- A blurred orange shape called “topology” without graph and identity semantics.
- Hundreds of effects with no perf counter or fallback tier.
- A game-like particle system that destroys legibility around text.
- CPU fallback that returns a blank frame or only supports rectangles.
- Re-adding a Qt/C++ mini-engine as a hidden dependency.
- Using heatmaps as passive behavioral tracking.
- Calling GPUI connected because a crate imports GPUI.
- Treating archived GPU checksums as new physical GPU evidence.
