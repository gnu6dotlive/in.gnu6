# GEO-01 — Partial conservative mesh queries

Status: EXPERIMENTAL / NOT CP01 QUALIFIED.

Geometry: f64 ray-triangle tests, closest-point surface queries and a narrow convex-solid constructor. The constructor checks triangle incidence, outward-supporting planes, connected face fan, orientation, Euler characteristic and geometry scale. Its type name is *CertifiedConvexSolid*; it does not claim certification of arbitrary closed meshes.

Evidence: 21 test functions in geometry_seed and mesh_queries; rustfmt and clippy -D warnings. Test fixtures include tetrahedron, open surface, reversed face, duplicate points, duplicate facets and ray corner ambiguity.

Known boundary: convex-only, max 512 vertices and 1024 triangles; no independent mesh oracle or large adversarial matrix, no GLB import, no general nonconvex self-intersection predicate, no BVH, no guarantee for near-coplanar floating-point degenerate cases. Potentially ambiguous geometry is rejected; classification thresholds require future adversarial review. A certified-convex mesh is not yet wired as SceneDocument MeshSolid or GPU hybrid CSG.

Stop: no promotion of CP01 until all six ratified gates and numeric budgets are independently demonstrated. No edits to BLOB-FRAME-v1, stable runtime or GUI.
