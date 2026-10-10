# GEO-03 — GLB/glTF untrusted-surface import + CPU perspective camera

Truth class: LOCALLY TESTED partial CP01 implementation. CP01 NOT QUALIFIED.
Branch: feat/blobin-cp01-hybrid-foundation-20261010.
Authority: BLOB-NEXT-01 CP01 HYBRID v0.4. No weakening of G01–G06.

## Scope actually implemented

- No-filesystem, no-network parser for a constrained static glTF 2.0 mesh:
  one buffer, one mesh, one TRIANGLES primitive, POSITION float32 VEC3,
  indexed U8/U16/U32 SCALAR or unindexed triangle list.
- GLB 2.0 JSON/BIN alignment/length validation; .gltf JSON with externally
  supplied bytes for a single safe simple .bin URI, no automatic resolution.
- Optional single identity mesh node and one scene. Node transforms, skins,
  animations, morph targets, required extensions, external URLs, data URIs,
  multi-mesh scenes, sparse accessors and unsupported topologies fail closed.
- Every imported asset is ImportedSurface/MeshSurface. MeshSolid is NEVER
  implicitly certified. The existing narrow convex certificate is an explicit
  separate constructor; generic nonconvex imported topology remains unsupported.
- Bounds, finite vertex positions, index range, triangle degeneracy, offset,
  alignment, stride and buffer extent are validated. Input <=16 MiB; JSON <=2 MiB.
- Right-handed +Y-up f64 perspective camera with ray generation for real pixel
  centers, 3 camera poses, analytic plane ray/depth oracle.

## Local qualification evidence

- 44 Rust unit/integration tests, clippy -D warnings, rustfmt.
- Perspective camera: 2,883 rays, hit=1,333 / miss=1,550, zero unexpected
  classification on 3 fixtures/poses; no claim this is CSG raymarch truth.
- GLB mutation test: 500 byte mutations, 451 rejected, no process panic.
- Canonical glTF/GLB assets checked into fixtures/, generated from stdlib
  scripts/generate_static_fixtures.py and compared via include_bytes!.
- Existing GEO-02 10,000 volume + 2,000 mixed samples, 1,000 independent
  ray fixtures remain passing.

## Blocking gaps and cautions

- No external Khronos glTF-Validator binary available on this workstation at
  time of this pass; test acceptance != Khronos-wide spec certification.
- Subset intentionally excludes commonplace glTF node transforms and external
  data URIs; broader native glTF support is NOT claimed.
- No general imported nonconvex solid certification; arbitrary self-intersection
  and winding oracle, BVH acceleration and CSG hit/miss unresolved semantics
  remain unfinished.
- No independently qualified save/open scene semantic hashes, normals + three
  camera poses for full CP01 hero, or CPU↔GPU numerical parity.
- CP02 real GPU mixed CSG, CP03 compositor-presented >=60 FPS are NOT proved.
- CP00 remains pending distinct integrator review; no service mutated.

## Next proposal: GEO-04

1. Strict parser/validator bridge to genuine Khronos Validator using trusted
   offline dependency and archived diagnostic reports, if available.
2. Geometry certification for imported closed manifold with robust self-
   intersection, nesting and signed winding — do not treat closed edges alone
   as a proof; refuse ambiguous inputs.
3. BVH vs independent brute-force oracle and explicit ray Unresolved semantics.
4. Versioned semantic hashing/reload fixture, 6 required CP01 gates and exact
   normalized tolerances with immutable preregistration.
