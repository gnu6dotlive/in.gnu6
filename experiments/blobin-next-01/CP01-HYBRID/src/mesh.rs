//! GEO-01 conservative mesh queries. Certified solids are strictly convex,
//! one connected, genus-zero, oriented, manifold triangle surfaces.
//! This is not a general arbitrary-mesh/robust predicate certification.
use crate::{GeometryError, MeshAsset, Vec3};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
const MAX_CERT_VERTICES: usize = 512;
const MAX_CERT_TRIANGLES: usize = 1024;
const REL_EPS: f64 = 1.0e-10;

#[derive(Debug, Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}
impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Result<Self, GeometryError> {
        if !origin.finite() || !direction.finite() || (direction.norm() - 1.0).abs() > 1.0e-9 {
            return Err(GeometryError::InvalidInput);
        }
        Ok(Self { origin, direction })
    }
}
#[derive(Debug, Clone, Copy)]
pub struct RayHit {
    pub triangle_index: usize,
    pub t: f64,
    pub barycentric: [f64; 3],
}
#[derive(Debug, Clone, Copy)]
pub enum RayTrace {
    Miss,
    Hit(RayHit),
    AmbiguousSharedEdge { t: f64, count: usize },
}
#[derive(Debug, Clone, Copy)]
pub struct ClosestSurface {
    pub triangle_index: usize,
    pub unsigned_distance: f64,
    pub position: Vec3,
}
fn tri(mesh: &MeshAsset, i: usize) -> (Vec3, Vec3, Vec3) {
    let [a, b, c] = mesh.triangles[i];
    (
        mesh.vertices[a as usize],
        mesh.vertices[b as usize],
        mesh.vertices[c as usize],
    )
}
/// Ericson region test; works for nondegenerate triangles prevalidated by mesh.
pub fn closest_point_triangle(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> Vec3 {
    let ab = b.minus(a);
    let ac = c.minus(a);
    let ap = p.minus(a);
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return a;
    }
    let bp = p.minus(b);
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return b;
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        return a.plus(ab.scale(d1 / (d1 - d3)));
    }
    let cp = p.minus(c);
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return c;
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        return a.plus(ac.scale(d2 / (d2 - d6)));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let bc = c.minus(b);
        return b.plus(bc.scale((d4 - d3) / ((d4 - d3) + (d5 - d6))));
    }
    let denom = 1.0 / (va + vb + vc);
    a.plus(ab.scale(vb * denom)).plus(ac.scale(vc * denom))
}
impl MeshAsset {
    pub fn closest_surface(&self, p: Vec3) -> Result<ClosestSurface, GeometryError> {
        if !p.finite() {
            return Err(GeometryError::InvalidInput);
        }
        self.preflight()?;
        let mut best: Option<ClosestSurface> = None;
        for i in 0..self.triangles.len() {
            let (a, b, c) = tri(self, i);
            let point = closest_point_triangle(p, a, b, c);
            let d = p.minus(point).norm();
            if !d.is_finite() {
                return Err(GeometryError::NonFiniteResult);
            }
            if best.is_none_or(|v| d < v.unsigned_distance) {
                best = Some(ClosestSurface {
                    triangle_index: i,
                    unsigned_distance: d,
                    position: point,
                });
            }
        }
        best.ok_or(GeometryError::InvalidMesh)
    }

    /// Two-sided Möller–Trumbore; t is in world metres for unit rays.
    /// Exact edge/tangent hits are deliberately ambiguous, not silent misses.
    pub fn ray_trace(&self, ray: Ray) -> Result<RayTrace, GeometryError> {
        self.preflight()?;
        let mut hits: Vec<(RayHit, bool)> = Vec::new();
        for i in 0..self.triangles.len() {
            let (a, b, c) = tri(self, i);
            let e1 = b.minus(a);
            let e2 = c.minus(a);
            let n = e1.cross(e2);
            let det = e1.dot(ray.direction.cross(e2));
            if det.abs() <= 1.0e-12 * n.norm() {
                continue;
            }
            let inv = 1.0 / det;
            let tvec = ray.origin.minus(a);
            let u = tvec.dot(ray.direction.cross(e2)) * inv;
            let v = ray.direction.dot(tvec.cross(e1)) * inv;
            let w = 1.0 - u - v;
            let eps = 1.0e-10;
            if u < -eps || v < -eps || w < -eps {
                continue;
            }
            let t = e2.dot(tvec.cross(e1)) * inv;
            if t <= eps {
                continue;
            }
            if !t.is_finite() {
                return Err(GeometryError::NonFiniteResult);
            }
            let on_edge = u <= eps || v <= eps || w <= eps;
            hits.push((
                RayHit {
                    triangle_index: i,
                    t,
                    barycentric: [w, u, v],
                },
                on_edge,
            ));
        }
        hits.sort_by(|a, b| a.0.t.total_cmp(&b.0.t));
        match hits.first().copied() {
            None => Ok(RayTrace::Miss),
            Some((first, on_edge)) => {
                let epsilon = 1.0e-8 * first.t.max(1.0);
                let same = hits
                    .iter()
                    .take_while(|h| (h.0.t - first.t).abs() <= epsilon)
                    .count();
                if on_edge || same > 1 {
                    Ok(RayTrace::AmbiguousSharedEdge {
                        t: first.t,
                        count: same,
                    })
                } else {
                    Ok(RayTrace::Hit(first))
                }
            }
        }
    }
}

/// Constructor only: cannot be forged by enum variant or by direct fields.
#[derive(Debug, Clone)]
pub struct CertifiedConvexSolid {
    mesh: MeshAsset,
    diameter: f64,
}
impl CertifiedConvexSolid {
    pub fn mesh(&self) -> &MeshAsset {
        &self.mesh
    }
    pub fn certify(mesh: MeshAsset) -> Result<Self, GeometryError> {
        mesh.preflight()?;
        if mesh.vertices.len() > MAX_CERT_VERTICES || mesh.triangles.len() > MAX_CERT_TRIANGLES {
            return Err(GeometryError::TooManyTriangles);
        }
        let mut low = mesh.vertices[0];
        let mut high = low;
        for v in &mesh.vertices {
            low = Vec3::new(low.x.min(v.x), low.y.min(v.y), low.z.min(v.z));
            high = Vec3::new(high.x.max(v.x), high.y.max(v.y), high.z.max(v.z));
        }
        let diameter = high.minus(low).norm();
        if !(1.0e-3..=1.0e3).contains(&diameter) {
            return Err(GeometryError::AmbiguousGeometry);
        }
        let eps = diameter * REL_EPS;
        let nvert = mesh.vertices.len();
        let mut edges: BTreeMap<(u32, u32), Vec<(usize, i8)>> = BTreeMap::new();
        let mut neighbors = vec![BTreeSet::new(); mesh.triangles.len()];
        let mut incident = vec![Vec::<usize>::new(); nvert];
        let mut volume = 0.0;
        for (i, indices) in mesh.triangles.iter().enumerate() {
            let [a, b, c] = *indices;
            let (pa, pb, pc) = tri(&mesh, i);
            let normal = pb.minus(pa).cross(pc.minus(pa));
            if normal.norm() <= eps * eps {
                return Err(GeometryError::AmbiguousGeometry);
            }
            // Each face is an outward-supporting face of the convex hull.
            for v in &mesh.vertices {
                if normal.dot(v.minus(pa)) / normal.norm() > eps {
                    return Err(GeometryError::UnsupportedNonConvex);
                }
            }
            volume += pa.dot(pb.cross(pc)) / 6.0;
            for &v in indices {
                incident[v as usize].push(i);
            }
            for (u, v) in [(a, b), (b, c), (c, a)] {
                edges
                    .entry((u.min(v), u.max(v)))
                    .or_default()
                    .push((i, if u < v { 1 } else { -1 }));
            }
        }
        if !volume.is_finite() || volume <= eps.powi(3) {
            return Err(GeometryError::WrongWinding);
        }
        for memberships in edges.values() {
            if memberships.len() != 2 || memberships[0].1 + memberships[1].1 != 0 {
                return Err(GeometryError::NonManifold);
            }
            let a = memberships[0].0;
            let b = memberships[1].0;
            neighbors[a].insert(b);
            neighbors[b].insert(a);
        }
        if nvert
            .checked_add(mesh.triangles.len())
            .and_then(|n| n.checked_sub(edges.len()))
            != Some(2)
        {
            return Err(GeometryError::NonManifold);
        }
        // Closed link around each vertex: one fan, no pinch / bowtie.
        for (vertex, faces) in incident.iter().enumerate() {
            if faces.len() < 3 {
                return Err(GeometryError::NonManifold);
            }
            let mut around = BTreeMap::<usize, BTreeSet<usize>>::new();
            for &face in faces {
                let mut connected = BTreeSet::new();
                for &n in &neighbors[face] {
                    if mesh.triangles[n].contains(&(vertex as u32)) {
                        connected.insert(n);
                    }
                }
                if connected.len() != 2 {
                    return Err(GeometryError::NonManifold);
                }
                around.insert(face, connected);
            }
            let mut seen = BTreeSet::new();
            let mut todo = vec![faces[0]];
            while let Some(f) = todo.pop() {
                if seen.insert(f) {
                    todo.extend(around[&f].iter().copied());
                }
            }
            if seen.len() != faces.len() {
                return Err(GeometryError::NonManifold);
            }
        }
        if incident.iter().any(Vec::is_empty) {
            return Err(GeometryError::NonManifold);
        }
        // Two vertices at effectively identical positions could hide a geometric seam.
        for i in 0..nvert {
            for j in i + 1..nvert {
                if mesh.vertices[i].minus(mesh.vertices[j]).norm() <= eps {
                    return Err(GeometryError::AmbiguousGeometry);
                }
            }
        }
        let mut seen = BTreeSet::new();
        let mut todo = VecDeque::from([0usize]);
        while let Some(i) = todo.pop_front() {
            if seen.insert(i) {
                todo.extend(neighbors[i].iter().copied());
            }
        }
        if seen.len() != mesh.triangles.len() {
            return Err(GeometryError::NonManifold);
        }
        Ok(Self { mesh, diameter })
    }
    pub fn signed_distance(&self, p: Vec3) -> Result<f64, GeometryError> {
        if !p.finite() {
            return Err(GeometryError::InvalidInput);
        }
        let nearest = self.mesh.closest_surface(p)?.unsigned_distance;
        let eps = self.diameter * REL_EPS;
        if nearest <= eps {
            return Ok(0.0);
        }
        let outside = self.mesh.triangles.iter().enumerate().any(|(i, _)| {
            let (a, b, c) = tri(&self.mesh, i);
            let n = b.minus(a).cross(c.minus(a));
            n.dot(p.minus(a)) / n.norm() > eps
        });
        Ok(if outside { nearest } else { -nearest })
    }
    pub fn ray_trace(&self, ray: Ray) -> Result<RayTrace, GeometryError> {
        self.mesh.ray_trace(ray)
    }
}

/// A rigorously scoped non-convex solid created from a *simple*, consistently
/// CCW polygon extruded along Z. This constructor proves validity by geometry
/// generation, NOT by accepting and "certifying" arbitrary imported meshes.
#[derive(Debug, Clone)]
pub struct ExtrudedPrism {
    mesh: MeshAsset,
    polygon: Vec<[f64; 2]>,
    z_min: f64,
    z_max: f64,
    diameter: f64,
}

fn orient2(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn between_points_2d(a: [f64; 2], b: [f64; 2], c: [f64; 2], eps: f64) -> bool {
    // c on or near segment ab, includes endpoints.
    orient2(a, b, c).abs() <= eps
        && c[0] >= a[0].min(b[0]) - eps
        && c[0] <= a[0].max(b[0]) + eps
        && c[1] >= a[1].min(b[1]) - eps
        && c[1] <= a[1].max(b[1]) + eps
}

fn segments_touch_or_cross(a: [f64; 2], b: [f64; 2], c: [f64; 2], d: [f64; 2], eps: f64) -> bool {
    let o1 = orient2(a, b, c);
    let o2 = orient2(a, b, d);
    let o3 = orient2(c, d, a);
    let o4 = orient2(c, d, b);
    // A distant endpoint on the infinite supporting line is not a segment
    // intersection; e.g. the disjoint collinear edges of an L-shaped contour.
    if (o1.abs() <= eps && between_points_2d(a, b, c, eps))
        || (o2.abs() <= eps && between_points_2d(a, b, d, eps))
        || (o3.abs() <= eps && between_points_2d(c, d, a, eps))
        || (o4.abs() <= eps && between_points_2d(c, d, b, eps))
    {
        return true;
    }
    if [o1, o2, o3, o4].iter().any(|v| v.abs() <= eps) {
        return false; // parallel/disjoint edge, or a conservative near-touch excluded above
    }
    (o1.is_sign_positive() != o2.is_sign_positive())
        && (o3.is_sign_positive() != o4.is_sign_positive())
}

fn polygon_contains_xy(poly: &[[f64; 2]], x: f64, y: f64) -> bool {
    let mut inside = false;
    for i in 0..poly.len() {
        let a = poly[i];
        let b = poly[(i + 1) % poly.len()];
        if (a[1] > y) != (b[1] > y) {
            let crossing_x = a[0] + (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]);
            if crossing_x > x {
                inside = !inside;
            }
        }
    }
    inside
}

impl ExtrudedPrism {
    pub fn build(poly: Vec<[f64; 2]>, z_min: f64, z_max: f64) -> Result<Self, GeometryError> {
        const MAX_POLYGON_VERTICES: usize = 64;
        if poly.len() < 3
            || poly.len() > MAX_POLYGON_VERTICES
            || !z_min.is_finite()
            || !z_max.is_finite()
            || z_max <= z_min
            || !poly.iter().flatten().all(|v| v.is_finite())
        {
            return Err(GeometryError::InvalidInput);
        }
        let mut lo = [poly[0][0], poly[0][1], z_min];
        let mut hi = [poly[0][0], poly[0][1], z_max];
        for v in &poly {
            for j in 0..2 {
                lo[j] = lo[j].min(v[j]);
                hi[j] = hi[j].max(v[j]);
            }
        }
        let diameter = Vec3::new(hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2]).norm();
        if !(1e-3..=1e3).contains(&diameter) {
            return Err(GeometryError::AmbiguousGeometry);
        }
        let eps = diameter * 1e-10;
        let eps_area = eps * diameter;
        let n = poly.len();
        let mut area2 = 0.0;
        for i in 0..n {
            let a = poly[i];
            let b = poly[(i + 1) % n];
            area2 += a[0] * b[1] - a[1] * b[0];
            if (b[0] - a[0]).hypot(b[1] - a[1]) < eps {
                return Err(GeometryError::AmbiguousGeometry);
            }
            if orient2(poly[(i + n - 1) % n], a, b).abs() <= eps_area {
                return Err(GeometryError::AmbiguousGeometry);
            }
        }
        if area2 <= eps_area {
            return Err(GeometryError::WrongWinding);
        }
        for i in 0..n {
            for j in (i + 1)..n {
                if j == i + 1 || (i == 0 && j == n - 1) {
                    continue;
                }
                if segments_touch_or_cross(
                    poly[i],
                    poly[(i + 1) % n],
                    poly[j],
                    poly[(j + 1) % n],
                    eps_area,
                ) {
                    return Err(GeometryError::AmbiguousGeometry);
                }
            }
        }
        // Ear clipping on a proven simple CCW polygon.
        let mut loop_ids: Vec<usize> = (0..n).collect();
        let mut caps: Vec<[usize; 3]> = Vec::with_capacity(n - 2);
        while loop_ids.len() > 3 {
            let mut ear = None;
            for k in 0..loop_ids.len() {
                let ia = loop_ids[(k + loop_ids.len() - 1) % loop_ids.len()];
                let ib = loop_ids[k];
                let ic = loop_ids[(k + 1) % loop_ids.len()];
                let a = poly[ia];
                let b = poly[ib];
                let c = poly[ic];
                if orient2(a, b, c) <= eps_area {
                    continue;
                }
                let contains_any = loop_ids.iter().copied().any(|m| {
                    if m == ia || m == ib || m == ic {
                        return false;
                    }
                    let p = poly[m];
                    orient2(a, b, p) >= -eps_area
                        && orient2(b, c, p) >= -eps_area
                        && orient2(c, a, p) >= -eps_area
                });
                if !contains_any {
                    ear = Some((k, [ia, ib, ic]));
                    break;
                }
            }
            let (k, indices) = ear.ok_or(GeometryError::AmbiguousGeometry)?;
            caps.push(indices);
            loop_ids.remove(k);
        }
        caps.push([loop_ids[0], loop_ids[1], loop_ids[2]]);
        let mut vertices = Vec::with_capacity(2 * n);
        for &z in &[z_min, z_max] {
            vertices.extend(poly.iter().map(|p| Vec3::new(p[0], p[1], z)));
        }
        let mut triangles = Vec::with_capacity(4 * n - 4);
        for [a, b, c] in caps {
            triangles.push([c as u32, b as u32, a as u32]);
            triangles.push([(n + a) as u32, (n + b) as u32, (n + c) as u32]);
        }
        for i in 0..n {
            let j = (i + 1) % n;
            triangles.push([i as u32, j as u32, (n + j) as u32]);
            triangles.push([i as u32, (n + j) as u32, (n + i) as u32]);
        }
        let mesh = MeshAsset {
            vertices,
            triangles,
        };
        if mesh.preflight()? != crate::MeshPreflight::ClosedCandidateNotCertified {
            return Err(GeometryError::NonManifold);
        }
        Ok(Self {
            mesh,
            polygon: poly,
            z_min,
            z_max,
            diameter,
        })
    }
    pub fn mesh(&self) -> &MeshAsset {
        &self.mesh
    }
    pub fn signed_distance(&self, p: Vec3) -> Result<f64, GeometryError> {
        if !p.finite() {
            return Err(GeometryError::InvalidInput);
        }
        let nearest = self.mesh.closest_surface(p)?.unsigned_distance;
        if nearest <= self.diameter * REL_EPS {
            return Ok(0.0);
        }
        let inside =
            p.z > self.z_min && p.z < self.z_max && polygon_contains_xy(&self.polygon, p.x, p.y);
        Ok(if inside { -nearest } else { nearest })
    }
    pub fn ray_trace(&self, ray: Ray) -> Result<RayTrace, GeometryError> {
        self.mesh.ray_trace(ray)
    }
}

/// No caller may make an arbitrary imported surface a signed volumetric mesh.
/// Both variants require a checked constructor.
#[derive(Debug, Clone)]
pub enum CertifiedMeshSolid {
    Convex(CertifiedConvexSolid),
    Extruded(ExtrudedPrism),
}
impl CertifiedMeshSolid {
    pub fn signed_distance(&self, p: Vec3) -> Result<f64, GeometryError> {
        match self {
            Self::Convex(s) => s.signed_distance(p),
            Self::Extruded(s) => s.signed_distance(p),
        }
    }
    pub fn mesh(&self) -> &MeshAsset {
        match self {
            Self::Convex(s) => s.mesh(),
            Self::Extruded(s) => s.mesh(),
        }
    }
}
