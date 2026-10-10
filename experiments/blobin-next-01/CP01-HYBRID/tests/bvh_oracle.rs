use blobin_cp01_hybrid::{
    bvh::TriangleBvh,
    mesh::{ExtrudedPrism, Ray, RayTrace},
    MeshAsset, Vec3,
};
fn p(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3::new(x, y, z)
}
fn rand(state: &mut u64) -> f64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((*state >> 11) as f64) / ((1u64 << 53) as f64)
}
fn l_shape() -> MeshAsset {
    ExtrudedPrism::build(
        vec![
            [0.0, 0.0],
            [3.0, 0.0],
            [3.0, 1.0],
            [1.0, 1.0],
            [1.0, 3.0],
            [0.0, 3.0],
        ],
        -1.0,
        1.0,
    )
    .unwrap()
    .mesh()
    .clone()
}
fn grid(rows: u32) -> MeshAsset {
    let mut vertices = Vec::new();
    for y in 0..=rows {
        for x in 0..=rows {
            vertices.push(p(x as f64 / rows as f64, y as f64 / rows as f64, 0.0));
        }
    }
    let mut triangles = Vec::new();
    for y in 0..rows {
        for x in 0..rows {
            let a = y * (rows + 1) + x;
            let b = a + 1;
            let c = a + (rows + 1);
            let d = c + 1;
            triangles.push([a, b, d]);
            triangles.push([a, d, c]);
        }
    }
    MeshAsset {
        vertices,
        triangles,
    }
}
fn compare_ray(a: RayTrace, b: RayTrace) {
    match (a, b) {
        (RayTrace::Miss, RayTrace::Miss) => {}
        (RayTrace::Hit(a), RayTrace::Hit(b)) => {
            assert!((a.t - b.t).abs() < 1e-9, "hit t: {} != {}", a.t, b.t);
            assert_eq!(a.triangle_index, b.triangle_index);
        }
        (
            RayTrace::AmbiguousSharedEdge { t: a, count: c },
            RayTrace::AmbiguousSharedEdge { t: b, count: d },
        ) => {
            assert!((a - b).abs() < 1e-9);
            assert_eq!(c, d);
        }
        (lhs, rhs) => panic!("BVH diverged brute-force: {lhs:?} vs {rhs:?}"),
    }
}
#[test]
fn bvh_independently_matches_brute_force_10k_points_and_10k_rays() {
    let meshes = vec![l_shape(), grid(12)];
    let mut seed = 0x15c5d285aa73d421u64;
    for (mesh_index, mesh) in meshes.into_iter().enumerate() {
        let bvh = TriangleBvh::build(mesh.clone()).unwrap();
        assert_eq!(bvh.triangle_count(), mesh.triangles.len());
        assert!(bvh.node_count() > 1);
        for i in 0..5_000 {
            let pos = p(
                rand(&mut seed) * 5.0 - 1.0,
                rand(&mut seed) * 5.0 - 1.0,
                rand(&mut seed) * 4.0 - 2.0,
            );
            let lhs = mesh.closest_surface(pos).unwrap();
            let rhs = bvh.closest_surface(pos).unwrap();
            assert!((lhs.unsigned_distance - rhs.unsigned_distance).abs() < 1e-10);
            assert!((lhs.position.minus(rhs.position)).norm() < 1e-10);
            assert_eq!(
                lhs.triangle_index, rhs.triangle_index,
                "mesh={mesh_index} point={i}"
            );
            let origin = p(
                rand(&mut seed) * 5.0 - 1.0,
                rand(&mut seed) * 5.0 - 1.0,
                rand(&mut seed) * 4.0 + 0.1,
            );
            let direction = p(
                rand(&mut seed) * 2.0 - 1.0,
                rand(&mut seed) * 2.0 - 1.0,
                -1.0,
            );
            let ray = Ray::new(origin, direction.scale(1.0 / direction.norm())).unwrap();
            compare_ray(mesh.ray_trace(ray).unwrap(), bvh.ray_trace(ray).unwrap());
        }
    }
    eprintln!("GEO04_BVH_PARITY closest=10000 rays=10000 distributions=l-prism,grid12");
}
#[test]
fn bvh_preserves_ambiguous_edges_and_early_ray_exits() {
    let mesh = grid(4);
    let bvh = TriangleBvh::build(mesh.clone()).unwrap();
    for (x, y) in [
        (0.0, 0.0),
        (0.5, 0.5),
        (1.0, 1.0),
        (0.3, 0.6),
        (1.5, 1.5),
        (-1.0, 0.5),
    ] {
        for z in [-2.0, 2.0] {
            let ray = Ray::new(p(x, y, z), p(0.0, 0.0, if z > 0.0 { -1.0 } else { 1.0 })).unwrap();
            compare_ray(mesh.ray_trace(ray).unwrap(), bvh.ray_trace(ray).unwrap());
        }
    }
}
