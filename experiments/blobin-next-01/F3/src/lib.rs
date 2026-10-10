//! Private original F3 material/height engine. No historical blob.in source imported.
//! Geometry and scene identity are owned by F2; backend output is projection only.
pub mod gpu;
use blobin_f2_topology::{Layout, Scene};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Material {
    Ceramic,
    Gel,
    BrushedMetal,
    Paper,
}
impl Material {
    pub const ALL: [Self; 4] = [Self::Ceramic, Self::Gel, Self::BrushedMetal, Self::Paper];
    pub fn code(self) -> u32 {
        match self {
            Self::Ceramic => 0,
            Self::Gel => 1,
            Self::BrushedMetal => 2,
            Self::Paper => 3,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Ceramic => "ceramic",
            Self::Gel => "gel",
            Self::BrushedMetal => "metal",
            Self::Paper => "paper",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Params {
    pub material: Material,
    pub heat_strength: f32,
}
impl Default for Params {
    fn default() -> Self {
        Self {
            material: Material::Ceramic,
            heat_strength: 0.62,
        }
    }
}
impl Params {
    pub fn validate(self) -> Result<(), String> {
        if !self.heat_strength.is_finite() || !(0.0..=1.0).contains(&self.heat_strength) {
            return Err("F3 INVALID_HEAT_STRENGTH".into());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Pixel {
    pub rgba: [u8; 4],
    pub mask: u32,
    pub height: f32,
    pub heat: f32,
    pub normal_z: f32,
}
pub struct CpuFrame {
    pub rgba: Vec<u8>,
    pub mask: Vec<u32>,
    pub height: Vec<f32>,
    pub heat: Vec<f32>,
    pub normal_z: Vec<f32>,
    pub semantic_digest: u64,
}
fn coords(scene: Scene, fx: f32, fy: f32) -> (f32, f32) {
    let scale = scene.logical_width.min(scene.logical_height) as f32;
    (
        (fx / scene.dpr as f32 - scene.logical_width as f32 * 0.5) / scale,
        (fy / scene.dpr as f32 - scene.logical_height as f32 * 0.5) / scale,
    )
}
fn field(scene: Scene, fx: f32, fy: f32) -> f32 {
    let (x, y) = coords(scene, fx, fy);
    let sep = if scene.layout == Layout::Split {
        0.31
    } else {
        0.15
    };
    let lobe = |cx: f32| {
        let px = x - cx;
        let theta = y.atan2(px);
        let r = 0.235
            + 0.008 * (3.0 * theta + scene.phase).sin()
            + 0.006 * (5.0 * theta - scene.phase).cos();
        (px * px + y * y).sqrt() - r
    };
    let combined = lobe(-sep).min(lobe(sep));
    if scene.layout == Layout::Hollow {
        combined.max(0.055 - (x * x + y * y).sqrt())
    } else {
        combined
    }
}
fn height_at(scene: Scene, fx: f32, fy: f32) -> f32 {
    let d = field(scene, fx, fy);
    let aa = 1.2 / (scene.logical_width.min(scene.logical_height) * scene.dpr) as f32;
    let coverage = (0.5 - d / aa).clamp(0.0, 1.0);
    let rim = (1.0 - d.abs() / 0.045).clamp(0.0, 1.0);
    (0.11 * coverage + 0.24 * rim * coverage).clamp(0.0, 0.35)
}
fn heat_at(scene: Scene, fx: f32, fy: f32, strength: f32) -> f32 {
    let (x, y) = coords(scene, fx, fy);
    let a = (-((x + 0.16).powi(2) + (y + 0.09).powi(2)) * 46.0).exp();
    let b = (-((x - 0.19).powi(2) + (y - 0.07).powi(2)) * 68.0).exp();
    (strength * (0.78 * a + 0.48 * b)).clamp(0.0, 1.0)
}
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let n = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2])
        .sqrt()
        .max(0.00001);
    [v[0] / n, v[1] / n, v[2] / n]
}
pub fn srgb_encode(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.0031308 {
        12.92 * v
    } else {
        1.055 * v.powf(1.0 / 2.4) - 0.055
    }
}
pub fn linear_decode(v: f32) -> f32 {
    let v = v.clamp(0.0, 1.0);
    if v <= 0.04045 {
        v / 12.92
    } else {
        ((v + 0.055) / 1.055).powf(2.4)
    }
}
fn over(fg: [f32; 4], bg: [f32; 4]) -> [f32; 4] {
    // Each source is straight linear RGB; returned RGB is premultiplied exactly once.
    let fa = fg[3].clamp(0.0, 1.0);
    let ba = bg[3].clamp(0.0, 1.0);
    let alpha = fa + ba * (1.0 - fa);
    [
        fg[0] * fa + bg[0] * ba * (1.0 - fa),
        fg[1] * fa + bg[1] * ba * (1.0 - fa),
        fg[2] * fa + bg[2] * ba * (1.0 - fa),
        alpha,
    ]
}
pub fn sample(scene: Scene, p: Params, x: u32, y: u32) -> Pixel {
    let fx = x as f32 + 0.5;
    let fy = y as f32 + 0.5;
    let d = field(scene, fx, fy);
    let mask = u32::from(d < 0.0);
    let aa = 1.2 / (scene.logical_width.min(scene.logical_height) * scene.dpr) as f32;
    let coverage = (0.5 - d / aa).clamp(0.0, 1.0);
    let height = height_at(scene, fx, fy);
    let dx = (height_at(scene, fx + 1.0, fy) - height_at(scene, fx - 1.0, fy)) * 0.5;
    let dy = (height_at(scene, fx, fy + 1.0) - height_at(scene, fx, fy - 1.0)) * 0.5;
    let normal = normalize([-dx * 2.0, -dy * 2.0, 0.20]);
    let light = normalize([-0.42, -0.58, 0.92]);
    let ndotl = (normal[0] * light[0] + normal[1] * light[1] + normal[2] * light[2]).max(0.0);
    let rim = (1.0 - d.abs() / 0.045).clamp(0.0, 1.0);
    let (_, qy) = coords(scene, fx, fy);
    let (base, spec) = match p.material {
        Material::Ceramic => ([0.07, 0.18, 0.26], 0.34),
        Material::Gel => ([0.20, 0.10, 0.29], 0.15),
        Material::BrushedMetal => ([0.23, 0.26, 0.24], 0.27),
        Material::Paper => ([0.34, 0.29, 0.20], 0.05),
    };
    let grain = match p.material {
        Material::Paper => {
            let (qx, _) = coords(scene, fx, fy);
            0.013 * (qx * 149.0 + qy * 73.0).sin()
        }
        Material::BrushedMetal => 0.016 * (qy * 172.0 + scene.phase).sin(),
        _ => 0.0,
    };
    let relief = rim * ndotl * spec;
    let gradient = 0.052 * (0.5 - qy).clamp(-0.5, 1.5);
    let material = [
        (base[0] + relief + gradient + grain).clamp(0.0, 1.0),
        (base[1] + relief + gradient + grain).clamp(0.0, 1.0),
        (base[2] + relief + gradient + grain).clamp(0.0, 1.0),
        coverage,
    ];
    let heat = heat_at(scene, fx, fy, p.heat_strength);
    let layer = over(material, [0.018, 0.026, 0.035, 1.0]);
    let result = over([0.62, 0.20, 0.07, heat * coverage * 0.24], layer);
    let mut rgba = [0u8; 4];
    for i in 0..3 {
        rgba[i] = (srgb_encode(result[i]) * 255.0).round() as u8;
    }
    rgba[3] = (result[3] * 255.0).round() as u8;
    Pixel {
        rgba,
        mask,
        height,
        heat,
        normal_z: normal[2],
    }
}
pub fn cpu_render(scene: Scene, p: Params) -> Result<CpuFrame, String> {
    scene.validate()?;
    p.validate()?;
    let n = scene.width() as usize * scene.height() as usize;
    let mut output = CpuFrame {
        rgba: Vec::with_capacity(n * 4),
        mask: Vec::with_capacity(n),
        height: Vec::with_capacity(n),
        heat: Vec::with_capacity(n),
        normal_z: Vec::with_capacity(n),
        semantic_digest: scene.digest(),
    };
    for y in 0..scene.height() {
        for x in 0..scene.width() {
            let px = sample(scene, p, x, y);
            output.rgba.extend(px.rgba);
            output.mask.push(px.mask);
            output.height.push(px.height);
            output.heat.push(px.heat);
            output.normal_z.push(px.normal_z);
        }
    }
    Ok(output)
}
pub fn geometry_agreement(scene: Scene, p: Params) -> Result<(), String> {
    let result = cpu_render(scene, p)?;
    let mask = blobin_f2_topology::cpu_mask(scene)?;
    for (i, (a, b)) in result.mask.iter().zip(mask.iter()).enumerate() {
        if *a != u32::from(*b) {
            return Err(format!("F3_F2_GEOMETRY_DRIFT index={i}"));
        }
    }
    let topology = blobin_f2_topology::measure_mask(&mask, scene.width(), scene.height())?;
    if topology != scene.layout.expected_topology() {
        return Err("F3_TOPOLOGY_DRIFT".into());
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn material_changes_never_change_topology() {
        for mut s in blobin_f2_topology::qualification_trace().unwrap() {
            for dpr in [1, 2] {
                s.dpr = dpr;
                for material in Material::ALL {
                    geometry_agreement(
                        s,
                        Params {
                            material,
                            ..Params::default()
                        },
                    )
                    .unwrap();
                }
            }
        }
    }
    #[test]
    fn deterministic_finite_and_bounded() {
        let s = Scene::default();
        let p = Params::default();
        let a = cpu_render(s, p).unwrap();
        let b = cpu_render(s, p).unwrap();
        assert_eq!(a.rgba, b.rgba);
        assert_eq!(a.mask, b.mask);
        assert_eq!(a.semantic_digest, s.digest());
        assert!(a
            .height
            .iter()
            .all(|x| x.is_finite() && *x >= 0.0 && *x <= 0.35));
        assert!(a
            .heat
            .iter()
            .all(|x| x.is_finite() && (0.0..=1.0).contains(x)));
        assert!(a
            .normal_z
            .iter()
            .all(|x| x.is_finite() && (0.0..=1.0).contains(x)));
        assert!(a.rgba.chunks_exact(4).all(|x| x[3] == 255));
    }
    #[test]
    fn cold_heatfield_is_zero() {
        let s = Scene::default();
        let cold = cpu_render(
            s,
            Params {
                heat_strength: 0.0,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(cold.heat.iter().all(|&v| v == 0.0));
        let hot = cpu_render(
            s,
            Params {
                heat_strength: 1.0,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(hot.heat.iter().any(|&v| v > 0.3));
    }
    #[test]
    fn premultiply_exactly_once() {
        let composite = over([0.8, 0.0, 0.0, 0.5], [0.0, 0.0, 0.0, 1.0]);
        assert!((composite[0] - 0.4).abs() < 0.000001);
        assert_eq!(composite[3], 1.0);
    }
    #[test]
    fn color_roundtrip() {
        for v in [0.0, 0.001, 0.01, 0.18, 0.5, 1.0] {
            assert!((linear_decode(srgb_encode(v)) - v).abs() < 0.000001);
        }
    }
    #[test]
    fn nonfinite_and_out_of_range_heat_rejected() {
        for v in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            assert!(cpu_render(
                Scene::default(),
                Params {
                    heat_strength: v,
                    ..Default::default()
                }
            )
            .is_err());
        }
    }
    #[test]
    fn punched_hole_never_creates_hit_target() {
        let mut e = blobin_f2_topology::Engine::new(Scene::default()).unwrap();
        e.transition(blobin_f2_topology::Action::Merge).unwrap();
        e.transition(blobin_f2_topology::Action::PunchHole).unwrap();
        let s = e.scene;
        let middle = (s.width() / 2, s.height() / 2);
        assert_eq!(s.hit_test(middle.0, middle.1), None);
        for material in Material::ALL {
            assert_eq!(
                sample(
                    s,
                    Params {
                        material,
                        ..Default::default()
                    },
                    middle.0,
                    middle.1
                )
                .mask,
                0
            );
        }
    }
    #[test]
    fn shader_naga_parse_and_validate() {
        let m = naga::front::wgsl::parse_str(include_str!("../shaders/f3.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&m)
        .unwrap();
    }
}
