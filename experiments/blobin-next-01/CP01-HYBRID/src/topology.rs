//! GEO-04 conservative closed-surface diagnostics.
//! A report means *candidate topology only*, NOT proof of signed interior.
//! The checker deliberately rejects detectable disjoint-triangle collisions.
use crate::{MeshAsset, Vec3};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const MAX_AUDIT_TRIANGLES: usize = 1024;
pub const MAX_AUDIT_VERTICES: usize = 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologyIssue {
    TooLarge,
    InvalidSurface,
    DegenerateOrCoincidentVertices,
    BoundaryOrNonManifoldEdge,
    InconsistentWinding,
    DisconnectedShell,
    PinchedVertexFan,
    DetectedSelfIntersection,
    AmbiguousVolume,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClosedTopologyCandidate {
    pub vertices: usize,
    pub triangles: usize,
    pub edges: usize,
    pub euler_characteristic: isize,
}
fn tri(m: &MeshAsset, i: usize) -> [Vec3; 3] {
    let [a, b, c] = m.triangles[i];
    [
        m.vertices[a as usize],
        m.vertices[b as usize],
        m.vertices[c as usize],
    ]
}
fn bbox(t: [Vec3; 3]) -> (Vec3, Vec3) {
    let mut lo = t[0];
    let mut hi = t[0];
    for p in &t[1..] {
        lo = Vec3::new(lo.x.min(p.x), lo.y.min(p.y), lo.z.min(p.z));
        hi = Vec3::new(hi.x.max(p.x), hi.y.max(p.y), hi.z.max(p.z));
    }
    (lo, hi)
}
fn tri_pair_bounds_overlap(a: [Vec3; 3], b: [Vec3; 3], eps: f64) -> bool {
    let (al, ah) = bbox(a);
    let (bl, bh) = bbox(b);
    !(al.x > bh.x + eps
        || bl.x > ah.x + eps
        || al.y > bh.y + eps
        || bl.y > ah.y + eps
        || al.z > bh.z + eps
        || bl.z > ah.z + eps)
}
fn segment_cross_triangle(a: Vec3, b: Vec3, tri: [Vec3; 3], eps: f64) -> bool {
    let [p, q, r] = tri;
    let direction = b.minus(a);
    let edge1 = q.minus(p);
    let edge2 = r.minus(p);
    let n = edge1.cross(edge2);
    let d = n.dot(direction);
    if d.abs() < eps * n.norm() {
        return false;
    } // coplanar/parallel separately
    let t = n.dot(p.minus(a)) / d;
    if t < -eps || t > 1.0 + eps {
        return false;
    }
    let point = a.plus(direction.scale(t));
    let v = point.minus(p);
    let dot00 = edge1.dot(edge1);
    let dot01 = edge1.dot(edge2);
    let dot11 = edge2.dot(edge2);
    let dot02 = edge1.dot(v);
    let dot12 = edge2.dot(v);
    let denom = dot00 * dot11 - dot01 * dot01;
    if denom <= eps * eps {
        return true;
    } // suspicious near-degenerate
    let u = (dot11 * dot02 - dot01 * dot12) / denom;
    let w = (dot00 * dot12 - dot01 * dot02) / denom;
    u >= -eps && w >= -eps && u + w <= 1.0 + eps
}
fn drop_axis(p: Vec3, axis: usize) -> [f64; 2] {
    match axis {
        0 => [p.y, p.z],
        1 => [p.x, p.z],
        _ => [p.x, p.y],
    }
}
fn cross(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}
fn within(p: [f64; 2], t: [[f64; 2]; 3], eps: f64) -> bool {
    let a = cross(t[0], t[1], p);
    let b = cross(t[1], t[2], p);
    let c = cross(t[2], t[0], p);
    (a >= -eps && b >= -eps && c >= -eps) || (a <= eps && b <= eps && c <= eps)
}
fn touch_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2], eps: f64) -> bool {
    let o = [
        cross(a, b, c),
        cross(a, b, d),
        cross(c, d, a),
        cross(c, d, b),
    ];
    if (o[0] > eps && o[1] < -eps || o[0] < -eps && o[1] > eps)
        && (o[2] > eps && o[3] < -eps || o[2] < -eps && o[3] > eps)
    {
        return true;
    }
    for (k, p, l, r) in [(0, c, a, b), (1, d, a, b), (2, a, c, d), (3, b, c, d)] {
        if o[k].abs() <= eps
            && p[0] >= l[0].min(r[0]) - eps
            && p[0] <= l[0].max(r[0]) + eps
            && p[1] >= l[1].min(r[1]) - eps
            && p[1] <= l[1].max(r[1]) + eps
        {
            return true;
        }
    }
    false
}
fn triangle_pair_intersects(a: [Vec3; 3], b: [Vec3; 3], eps: f64) -> bool {
    if !tri_pair_bounds_overlap(a, b, eps) {
        return false;
    }
    for i in 0..3 {
        if segment_cross_triangle(a[i], a[(i + 1) % 3], b, eps)
            || segment_cross_triangle(b[i], b[(i + 1) % 3], a, eps)
        {
            return true;
        }
    }
    let n = a[1].minus(a[0]).cross(a[2].minus(a[0]));
    let nb = b[1].minus(b[0]).cross(b[2].minus(b[0]));
    if n.cross(nb).norm() > eps * n.norm() * nb.norm() {
        return false;
    }
    if b.iter()
        .any(|v| n.dot(v.minus(a[0])).abs() > eps * n.norm())
    {
        return false;
    }
    let components = [n.x.abs(), n.y.abs(), n.z.abs()];
    let axis = if components[0] >= components[1] && components[0] >= components[2] {
        0
    } else if components[1] >= components[2] {
        1
    } else {
        2
    };
    let a = a.map(|p| drop_axis(p, axis));
    let b = b.map(|p| drop_axis(p, axis));
    if within(a[0], b, eps) || within(b[0], a, eps) {
        return true;
    }
    for i in 0..3 {
        for j in 0..3 {
            if touch_2d(a[i], a[(i + 1) % 3], b[j], b[(j + 1) % 3], eps) {
                return true;
            }
        }
    }
    false
}
pub fn inspect_closed_topology(mesh: &MeshAsset) -> Result<ClosedTopologyCandidate, TopologyIssue> {
    if mesh.triangles.len() > MAX_AUDIT_TRIANGLES || mesh.vertices.len() > MAX_AUDIT_VERTICES {
        return Err(TopologyIssue::TooLarge);
    }
    mesh.preflight()
        .map_err(|_| TopologyIssue::InvalidSurface)?;
    let (mut low, mut high) = (mesh.vertices[0], mesh.vertices[0]);
    for p in &mesh.vertices {
        low = Vec3::new(low.x.min(p.x), low.y.min(p.y), low.z.min(p.z));
        high = Vec3::new(high.x.max(p.x), high.y.max(p.y), high.z.max(p.z));
    }
    let diameter = high.minus(low).norm();
    if !(1e-3..=1e3).contains(&diameter) {
        return Err(TopologyIssue::AmbiguousVolume);
    }
    let eps = diameter * 1e-10;
    for (i, p) in mesh.vertices.iter().enumerate() {
        for q in &mesh.vertices[i + 1..] {
            if p.minus(*q).norm() <= eps {
                return Err(TopologyIssue::DegenerateOrCoincidentVertices);
            }
        }
    }
    let mut edges = BTreeMap::<(u32, u32), Vec<(usize, i8)>>::new();
    let mut vertex_faces = vec![Vec::new(); mesh.vertices.len()];
    let mut signed_volume = 0.0f64;
    for (i, t) in mesh.triangles.iter().enumerate() {
        let [a, b, c] = *t;
        for &k in t {
            vertex_faces[k as usize].push(i);
        }
        for (u, v) in [(a, b), (b, c), (c, a)] {
            edges
                .entry((u.min(v), u.max(v)))
                .or_default()
                .push((i, if u < v { 1 } else { -1 }));
        }
        let [p, q, r] = tri(mesh, i);
        signed_volume += p.dot(q.cross(r)) / 6.0;
    }
    let mut adjacency = vec![BTreeSet::new(); mesh.triangles.len()];
    for members in edges.values() {
        if members.len() != 2 {
            return Err(TopologyIssue::BoundaryOrNonManifoldEdge);
        }
        if members[0].1 + members[1].1 != 0 {
            return Err(TopologyIssue::InconsistentWinding);
        }
        adjacency[members[0].0].insert(members[1].0);
        adjacency[members[1].0].insert(members[0].0);
    }
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::from([0usize]);
    while let Some(x) = queue.pop_front() {
        if visited.insert(x) {
            queue.extend(adjacency[x].iter().copied());
        }
    }
    if visited.len() != mesh.triangles.len() {
        return Err(TopologyIssue::DisconnectedShell);
    }
    for (vertex, faces) in vertex_faces.iter().enumerate() {
        if faces.len() < 3 {
            return Err(TopologyIssue::PinchedVertexFan);
        }
        let mut neighbors = BTreeMap::<usize, BTreeSet<usize>>::new();
        for &face in faces {
            let around = adjacency[face]
                .iter()
                .copied()
                .filter(|&f| mesh.triangles[f].contains(&(vertex as u32)))
                .collect::<BTreeSet<_>>();
            if around.len() != 2 {
                return Err(TopologyIssue::PinchedVertexFan);
            }
            neighbors.insert(face, around);
        }
        let mut discovered = BTreeSet::new();
        let mut to_visit = vec![faces[0]];
        while let Some(f) = to_visit.pop() {
            if discovered.insert(f) {
                to_visit.extend(neighbors[&f].iter().copied());
            }
        }
        if discovered.len() != faces.len() {
            return Err(TopologyIssue::PinchedVertexFan);
        }
    }
    if signed_volume.abs() <= eps.powi(3) {
        return Err(TopologyIssue::AmbiguousVolume);
    }
    if signed_volume < 0.0 {
        return Err(TopologyIssue::InconsistentWinding);
    }
    // Detect many non-adjacent self-collisions. This does NOT certify against
    // near-degenerate or shared-vertex/edge overlapping triangles.
    for i in 0..mesh.triangles.len() {
        for j in i + 1..mesh.triangles.len() {
            if mesh.triangles[i]
                .iter()
                .any(|x| mesh.triangles[j].contains(x))
            {
                continue;
            }
            if triangle_pair_intersects(tri(mesh, i), tri(mesh, j), eps) {
                return Err(TopologyIssue::DetectedSelfIntersection);
            }
        }
    }
    Ok(ClosedTopologyCandidate {
        vertices: mesh.vertices.len(),
        triangles: mesh.triangles.len(),
        edges: edges.len(),
        euler_characteristic: mesh.vertices.len() as isize - edges.len() as isize
            + mesh.triangles.len() as isize,
    })
}
