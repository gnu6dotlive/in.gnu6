use blobin_cp01_hybrid::*;
use std::collections::BTreeMap;
fn v(x: f64, y: f64, z: f64) -> Vec3 {
    Vec3::new(x, y, z)
}
fn sdf(shape: Primitive) -> NodeKind {
    NodeKind::Sdf(SdfLeaf {
        shape,
        transform: Transform::IDENTITY,
    })
}
fn doc(nodes: Vec<(u64, NodeKind)>, root: u64) -> SceneDocument {
    SceneDocument {
        version: 1,
        roots: vec![NodeId(root)],
        nodes: nodes
            .into_iter()
            .map(|(n, v)| (NodeId(n), v))
            .collect::<BTreeMap<_, _>>(),
    }
}
fn near(a: f64, b: f64) {
    assert!((a - b).abs() < 1e-10, "{a} != {b}");
}

#[test]
fn sphere_signed_distance_analytic_fixture() {
    let s = doc(vec![(1, sdf(Primitive::Sphere { radius: 1. }))], 1);
    for (p, d) in [
        (v(0., 0., 0.), -1.),
        (v(1., 0., 0.), 0.),
        (v(2., 0., 0.), 1.),
    ] {
        let sample = s.sample(NodeId(1), p).unwrap();
        near(sample.distance, d);
        assert_eq!(sample.semantic, Semantic::ExactDistance);
    }
}
#[test]
fn box_outside_inside_face_and_corner() {
    let s = doc(
        vec![(
            1,
            sdf(Primitive::Box {
                half_extents: v(1., 1., 1.),
            }),
        )],
        1,
    );
    for (p, d) in [
        (v(0., 0., 0.), -1.),
        (v(1., 0., 0.), 0.),
        (v(2., 0., 0.), 1.),
        (v(2., 2., 0.), 2f64.sqrt()),
    ] {
        near(s.sample(NodeId(1), p).unwrap().distance, d);
    }
}
#[test]
fn capsule_segment_ends() {
    let s = doc(
        vec![(
            1,
            sdf(Primitive::Capsule {
                a: v(0., -1., 0.),
                b: v(0., 1., 0.),
                radius: 0.5,
            }),
        )],
        1,
    );
    for (p, d) in [
        (v(0., 0., 0.), -0.5),
        (v(0., 2., 0.), 0.5),
        (v(1., 0., 0.), 0.5),
    ] {
        near(s.sample(NodeId(1), p).unwrap().distance, d);
    }
}
#[test]
fn transformed_sdf_still_has_true_world_distance() {
    let shape = Primitive::Sphere { radius: 1. };
    let s = doc(
        vec![(
            1,
            NodeKind::Sdf(SdfLeaf {
                shape,
                transform: Transform {
                    translation: v(2., 0., 0.),
                    rotation: Quat::IDENTITY,
                    scale: 2.,
                },
            }),
        )],
        1,
    );
    near(s.sample(NodeId(1), v(2., 0., 0.)).unwrap().distance, -2.);
    near(s.sample(NodeId(1), v(4., 0., 0.)).unwrap().distance, 0.);
    near(s.sample(NodeId(1), v(5., 0., 0.)).unwrap().distance, 1.);
}
#[test]
fn rotation_applies_to_box() {
    let z90 = Quat {
        w: 0.5f64.sqrt(),
        x: 0.,
        y: 0.,
        z: 0.5f64.sqrt(),
    };
    let s = doc(
        vec![(
            1,
            NodeKind::Sdf(SdfLeaf {
                shape: Primitive::Box {
                    half_extents: v(2., 1., 1.),
                },
                transform: Transform {
                    translation: v(0., 0., 0.),
                    rotation: z90,
                    scale: 1.,
                },
            }),
        )],
        1,
    );
    near(s.sample(NodeId(1), v(0., 2., 0.)).unwrap().distance, 0.);
    near(s.sample(NodeId(1), v(2., 0., 0.)).unwrap().distance, 1.);
}
#[test]
fn csg_difference_creates_a_cavity() {
    let s = doc(
        vec![
            (1, sdf(Primitive::Sphere { radius: 1. })),
            (2, sdf(Primitive::Sphere { radius: 0.5 })),
            (
                3,
                NodeKind::Csg {
                    op: CsgOp::Difference,
                    left: NodeId(1),
                    right: NodeId(2),
                },
            ),
        ],
        3,
    );
    let center = s.sample(NodeId(3), v(0., 0., 0.)).unwrap();
    near(center.distance, 0.5);
    assert_eq!(center.semantic, Semantic::FieldEstimate);
    near(
        s.sample(NodeId(3), v(0.75, 0., 0.)).unwrap().distance,
        -0.25,
    );
}
#[test]
fn union_and_intersection_distinct() {
    let sphere = sdf(Primitive::Sphere { radius: 1. });
    for (op, expected) in [(CsgOp::Union, -1.), (CsgOp::Intersection, -0.5)] {
        let s = doc(
            vec![
                (1, sphere.clone()),
                (2, sdf(Primitive::Sphere { radius: 0.5 })),
                (
                    3,
                    NodeKind::Csg {
                        op,
                        left: NodeId(1),
                        right: NodeId(2),
                    },
                ),
            ],
            3,
        );
        near(
            s.sample(NodeId(3), v(0., 0., 0.)).unwrap().distance,
            expected,
        );
    }
}
#[test]
fn smooth_union_reports_field_not_exact_distance() {
    let s = doc(
        vec![
            (1, sdf(Primitive::Sphere { radius: 1. })),
            (2, sdf(Primitive::Sphere { radius: 1. })),
            (
                3,
                NodeKind::Csg {
                    op: CsgOp::SmoothUnion { k: 0.5 },
                    left: NodeId(1),
                    right: NodeId(2),
                },
            ),
        ],
        3,
    );
    near(s.sample(NodeId(3), v(0., 0., 0.)).unwrap().distance, -1.125);
    assert_eq!(
        s.sample(NodeId(3), v(0., 0., 0.)).unwrap().semantic,
        Semantic::FieldEstimate
    );
}
#[test]
fn preflight_never_certifies_solidity_from_edge_counts_alone() {
    let vertices = vec![v(0., 0., 0.), v(1., 0., 0.), v(0., 1., 0.), v(0., 0., 1.)];
    let tetra = MeshAsset {
        vertices: vertices.clone(),
        triangles: vec![[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]],
    };
    assert_eq!(
        tetra.preflight(),
        Ok(MeshPreflight::ClosedCandidateNotCertified)
    );
    let open = MeshAsset {
        vertices,
        triangles: vec![[0, 2, 1]],
    };
    assert_eq!(open.preflight(), Ok(MeshPreflight::SurfaceOnly));
}
#[test]
fn surface_mesh_refused_as_signed_csg_operand() {
    let open = MeshAsset {
        vertices: vec![v(0., 0., 0.), v(1., 0., 0.), v(0., 1., 0.)],
        triangles: vec![[0, 1, 2]],
    };
    let s = doc(
        vec![
            (1, sdf(Primitive::Sphere { radius: 1. })),
            (2, NodeKind::MeshSurface(open)),
            (
                3,
                NodeKind::Csg {
                    op: CsgOp::Difference,
                    left: NodeId(1),
                    right: NodeId(2),
                },
            ),
        ],
        3,
    );
    assert_eq!(
        s.sample(NodeId(3), v(0., 0., 0.)).unwrap_err(),
        GeometryError::SurfaceHasNoSignedField
    );
}
#[test]
fn invalid_mesh_and_nonfinite_inputs_rejected() {
    let mesh = MeshAsset {
        vertices: vec![v(0., 0., 0.), v(1., 0., 0.), v(2., 0., 0.)],
        triangles: vec![[0, 1, 2]],
    };
    assert_eq!(mesh.preflight(), Err(GeometryError::InvalidMesh));
    let s = doc(vec![(1, sdf(Primitive::Sphere { radius: 1. }))], 1);
    assert_eq!(
        s.sample(NodeId(1), v(f64::NAN, 0., 0.)).unwrap_err(),
        GeometryError::InvalidInput
    );
}
#[test]
fn graph_cycles_and_unknown_references_rejected() {
    let cycle = doc(
        vec![
            (
                1,
                NodeKind::Csg {
                    op: CsgOp::Union,
                    left: NodeId(1),
                    right: NodeId(2),
                },
            ),
            (2, sdf(Primitive::Sphere { radius: 1. })),
        ],
        1,
    );
    assert_eq!(cycle.validate(), Err(GeometryError::GraphCycle));
    let missing = doc(
        vec![
            (
                1,
                NodeKind::Csg {
                    op: CsgOp::Union,
                    left: NodeId(2),
                    right: NodeId(3),
                },
            ),
            (2, sdf(Primitive::Sphere { radius: 1. })),
        ],
        1,
    );
    assert_eq!(missing.validate(), Err(GeometryError::MissingNode));
}
#[test]
fn invalid_scale_quat_and_smooth_radius_rejected() {
    let bad = doc(
        vec![(
            1,
            NodeKind::Sdf(SdfLeaf {
                shape: Primitive::Sphere { radius: 1. },
                transform: Transform {
                    scale: 0.,
                    ..Transform::IDENTITY
                },
            }),
        )],
        1,
    );
    assert_eq!(bad.validate(), Err(GeometryError::InvalidInput));
    let bad = doc(
        vec![(
            1,
            NodeKind::Sdf(SdfLeaf {
                shape: Primitive::Sphere { radius: 1. },
                transform: Transform {
                    rotation: Quat {
                        w: 0.,
                        x: 0.,
                        y: 0.,
                        z: 0.,
                    },
                    ..Transform::IDENTITY
                },
            }),
        )],
        1,
    );
    assert_eq!(bad.validate(), Err(GeometryError::InvalidInput));
}

#[test]
fn shared_subgraphs_do_not_expand_exponentially() {
    let mut nodes = vec![(1, sdf(Primitive::Sphere { radius: 1. }))];
    // ~2^49 naive expansions; memoized graph has only 50 nodes.
    for n in 2..=50 {
        nodes.push((
            n,
            NodeKind::Csg {
                op: CsgOp::Union,
                left: NodeId(n - 1),
                right: NodeId(n - 1),
            },
        ));
    }
    let s = doc(nodes, 50);
    s.validate().unwrap();
    near(s.sample(NodeId(50), v(0., 0., 0.)).unwrap().distance, -1.);
}
#[test]
fn graph_depth_is_bounded_even_with_shared_nodes() {
    let mut nodes = vec![(1, sdf(Primitive::Sphere { radius: 1. }))];
    for n in 2..=65 {
        nodes.push((
            n,
            NodeKind::Csg {
                op: CsgOp::Union,
                left: NodeId(n - 1),
                right: NodeId(1),
            },
        ));
    }
    assert_eq!(doc(nodes, 65).validate(), Err(GeometryError::GraphTooDeep));
}
#[test]
fn insertion_rejects_reused_ids_and_zero_ids() {
    let mut s = doc(vec![(1, sdf(Primitive::Sphere { radius: 1. }))], 1);
    assert_eq!(
        s.insert(NodeId(1), sdf(Primitive::Sphere { radius: 2. })),
        Err(GeometryError::DuplicateId)
    );
    assert_eq!(
        s.insert(NodeId(0), sdf(Primitive::Sphere { radius: 2. })),
        Err(GeometryError::InvalidInput)
    );
}
