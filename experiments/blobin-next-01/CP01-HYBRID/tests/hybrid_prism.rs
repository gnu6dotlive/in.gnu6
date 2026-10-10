use blobin_cp01_hybrid::{
    mesh::{CertifiedConvexSolid, CertifiedMeshSolid, ExtrudedPrism, Ray, RayTrace},
    CsgOp, FieldSample, GeometryError, MeshAsset, MeshPreflight, NodeId, NodeKind, Primitive,
    SceneDocument, SdfLeaf, Semantic, Transform, Vec3,
};
use std::collections::BTreeMap;
fn p(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3::new(x, y, z)
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() <= 1e-8, "a={a} b={b}");
}
fn l_poly() -> Vec<[f64; 2]> {
    vec![[0., 0.], [3., 0.], [3., 1.], [1., 1.], [1., 3.], [0., 3.]]
}
fn prism() -> ExtrudedPrism {
    ExtrudedPrism::build(l_poly(), -1., 1.).unwrap()
}
fn tetra() -> MeshAsset {
    MeshAsset {
        vertices: vec![p(0., 0., 0.), p(1., 0., 0.), p(0., 1., 0.), p(0., 0., 1.)],
        triangles: vec![[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]],
    }
}
fn sample(s: &SceneDocument, id: u64, x: f64, y: f64, z: f64) -> FieldSample {
    s.sample(NodeId(id), p(x, y, z)).unwrap()
}
fn scene_with_solid(solid: CertifiedMeshSolid) -> SceneDocument {
    let mut nodes = BTreeMap::new();
    nodes.insert(NodeId(1), NodeKind::MeshSolid(solid));
    nodes.insert(
        NodeId(2),
        NodeKind::Sdf(SdfLeaf {
            shape: Primitive::Sphere { radius: 1. },
            transform: Transform {
                translation: p(0.5, 0.5, 0.),
                ..Transform::IDENTITY
            },
        }),
    );
    nodes.insert(
        NodeId(3),
        NodeKind::Csg {
            op: CsgOp::Difference,
            left: NodeId(1),
            right: NodeId(2),
        },
    );
    nodes.insert(
        NodeId(4),
        NodeKind::Csg {
            op: CsgOp::Difference,
            left: NodeId(2),
            right: NodeId(1),
        },
    );
    nodes.insert(
        NodeId(5),
        NodeKind::Csg {
            op: CsgOp::Intersection,
            left: NodeId(1),
            right: NodeId(2),
        },
    );
    nodes.insert(
        NodeId(6),
        NodeKind::Csg {
            op: CsgOp::Union,
            left: NodeId(1),
            right: NodeId(2),
        },
    );
    SceneDocument {
        version: 1,
        nodes,
        roots: vec![NodeId(3), NodeId(4), NodeId(5), NodeId(6)],
    }
}
#[test]
fn concave_l_shape_has_closed_triangles_and_signed_distance() {
    let s = prism();
    assert_eq!(
        s.mesh().preflight().unwrap(),
        MeshPreflight::ClosedCandidateNotCertified
    );
    assert!(s.mesh().triangles.len() > 12);
    for (pt, expected) in [
        (p(0.5, 0.5, 0.), -0.5),
        (p(2., 0.5, 0.), -0.5),
        (p(0.5, 2., 0.), -0.5),
        (p(2., 2., 0.), 1.),
        (p(-1., 0.5, 0.), 1.),
        (p(0.5, 0.5, 2.), 1.),
    ] {
        near(s.signed_distance(pt).unwrap(), expected);
    }
    let r = Ray::new(p(2., 2., 3.), p(0., 0., -1.)).unwrap();
    assert!(matches!(s.ray_trace(r).unwrap(), RayTrace::Miss));
    let r = Ray::new(p(2., 0.5, 3.), p(0., 0., -1.)).unwrap();
    assert!(matches!(
        s.ray_trace(r).unwrap(),
        RayTrace::Hit(_) | RayTrace::AmbiguousSharedEdge { .. }
    ));
}
#[test]
fn sealed_nonconvex_mesh_participates_in_real_mixed_csg() {
    let scene = scene_with_solid(CertifiedMeshSolid::Extruded(prism()));
    scene.validate().unwrap();
    near(sample(&scene, 1, 0.5, 0.5, 0.).distance, -0.5);
    near(sample(&scene, 2, 0.5, 0.5, 0.).distance, -1.);
    near(sample(&scene, 3, 0.5, 0.5, 0.).distance, 1.); // prism minus sphere -> cavity
    near(sample(&scene, 4, 0.5, 0.5, 0.).distance, 0.5); // sphere minus prism
    near(sample(&scene, 5, 0.5, 0.5, 0.).distance, -0.5);
    near(sample(&scene, 6, 0.5, 0.5, 0.).distance, -1.);
    for root in [3, 4, 5, 6] {
        assert_eq!(
            sample(&scene, root, 0.5, 0.5, 0.).semantic,
            Semantic::FieldEstimate
        );
    }
}
#[test]
fn convex_mesh_can_also_become_a_signed_csg_operand() {
    let c = CertifiedConvexSolid::certify(tetra()).unwrap();
    let scene = scene_with_solid(CertifiedMeshSolid::Convex(c));
    scene.validate().unwrap();
    assert_eq!(
        sample(&scene, 1, 0.1, 0.1, 0.1).semantic,
        Semantic::ExactDistance
    );
    let d = sample(&scene, 3, 0.1, 0.1, 0.1).distance;
    assert!(d > 0.0);
}
#[test]
fn adversarial_polygon_is_rejected() {
    let bowtie = vec![[0., 0.], [2., 2.], [0., 2.], [2., 0.]];
    assert!(ExtrudedPrism::build(bowtie, -1., 1.).is_err());
    let open_corner = vec![[0., 0.], [3., 0.], [3., 1.], [1., 1.], [1., 1.], [0., 3.]];
    assert!(ExtrudedPrism::build(open_corner, -1., 1.).is_err());
    let mut reversed = l_poly();
    reversed.reverse();
    assert!(matches!(
        ExtrudedPrism::build(reversed, -1., 1.),
        Err(GeometryError::WrongWinding)
    ));
    assert!(ExtrudedPrism::build(l_poly(), 1., -1.).is_err());
    assert!(ExtrudedPrism::build(l_poly(), 0., f64::INFINITY).is_err());
    let mut self_touch = l_poly();
    self_touch[3] = [0., 0.];
    assert!(ExtrudedPrism::build(self_touch, -1., 1.).is_err());
}
