use blobin_cp01_hybrid::{
    gltf::import_glb,
    mesh::ExtrudedPrism,
    topology::{inspect_closed_topology, TopologyIssue},
    MeshAsset, Vec3,
};
fn p(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3::new(x, y, z)
}
fn tetra() -> MeshAsset {
    import_glb(include_bytes!("../fixtures/tetra-indexed.glb"))
        .unwrap()
        .mesh
}
fn l_prism() -> MeshAsset {
    ExtrudedPrism::build(
        vec![[0., 0.], [3., 0.], [3., 1.], [1., 1.], [1., 3.], [0., 3.]],
        -1.,
        1.,
    )
    .unwrap()
    .mesh()
    .clone()
}
#[test]
fn imported_tetra_has_only_candidate_topology() {
    let result = inspect_closed_topology(&tetra()).unwrap();
    assert_eq!(result.vertices, 4);
    assert_eq!(result.triangles, 4);
    assert_eq!(result.edges, 6);
    assert_eq!(result.euler_characteristic, 2);
}
#[test]
fn generated_nonconvex_prism_remains_topology_candidate_not_general_certificate() {
    let mesh = l_prism();
    let result = inspect_closed_topology(&mesh).unwrap();
    assert_eq!(result.euler_characteristic, 2);
    assert_eq!(result.triangles, 20);
    assert_eq!(result.vertices, 12);
}
#[test]
fn reject_open_reversed_and_duplicate_vertices() {
    let mut mesh = tetra();
    mesh.triangles.remove(1);
    assert_eq!(
        inspect_closed_topology(&mesh).unwrap_err(),
        TopologyIssue::BoundaryOrNonManifoldEdge
    );
    let mut mesh = tetra();
    mesh.triangles[1].swap(0, 1);
    assert_eq!(
        inspect_closed_topology(&mesh).unwrap_err(),
        TopologyIssue::InconsistentWinding
    );
    let mut mesh = tetra();
    mesh.vertices[1] = mesh.vertices[0];
    assert!(matches!(
        inspect_closed_topology(&mesh),
        Err(TopologyIssue::InvalidSurface | TopologyIssue::DegenerateOrCoincidentVertices)
    ));
}
#[test]
fn disconnected_closed_shells_do_not_gain_solid_authority() {
    let mut m = tetra();
    let other = tetra();
    let offset = m.vertices.len() as u32;
    m.vertices
        .extend(other.vertices.into_iter().map(|v| v.plus(p(3., 0., 0.))));
    m.triangles.extend(
        other
            .triangles
            .iter()
            .map(|t| [t[0] + offset, t[1] + offset, t[2] + offset]),
    );
    assert_eq!(
        inspect_closed_topology(&m).unwrap_err(),
        TopologyIssue::DisconnectedShell
    );
}
#[test]
fn bounded_topology_audit_rejects_excessive_triangle_budget() {
    let mut mesh = tetra();
    mesh.triangles.resize(1025, [0, 1, 2]);
    assert_eq!(
        inspect_closed_topology(&mesh).unwrap_err(),
        TopologyIssue::TooLarge
    );
}
#[test]
fn twisted_cube_variants_are_never_silently_treated_as_certified_solids() {
    let mesh = ExtrudedPrism::build(vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]], -1., 1.)
        .unwrap()
        .mesh()
        .clone();
    let mut suspect = 0;
    let mut crossed = 0;
    for x in [0.2, 0.4, 0.6, 1.5] {
        for y in [0.2, 0.4, 1.3] {
            for z in [-1.5, -0.5, 0.0] {
                let mut altered = mesh.clone();
                altered.vertices[6] = p(x, y, z);
                let result = inspect_closed_topology(&altered);
                if result == Err(TopologyIssue::DetectedSelfIntersection) {
                    crossed += 1;
                }
                if result.is_err() {
                    suspect += 1;
                }
            }
        }
    }
    eprintln!("GEO04_TOPOLOGY_TWISTED_SUSPECT={suspect} DETECTED_SELF_INTERSECTIONS={crossed}");
    assert!(
        crossed > 0,
        "a true non-adjacent triangle collision must be diagnosed"
    );
    assert!(
        suspect > 0,
        "twisted candidate meshes must trigger diagnostic scrutiny"
    );
}
