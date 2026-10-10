use blobin_cp01_hybrid::{
    mesh::{CertifiedConvexSolid, Ray, RayTrace},
    *,
};
fn p(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3::new(x, y, z)
}
fn tetra() -> MeshAsset {
    MeshAsset {
        vertices: vec![p(0., 0., 0.), p(1., 0., 0.), p(0., 1., 0.), p(0., 0., 1.)],
        triangles: vec![[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]],
    }
}
fn near(x: f64, y: f64) {
    assert!((x - y).abs() < 1e-9, "{x} != {y}");
}
#[test]
fn convex_tetra_certifies_signed_distance() {
    let c = CertifiedConvexSolid::certify(tetra()).unwrap();
    near(c.signed_distance(p(0.1, 0.1, 0.1)).unwrap(), -0.1);
    near(c.signed_distance(p(-1., 0.1, 0.1)).unwrap(), 1.);
    near(c.signed_distance(p(0., 0.2, 0.2)).unwrap(), 0.);
}
#[test]
fn ray_triangle_hit_miss_and_edge_ambiguity() {
    let m = MeshAsset {
        vertices: vec![p(0., 0., 0.), p(1., 0., 0.), p(0., 1., 0.)],
        triangles: vec![[0, 1, 2]],
    };
    let r = Ray::new(p(0.2, 0.2, 1.), p(0., 0., -1.)).unwrap();
    match m.ray_trace(r).unwrap() {
        RayTrace::Hit(h) => near(h.t, 1.),
        r => panic!("{r:?}"),
    }
    assert!(matches!(
        m.ray_trace(Ray::new(p(2., 2., 1.), p(0., 0., -1.)).unwrap())
            .unwrap(),
        RayTrace::Miss
    ));
    assert!(matches!(
        m.ray_trace(Ray::new(p(0., 0., 1.), p(0., 0., -1.)).unwrap())
            .unwrap(),
        RayTrace::AmbiguousSharedEdge { .. }
    ));
}
#[test]
fn invalid_topology_refused() {
    let mut m = tetra();
    m.triangles.pop();
    assert!(CertifiedConvexSolid::certify(m).is_err());
    let mut m = tetra();
    m.triangles[0].swap(0, 1);
    assert!(CertifiedConvexSolid::certify(m).is_err());
    let mut m = tetra();
    m.vertices.push(p(0., 0., 0.));
    assert!(CertifiedConvexSolid::certify(m).is_err());
}
#[test]
fn offset_duplicated_facets_not_solid() {
    let mut m = tetra();
    m.triangles.extend(m.triangles.clone());
    assert!(CertifiedConvexSolid::certify(m).is_err());
}
#[test]
fn nonconvex_closed_is_not_certified() {
    let mut m = tetra();
    m.vertices[3] = p(0., 0., -1.);
    assert!(CertifiedConvexSolid::certify(m).is_err());
}
