//! BLOB-NEXT-01 CP01 GEO-02: bounded 3D geometry + CPU SDF/mesh CSG; NOT qualified CP01.
//! Generic imported meshes are NOT certified. Mixed CPU SDF/MeshSolid booleans operate only on sealed convex meshes or bounded generated non-convex extrusions; the GPU path is not implemented.
pub mod camera;
pub mod gltf;
pub mod mesh;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub const MAX_NODES: usize = 256;
pub const MAX_DEPTH: usize = 64;
pub const MAX_TRIANGLES: usize = 65_536;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl Vec3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
    pub fn finite(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }
    pub fn dot(self, b: Self) -> f64 {
        self.x * b.x + self.y * b.y + self.z * b.z
    }
    pub fn norm(self) -> f64 {
        self.dot(self).sqrt()
    }
    pub fn minus(self, b: Self) -> Self {
        Self::new(self.x - b.x, self.y - b.y, self.z - b.z)
    }
    pub fn plus(self, b: Self) -> Self {
        Self::new(self.x + b.x, self.y + b.y, self.z + b.z)
    }
    pub fn scale(self, k: f64) -> Self {
        Self::new(self.x * k, self.y * k, self.z * k)
    }
    pub fn cross(self, b: Self) -> Self {
        Self::new(
            self.y * b.z - self.z * b.y,
            self.z * b.x - self.x * b.z,
            self.x * b.y - self.y * b.x,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Quat {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}
impl Quat {
    pub const IDENTITY: Self = Self {
        w: 1.,
        x: 0.,
        y: 0.,
        z: 0.,
    };
    fn norm2(self) -> f64 {
        self.w * self.w + self.x * self.x + self.y * self.y + self.z * self.z
    }
    pub fn valid(self) -> bool {
        [self.w, self.x, self.y, self.z]
            .iter()
            .all(|v| v.is_finite())
            && (self.norm2() - 1.).abs() < 1e-9
    }
    pub fn conjugate_rotate(self, v: Vec3) -> Vec3 {
        let q = Vec3::new(-self.x, -self.y, -self.z);
        let t = q.cross(v).scale(2.);
        v.plus(t.scale(self.w)).plus(q.cross(t))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: f64,
}
impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::new(0., 0., 0.),
        rotation: Quat::IDENTITY,
        scale: 1.,
    };
    pub fn valid(self) -> bool {
        self.translation.finite()
            && self.rotation.valid()
            && self.scale.is_finite()
            && self.scale > 0.
    }
    pub fn inverse_point(self, p: Vec3) -> Vec3 {
        self.rotation
            .conjugate_rotate(p.minus(self.translation))
            .scale(1. / self.scale)
    }
}

#[derive(Debug, Clone)]
pub enum Primitive {
    Sphere { radius: f64 },
    Box { half_extents: Vec3 },
    Capsule { a: Vec3, b: Vec3, radius: f64 },
}
impl Primitive {
    fn valid(&self) -> bool {
        match self {
            Self::Sphere { radius } => radius.is_finite() && *radius > 0.,
            Self::Box { half_extents: h } => h.finite() && h.x > 0. && h.y > 0. && h.z > 0.,
            Self::Capsule { a, b, radius: r } => {
                a.finite()
                    && b.finite()
                    && r.is_finite()
                    && *r > 0.
                    && b.minus(*a).dot(b.minus(*a)) > 1e-16
            }
        }
    }
    fn distance(&self, p: Vec3) -> f64 {
        match self {
            Self::Sphere { radius: r } => p.norm() - r,
            Self::Box { half_extents: h } => {
                let q = Vec3::new(p.x.abs() - h.x, p.y.abs() - h.y, p.z.abs() - h.z);
                let outside = Vec3::new(q.x.max(0.), q.y.max(0.), q.z.max(0.)).norm();
                outside + q.x.max(q.y).max(q.z).min(0.)
            }
            Self::Capsule { a, b, radius: r } => {
                let seg = b.minus(*a);
                let t = (p.minus(*a).dot(seg) / seg.dot(seg)).clamp(0., 1.);
                p.minus(a.plus(seg.scale(t))).norm() - r
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Semantic {
    ExactDistance,
    FieldEstimate,
}
#[derive(Debug, Clone, Copy)]
pub struct FieldSample {
    pub distance: f64,
    pub semantic: Semantic,
}
#[derive(Debug, Clone, Copy)]
pub enum CsgOp {
    Union,
    Intersection,
    Difference,
    SmoothUnion { k: f64 },
}
#[derive(Debug, Clone)]
pub struct SdfLeaf {
    pub shape: Primitive,
    pub transform: Transform,
}
#[derive(Debug, Clone)]
pub struct MeshAsset {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[u32; 3]>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshPreflight {
    SurfaceOnly,
    ClosedCandidateNotCertified,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeometryError {
    InvalidInput,
    TooManyNodes,
    GraphCycle,
    GraphTooDeep,
    DuplicateId,
    MissingNode,
    SurfaceHasNoSignedField,
    InvalidMesh,
    TooManyTriangles,
    NonFiniteResult,
    NonManifold,
    WrongWinding,
    AmbiguousGeometry,
    UnsupportedNonConvex,
}
impl MeshAsset {
    /// Preflight is deliberately NOT a solid certification: lacks self-intersection
    /// proof, signed winding and robust inside/outside oracle.
    pub fn preflight(&self) -> Result<MeshPreflight, GeometryError> {
        if self.vertices.len() < 3
            || self.triangles.is_empty()
            || self.triangles.len() > MAX_TRIANGLES
            || !self.vertices.iter().all(|v| v.finite())
        {
            return Err(GeometryError::InvalidMesh);
        }
        let mut edges: HashMap<(u32, u32), (u32, i32)> = HashMap::new();
        for triangle in &self.triangles {
            let [a, b, c] = *triangle;
            if [a, b, c].iter().any(|&v| v as usize >= self.vertices.len())
                || a == b
                || b == c
                || a == c
            {
                return Err(GeometryError::InvalidMesh);
            }
            let ab = self.vertices[b as usize].minus(self.vertices[a as usize]);
            let ac = self.vertices[c as usize].minus(self.vertices[a as usize]);
            if ab.cross(ac).norm() <= 1e-12 {
                return Err(GeometryError::InvalidMesh);
            }
            for (i, j) in [(a, b), (b, c), (c, a)] {
                let key = (i.min(j), i.max(j));
                let entry = edges.entry(key).or_insert((0, 0));
                entry.0 += 1;
                entry.1 += if i < j { 1 } else { -1 };
            }
        }
        if edges
            .values()
            .all(|(count, orient)| *count == 2 && *orient == 0)
        {
            Ok(MeshPreflight::ClosedCandidateNotCertified)
        } else {
            Ok(MeshPreflight::SurfaceOnly)
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u64);
#[derive(Debug, Clone)]
pub enum NodeKind {
    Sdf(SdfLeaf),
    MeshSurface(MeshAsset),
    MeshSolid(mesh::CertifiedMeshSolid),
    /// Strictly convex certified mesh; arbitrary mesh certification remains unavailable.
    ConvexSolid(mesh::CertifiedConvexSolid),
    Csg {
        op: CsgOp,
        left: NodeId,
        right: NodeId,
    },
}
#[derive(Debug, Clone)]
pub struct SceneDocument {
    pub version: u32,
    pub nodes: BTreeMap<NodeId, NodeKind>,
    pub roots: Vec<NodeId>,
}
impl SceneDocument {
    /// A construction API that rejects duplicate IDs instead of overwriting an entry.
    pub fn insert(&mut self, id: NodeId, kind: NodeKind) -> Result<(), GeometryError> {
        if id.0 == 0 {
            return Err(GeometryError::InvalidInput);
        }
        if self.nodes.contains_key(&id) {
            return Err(GeometryError::DuplicateId);
        }
        if self.nodes.len() >= MAX_NODES {
            return Err(GeometryError::TooManyNodes);
        }
        self.nodes.insert(id, kind);
        Ok(())
    }
    pub fn validate(&self) -> Result<(), GeometryError> {
        if self.version != 1 {
            return Err(GeometryError::InvalidInput);
        }
        if self.nodes.len() > MAX_NODES {
            return Err(GeometryError::TooManyNodes);
        }
        let mut depths = BTreeMap::new();
        for &root in &self.roots {
            self.depth(root, &mut BTreeSet::new(), &mut depths)?;
        }
        for (&id, kind) in &self.nodes {
            if id.0 == 0 {
                return Err(GeometryError::InvalidInput);
            }
            if let NodeKind::Sdf(leaf) = kind {
                if !leaf.shape.valid() || !leaf.transform.valid() {
                    return Err(GeometryError::InvalidInput);
                }
            }
            if let NodeKind::MeshSurface(mesh) = kind {
                mesh.preflight()?;
            }
            if let NodeKind::Csg {
                op: CsgOp::SmoothUnion { k },
                ..
            } = kind
            {
                if !k.is_finite() || *k <= 0. {
                    return Err(GeometryError::InvalidInput);
                }
            }
            self.depth(id, &mut BTreeSet::new(), &mut depths)?;
        }
        Ok(())
    }
    /// Memoized DAG height: shared subtrees cannot grow evaluation exponentially.
    fn depth(
        &self,
        id: NodeId,
        visiting: &mut BTreeSet<NodeId>,
        depths: &mut BTreeMap<NodeId, usize>,
    ) -> Result<usize, GeometryError> {
        if visiting.contains(&id) {
            return Err(GeometryError::GraphCycle);
        }
        if let Some(&height) = depths.get(&id) {
            return Ok(height);
        }
        let kind = self.nodes.get(&id).ok_or(GeometryError::MissingNode)?;
        visiting.insert(id);
        let height = match kind {
            NodeKind::Csg { left, right, .. } => {
                1 + self
                    .depth(*left, visiting, depths)?
                    .max(self.depth(*right, visiting, depths)?)
            }
            _ => 1,
        };
        visiting.remove(&id);
        if height > MAX_DEPTH {
            return Err(GeometryError::GraphTooDeep);
        }
        depths.insert(id, height);
        Ok(height)
    }
    pub fn sample(&self, root: NodeId, p: Vec3) -> Result<FieldSample, GeometryError> {
        if !p.finite() {
            return Err(GeometryError::InvalidInput);
        }
        self.validate()?;
        self.eval(root, p, &mut BTreeMap::new())
    }
    fn eval(
        &self,
        id: NodeId,
        p: Vec3,
        memo: &mut BTreeMap<NodeId, FieldSample>,
    ) -> Result<FieldSample, GeometryError> {
        if let Some(&cached) = memo.get(&id) {
            return Ok(cached);
        }
        let kind = self.nodes.get(&id).ok_or(GeometryError::MissingNode)?;
        let sample = match kind {
            NodeKind::Sdf(s) => FieldSample {
                distance: s.shape.distance(s.transform.inverse_point(p)) * s.transform.scale,
                semantic: Semantic::ExactDistance,
            },
            NodeKind::MeshSurface(_) => return Err(GeometryError::SurfaceHasNoSignedField),
            NodeKind::MeshSolid(s) => FieldSample {
                distance: s.signed_distance(p)?,
                semantic: Semantic::ExactDistance,
            },
            NodeKind::ConvexSolid(s) => FieldSample {
                distance: s.signed_distance(p)?,
                semantic: Semantic::ExactDistance,
            },
            NodeKind::Csg { op, left, right } => {
                let a = self.eval(*left, p, memo)?.distance;
                let b = self.eval(*right, p, memo)?.distance;
                let distance = match op {
                    CsgOp::Union => a.min(b),
                    CsgOp::Intersection => a.max(b),
                    CsgOp::Difference => a.max(-b),
                    CsgOp::SmoothUnion { k } => {
                        let h = (0.5 + 0.5 * (b - a) / k).clamp(0., 1.);
                        b * (1. - h) + a * h - k * h * (1. - h)
                    }
                };
                FieldSample {
                    distance,
                    semantic: Semantic::FieldEstimate,
                }
            }
        };
        if sample.distance.is_finite() {
            memo.insert(id, sample);
            Ok(sample)
        } else {
            Err(GeometryError::NonFiniteResult)
        }
    }
}
