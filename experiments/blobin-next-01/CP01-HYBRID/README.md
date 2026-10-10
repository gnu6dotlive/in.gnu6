# CP01-HYBRID — GEO-00 experimental Rust foundation

**NOT CP01 QUALIFIED.** Initial pure Rust mathematical and typed scene scaffold from ratified BLOB-NEXT-01. Does not implement signed mesh interiors, glTF import, BVH, signed mesh distance, hybrid booleans, camera/rays or full independent f64 oracle.

Implemented: bounded SDF sphere/box/capsule, rigid transform + uniform scale, CSG union/intersection/difference/smooth union, typed NodeId, cycle/depth validation, memoized DAG evaluation, conservative mesh preflight. Even a closed-looking tetrahedron is only ClosedCandidateNotCertified, not MeshSolid.

Reproduce offline:
1. CARGO_TARGET_DIR=/mnt/workbench/build/blobin-cp01-geometry-20261010 cargo test --offline
2. CARGO_TARGET_DIR=/mnt/workbench/build/blobin-cp01-geometry-20261010 cargo clippy --offline --all-targets -- -D warnings
3. cargo fmt -- --check

Next GEO-01: independently instrumented mesh topology, self-intersection, closest-point and inside/outside reference; static glTF subset; certified MeshSolid and real mixed CSG. Freeze fixtures (10,000 samples / 1,000 rays), record tolerance violations and adversarial failures. No claim of 60 FPS or GPU support follows from this crate.

## GEO-02 scope

Sealed CertifiedMeshSolid supports conservatively validated convex meshes and generated non-convex prisms from simple CCW polygons (no arbitrary mesh import). SceneDocument evaluates unions, intersections, and both subtraction directions between these meshes and SDFs on CPU. A separate analytical L-prism oracle tests 10,000 signed-distance points, 2,000 mixed CSG points and 1,000 rays. **This is not a qualified CP01**, nor any Vulkan GPU hybrid rendering or signed certification of imported arbitrary meshes.
