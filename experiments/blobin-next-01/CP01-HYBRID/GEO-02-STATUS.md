# GEO-02 — Typed CPU hybrid CSG with a bounded non-convex solid class

Truth class: IMPLEMENTED + LOCALLY OBSERVED; NOT CP01 QUALIFIED.
Contract: BLOB-NEXT-01 CP01 HYBRID v0.4; no CPU/GPU checkpoint promotion.

## Delivered
- CertifiedMeshSolid sealed type with Convex(CertifiedConvexSolid) and Extruded(ExtrudedPrism).
- Explicitly generated, non-convex triangle mesh from a simple, CCW 2-D polygon and finite extrusion height. Constructed with self-crossing checks and ear-clipped caps, closed oriented side walls and conservative triangle preflight.
- Signed distance to the generated triangulated surface (not a fabricated bounding box), inside/outside sign from the validated source polygon.
- Typed SceneDocument::MeshSolid; CPU CSG union, intersection, SDF minus MeshSolid and MeshSolid minus SDF. All report estimated-field CSG semantics, not exact Euclidean distance for composites.
- Existing MeshSurface rejected as a signed field operand.
- Real ray/triangle queries; exact edge ambiguity explicitly classified instead of silently claimed hit.

## Falsification & numerical oracle
- A separate 2-D L-shape oracle encodes the concave area as a union of two rectangles and computes signed 3-D prism distance from line segments, independent of the SDF triangle closest-point solver.
- Fixed pseudo-random seed and 10,000 volume points; 2,000 points of 4 mixed CSG operations; 1,000 vertical rays.
- cargo test --offline -- --nocapture, cargo clippy (all targets, -D warnings), and rustfmt.
- Scope of the error claim is this fixture and these coordinates, not all possible nonconvex meshes or a general robust predicate theorem.

## NOT DELIVERED / explicit failures to claim
- No support for arbitrary, untrusted glTF/GLB imported triangle soups becoming MeshSolid.
- No general non-convex topology/self-intersection certification, arbitrary genus or near-degenerate geometry robustness.
- No BVH, GPU shaders, Vulkan/wgpu hybrid rendering or actual 60 FPS.
- No complete independent 10k/1k fixture matrix across all CP01 geometry classes, camera poses, normals, analytic capsule/oracle and save/reopen with stable document digests.
- CP01 remains NOT QUALIFIED, CP02/CP03 remain NOT STARTED.

## Handoff
GEO-03 should add an import-without-trust path for glTF/GLB static subset, independent robust mesh validity or exact reversible construction alternatives, and CPU camera/ray queries. Only after a validated independent oracle should a generic imported closed mesh be promotable to MeshSolid. Preserve the sealed topology authority and report ambiguous predicates as errors.
