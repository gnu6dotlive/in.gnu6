use blobin_cp01_hybrid::{camera::PerspectiveCamera, mesh::RayTrace, MeshAsset, Vec3};

fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3::new(x, y, z)
}
fn planar_triangle() -> MeshAsset {
    MeshAsset {
        vertices: vec![v(0.0, 0.0, 0.0), v(3.0, 0.0, 0.0), v(0.0, 3.0, 0.0)],
        triangles: vec![[0, 1, 2]],
    }
}
#[test]
fn three_camera_poses_center_depth_and_unit_direction() {
    let tri = planar_triangle();
    for eye in [v(0.42, 0.36, 3.0), v(0.2, 0.15, 4.0), v(0.9, 0.3, 2.0)] {
        let target = v(eye.x, eye.y, 0.0);
        let cam = PerspectiveCamera::look_at(eye, target, v(0.0, 1.0, 0.0), 0.8).unwrap();
        let ray = cam.ray_for_pixel(50, 50, 101, 101).unwrap();
        assert!((ray.direction.z + 1.0).abs() < 1e-12);
        assert!(ray.direction.x.abs() < 1e-12 && ray.direction.y.abs() < 1e-12);
        assert!((ray.direction.norm() - 1.0).abs() < 1e-12);
        match tri.ray_trace(ray).unwrap() {
            RayTrace::Hit(h) => assert!((h.t - eye.z).abs() < 1e-9, "t={} z={}", h.t, eye.z),
            other => panic!("center should have an unambiguous hit: {other:?}"),
        }
    }
}
#[test]
fn analytic_plane_oracle_three_poses_nearly_three_thousand_rays() {
    let tri = planar_triangle();
    let mut checked = 0;
    let mut hits = 0;
    let mut misses = 0;
    for eye in [v(0.42, 0.36, 3.0), v(0.2, 0.15, 4.0), v(0.9, 0.3, 2.0)] {
        let camera =
            PerspectiveCamera::look_at(eye, v(eye.x, eye.y, 0.0), v(0.0, 1.0, 0.0), 0.8).unwrap();
        for py in 0..31 {
            for px in 0..31 {
                let ray = camera.ray_for_pixel(px, py, 31, 31).unwrap();
                let t = -ray.origin.z / ray.direction.z;
                let at = ray.origin.plus(ray.direction.scale(t));
                let margin = [at.x, at.y, 3.0 - at.x - at.y]
                    .into_iter()
                    .map(f64::abs)
                    .fold(f64::INFINITY, f64::min);
                let predicted = at.x > 0.0 && at.y > 0.0 && at.x + at.y < 3.0;
                // Boundary rays get their own Unresolved policy; never
                // silently exclude without recording the explicit predicate.
                if margin < 1e-8 {
                    continue;
                }
                match tri.ray_trace(ray).unwrap() {
                    RayTrace::Miss => {
                        assert!(!predicted, "lost ray pose={eye:?}, pixel={px},{py}");
                        misses += 1;
                    }
                    RayTrace::Hit(h) => {
                        assert!(predicted, "false ray pose={eye:?}, pixel={px},{py}");
                        assert!((h.t - t).abs() < 1e-4);
                        hits += 1;
                    }
                    RayTrace::AmbiguousSharedEdge { .. } => {
                        panic!("unexpected ambiguity away from boundary")
                    }
                }
                checked += 1;
            }
        }
    }
    eprintln!("GEO03_CAMERA_ORACLE poses=3 rays={checked} hit={hits} miss={misses}");
    assert!(checked >= 2800 && hits > 100 && misses > 100);
}
#[test]
fn camera_rejects_invalid_or_parallel_reference_vectors() {
    assert!(
        PerspectiveCamera::look_at(v(0.0, 0.0, 0.0), v(0.0, 0.0, 0.0), v(0.0, 1.0, 0.0), 1.0)
            .is_err()
    );
    assert!(
        PerspectiveCamera::look_at(v(0.0, 0.0, 1.0), v(0.0, 0.0, 0.0), v(0.0, 0.0, -1.0), 1.0)
            .is_err()
    );
    assert!(PerspectiveCamera::look_at(
        v(0.0, 0.0, 1.0),
        v(0.0, 0.0, 0.0),
        v(0.0, 1.0, 0.0),
        f64::NAN
    )
    .is_err());
    let c = PerspectiveCamera::look_at(v(0.0, 0.0, 1.0), v(0.0, 0.0, 0.0), v(0.0, 1.0, 0.0), 1.0)
        .unwrap();
    assert!(c.ray_for_pixel(0, 0, 0, 10).is_err());
    assert!(c.ray_for_pixel(10, 0, 10, 10).is_err());
}
