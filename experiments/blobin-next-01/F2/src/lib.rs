//! Private clean-room F2: typed topology authority, independently falsifiable by a raster oracle.
//! Geometry models two organic lobes. Renderer providers do not own entity identity or epochs.

pub mod gpu;

use std::collections::VecDeque;

pub const LEFT_ID: u64 = 0xB10B_0001;
pub const RIGHT_ID: u64 = 0xB10B_0002;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Layout {
    Split,
    Merged,
    Hollow,
}
impl Layout {
    pub fn code(self) -> u32 {
        match self {
            Self::Split => 0,
            Self::Merged => 1,
            Self::Hollow => 2,
        }
    }
    pub fn expected_topology(self) -> Topology {
        match self {
            Self::Split => Topology {
                components: 2,
                holes: 0,
            },
            Self::Merged => Topology {
                components: 1,
                holes: 0,
            },
            Self::Hollow => Topology {
                components: 1,
                holes: 1,
            },
        }
    }
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Action {
    Merge,
    Split,
    PunchHole,
    SealHole,
}
impl Action {
    pub fn name(self) -> &'static str {
        match self {
            Self::Merge => "Merge",
            Self::Split => "Split",
            Self::PunchHole => "PunchHole",
            Self::SealHole => "SealHole",
        }
    }
}
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Topology {
    pub components: u32,
    pub holes: u32,
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Scene {
    pub layout: Layout,
    pub epoch: u64,
    pub left_id: u64,
    pub right_id: u64,
    pub logical_width: u32,
    pub logical_height: u32,
    pub dpr: u32,
    pub phase: f32,
}
impl Default for Scene {
    fn default() -> Self {
        Self {
            layout: Layout::Split,
            epoch: 0,
            left_id: LEFT_ID,
            right_id: RIGHT_ID,
            logical_width: 256,
            logical_height: 192,
            dpr: 1,
            phase: 0.35,
        }
    }
}
impl Scene {
    pub fn width(self) -> u32 {
        self.logical_width * self.dpr
    }
    pub fn height(self) -> u32 {
        self.logical_height * self.dpr
    }
    pub fn validate(self) -> Result<(), String> {
        if !matches!(self.dpr, 1 | 2)
            || self.logical_width == 0
            || self.logical_height == 0
            || self.logical_width > 2048
            || self.logical_height > 2048
            || !self.phase.is_finite()
            || self.phase.abs() > 10.0
            || self.left_id == self.right_id
            || self.left_id == 0
            || self.right_id == 0
        {
            return Err("F2: invalid scene, dimension, DPR, phase, or IDs".into());
        }
        Ok(())
    }
    pub fn digest(self) -> u64 {
        let mut state = 0xcbf29ce484222325u64;
        for word in [
            self.left_id,
            self.right_id,
            self.epoch,
            self.layout.code() as u64,
            self.logical_width as u64,
            self.logical_height as u64,
            self.phase.to_bits() as u64,
        ] {
            for byte in word.to_le_bytes() {
                state ^= byte as u64;
                state = state.wrapping_mul(0x100000001b3);
            }
        }
        state
    }
    /// Explicit hit ownership: overlap chooses the nearer lobe; hole and exterior are noninteractive.
    /// This never derives authority from a renderer's pixel buffer.
    pub fn hit_test(self, x: u32, y: u32) -> Option<u64> {
        let (field, a, b) = signed_field(self, x, y);
        if field >= 0.0 {
            return None;
        }
        if a <= b {
            Some(self.left_id)
        } else {
            Some(self.right_id)
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TopologyEvent {
    pub action: Action,
    pub from_epoch: u64,
    pub to_epoch: u64,
    pub before: Topology,
    pub after: Topology,
    pub left_id: u64,
    pub right_id: u64,
    pub before_digest: u64,
    pub after_digest: u64,
}

pub struct Engine {
    pub scene: Scene,
    pub events: Vec<TopologyEvent>,
}
impl Engine {
    pub fn new(scene: Scene) -> Result<Self, String> {
        scene.validate()?;
        Ok(Self {
            scene,
            events: Vec::new(),
        })
    }
    pub fn transition(&mut self, action: Action) -> Result<&TopologyEvent, String> {
        let to = match (self.scene.layout, action) {
            (Layout::Split, Action::Merge) => Layout::Merged,
            (Layout::Merged, Action::Split) => Layout::Split,
            (Layout::Merged, Action::PunchHole) => Layout::Hollow,
            (Layout::Hollow, Action::SealHole) => Layout::Merged,
            _ => {
                return Err(format!(
                    "F2 INVALID_TRANSITION: {:?} / {}",
                    self.scene.layout,
                    action.name()
                ))
            }
        };
        let before = self.scene;
        let next_epoch = before
            .epoch
            .checked_add(1)
            .ok_or("topology epoch overflow")?;
        let mut next = before;
        next.layout = to;
        next.epoch = next_epoch;
        let event = TopologyEvent {
            action,
            from_epoch: before.epoch,
            to_epoch: next_epoch,
            before: before.layout.expected_topology(),
            after: to.expected_topology(),
            left_id: before.left_id,
            right_id: before.right_id,
            before_digest: before.digest(),
            after_digest: next.digest(),
        };
        self.scene = next;
        self.events.push(event);
        Ok(self.events.last().expect("just pushed"))
    }
}
fn coords(scene: Scene, x: u32, y: u32) -> (f32, f32) {
    let scale = scene.logical_width.min(scene.logical_height) as f32;
    (
        ((x as f32 + 0.5) / scene.dpr as f32 - scene.logical_width as f32 * 0.5) / scale,
        ((y as f32 + 0.5) / scene.dpr as f32 - scene.logical_height as f32 * 0.5) / scale,
    )
}
fn lobe(x: f32, y: f32, cx: f32, phase: f32) -> f32 {
    let px = x - cx;
    let theta = y.atan2(px);
    let r = 0.235 + 0.008 * (3.0 * theta + phase).sin() + 0.006 * (5.0 * theta - phase).cos();
    (px * px + y * y).sqrt() - r
}
/// Approximate signed contour field, negative inside material.
/// Exposes both lobes independently for stable hit ownership.
pub fn signed_field(scene: Scene, x: u32, y: u32) -> (f32, f32, f32) {
    let (qx, qy) = coords(scene, x, y);
    let separation = if scene.layout == Layout::Split {
        0.31
    } else {
        0.15
    };
    let a = lobe(qx, qy, -separation, scene.phase);
    let b = lobe(qx, qy, separation, scene.phase);
    let union = a.min(b);
    let d = if scene.layout == Layout::Hollow {
        union.max(0.055 - (qx * qx + qy * qy).sqrt())
    } else {
        union
    };
    (d, a, b)
}
pub fn cpu_mask(scene: Scene) -> Result<Vec<u8>, String> {
    scene.validate()?;
    let mut mask = Vec::with_capacity((scene.width() * scene.height()) as usize);
    for y in 0..scene.height() {
        for x in 0..scene.width() {
            mask.push(u8::from(signed_field(scene, x, y).0 < 0.0));
        }
    }
    Ok(mask)
}
pub fn cpu_render(scene: Scene) -> Result<Vec<u8>, String> {
    scene.validate()?;
    let scale = scene.logical_width.min(scene.logical_height) as f32;
    let aa = 1.2 / (scale * scene.dpr as f32);
    let mut rgba = Vec::with_capacity((scene.width() * scene.height() * 4) as usize);
    for y in 0..scene.height() {
        for x in 0..scene.width() {
            let (d, _, _) = signed_field(scene, x, y);
            let (qx, qy) = coords(scene, x, y);
            let coverage = (0.5 - d / aa).clamp(0.0, 1.0);
            let bevel = (1.0 - d.abs() / 0.028).clamp(0.0, 1.0);
            let len = (qx * qx + qy * qy).sqrt().max(0.00001);
            let incidence = (0.52 - (qx / len * 0.33 - qy / len * 0.52)).clamp(0.0, 1.0);
            let lighting = bevel * incidence * 0.22;
            let fill = [0.11f32, 0.28, 0.31];
            let back = [0.025f32, 0.033, 0.043];
            for i in 0..3 {
                let v =
                    back[i] * (1.0 - coverage) + (fill[i] + lighting).clamp(0.0, 1.0) * coverage;
                rgba.push((v * 255.0).round().clamp(0.0, 255.0) as u8);
            }
            rgba.push(255);
        }
    }
    Ok(rgba)
}

/// Extracts a binary mask from the actual RGBA projection, without consulting Scene.mode.
/// Threshold is deliberately fixed before comparing renders. An oracle can therefore fail.
pub fn image_mask(rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>, String> {
    if rgba.len() != width as usize * height as usize * 4 {
        return Err("RGBA input does not match dimensions".into());
    }
    Ok(rgba
        .chunks_exact(4)
        .map(|p| u8::from(p[1] > 39 && p[3] == 255))
        .collect())
}
/// Flood-fill observed 4-connected material and 4-connected background holes
/// (background components that DO NOT touch viewport boundary).
/// This is a raster topology oracle at the tested resolution, not a continuous proof.
pub fn measure_mask(mask: &[u8], width: u32, height: u32) -> Result<Topology, String> {
    let count = width as usize * height as usize;
    if width < 2 || height < 2 || mask.len() != count || mask.iter().any(|&p| p > 1) {
        return Err("invalid binary mask or viewport".into());
    }
    let w = width as usize;
    let h = height as usize;
    let mut seen = vec![false; count];
    let mut topology = Topology {
        components: 0,
        holes: 0,
    };
    let mut queue = VecDeque::new();
    for start in 0..count {
        if seen[start] {
            continue;
        }
        let target = mask[start];
        let mut touches_edge = false;
        seen[start] = true;
        queue.push_back(start);
        while let Some(i) = queue.pop_front() {
            let x = i % w;
            let y = i / w;
            if x == 0 || y == 0 || x + 1 == w || y + 1 == h {
                touches_edge = true;
            }
            for n in [
                if x > 0 { Some(i - 1) } else { None },
                if x + 1 < w { Some(i + 1) } else { None },
                if y > 0 { Some(i - w) } else { None },
                if y + 1 < h { Some(i + w) } else { None },
            ]
            .into_iter()
            .flatten()
            {
                let j = n;
                if !seen[j] && mask[j] == target {
                    seen[j] = true;
                    queue.push_back(j);
                }
            }
        }
        if target == 1 {
            topology.components += 1;
        } else if !touches_edge {
            topology.holes += 1;
        }
    }
    Ok(topology)
}
pub fn measure_image(rgba: &[u8], w: u32, h: u32) -> Result<Topology, String> {
    measure_mask(&image_mask(rgba, w, h)?, w, h)
}
pub fn qualification_trace() -> Result<Vec<Scene>, String> {
    let mut engine = Engine::new(Scene::default())?;
    let mut scenes = vec![engine.scene];
    for action in [
        Action::Merge,
        Action::PunchHole,
        Action::SealHole,
        Action::Split,
    ] {
        engine.transition(action)?;
        scenes.push(engine.scene);
    }
    Ok(scenes)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_trace_and_identity_survival() {
        let mut e = Engine::new(Scene::default()).unwrap();
        let digest0 = e.scene.digest();
        for (i, kind) in [
            Action::Merge,
            Action::PunchHole,
            Action::SealHole,
            Action::Split,
        ]
        .into_iter()
        .enumerate()
        {
            let evt = e.transition(kind).unwrap();
            assert_eq!(evt.to_epoch, (i + 1) as u64);
            assert_eq!((evt.left_id, evt.right_id), (LEFT_ID, RIGHT_ID));
            assert_ne!(evt.before_digest, evt.after_digest);
        }
        assert_eq!(e.scene.layout, Layout::Split);
        assert_ne!(e.scene.digest(), digest0);
        assert_eq!(e.events.len(), 4);
        assert_eq!(
            format!("{:?}", e.events),
            format!("{:?}", {
                let mut r = Engine::new(Scene::default()).unwrap();
                for a in [
                    Action::Merge,
                    Action::PunchHole,
                    Action::SealHole,
                    Action::Split,
                ] {
                    r.transition(a).unwrap();
                }
                r.events
            })
        );
    }
    #[test]
    fn invalid_transition_is_transactional() {
        let mut e = Engine::new(Scene::default()).unwrap();
        let state = e.scene;
        assert!(e.transition(Action::PunchHole).is_err());
        assert_eq!(e.scene, state);
        assert!(e.events.is_empty());
        assert!(e.transition(Action::Split).is_err());
        assert_eq!(e.scene, state);
    }
    #[test]
    fn topology_oracle_matches_real_geometry_at_both_dprs() {
        for dpr in [1, 2] {
            for mut scene in qualification_trace().unwrap() {
                scene.dpr = dpr;
                let mask = cpu_mask(scene).unwrap();
                let observed = measure_mask(&mask, scene.width(), scene.height()).unwrap();
                assert_eq!(
                    observed,
                    scene.layout.expected_topology(),
                    "dpr={dpr} layout={:?}",
                    scene.layout
                );
                let image = cpu_render(scene).unwrap();
                assert_eq!(
                    measure_image(&image, scene.width(), scene.height()).unwrap(),
                    observed
                );
            }
        }
    }
    #[test]
    fn hole_has_no_owner_and_lobes_retain_identity() {
        let mut e = Engine::new(Scene::default()).unwrap();
        let x = e.scene.logical_width / 2;
        let y = e.scene.logical_height / 2;
        e.transition(Action::Merge).unwrap();
        assert!(e.scene.hit_test(x, y).is_some());
        e.transition(Action::PunchHole).unwrap();
        assert_eq!(e.scene.hit_test(x, y), None);
        let left = e.scene.hit_test(x - 33, y);
        let right = e.scene.hit_test(x + 33, y);
        assert_eq!(left, Some(LEFT_ID));
        assert_eq!(right, Some(RIGHT_ID));
    }
    #[test]
    fn negative_topology_falsifier() {
        let scene = Scene::default();
        let mut mask = cpu_mask(scene).unwrap();
        let old = measure_mask(&mask, scene.width(), scene.height()).unwrap();
        assert_eq!(old.components, 2);
        // Synthetic causal mutation: fill the central gap, connecting two blobs.
        for x in 70..190 {
            let j = (scene.logical_height / 2 * scene.logical_width + x) as usize;
            mask[j] = 1;
        }
        let altered = measure_mask(&mask, scene.width(), scene.height()).unwrap();
        assert_ne!(old, altered, "oracle MUST reject a changed topology");
    }
    #[test]
    fn reject_nonfinite_dpr_and_duplicate_ids() {
        assert!(Scene {
            phase: f32::NAN,
            ..Scene::default()
        }
        .validate()
        .is_err());
        assert!(Scene {
            dpr: 0,
            ..Scene::default()
        }
        .validate()
        .is_err());
        assert!(Scene {
            left_id: RIGHT_ID,
            ..Scene::default()
        }
        .validate()
        .is_err());
    }
    #[test]
    fn shader_naga_parse_validate() {
        let module = naga::front::wgsl::parse_str(include_str!("../shaders/f2.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
    }
}
