//! CUDA-02 private unified CSG provider. Authority remains on the Rust/F2 side.
//! Three *compute* implementations receive identical validated immutable primitives.
//! This does not extend F2's semantic topology grammar to arbitrary CSG.
pub mod cuda;
pub mod vulkan;

use blobin_f2_topology::Scene;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Primitive {
    pub x: f32,
    pub y: f32,
    pub radius: f32,
    pub op: u32,
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Sample {
    pub distance: f32,
    pub nx: f32,
    pub ny: f32,
    pub coverage: f32,
}
const _: () = {
    assert!(std::mem::size_of::<Primitive>() == 16);
    assert!(std::mem::size_of::<Sample>() == 16);
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneAuthority {
    pub epoch: u64,
    pub left_id: u64,
    pub right_id: u64,
    pub semantic_digest: u64,
}
impl From<Scene> for SceneAuthority {
    fn from(s: Scene) -> Self {
        Self {
            epoch: s.epoch,
            left_id: s.left_id,
            right_id: s.right_id,
            semantic_digest: s.digest(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Request {
    authority: SceneAuthority,
    width: u32,
    height: u32,
    primitives: Vec<Primitive>,
}
impl Request {
    /// F2-derived provenance metadata, not a cryptographic execution capability.
    pub fn authority(&self) -> SceneAuthority {
        self.authority
    }
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    pub fn primitives(&self) -> &[Primitive] {
        &self.primitives
    }
    pub fn from_f2(s: Scene, primitives: Vec<Primitive>) -> Result<Self, String> {
        s.validate()?;
        let value = Self {
            authority: s.into(),
            width: s.width(),
            height: s.height(),
            primitives,
        };
        value.validate()?;
        Ok(value)
    }
    pub fn validate(&self) -> Result<(), String> {
        let pixels = (self.width as usize)
            .checked_mul(self.height as usize)
            .ok_or("CUDA02 PIXEL OVERFLOW")?;
        if self.width < 16
            || self.height < 16
            || self.width > 2048
            || self.height > 2048
            || pixels > 1_048_576
            || self.primitives.is_empty()
            || self.primitives.len() > 256
        {
            return Err("CUDA02 SCENE BOUNDS".into());
        }
        for p in &self.primitives {
            if !p.x.is_finite()
                || !p.y.is_finite()
                || !p.radius.is_finite()
                || p.radius <= 0.0
                || p.op > 3
            {
                return Err("CUDA02 INVALID PRIMITIVE".into());
            }
        }
        if self.primitives[0].op != 0 {
            return Err("CUDA02 FIRST PRIMITIVE MUST BE UNION".into());
        }
        Ok(())
    }
}
pub fn generate_scene(count: usize) -> Vec<Primitive> {
    let mut out = Vec::with_capacity(count);
    out.push(Primitive {
        x: 0.0,
        y: 0.0,
        radius: 0.245,
        op: 0,
    });
    for i in 1..count {
        let theta = i as f32 * 2.399_963_1_f32;
        let ring = 0.04 + (i % 13) as f32 * 0.024;
        let (x, y) = (ring * theta.cos(), ring * theta.sin());
        let op = if i % 17 == 0 {
            2
        } else if i % 13 == 0 {
            1
        } else if i % 4 == 0 {
            3
        } else {
            0
        };
        let radius = if op == 2 {
            0.68
        } else if op == 1 {
            0.034
        } else {
            0.035 + 0.004 * (i % 7) as f32
        };
        out.push(Primitive { x, y, radius, op });
    }
    out
}
fn circle(x: f32, y: f32, p: Primitive) -> f32 {
    let dx = x - p.x;
    let dy = y - p.y;
    (dx * dx + dy * dy).sqrt() - p.radius
}
fn signed_field(x: f32, y: f32, primitives: &[Primitive]) -> f32 {
    let mut d = circle(x, y, primitives[0]);
    for p in &primitives[1..] {
        let v = circle(x, y, *p);
        d = match p.op {
            1 => d.max(-v),
            2 => d.max(v),
            3 => {
                let k = 0.08_f32;
                let h = (k - (d - v).abs()).max(0.0) / k;
                d.min(v) - h * h * k * 0.25
            }
            _ => d.min(v),
        };
    }
    d
}
fn evaluate_pixel(ix: u32, iy: u32, req: &Request) -> Sample {
    let scale = req.width.min(req.height) as f32;
    let x = (ix as f32 + 0.5 - req.width as f32 * 0.5) / scale;
    let y = (iy as f32 + 0.5 - req.height as f32 * 0.5) / scale;
    let epsilon = 1.0 / scale;
    let p = &req.primitives;
    let distance = signed_field(x, y, p);
    let dx = (signed_field(x + epsilon, y, p) - signed_field(x - epsilon, y, p)) * 0.5;
    let dy = (signed_field(x, y + epsilon, p) - signed_field(x, y - epsilon, p)) * 0.5;
    let mag = (dx * dx + dy * dy).sqrt();
    let (nx, ny) = if mag > 1e-8 {
        (dx / mag, dy / mag)
    } else {
        (0.0, 0.0)
    };
    let coverage = (0.5 - distance / (1.2 / scale)).clamp(0.0, 1.0);
    Sample {
        distance,
        nx,
        ny,
        coverage,
    }
}
pub fn cpu(req: &Request) -> Result<Vec<Sample>, String> {
    req.validate()?;
    let mut out = Vec::with_capacity(req.width as usize * req.height as usize);
    for iy in 0..req.height {
        for ix in 0..req.width {
            out.push(evaluate_pixel(ix, iy, req));
        }
    }
    Ok(out)
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Backend {
    Cpu,
    Vulkan,
    Cuda,
    Auto,
}
#[derive(Debug)]
pub struct QualifiedFrame {
    pub authority: SceneAuthority,
    pub backend: Backend,
    pub samples: Vec<Sample>,
}
pub fn evaluate(
    req: &Request,
    backend: Backend,
    cuda_library: Option<&std::path::Path>,
) -> Result<QualifiedFrame, String> {
    req.validate()?;
    let (selected, samples) = match backend {
        Backend::Cpu => (Backend::Cpu, cpu(req)?),
        Backend::Vulkan => (Backend::Vulkan, vulkan::evaluate(req)?),
        Backend::Cuda => {
            let library = cuda_library.ok_or("CUDA02 CUDA LIBRARY REQUIRED")?;
            (Backend::Cuda, cuda::evaluate(req, library)?)
        }
        Backend::Auto => {
            if let Some(path) = cuda_library {
                if let Ok(out) = cuda::evaluate(req, path) {
                    return Ok(QualifiedFrame {
                        authority: req.authority,
                        backend: Backend::Cuda,
                        samples: out,
                    });
                }
            }
            match vulkan::evaluate(req) {
                Ok(out) => (Backend::Vulkan, out),
                Err(_) => (Backend::Cpu, cpu(req)?),
            }
        }
    };
    // Provider cannot mint identities; the Rust orchestrator attaches the original input authority.
    Ok(QualifiedFrame {
        authority: req.authority,
        backend: selected,
        samples,
    })
}
#[derive(Clone, Copy, Debug, Default)]
pub struct ErrorMetrics {
    pub max_distance: f32,
    pub max_normal: f32,
    pub max_coverage: f32,
    pub occupancy_mismatch: u32,
    pub near_boundary_mismatch: u32,
}
pub fn compare(reference: &[Sample], observed: &[Sample]) -> Result<ErrorMetrics, String> {
    if reference.len() != observed.len() {
        return Err("CUDA02 SAMPLE COUNT".into());
    }
    let mut m = ErrorMetrics::default();
    for (a, b) in reference.iter().zip(observed) {
        for v in [
            a.distance, a.nx, a.ny, a.coverage, b.distance, b.nx, b.ny, b.coverage,
        ] {
            if !v.is_finite() {
                return Err("CUDA02 NONFINITE SAMPLE".into());
            }
        }
        m.max_distance = m.max_distance.max((a.distance - b.distance).abs());
        m.max_normal = m
            .max_normal
            .max((a.nx - b.nx).abs().max((a.ny - b.ny).abs()));
        m.max_coverage = m.max_coverage.max((a.coverage - b.coverage).abs());
        if (a.distance < 0.0) != (b.distance < 0.0) {
            if a.distance.abs() <= 1e-4 {
                m.near_boundary_mismatch += 1;
            } else {
                m.occupancy_mismatch += 1;
            }
        }
    }
    if m.occupancy_mismatch != 0
        || m.near_boundary_mismatch != 0
        || m.max_distance > 1e-4
        || m.max_normal > 0.006
        || m.max_coverage > 0.007
    {
        return Err(format!("CUDA02 PARITY_RED {m:?}"));
    }
    Ok(m)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn f2_authority_is_passthrough_not_geometry_owner() {
        let scene = Scene::default();
        let req = Request::from_f2(scene, generate_scene(16)).unwrap();
        let frame = evaluate(&req, Backend::Cpu, None).unwrap();
        assert_eq!(frame.authority, req.authority);
        assert_eq!(frame.authority.semantic_digest, scene.digest());
        assert_eq!(frame.authority.left_id, scene.left_id);
        assert_eq!(frame.authority.right_id, scene.right_id);
    }
    #[test]
    fn invalid_primitives_fail_closed() {
        let scene = Scene::default();
        let mut req = Request::from_f2(scene, generate_scene(16)).unwrap();
        req.primitives[1].op = 44;
        assert!(req.validate().is_err());
        req.primitives[1].op = 0;
        req.primitives[0].radius = f32::NAN;
        assert!(req.validate().is_err());
        req.primitives[0].radius = 0.245;
        req.primitives.clear();
        assert!(req.validate().is_err());
    }
    #[test]
    fn cpu_deterministic_finite_and_falsifiable() {
        let req = Request::from_f2(Scene::default(), generate_scene(16)).unwrap();
        let a = cpu(&req).unwrap();
        let b = cpu(&req).unwrap();
        assert_eq!(a.len(), b.len());
        assert!(compare(&a, &b).is_ok());
        let mut altered = a.clone();
        altered[0].distance += 0.5;
        assert!(compare(&a, &altered).is_err());
    }
    #[test]
    fn cuda_missing_is_error_not_fabricated_success() {
        let req = Request::from_f2(Scene::default(), generate_scene(16)).unwrap();
        assert!(evaluate(&req, Backend::Cuda, None).is_err());
    }
    #[test]
    fn every_f2_epoch_preserves_source_provenance() {
        for scene in blobin_f2_topology::qualification_trace().unwrap() {
            let req = Request::from_f2(scene, generate_scene(16)).unwrap();
            let result = evaluate(&req, Backend::Cpu, None).unwrap();
            assert_eq!(result.authority, req.authority());
            assert_eq!(result.authority.semantic_digest, scene.digest());
            assert_eq!(result.authority.epoch, scene.epoch);
            assert_eq!(result.authority.left_id, scene.left_id);
            assert_eq!(result.authority.right_id, scene.right_id);
        }
    }
    #[test]
    fn source_shader_naga_validates() {
        let m = naga::front::wgsl::parse_str(include_str!("../shaders/geometry.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&m)
        .unwrap();
    }
}
