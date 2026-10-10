// Independent oracle for a known 3-D concave L prism.
// Computes unsigned distance to 2-D outline analytically and signs it from
// *two rectangles*, not from triangle projection, ear clipping or mesh parity.
use blobin_cp01_hybrid::{
    mesh::{CertifiedMeshSolid, ExtrudedPrism, Ray, RayTrace},
    CsgOp, NodeId, NodeKind, Primitive, SceneDocument, SdfLeaf, Transform, Vec3,
};
use std::collections::BTreeMap;

const OUTLINE: [[f64; 2]; 6] = [
    [0.0, 0.0],
    [3.0, 0.0],
    [3.0, 1.0],
    [1.0, 1.0],
    [1.0, 3.0],
    [0.0, 3.0],
];
fn rand01(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1_u64 << 53) as f64)
}
fn inside_l(x: f64, y: f64) -> bool {
    ((0.0..3.0).contains(&x) && (0.0..1.0).contains(&y))
        || ((0.0..1.0).contains(&x) && (0.0..3.0).contains(&y))
}
fn oracle_l_prism(x: f64, y: f64, z: f64) -> f64 {
    let mut d_xy = f64::INFINITY;
    for i in 0..OUTLINE.len() {
        let a = OUTLINE[i];
        let b = OUTLINE[(i + 1) % OUTLINE.len()];
        let ux = b[0] - a[0];
        let uy = b[1] - a[1];
        let t = (((x - a[0]) * ux + (y - a[1]) * uy) / (ux * ux + uy * uy)).clamp(0.0, 1.0);
        d_xy = d_xy.min(((x - a[0] - t * ux).powi(2) + (y - a[1] - t * uy).powi(2)).sqrt());
    }
    let within_xy = inside_l(x, y);
    let within_z = z > -1.0 && z < 1.0;
    let dz = (z.abs() - 1.0).max(0.0);
    match (within_xy, within_z) {
        (true, true) => -d_xy.min(1.0 - z.abs()),
        (true, false) => dz,
        (false, true) => d_xy,
        (false, false) => d_xy.hypot(dz),
    }
}
fn prism() -> ExtrudedPrism {
    ExtrudedPrism::build(OUTLINE.to_vec(), -1.0, 1.0).unwrap()
}
fn document() -> SceneDocument {
    let mut nodes = BTreeMap::new();
    nodes.insert(
        NodeId(1),
        NodeKind::MeshSolid(CertifiedMeshSolid::Extruded(prism())),
    );
    nodes.insert(
        NodeId(2),
        NodeKind::Sdf(SdfLeaf {
            shape: Primitive::Sphere { radius: 1.0 },
            transform: Transform {
                translation: Vec3::new(0.5, 0.5, 0.0),
                ..Transform::IDENTITY
            },
        }),
    );
    for (i, op) in [CsgOp::Union, CsgOp::Intersection, CsgOp::Difference]
        .into_iter()
        .enumerate()
    {
        nodes.insert(
            NodeId(3 + i as u64),
            NodeKind::Csg {
                op,
                left: NodeId(1),
                right: NodeId(2),
            },
        );
    }
    nodes.insert(
        NodeId(6),
        NodeKind::Csg {
            op: CsgOp::Difference,
            left: NodeId(2),
            right: NodeId(1),
        },
    );
    SceneDocument {
        version: 1,
        nodes,
        roots: vec![NodeId(3), NodeId(4), NodeId(5), NodeId(6)],
    }
}
#[test]
fn oracle_ten_thousand_volume_queries() {
    let solid = prism();
    let scene = document();
    let mut rng = 0x91cd_216c_a55d_17f9_u64;
    let mut max_err: f64 = 0.0;
    let mut sign_disagree = 0_usize;
    let mut mixed_max_err: f64 = 0.0;
    for i in 0..10_000 {
        let x = rand01(&mut rng) * 5.0 - 1.0;
        let y = rand01(&mut rng) * 5.0 - 1.0;
        let z = rand01(&mut rng) * 4.0 - 2.0;
        let oracle = oracle_l_prism(x, y, z);
        let measured = solid.signed_distance(Vec3::new(x, y, z)).unwrap();
        max_err = max_err.max((oracle - measured).abs());
        if oracle.abs() > 1e-7 && measured.signum() != oracle.signum() {
            sign_disagree += 1;
        }
        // Every 5th point passes through the typed SceneDocument mixed operators.
        if i % 5 == 0 {
            let sphere = ((x - 0.5).powi(2) + (y - 0.5).powi(2) + z * z).sqrt() - 1.0;
            for (root, expected) in [
                (3, oracle.min(sphere)),
                (4, oracle.max(sphere)),
                (5, oracle.max(-sphere)),
                (6, sphere.max(-oracle)),
            ] {
                let sample = scene.sample(NodeId(root), Vec3::new(x, y, z)).unwrap();
                mixed_max_err = mixed_max_err.max((sample.distance - expected).abs());
            }
        }
    }
    eprintln!("GEO02_ORACLE_VOLUME points=10000 mixed_points=2000 max_abs_err={max_err:.12e} mixed_max_abs_err={mixed_max_err:.12e} sign_mismatch={sign_disagree}");
    assert_eq!(sign_disagree, 0);
    assert!(max_err <= 0.0002);
    assert!(mixed_max_err <= 0.0002);
}
#[test]
fn oracle_one_thousand_vertical_ray_queries() {
    let prism = prism();
    let mut rng = 0x2142_90b4_e811_7124_u64;
    let mut hit = 0;
    let mut miss = 0;
    let mut ambiguous = 0;
    for _ in 0..1_000 {
        let x = rand01(&mut rng) * 5.0 - 1.0;
        let y = rand01(&mut rng) * 5.0 - 1.0;
        let ray = Ray::new(Vec3::new(x, y, 3.0), Vec3::new(0.0, 0.0, -1.0)).unwrap();
        match prism.ray_trace(ray).unwrap() {
            RayTrace::Miss => {
                assert!(!inside_l(x, y), "unreported hit x={x} y={y}");
                miss += 1;
            }
            RayTrace::Hit(h) => {
                assert!(inside_l(x, y), "spurious hit x={x} y={y}");
                assert!((h.t - 2.0).abs() < 0.0001, "depth={} x={x} y={y}", h.t);
                hit += 1;
            }
            RayTrace::AmbiguousSharedEdge { t, count } => {
                // Never silently reinterpret an ambiguous mesh intersection.
                assert!(inside_l(x, y));
                assert!((t - 2.0).abs() < 0.0001);
                assert!(count >= 1);
                ambiguous += 1;
            }
        }
    }
    eprintln!("GEO02_ORACLE_RAYS total=1000 hit={hit} miss={miss} ambiguous={ambiguous}");
    assert_eq!(hit + miss + ambiguous, 1000);
    assert!(hit > 100 && miss > 100);
    assert!(ambiguous < 5);
}
