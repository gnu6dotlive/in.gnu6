# GEO-04 — Imported convex certificate + BVH + nonconvex topology inventory

Truth class: IMPLEMENTED/PARTIALLY QUALIFIED LOCALLY; checkpoint CP01 is NOT QUALIFIED.
Source worktree: feat/blobin-cp01-hybrid-foundation-20261010.
Contract authority: BLOB-NEXT-01 CP01 HYBRID v0.4. All CPU and GPU checkpoints retain original gates.

## What now works
- Imported GLB tetrahedron is initially ImportedSurface. A **separate, explicit**
  try_certify_convex consumes it and checks the conservative convex certificate.
  It can then be a real signed MeshSolid operand of union/intersection and
  both SDF minus MeshSolid / MeshSolid minus SDF CPU field operations.
- Imported nonconvex closed L-prism stays MeshSurface/topology candidate;
  **try_certify_convex must reject it**. Only generated extrusions preserve the
  controlled source polygon needed for our currently approved signed interior.
- Triangle BVH builds from a preflighted mesh; closest-point query and ray
  intersection independently compared to existing brute-force reference:
  10,000 points + 10,000 rays over a concave extruded L-mesh and grid surface;
  hit/miss/edge ambiguity retained, identical nearest original triangle IDs.
- Conservative bounded topology diagnostics: connectedness, two incident
  oppositely oriented triangles per edge, vertex fan, coincident vertices,
  signed volume and explicit disjoint-triangle intersection alarms.
  This returns ClosedTopologyCandidate **without** granting a signed distance
  field or certification of arbitrary imported meshes.
- Reproducible checked-in nonconvex GLB fixture; same generator as GEO-03.

## Positive and negative scenarios
- Known imported convex tetra: explicit certification accepted and mixed CPU
  booleans numerically verify cavity/noncavity.
- Imported nonconvex L-shape: closed candidate topology, convex certificate
  rejected (correct).
- Twisted cube variants: 14 detected non-adjacent self-intersections in
  preregistered sweep; no silent promotion.
- Degenerate/open/reversed/disconnected/unbounded meshes rejected.
- BVH tested against brute-force; equality is scoped to the sampled fixtures
  and fixed pseudo-random sequences, not a robust-predicate theorem.

## What remains BLOCKED
- General imported nonconvex solid certification; topological closure is NOT
  enough, and the detector does not fully decide intersecting triangles which
  SHARE an edge or vertex. Near-coplanarity and floating-point ambiguity also
  need stronger exact/adaptive predicates and independent validator.
- Robust nonzero winding / signed occupancy for arbitrary genuses/holes,
  shell nesting, normals oracle, large-scale CPU performance.
- Stable scene serialization/reload digest, glTF Validator independent
  external verification, complete CP01 G01–G06 and all CPU/GPU strict budgets.
- CP02 Vulkan GPU evaluation of actual mixed booleans, CP03 *compositor
  presented* 60 FPS, CP07 alpha. No claim of these.
- CP00 distinct integrator review still pending.

## Operational boundaries
No edits to original runtime/viewer, stable BLOB-FRAME-v1, systemd, or VPS.
No merge/push/publication. Tests and fixtures confined to this worktree.
