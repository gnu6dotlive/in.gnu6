use blobin_cp01_hybrid::{
    gltf::{import_glb, import_gltf_json_with_supplied_buffer, ImportError, SourceFormat},
    mesh::CertifiedConvexSolid,
    MeshPreflight,
};
use serde_json::{json, Value};

fn tetra(indexed: bool) -> (Value, Vec<u8>) {
    let vertices: Vec<[f32; 3]> = vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];
    let indices: [u16; 12] = [0, 2, 1, 0, 1, 3, 0, 3, 2, 1, 2, 3];
    let mut bytes = Vec::new();
    let pos: Vec<[f32; 3]> = if indexed {
        vertices
    } else {
        indices.iter().map(|&v| vertices[v as usize]).collect()
    };
    for p in &pos {
        for n in p {
            bytes.extend_from_slice(&n.to_le_bytes());
        }
    }
    let mut views = vec![json!({"buffer":0,"byteOffset":0,"byteLength":bytes.len()})];
    let mut acc =
        vec![json!({"bufferView":0,"componentType":5126,"count":pos.len(),"type":"VEC3"})];
    let mut prim = json!({"attributes":{"POSITION":0},"mode":4});
    if indexed {
        let offset = bytes.len();
        for n in indices {
            bytes.extend_from_slice(&n.to_le_bytes());
        }
        views.push(json!({"buffer":0,"byteOffset":offset,"byteLength":24}));
        acc.push(json!({"bufferView":1,"componentType":5123,"count":12,"type":"SCALAR"}));
        prim["indices"] = json!(1);
    }
    let doc = json!({"asset":{"version":"2.0"},"buffers":[{"byteLength":bytes.len()}],
      "bufferViews":views,"accessors":acc,"meshes":[{"primitives":[prim]}],
      "nodes":[{"mesh":0}],"scenes":[{"nodes":[0]}],"scene":0});
    (doc, bytes)
}
fn glb(doc: &Value, data: &[u8]) -> Vec<u8> {
    let mut j = serde_json::to_vec(doc).unwrap();
    while !j.len().is_multiple_of(4) {
        j.push(b' ');
    }
    let mut bin = data.to_vec();
    while !bin.len().is_multiple_of(4) {
        bin.push(0);
    }
    let mut result = Vec::new();
    result.extend_from_slice(b"glTF");
    result.extend_from_slice(&2u32.to_le_bytes());
    let len = 12 + 8 + j.len() + 8 + bin.len();
    result.extend_from_slice(&(len as u32).to_le_bytes());
    result.extend_from_slice(&(j.len() as u32).to_le_bytes());
    result.extend_from_slice(&0x4e4f534au32.to_le_bytes());
    result.extend(j);
    result.extend_from_slice(&(bin.len() as u32).to_le_bytes());
    result.extend_from_slice(&0x004e4942u32.to_le_bytes());
    result.extend(bin);
    result
}
fn assert_err_eq(doc: &Value, data: &[u8], error: ImportError) {
    assert_eq!(import_glb(&glb(doc, data)).unwrap_err(), error);
}
#[test]
fn glb_indexed_tetra_imported_as_surface_not_certified() {
    let (doc, data) = tetra(true);
    let item = import_glb(&glb(&doc, &data)).unwrap();
    assert_eq!(item.source, SourceFormat::Glb);
    assert_eq!(item.mesh.vertices.len(), 4);
    assert_eq!(item.mesh.triangles.len(), 4);
    assert_eq!(
        item.mesh.preflight().unwrap(),
        MeshPreflight::ClosedCandidateNotCertified
    );
    // Promotion to convex solid is a separate explicit operation, not import behaviour.
    let certified = CertifiedConvexSolid::certify(item.mesh).unwrap();
    assert!(
        certified
            .signed_distance(blobin_cp01_hybrid::Vec3::new(0.1, 0.1, 0.1))
            .unwrap()
            < 0.0
    );
}
#[test]
fn glb_nonindexed_tetra_surface() {
    let (doc, data) = tetra(false);
    let item = import_glb(&glb(&doc, &data)).unwrap();
    assert_eq!(item.mesh.triangles.len(), 4);
    assert_eq!(item.mesh.vertices.len(), 12);
}
#[test]
fn static_gltf_json_external_buffer_is_explicitly_supplied() {
    let (mut doc, data) = tetra(true);
    doc["buffers"][0]["uri"] = json!("tetra.bin");
    let json = serde_json::to_vec(&doc).unwrap();
    let item = import_gltf_json_with_supplied_buffer(&json, &data).unwrap();
    assert_eq!(item.source, SourceFormat::GltfJsonExternalBuffer);
    assert_eq!(item.mesh.triangles[0], [0, 2, 1]);
}
#[test]
fn external_resource_path_traversal_and_network_rejected() {
    let (mut doc, data) = tetra(true);
    for uri in [
        "../outside.bin",
        "http://site/file.bin",
        "subdir/asset.bin",
        r"folder\mesh.bin",
        "data:application/octet-stream;base64,AA==",
    ] {
        doc["buffers"][0]["uri"] = json!(uri);
        assert_eq!(
            import_gltf_json_with_supplied_buffer(&serde_json::to_vec(&doc).unwrap(), &data)
                .unwrap_err(),
            ImportError::UnsupportedUri,
            "{uri}"
        );
    }
}
#[test]
fn glb_invalid_header_length_chunks_and_alignment() {
    let (doc, data) = tetra(true);
    let good = glb(&doc, &data);
    let mut bad = good.clone();
    bad[0] = b'B';
    assert_eq!(import_glb(&bad).unwrap_err(), ImportError::GlbHeader);
    let mut bad = good.clone();
    bad[8..12].copy_from_slice(&(good.len() as u32 + 1).to_le_bytes());
    assert_eq!(import_glb(&bad).unwrap_err(), ImportError::GlbHeader);
    let mut bad = good.clone();
    bad[12..16].copy_from_slice(&3u32.to_le_bytes());
    assert_eq!(import_glb(&bad).unwrap_err(), ImportError::GlbChunk);
    let mut bad = good.clone();
    bad.truncate(good.len() - 1);
    assert_eq!(import_glb(&bad).unwrap_err(), ImportError::GlbHeader);
}
#[test]
fn oversized_counts_short_buffers_and_offset_overflow() {
    let (mut doc, data) = tetra(true);
    doc["accessors"][0]["count"] = json!(u64::MAX);
    assert_err_eq(&doc, &data, ImportError::TooLarge);
    let (mut doc, data) = tetra(true);
    doc["bufferViews"][0]["byteLength"] = json!(data.len() + 1);
    assert_err_eq(&doc, &data, ImportError::OutOfBounds);
    let (mut doc, data) = tetra(true);
    doc["accessors"][0]["byteOffset"] = json!(u64::MAX);
    assert!(matches!(
        import_glb(&glb(&doc, &data)),
        Err(ImportError::Alignment | ImportError::OutOfBounds | ImportError::InvalidAccessor)
    ));
}
#[test]
fn invalid_f32_nan_and_infinity_are_not_geometry() {
    for bits in [f32::NAN.to_bits(), f32::INFINITY.to_bits()] {
        let (doc, mut data) = tetra(true);
        data[0..4].copy_from_slice(&bits.to_le_bytes());
        assert_err_eq(&doc, &data, ImportError::InvalidGeometry);
    }
}
#[test]
fn unsupported_morph_animation_skin_and_node_transform_fail_closed() {
    let (base, data) = tetra(true);
    for path in 0..5 {
        let mut doc = base.clone();
        match path {
            0 => doc["animations"] = json!([{}]),
            1 => doc["skins"] = json!([{}]),
            2 => doc["nodes"][0]["translation"] = json!([1.0, 0.0, 0.0]),
            3 => doc["meshes"][0]["primitives"][0]["targets"] = json!([]),
            _ => doc["extensionsRequired"] = json!(["KHR_draco_mesh_compression"]),
        }
        assert_err_eq(&doc, &data, ImportError::Unsupported);
    }
}
#[test]
fn glb_does_not_accept_buffer_uri_even_if_safe() {
    let (mut doc, data) = tetra(true);
    doc["buffers"][0]["uri"] = json!("tetra.bin");
    assert_err_eq(&doc, &data, ImportError::UnsupportedUri);
}
#[test]
fn indices_and_topology_are_validated_before_return() {
    let (mut doc, mut data) = tetra(true);
    // First triangle degenerate, index 2 equals index 0.
    data[48 + 4..48 + 6].copy_from_slice(&0u16.to_le_bytes());
    assert_err_eq(&doc, &data, ImportError::InvalidGeometry);
    doc["meshes"][0]["primitives"][0]["mode"] = json!(1);
    assert_err_eq(&doc, &data, ImportError::Unsupported);
}

#[test]
fn sparse_stride_and_misalignment_rejected() {
    let (base, bytes) = tetra(true);
    let mut doc = base.clone();
    doc["accessors"][0]["sparse"] = json!({"count":1});
    assert_err_eq(&doc, &bytes, ImportError::Unsupported);
    let mut doc = base.clone();
    doc["bufferViews"][0]["byteStride"] = json!(3);
    assert_err_eq(&doc, &bytes, ImportError::Alignment);
    let mut doc = base.clone();
    doc["accessors"][0]["byteOffset"] = json!(1);
    assert_err_eq(&doc, &bytes, ImportError::Alignment);
}
#[test]
fn importer_fuzzes_500_mutated_containers_without_panicking() {
    let (doc, data) = tetra(true);
    let seed = glb(&doc, &data);
    let mut state = 0x3e77d1820b2f68a1u64;
    let mut rejected = 0usize;
    for _ in 0..500 {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let mut candidate = seed.clone();
        let offset = (state as usize) % candidate.len();
        candidate[offset] ^= ((state >> 41) as u8) | 1;
        if import_glb(&candidate).is_err() {
            rejected += 1;
        }
    }
    eprintln!("GEO03_GLTF_MUTATED_CONTAINERS trials=500 rejected={rejected}");
    assert!(rejected > 0);
}

#[test]
fn indices_must_be_tightly_packed_and_vertex_stride_bounded() {
    let (mut doc, data) = tetra(true);
    doc["bufferViews"][1]["byteStride"] = json!(2);
    assert_err_eq(&doc, &data, ImportError::Unsupported);
    let (mut doc, data) = tetra(true);
    doc["bufferViews"][0]["byteStride"] = json!(256);
    assert_err_eq(&doc, &data, ImportError::Alignment);
}

#[test]
fn canonical_static_fixtures_are_reimportable_without_filesystem_access() {
    const GLB: &[u8] = include_bytes!("../fixtures/tetra-indexed.glb");
    const GLTF: &[u8] = include_bytes!("../fixtures/tetra.gltf");
    const BIN: &[u8] = include_bytes!("../fixtures/tetra.bin");
    let binary = import_glb(GLB).unwrap();
    let json = import_gltf_json_with_supplied_buffer(GLTF, BIN).unwrap();
    assert_eq!(binary.mesh.vertices, json.mesh.vertices);
    assert_eq!(binary.mesh.triangles, json.mesh.triangles);
    assert_eq!(
        binary.mesh.preflight().unwrap(),
        MeshPreflight::ClosedCandidateNotCertified
    );
}

#[test]
fn imported_tetra_requires_explicit_convex_certification_for_mixed_csg() {
    use blobin_cp01_hybrid::{
        mesh::CertifiedMeshSolid, CsgOp, NodeId, NodeKind, Primitive, SceneDocument, SdfLeaf,
        Semantic, Transform, Vec3,
    };
    use std::collections::BTreeMap;
    let item = import_glb(include_bytes!("../fixtures/tetra-indexed.glb")).unwrap();
    let convex = item.try_certify_convex().unwrap();
    let mut nodes = BTreeMap::new();
    nodes.insert(
        NodeId(1),
        NodeKind::MeshSolid(CertifiedMeshSolid::Convex(convex)),
    );
    nodes.insert(
        NodeId(2),
        NodeKind::Sdf(SdfLeaf {
            shape: Primitive::Sphere { radius: 0.30 },
            transform: Transform {
                translation: Vec3::new(0.15, 0.15, 0.15),
                ..Transform::IDENTITY
            },
        }),
    );
    for (root, op, left, right) in [
        (3, CsgOp::Difference, 1, 2),
        (4, CsgOp::Difference, 2, 1),
        (5, CsgOp::Intersection, 1, 2),
        (6, CsgOp::Union, 1, 2),
    ] {
        nodes.insert(
            NodeId(root),
            NodeKind::Csg {
                op,
                left: NodeId(left),
                right: NodeId(right),
            },
        );
    }
    let scene = SceneDocument {
        version: 1,
        nodes,
        roots: vec![NodeId(3), NodeId(4), NodeId(5), NodeId(6)],
    };
    scene.validate().unwrap();
    let sample = |n| {
        scene
            .sample(NodeId(n), Vec3::new(0.15, 0.15, 0.15))
            .unwrap()
    };
    assert!(sample(1).distance < 0.0 && sample(2).distance < 0.0);
    assert!(sample(3).distance > 0.0);
    assert!(sample(4).distance > 0.0);
    assert!(sample(5).distance < 0.0);
    assert!(sample(6).distance < 0.0);
    for n in 3..=6 {
        assert_eq!(sample(n).semantic, Semantic::FieldEstimate);
    }
}

#[test]
fn imported_nonconvex_l_prism_is_only_topology_candidate_not_meshsolid() {
    use blobin_cp01_hybrid::topology::inspect_closed_topology;
    let item = import_glb(include_bytes!("../fixtures/l-prism-nonconvex.glb")).unwrap();
    assert_eq!(item.mesh.triangles.len(), 20);
    let inventory = inspect_closed_topology(&item.mesh).unwrap();
    assert_eq!(inventory.euler_characteristic, 2);
    assert!(
        item.try_certify_convex().is_err(),
        "topologically closed nonconvex mesh must never gain convex certificate"
    );
}
