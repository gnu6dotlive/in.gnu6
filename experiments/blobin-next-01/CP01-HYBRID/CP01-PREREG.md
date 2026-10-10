# CP01-HYBRID — GEO-00 acquisition / preregistration

State: EXPERIMENTAL FOUNDATION. NOT CP01 QUALIFIED.
Authority: BLOB-NEXT-01-CHECKPOINTS-v1.2-RATIFIE.md (2026-10-10).
Owned branch: feat/blobin-cp01-hybrid-foundation-20261010.
Isolation: no BLOB-FRAME-v1 change; no CUDA-05 service or viewer change.

## Initial falsifiable slice
- Typed world coordinates (right-handed, Y-up, meters) and f64 field samples.
- SDF sphere/box/capsule analytic distance, transforms restricted to quaternion rigid rotation + strictly positive uniform scale.
- CSG min/max/difference/smooth_union preserve FIELD_ESTIMATE semantics.
- Deterministic NodeId graph, no missing references, cycles, or depths >64; max 256 nodes.
- Triangle mesh asset checked for finite positions, positive triangle area, indices, and size bounds.
- Meshes remain SurfaceOnly or UNQUALIFIED until orientation/manifold/self-intersection, reliable inside/outside, and closest-point oracle are implemented. A superficially closed mesh MUST NOT be promoted.
- No GPU implementation, glTF import, certified MeshSolid, hybrid booleans, signed mesh distance, full 10k/1k oracle matrix, or CP01 PASS implied.

## Gates to earn (NOT yet qualified)
G01 analytic SDF/CSG; G02 mesh validation + glTF + intersection; G03 signed MeshSolid hybrid CSG; G04 camera/rays/occlusion; G05 independent f64 oracle 10k points/1k rays; G06 malicious inputs/document replay and regression.

## Numerical acceptance
CP01 contract: |SDF delta| <=1e-4m, |mesh signed distance delta| <=2e-4m, rays <=1e-4m (mesh), <=1e-3m (SDF), normals dot>=0.995, no fake Hit/Miss, zero invalid topologies certified solid.

## Expansion gate
Next GEO worker may implement validated solids and exact mesh queries, but MUST preserve this rule: surface classification and signed solid certification are different. Use independent oracle reference, freeze fixture seeds before qualification. Keep the crate in this isolated worktree.
