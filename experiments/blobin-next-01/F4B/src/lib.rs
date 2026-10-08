//! F4B clean-room hard-resilience lab: persistent GPU region compositing and CPU continuity.
//! F4's effects are still FULL-FRAME. This module does not claim incremental effect evaluation.
pub mod gpu;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Damage {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}
impl Damage {
    pub fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }
    pub fn is_full(self, width: u32, height: u32) -> bool {
        self.x == 0 && self.y == 0 && self.w == width && self.h == height
    }
    pub fn contains(self, x: u32, y: u32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.w && y < self.y + self.h
    }
    pub fn validate(self, width: u32, height: u32) -> Result<(), String> {
        let right = self.x.checked_add(self.w).ok_or("F4B DAMAGE OVERFLOW X")?;
        let bottom = self.y.checked_add(self.h).ok_or("F4B DAMAGE OVERFLOW Y")?;
        if width == 0 || height == 0 || right > width || bottom > height {
            return Err("F4B DAMAGE OUTSIDE VIEWPORT".into());
        }
        Ok(())
    }
}
pub fn minimal_damage(
    previous: &[u8],
    next: &[u8],
    width: u32,
    height: u32,
) -> Result<Damage, String> {
    check_lengths(previous, next, width, height)?;
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut changed = false;
    for y in 0..height {
        for x in 0..width {
            let i = ((y * width + x) * 4) as usize;
            if previous[i..i + 4] != next[i..i + 4] {
                changed = true;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }
    }
    if !changed {
        return Ok(Damage {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
        });
    }
    Ok(Damage {
        x: min_x,
        y: min_y,
        w: max_x - min_x + 1,
        h: max_y - min_y + 1,
    })
}
fn check_lengths(before: &[u8], after: &[u8], width: u32, height: u32) -> Result<(), String> {
    let expected = (width as usize)
        .checked_mul(height as usize)
        .and_then(|x| x.checked_mul(4))
        .ok_or("F4B PIXEL LENGTH OVERFLOW")?;
    if width == 0
        || height == 0
        || width as usize * height as usize > 1_048_576
        || before.len() != expected
        || after.len() != expected
    {
        return Err("F4B IMAGE LENGTH/BUDGET".into());
    }
    Ok(())
}
/// Fail closed on a missing dirty bound. Unlike an ordinary scissor,
/// this oracle checks the actual source and target bytes first.
pub fn validate_damage(
    before: &[u8],
    after: &[u8],
    width: u32,
    height: u32,
    damage: Damage,
) -> Result<usize, String> {
    check_lengths(before, after, width, height)?;
    damage.validate(width, height)?;
    let mut changes = 0;
    for y in 0..height {
        for x in 0..width {
            let index = ((y * width + x) * 4) as usize;
            if before[index..index + 4] != after[index..index + 4] {
                if !damage.contains(x, y) {
                    return Err(format!("F4B UNDERDECLARED_DAMAGE x={x} y={y}"));
                }
                changes += 1;
            }
        }
    }
    if damage.is_empty() && changes != 0 {
        return Err("F4B NONEMPTY_IDLE".into());
    }
    Ok(changes)
}
/// Presentational overlay for F4B: creates an isolated dirty patch from a *precomputed*
/// F4 result, not a claim about partial shadow/bloom evaluation.
pub fn overlay(
    prev: &[u8],
    width: u32,
    height: u32,
    rect: Damage,
    seed: u8,
) -> Result<Vec<u8>, String> {
    rect.validate(width, height)?;
    check_lengths(prev, prev, width, height)?;
    let mut out = prev.to_vec();
    for y in rect.y..rect.y + rect.h {
        for x in rect.x..rect.x + rect.w {
            let i = ((y * width + x) * 4) as usize;
            out[i] = out[i].wrapping_add(seed);
            out[i + 1] = out[i + 1].wrapping_add(1);
            out[i + 2] = out[i + 2].wrapping_add(3);
        }
    }
    Ok(out)
}
#[cfg(test)]
mod tests {
    use super::*;
    const W: u32 = 128;
    const H: u32 = 96;
    fn blank() -> Vec<u8> {
        vec![41u8; (W * H * 4) as usize]
    }
    #[test]
    fn no_change_means_zero_damage() {
        let a = blank();
        assert_eq!(
            minimal_damage(&a, &a, W, H).unwrap(),
            Damage {
                x: 0,
                y: 0,
                w: 0,
                h: 0
            }
        );
        assert_eq!(
            validate_damage(
                &a,
                &a,
                W,
                H,
                Damage {
                    x: 0,
                    y: 0,
                    w: 0,
                    h: 0
                }
            )
            .unwrap(),
            0
        );
    }
    #[test]
    fn exact_rect_fills_and_minimal_bbox() {
        let a = blank();
        let r = Damage {
            x: 19,
            y: 25,
            w: 11,
            h: 9,
        };
        let b = overlay(&a, W, H, r, 1).unwrap();
        assert_eq!(minimal_damage(&a, &b, W, H).unwrap(), r);
        assert_eq!(validate_damage(&a, &b, W, H, r).unwrap(), 99);
    }
    #[test]
    fn underdeclared_halo_is_detected() {
        let a = blank();
        let big = Damage {
            x: 8,
            y: 8,
            w: 16,
            h: 16,
        };
        let b = overlay(&a, W, H, big, 13).unwrap();
        let bad = Damage {
            x: 9,
            y: 9,
            w: 14,
            h: 14,
        };
        assert!(validate_damage(&a, &b, W, H, bad)
            .unwrap_err()
            .contains("UNDERDECLARED"));
    }
    #[test]
    fn overflow_outside_and_lengths_fail_closed() {
        let a = blank();
        for r in [
            Damage {
                x: u32::MAX,
                y: 0,
                w: 12,
                h: 1,
            },
            Damage {
                x: 130,
                y: 0,
                w: 1,
                h: 1,
            },
            Damage {
                x: 0,
                y: 90,
                w: 2,
                h: 15,
            },
        ] {
            assert!(validate_damage(&a, &a, W, H, r).is_err());
        }
        assert!(minimal_damage(&a, &a[1..], W, H).is_err());
    }
    #[test]
    fn f4_scene_digest_survives_presentation_overlay() {
        let scene = blobin_f2_topology::Scene::default();
        let input = blobin_f3_material_height::Params::default();
        let base = blobin_f4_effects::process(
            scene,
            input,
            &blobin_f4_effects::Plan::default(),
            &mut blobin_f4_effects::Temporal::default(),
        )
        .unwrap();
        let original_digest = base.semantic_digest;
        let b = overlay(
            &base.rgba,
            scene.width(),
            scene.height(),
            Damage {
                x: 42,
                y: 42,
                w: 10,
                h: 10,
            },
            12,
        )
        .unwrap();
        assert!(validate_damage(
            &base.rgba,
            &b,
            scene.width(),
            scene.height(),
            Damage {
                x: 42,
                y: 42,
                w: 10,
                h: 10
            }
        )
        .is_ok());
        assert_eq!(original_digest, scene.digest());
        assert_eq!((base.components, base.holes), (2, 0));
    }
    #[test]
    fn wgsl_parse_and_validate() {
        let module = naga::front::wgsl::parse_str(include_str!("../shaders/roi.wgsl")).unwrap();
        naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
    }
}
