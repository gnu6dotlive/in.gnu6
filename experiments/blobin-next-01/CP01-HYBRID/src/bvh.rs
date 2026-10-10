//! GEO-04: CPU BVH acceleration whose numerical authority is the unaccelerated
//! mesh reference queries. Not a certification of arbitrary imported solids.
use crate::{
    mesh::{closest_point_triangle, ClosestSurface, Ray, RayHit, RayTrace},
    GeometryError, MeshAsset, Vec3,
};

const LEAF_TRIANGLES: usize = 6;
#[derive(Clone, Copy, Debug)]
struct Bounds {
    lo: Vec3,
    hi: Vec3,
}
impl Bounds {
    fn from_point(p: Vec3) -> Self {
        Self { lo: p, hi: p }
    }
    fn add(&mut self, p: Vec3) {
        self.lo = Vec3::new(self.lo.x.min(p.x), self.lo.y.min(p.y), self.lo.z.min(p.z));
        self.hi = Vec3::new(self.hi.x.max(p.x), self.hi.y.max(p.y), self.hi.z.max(p.z));
    }
    fn merge(&mut self, other: Self) {
        self.add(other.lo);
        self.add(other.hi);
    }
    fn nearest_sq(&self, p: Vec3) -> f64 {
        let x = (self.lo.x - p.x).max(0.0).max(p.x - self.hi.x);
        let y = (self.lo.y - p.y).max(0.0).max(p.y - self.hi.y);
        let z = (self.lo.z - p.z).max(0.0).max(p.z - self.hi.z);
        x * x + y * y + z * z
    }
    fn ray_intersects(&self, r: Ray) -> bool {
        // Conservative slab test, keeping edges/corners potentially ambiguous.
        let mut near = 0.0f64;
        let mut far = f64::INFINITY;
        for (l, h, o, d) in [
            (self.lo.x, self.hi.x, r.origin.x, r.direction.x),
            (self.lo.y, self.hi.y, r.origin.y, r.direction.y),
            (self.lo.z, self.hi.z, r.origin.z, r.direction.z),
        ] {
            let eps = 1e-10 * (h - l).abs().max(1.0);
            let l = l - eps;
            let h = h + eps;
            if d.abs() < 1e-15 {
                if o < l || o > h {
                    return false;
                }
            } else {
                let a = (l - o) / d;
                let b = (h - o) / d;
                near = near.max(a.min(b));
                far = far.min(a.max(b));
                if far < near {
                    return false;
                }
            }
        }
        far > 0.0
    }
}
#[derive(Clone, Debug)]
struct BvhNode {
    bounds: Bounds,
    left: Option<usize>,
    right: Option<usize>,
    tris: Vec<usize>,
}
pub struct TriangleBvh {
    mesh: MeshAsset,
    nodes: Vec<BvhNode>,
    root: usize,
}
impl TriangleBvh {
    pub fn build(mesh: MeshAsset) -> Result<Self, GeometryError> {
        mesh.preflight()?;
        let mut bvh = Self {
            mesh,
            nodes: Vec::new(),
            root: 0,
        };
        let indices: Vec<usize> = (0..bvh.mesh.triangles.len()).collect();
        bvh.root = bvh.build_node(indices);
        Ok(bvh)
    }
    fn tri(&self, id: usize) -> (Vec3, Vec3, Vec3) {
        let [a, b, c] = self.mesh.triangles[id];
        (
            self.mesh.vertices[a as usize],
            self.mesh.vertices[b as usize],
            self.mesh.vertices[c as usize],
        )
    }
    fn tri_bounds(&self, id: usize) -> Bounds {
        let (a, b, c) = self.tri(id);
        let mut bounds = Bounds::from_point(a);
        bounds.add(b);
        bounds.add(c);
        bounds
    }
    fn centroid(&self, id: usize) -> Vec3 {
        let (a, b, c) = self.tri(id);
        a.plus(b).plus(c).scale(1.0 / 3.0)
    }
    fn build_node(&mut self, mut tris: Vec<usize>) -> usize {
        let mut bounds = self.tri_bounds(tris[0]);
        for &t in &tris[1..] {
            bounds.merge(self.tri_bounds(t));
        }
        let me = self.nodes.len();
        self.nodes.push(BvhNode {
            bounds,
            left: None,
            right: None,
            tris: Vec::new(),
        });
        if tris.len() <= LEAF_TRIANGLES {
            self.nodes[me].tris = tris;
            return me;
        }
        let extent = bounds.hi.minus(bounds.lo);
        let axis = if extent.x >= extent.y && extent.x >= extent.z {
            0
        } else if extent.y >= extent.z {
            1
        } else {
            2
        };
        tris.sort_by(|&a, &b| {
            let x = self.centroid(a);
            let y = self.centroid(b);
            let ka = [x.x, x.y, x.z][axis];
            let kb = [y.x, y.y, y.z][axis];
            ka.total_cmp(&kb).then(a.cmp(&b))
        });
        let right = tris.split_off(tris.len() / 2);
        let left = self.build_node(tris);
        let right = self.build_node(right);
        self.nodes[me].left = Some(left);
        self.nodes[me].right = Some(right);
        me
    }
    pub fn triangle_count(&self) -> usize {
        self.mesh.triangles.len()
    }
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }
    pub fn closest_surface(&self, p: Vec3) -> Result<ClosestSurface, GeometryError> {
        if !p.finite() {
            return Err(GeometryError::InvalidInput);
        }
        let mut best: Option<ClosestSurface> = None;
        let mut stack = vec![self.root];
        while let Some(index) = stack.pop() {
            let node = &self.nodes[index];
            // Include ties: same-distance triangles must choose lowest original index.
            if best.is_some_and(|old| {
                node.bounds.nearest_sq(p) > old.unsigned_distance.powi(2) + 1e-13
            }) {
                continue;
            }
            for &i in &node.tris {
                let (a, b, c) = self.tri(i);
                let surface = closest_point_triangle(p, a, b, c);
                let distance = p.minus(surface).norm();
                if !distance.is_finite() {
                    return Err(GeometryError::NonFiniteResult);
                }
                if best.is_none_or(|old| {
                    distance < old.unsigned_distance
                        || (distance == old.unsigned_distance && i < old.triangle_index)
                }) {
                    best = Some(ClosestSurface {
                        triangle_index: i,
                        unsigned_distance: distance,
                        position: surface,
                    });
                }
            }
            if let Some(left) = node.left {
                stack.push(left);
            }
            if let Some(right) = node.right {
                stack.push(right);
            }
        }
        best.ok_or(GeometryError::InvalidMesh)
    }
    pub fn ray_trace(&self, ray: Ray) -> Result<RayTrace, GeometryError> {
        if !ray.origin.finite()
            || !ray.direction.finite()
            || (ray.direction.norm() - 1.0).abs() > 1e-9
        {
            return Err(GeometryError::InvalidInput);
        }
        let mut stack = vec![self.root];
        let mut hits: Vec<(RayHit, bool)> = Vec::new();
        while let Some(index) = stack.pop() {
            let node = &self.nodes[index];
            if !node.bounds.ray_intersects(ray) {
                continue;
            }
            for &i in &node.tris {
                let (a, b, c) = self.tri(i);
                let e1 = b.minus(a);
                let e2 = c.minus(a);
                let n = e1.cross(e2);
                let det = e1.dot(ray.direction.cross(e2));
                if det.abs() <= 1e-12 * n.norm() {
                    continue;
                }
                let inv = 1.0 / det;
                let tvec = ray.origin.minus(a);
                let u = tvec.dot(ray.direction.cross(e2)) * inv;
                let v = ray.direction.dot(tvec.cross(e1)) * inv;
                let w = 1.0 - u - v;
                let eps = 1e-10;
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
                let edge = u <= eps || v <= eps || w <= eps;
                hits.push((
                    RayHit {
                        triangle_index: i,
                        t,
                        barycentric: [w, u, v],
                    },
                    edge,
                ));
            }
            if let Some(left) = node.left {
                stack.push(left);
            }
            if let Some(right) = node.right {
                stack.push(right);
            }
        }
        hits.sort_by(|a, b| {
            a.0.t
                .total_cmp(&b.0.t)
                .then(a.0.triangle_index.cmp(&b.0.triangle_index))
        });
        match hits.first().copied() {
            None => Ok(RayTrace::Miss),
            Some((first, on_edge)) => {
                let epsilon = 1e-8 * first.t.max(1.0);
                let count = hits
                    .iter()
                    .take_while(|x| (x.0.t - first.t).abs() <= epsilon)
                    .count();
                if on_edge || count > 1 {
                    Ok(RayTrace::AmbiguousSharedEdge { t: first.t, count })
                } else {
                    Ok(RayTrace::Hit(first))
                }
            }
        }
    }
}
