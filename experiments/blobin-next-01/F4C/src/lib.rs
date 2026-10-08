//! F4C: separate falsifiable full-frame F4 oracle and conservative dirty rectangles.
pub mod gpu;
use blobin_f4b_resilience::{minimal_damage, validate_damage, Damage};
#[derive(Clone, Copy, Debug)]
pub struct SceneKey {
    pub width: u32,
    pub height: u32,
    pub digest: u64,
}
impl SceneKey {
    pub fn validate(self) -> Result<(), String> {
        let n = (self.width as usize)
            .checked_mul(self.height as usize)
            .ok_or("F4C SIZE OVERFLOW")?;
        if self.width == 0 || self.height == 0 || n > 1_048_576 {
            return Err("F4C PIXEL BUDGET".into());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Options {
    pub radius: u32,
    pub reduced_motion: bool,
}
impl Options {
    pub fn validate(self) -> Result<(), String> {
        if self.radius > 8 {
            Err("F4C RADIUS BUDGET".into())
        } else {
            Ok(())
        }
    }
}
pub fn empty() -> Damage {
    Damage {
        x: 0,
        y: 0,
        w: 0,
        h: 0,
    }
}
pub fn full(k: SceneKey) -> Damage {
    Damage {
        x: 0,
        y: 0,
        w: k.width,
        h: k.height,
    }
}
pub fn area(d: Damage) -> u64 {
    d.w as u64 * d.h as u64
}
pub fn union(a: Damage, b: Damage) -> Damage {
    if a.is_empty() {
        return b;
    }
    if b.is_empty() {
        return a;
    }
    let x = a.x.min(b.x);
    let y = a.y.min(b.y);
    Damage {
        x,
        y,
        w: (a.x + a.w).max(b.x + b.w) - x,
        h: (a.y + a.h).max(b.y + b.h) - y,
    }
}
pub fn expand(d: Damage, r: u32, k: SceneKey) -> Damage {
    if d.is_empty() {
        return empty();
    }
    let x = d.x.saturating_sub(r);
    let y = d.y.saturating_sub(r);
    let right = d.x.saturating_add(d.w).saturating_add(r).min(k.width);
    let bottom = d.y.saturating_add(d.h).saturating_add(r).min(k.height);
    Damage {
        x,
        y,
        w: right - x,
        h: bottom - y,
    }
}
/// For unchanged geometry: shadow inherits source damage; blur/bloom need input-luma radius.
/// Echo needs the full frame when a prior frame exists, even if source is unchanged.
pub fn invalidations(d: Damage, k: SceneKey, o: Options, history: bool) -> [Damage; 4] {
    let local = expand(d, o.radius, k);
    [
        d,
        local,
        local,
        if history && !o.reduced_motion {
            full(k)
        } else {
            local
        },
    ]
}
pub fn verify_damage(
    old: &[u8],
    new: &[u8],
    k: SceneKey,
    declared: Damage,
) -> Result<Damage, String> {
    k.validate()?;
    declared.validate(k.width, k.height)?;
    validate_damage(old, new, k.width, k.height, declared)?;
    minimal_damage(old, new, k.width, k.height)
}
fn smooth(src: &[f32], w: usize, h: usize, r: usize) -> Vec<f32> {
    if r == 0 {
        return src.to_vec();
    }
    let mut tmp = vec![0.; src.len()];
    let mut out = vec![0.; src.len()];
    for y in 0..h {
        for x in 0..w {
            let a = x.saturating_sub(r);
            let b = (x + r).min(w - 1);
            let mut sum = 0.0;
            for i in a..=b {
                sum += src[y * w + i];
            }
            tmp[y * w + x] = sum / (b - a + 1) as f32;
        }
    }
    for y in 0..h {
        for x in 0..w {
            let a = y.saturating_sub(r);
            let b = (y + r).min(h - 1);
            let mut sum = 0.0;
            for i in a..=b {
                sum += tmp[i * w + x];
            }
            out[y * w + x] = sum / (b - a + 1) as f32;
        }
    }
    out
}
/// Independent CPU full-frame reference; shares only the stated F4 numeric contract.
pub fn reference(
    k: SceneKey,
    o: Options,
    source: &[u8],
    mask: &[u8],
    history: Option<&[u8]>,
) -> Result<[Vec<u8>; 4], String> {
    k.validate()?;
    o.validate()?;
    let n = k.width as usize * k.height as usize;
    if source.len() != n * 4 || mask.len() != n || mask.iter().any(|&m| m > 1) {
        return Err("F4C INPUT SIZE/MASK".into());
    }
    if let Some(v) = history {
        if v.len() != source.len() {
            return Err("F4C HISTORY SIZE".into());
        }
    }
    let w = k.width as usize;
    let h = k.height as usize;
    let mut dst = source.to_vec();
    let luma: Vec<f32> = source
        .chunks_exact(4)
        .map(|p| (p[0] as f32 + p[1] as f32 + p[2] as f32) / (255.0 * 3.0))
        .collect();
    for y in 2..h {
        for x in 2..w {
            let i = y * w + x;
            if mask[(y - 2) * w + x - 2] != 0 && mask[i] == 0 {
                for byte in &mut dst[i * 4..i * 4 + 3] {
                    *byte = byte.saturating_sub(12);
                }
            }
        }
    }
    let shadow = dst.clone();
    let v = smooth(&luma, w, h, o.radius as usize);
    for i in 0..n {
        if mask[i] != 0 {
            let add = (v[i] * 10.0).round() as u8;
            for pixel in &mut dst[i * 4..i * 4 + 3] {
                *pixel = pixel.saturating_add(add);
            }
        }
    }
    let blurred = dst.clone();
    let highlights: Vec<f32> = luma.iter().map(|v| (v - 0.45).max(0.0)).collect();
    let glow = smooth(&highlights, w, h, o.radius as usize);
    for i in 0..n {
        let add = (glow[i] * 18.0).round() as u8;
        for pixel in &mut dst[i * 4..i * 4 + 3] {
            *pixel = pixel.saturating_add(add);
        }
    }
    let bloom = dst.clone();
    if !o.reduced_motion {
        if let Some(prev) = history {
            for (now, old) in dst.iter_mut().zip(prev) {
                *now = ((*now as u16 * 3 + *old as u16) / 4) as u8;
            }
        }
    }
    Ok([shadow, blurred, bloom, dst])
}
#[cfg(test)]
mod tests {
    use super::*;
    use blobin_f2_topology::{cpu_mask, qualification_trace};
    use blobin_f3_material_height::{cpu_render, Params};
    use blobin_f4_effects::{process, Plan, Rect, Temporal};
    #[test]
    fn independent_reference_matches_f4_two_frames() {
        for s in qualification_trace().unwrap() {
            let k = SceneKey {
                width: s.width(),
                height: s.height(),
                digest: s.digest(),
            };
            let opts = Options {
                radius: 3,
                reduced_motion: false,
            };
            let m = cpu_mask(s).unwrap();
            let mut temporal = Temporal::default();
            let p = Plan {
                dirty: Rect {
                    x: 0,
                    y: 0,
                    w: s.width(),
                    h: s.height(),
                },
                ..Default::default()
            };
            let a = cpu_render(
                s,
                Params {
                    heat_strength: 0.62,
                    ..Default::default()
                },
            )
            .unwrap();
            let stages = reference(k, opts, &a.rgba, &m, None).unwrap();
            assert_eq!(
                stages[3],
                process(
                    s,
                    Params {
                        heat_strength: 0.62,
                        ..Default::default()
                    },
                    &p,
                    &mut temporal
                )
                .unwrap()
                .rgba
            );
            let b = cpu_render(
                s,
                Params {
                    heat_strength: 0.18,
                    ..Default::default()
                },
            )
            .unwrap();
            let stages2 = reference(k, opts, &b.rgba, &m, Some(&stages[3])).unwrap();
            assert_eq!(
                stages2[3],
                process(
                    s,
                    Params {
                        heat_strength: 0.18,
                        ..Default::default()
                    },
                    &p,
                    &mut temporal
                )
                .unwrap()
                .rgba
            );
        }
    }
    #[test]
    fn halo_and_zero_idle() {
        let k = SceneKey {
            width: 256,
            height: 192,
            digest: 1,
        };
        let o = Options {
            radius: 3,
            reduced_motion: false,
        };
        assert!(invalidations(empty(), k, o, false)
            .iter()
            .all(|d| d.is_empty()));
        let d = Damage {
            x: 20,
            y: 30,
            w: 12,
            h: 10,
        };
        let r = invalidations(d, k, o, false);
        assert_eq!(r[0], d);
        assert_eq!(
            r[1],
            Damage {
                x: 17,
                y: 27,
                w: 18,
                h: 16
            }
        );
        assert_eq!(r[2], r[1]);
        assert_eq!(r[3], r[1]);
        assert_eq!(invalidations(empty(), k, o, true)[3], full(k));
    }
    #[test]
    fn invalid_damage_rejected() {
        let k = SceneKey {
            width: 128,
            height: 96,
            digest: 1,
        };
        let old = vec![50; 128 * 96 * 4];
        let d = Damage {
            x: 10,
            y: 11,
            w: 13,
            h: 8,
        };
        let new = blobin_f4b_resilience::overlay(&old, 128, 96, d, 13).unwrap();
        assert_eq!(verify_damage(&old, &new, k, d).unwrap(), d);
        assert!(verify_damage(
            &old,
            &new,
            k,
            Damage {
                x: 11,
                y: 11,
                w: 12,
                h: 8
            }
        )
        .is_err());
    }
    #[test]
    fn reduced_motion_has_no_history_halo() {
        let k = SceneKey {
            width: 256,
            height: 192,
            digest: 1,
        };
        let o = Options {
            radius: 3,
            reduced_motion: true,
        };
        assert!(invalidations(empty(), k, o, true)
            .iter()
            .all(|d| d.is_empty()));
    }
    #[test]
    fn shader_parses_and_validates() {
        let m = naga::front::wgsl::parse_str(include_str!("../shaders/f4c.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&m)
        .unwrap();
    }
}
