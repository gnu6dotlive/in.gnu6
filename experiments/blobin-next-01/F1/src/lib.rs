//! Newly authored F1 contract: semantic frame is independent of both renderers.
//! Non-production private clean-room lab. No historical blob.in code imported.
pub mod gpu;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameIr {
    pub logical_width: u32,
    pub logical_height: u32,
    pub dpr: u32,
    pub phase: f32,
    pub focused: bool,
    pub entity_id: u64,
    pub topology_epoch: u64,
}

impl Default for FrameIr {
    fn default() -> Self {
        Self {
            logical_width: 256,
            logical_height: 192,
            dpr: 1,
            phase: 0.35,
            focused: true,
            entity_id: 0xB10B_0001,
            topology_epoch: 0,
        }
    }
}

impl FrameIr {
    pub fn validate(&self) -> Result<(), String> {
        if self.logical_width == 0
            || self.logical_height == 0
            || !matches!(self.dpr, 1 | 2)
            || !self.phase.is_finite()
            || self.phase.abs() > 10.0
            || self.logical_width > 2048
            || self.logical_height > 2048
        {
            return Err("F1: invalid dimensions, DPR or phase".into());
        }
        Ok(())
    }
    pub fn pixel_width(&self) -> u32 {
        self.logical_width * self.dpr
    }
    pub fn pixel_height(&self) -> u32 {
        self.logical_height * self.dpr
    }
    pub fn semantic_digest(&self) -> u64 {
        // FNV-1a of the *semantic* IR, not any raster/provider id.
        let mut hash = 0xcbf29ce484222325u64;
        for bytes in [
            self.logical_width as u64,
            self.logical_height as u64,
            self.phase.to_bits() as u64,
            self.focused as u64,
            self.entity_id,
            self.topology_epoch,
        ] {
            for byte in bytes.to_le_bytes() {
                hash ^= byte as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        hash
    }
}

/// Approximate signed field: organic outer contour and a real circular inner hole.
/// The renderer sees the same field; topology ownership stays in FrameIr.
pub fn signed_field(frame: &FrameIr, x: u32, y: u32) -> (f32, f32, f32) {
    let scale = frame.logical_width.min(frame.logical_height) as f32;
    let qx = ((x as f32 + 0.5) / frame.dpr as f32 - frame.logical_width as f32 * 0.5) / scale;
    let qy = ((y as f32 + 0.5) / frame.dpr as f32 - frame.logical_height as f32 * 0.5) / scale;
    let theta = qy.atan2(qx);
    let radius = 0.36
        + 0.023 * (theta * 5.0 + frame.phase).sin()
        + 0.012 * (theta * 3.0 - frame.phase).cos();
    let outer = (qx * qx + qy * qy).sqrt() - radius;
    let hx = qx - 0.12;
    let hy = qy + 0.03;
    let hole = (hx * hx + hy * hy).sqrt() - 0.095;
    (outer.max(-hole), qx, qy)
}

pub fn cpu_render(frame: &FrameIr) -> Result<Vec<u8>, String> {
    frame.validate()?;
    let mut result = Vec::with_capacity((frame.pixel_width() * frame.pixel_height() * 4) as usize);
    let aa = 1.2 / (frame.logical_width.min(frame.logical_height) * frame.dpr) as f32;
    for y in 0..frame.pixel_height() {
        for x in 0..frame.pixel_width() {
            let (d, qx, qy) = signed_field(frame, x, y);
            let coverage = (0.5 - d / aa).clamp(0.0, 1.0);
            let bevel = (1.0 - d.abs() / 0.035).clamp(0.0, 1.0);
            let len = (qx * qx + qy * qy).sqrt().max(0.00001);
            let incidence = (0.52 - (qx / len * 0.33 - qy / len * 0.52)).clamp(0.0, 1.0);
            let illumination = bevel * incidence * if frame.focused { 0.22 } else { 0.12 };
            let base = [0.11f32, 0.28, 0.31];
            let back = [0.025f32, 0.033, 0.043];
            for i in 0..3 {
                let fill = (base[i] + illumination).clamp(0.0, 1.0);
                let value = back[i] * (1.0 - coverage) + fill * coverage;
                result.push((value * 255.0).round().clamp(0.0, 255.0) as u8);
            }
            result.push(255);
        }
    }
    Ok(result)
}

#[derive(Debug)]
pub struct ImageDiff {
    pub max_channel_error: u8,
    pub mean_abs_error: f64,
    pub changed_channels: usize,
    pub differing_pixels: usize,
    pub total_pixels: usize,
}

pub fn diff_images(a: &[u8], b: &[u8]) -> Result<(ImageDiff, Vec<u8>), String> {
    if a.len() != b.len() || !a.len().is_multiple_of(4) {
        return Err("image byte-length mismatch".into());
    }
    let mut max_error = 0;
    let mut sum: u64 = 0;
    let mut changed = 0;
    let mut differing_pixels = 0;
    let mut diff = Vec::with_capacity(a.len());
    for (ap, bp) in a.chunks_exact(4).zip(b.chunks_exact(4)) {
        let mut nonzero = false;
        for i in 0..3 {
            let d = ap[i].abs_diff(bp[i]);
            max_error = max_error.max(d);
            sum += d as u64;
            if d > 0 {
                changed += 1;
                nonzero = true;
            }
            diff.push(d.saturating_mul(16));
        }
        if nonzero {
            differing_pixels += 1;
        }
        diff.push(255);
    }
    let stats = ImageDiff {
        max_channel_error: max_error,
        mean_abs_error: sum as f64 / (a.len() / 4 * 3).max(1) as f64,
        changed_channels: changed,
        differing_pixels,
        total_pixels: a.len() / 4,
    };
    Ok((stats, diff))
}

pub fn bytes_digest(data: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for b in data {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cpu_repeatable_and_opaque() {
        let ir = FrameIr::default();
        let a = cpu_render(&ir).unwrap();
        assert_eq!(a, cpu_render(&ir).unwrap());
        assert_eq!(a.len(), (ir.pixel_width() * ir.pixel_height() * 4) as usize);
        assert!(a.chunks_exact(4).all(|p| p[3] == 255));
    }
    #[test]
    fn real_hole_and_separate_contour() {
        let ir = FrameIr::default();
        let mid_x = (ir.logical_width as f32 / 2.0 + 0.12 * 192.0) as u32;
        let mid_y = (ir.logical_height as f32 / 2.0 - 0.03 * 192.0) as u32;
        assert!(
            signed_field(&ir, mid_x, mid_y).0 > 0.05,
            "hole must be excluded"
        );
        assert!(signed_field(&ir, ir.logical_width / 2 - 35, ir.logical_height / 2).0 < 0.0);
        assert!(signed_field(&ir, 0, 0).0 > 0.0);
    }
    #[test]
    fn semantic_digest_independent_of_dpr_and_backend() {
        let a = FrameIr::default();
        let mut b = a;
        b.dpr = 2;
        assert_eq!(a.semantic_digest(), b.semantic_digest());
        assert_ne!(
            a.semantic_digest(),
            FrameIr { phase: 0.7, ..a }.semantic_digest()
        );
    }
    #[test]
    fn invalid_ir_rejected() {
        for ir in [
            FrameIr {
                dpr: 0,
                ..FrameIr::default()
            },
            FrameIr {
                logical_height: 0,
                ..FrameIr::default()
            },
            FrameIr {
                phase: f32::NAN,
                ..FrameIr::default()
            },
        ] {
            assert!(cpu_render(&ir).is_err());
        }
    }
    #[test]
    fn shader_parses_and_validates_with_naga() {
        let source = include_str!("../shaders/f1.wgsl");
        let module = naga::front::wgsl::parse_str(source).expect("WGSL syntax");
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .expect("WGSL semantic validation");
    }
    #[test]
    fn compare_falsifier() {
        let a = cpu_render(&FrameIr::default()).unwrap();
        let mut b = a.clone();
        b[0] += 5;
        let (stat, _) = diff_images(&a, &b).unwrap();
        assert_eq!(stat.max_channel_error, 5);
        assert_eq!(stat.differing_pixels, 1);
    }
}
