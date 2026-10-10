//! Private clean-room F4: bounded, typed postprocessing with explicit dirty and temporal contracts.
//! CPU oracle; GPU F3 source rendering is supported, but GPU F4 postpasses are NOT implemented.
pub mod gpu;
use blobin_f2_topology::{cpu_mask, measure_mask, Scene};
use blobin_f3_material_height::{cpu_render, Params};

pub const MAX_PIXELS: usize = 1024 * 1024;
pub const MAX_RADIUS: usize = 8;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Pass {
    Shadow,
    Blur,
    Bloom,
    Echo,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
impl Rect {
    pub fn expanded(self, by: u32, w: u32, h: u32) -> Self {
        let x = self.x.saturating_sub(by);
        let y = self.y.saturating_sub(by);
        let xmax = self.x.saturating_add(self.w).saturating_add(by).min(w);
        let ymax = self.y.saturating_add(self.h).saturating_add(by).min(h);
        Self {
            x,
            y,
            w: xmax.saturating_sub(x),
            h: ymax.saturating_sub(y),
        }
    }
}
#[derive(Clone, Debug)]
pub struct Plan {
    pub passes: Vec<Pass>,
    pub blur_radius: usize,
    pub reduced_motion: bool,
    pub dirty: Rect,
}
impl Plan {
    pub fn validate(&self, scene: Scene) -> Result<(), String> {
        scene.validate()?;
        if self.blur_radius > MAX_RADIUS || self.passes.len() > 4 {
            return Err("F4 BOUNDED_GRAPH".into());
        }
        for (index, pass) in self.passes.iter().enumerate() {
            let position = match pass {
                Pass::Shadow => 0,
                Pass::Blur => 1,
                Pass::Bloom => 2,
                Pass::Echo => 3,
            };
            if position != index {
                return Err("F4 INVALID_PASS_ORDER".into());
            }
        }
        if self.dirty.x > scene.width()
            || self.dirty.y > scene.height()
            || self.dirty.x.saturating_add(self.dirty.w) > scene.width()
            || self.dirty.y.saturating_add(self.dirty.h) > scene.height()
        {
            return Err("F4 INVALID_DIRTY_RECT".into());
        }
        if scene.width() as usize * scene.height() as usize > MAX_PIXELS {
            return Err("F4 PIXEL_BUDGET_EXCEEDED".into());
        }
        Ok(())
    }
    pub fn halo(&self) -> u32 {
        // shadow tap shift 2px and 2 separable blur passes, bloom uses blurred input
        2 + (self.blur_radius as u32) * 2
    }
    pub fn expanded_dirty(&self, scene: Scene) -> Rect {
        self.dirty
            .expanded(self.halo(), scene.width(), scene.height())
    }
}
impl Default for Plan {
    fn default() -> Self {
        Self {
            passes: vec![Pass::Shadow, Pass::Blur, Pass::Bloom, Pass::Echo],
            blur_radius: 3,
            reduced_motion: false,
            dirty: Rect {
                x: 0,
                y: 0,
                w: 256,
                h: 192,
            },
        }
    }
}
#[derive(Default)]
pub struct Temporal {
    previous: Option<Vec<u8>>,
    previous_digest: Option<u64>,
    pub resets: usize,
}
impl Temporal {
    pub fn reset(&mut self) {
        self.previous = None;
        self.previous_digest = None;
        self.resets += 1;
    }
    pub fn len(&self) -> usize {
        self.previous.as_ref().map_or(0, Vec::len)
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
pub struct Output {
    pub rgba: Vec<u8>,
    pub components: u32,
    pub holes: u32,
    pub dirty: Rect,
    pub semantic_digest: u64,
    pub temporal_bytes: usize,
}
fn blurred(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    if r == 0 {
        return src.to_vec();
    }
    let mut tmp = vec![0.; src.len()];
    let mut out = vec![0.; src.len()];
    for y in 0..h {
        for x in 0..w {
            let left = x.saturating_sub(r);
            let right = (x + r).min(w - 1);
            let mut sum = 0.;
            for u in left..=right {
                sum += src[y * w + u];
            }
            tmp[y * w + x] = sum / (right - left + 1) as f32;
        }
    }
    for y in 0..h {
        for x in 0..w {
            let top = y.saturating_sub(r);
            let bottom = (y + r).min(h - 1);
            let mut sum = 0.;
            for v in top..=bottom {
                sum += tmp[v * w + x];
            }
            out[y * w + x] = sum / (bottom - top + 1) as f32;
        }
    }
    out
}
/// F4 is a post-process projection; topology and hit target remain exclusively F2-owned.
pub fn process(
    scene: Scene,
    params: Params,
    plan: &Plan,
    temporal: &mut Temporal,
) -> Result<Output, String> {
    plan.validate(scene)?;
    params.validate()?;
    let source = cpu_render(scene, params)?;
    let w = scene.width() as usize;
    let h = scene.height() as usize;
    let mask = cpu_mask(scene)?;
    let topo = measure_mask(&mask, scene.width(), scene.height())?;
    if topo != scene.layout.expected_topology() {
        return Err("F4 TOPOLOGY_ORACLE_RED".into());
    }
    let n = w * h;
    // No implicit temporal history on reduced-motion, or following semantic epoch changes.
    if plan.reduced_motion || temporal.previous_digest != Some(scene.digest()) {
        temporal.reset();
    }
    let source_luma: Vec<f32> = source
        .rgba
        .chunks_exact(4)
        .map(|p| (p[0] as f32 + p[1] as f32 + p[2] as f32) / (3. * 255.))
        .collect();
    let mut dst = source.rgba.clone();
    for pass in &plan.passes {
        match pass {
            Pass::Shadow => {
                for y in 2..h {
                    for x in 2..w {
                        let pos = (y * w + x) * 4;
                        if mask[(y - 2) * w + x - 2] != 0 && mask[y * w + x] == 0 {
                            for v in &mut dst[pos..pos + 3] {
                                *v = v.saturating_sub(12);
                            }
                        }
                    }
                }
            }
            Pass::Blur => {
                let smooth = blurred(&source_luma, w, h, plan.blur_radius);
                for i in 0..n {
                    if mask[i] != 0 {
                        let inc = (smooth[i] * 10.).round() as u8;
                        for v in &mut dst[i * 4..i * 4 + 3] {
                            *v = v.saturating_add(inc);
                        }
                    }
                }
            }
            Pass::Bloom => {
                let highlights: Vec<f32> = source_luma.iter().map(|v| (v - 0.45).max(0.)).collect();
                let halo = blurred(&highlights, w, h, plan.blur_radius);
                for i in 0..n {
                    let inc = (halo[i] * 18.).round() as u8;
                    for v in &mut dst[i * 4..i * 4 + 3] {
                        *v = v.saturating_add(inc);
                    }
                }
            }
            Pass::Echo => {
                if !plan.reduced_motion {
                    if let Some(prev) = temporal.previous.as_ref() {
                        if prev.len() != dst.len() {
                            return Err("F4 TEMPORAL_SIZE_MISMATCH".into());
                        }
                        for (now, old) in dst.iter_mut().zip(prev) {
                            *now = ((*now as u16 * 3 + *old as u16) / 4) as u8;
                        }
                    }
                }
            }
        }
    }
    if plan.passes.contains(&Pass::Echo) && !plan.reduced_motion {
        temporal.previous = Some(dst.clone());
        temporal.previous_digest = Some(scene.digest());
    } else {
        temporal.previous = None;
        temporal.previous_digest = None;
    }
    if temporal.len() > n * 4 {
        return Err("F4 TEMPORAL_MEMORY_EXCEEDED".into());
    }
    Ok(Output {
        rgba: dst,
        components: topo.components,
        holes: topo.holes,
        dirty: plan.expanded_dirty(scene),
        semantic_digest: scene.digest(),
        temporal_bytes: temporal.len(),
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn scene() -> Scene {
        Scene::default()
    }
    #[test]
    fn shader_naga_validates() {
        let shader = naga::front::wgsl::parse_str(include_str!("../shaders/f4.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&shader)
        .unwrap();
    }
    #[test]
    fn all_passes_and_geometry() {
        let s = scene();
        let mut t = Temporal::default();
        let p = Plan::default();
        let first = process(s, Params::default(), &p, &mut t).unwrap();
        assert_eq!((first.components, first.holes), (2, 0));
        assert_eq!(first.semantic_digest, s.digest());
        assert_eq!(first.rgba.len(), (s.width() * s.height() * 4) as usize);
        assert_eq!(first.temporal_bytes, first.rgba.len());
        assert!(first.rgba.chunks_exact(4).all(|px| px[3] == 255));
        let _ = process(s, Params::default(), &p, &mut t).unwrap();
    }
    #[test]
    fn reduced_motion_does_not_store_echoes() {
        let s = scene();
        let mut t = Temporal::default();
        let p = Plan {
            reduced_motion: true,
            ..Plan::default()
        };
        let a = process(s, Params::default(), &p, &mut t).unwrap();
        let b = process(s, Params::default(), &p, &mut t).unwrap();
        assert_eq!(a.rgba, b.rgba);
        assert_eq!(t.len(), 0);
    }
    #[test]
    fn epoch_change_clears_temporal_feedback() {
        let s = scene();
        let mut t = Temporal::default();
        let p = Plan::default();
        process(s, Params::default(), &p, &mut t).unwrap();
        let count = t.resets;
        process(Scene { epoch: 1, ..s }, Params::default(), &p, &mut t).unwrap();
        assert_eq!(t.resets, count + 1);
    }
    #[test]
    fn dirty_halo_is_bounded_and_clipped() {
        let s = scene();
        let p = Plan {
            dirty: Rect {
                x: 246,
                y: 186,
                w: 10,
                h: 6,
            },
            ..Plan::default()
        };
        let d = p.expanded_dirty(s);
        assert_eq!((d.x + d.w, d.y + d.h), (256, 192));
        assert_eq!(d.x, 238);
        assert_eq!(d.y, 178);
    }
    #[test]
    fn invalid_graph_and_memory_budgets_rejected() {
        let s = scene();
        let mut p = Plan {
            blur_radius: 10000,
            ..Plan::default()
        };
        assert!(p.validate(s).is_err());
        p = Plan::default();
        p.passes.swap(0, 1);
        assert!(p.validate(s).is_err());
        p = Plan::default();
        p.dirty.x = 99999;
        assert!(p.validate(s).is_err());
        let wide = Scene {
            logical_width: 2048,
            logical_height: 1024,
            ..s
        };
        assert!(p.validate(wide).is_err());
    }
    #[test]
    fn trace_preserves_topology_and_identity() {
        let mut temporal = Temporal::default();
        for s in blobin_f2_topology::qualification_trace().unwrap() {
            let out = process(s, Params::default(), &Plan::default(), &mut temporal).unwrap();
            assert_eq!(
                (out.components, out.holes),
                (
                    s.layout.expected_topology().components,
                    s.layout.expected_topology().holes
                )
            );
            assert_eq!(out.semantic_digest, s.digest());
        }
    }
}
