//! CPU camera ray oracle. Y-up/right-handed, f64 normalized pixel centers.
//! Does NOT claim full CSG raymarch hit/miss qualification.
use crate::{mesh::Ray, GeometryError, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct PerspectiveCamera {
    pub eye: Vec3,
    forward: Vec3,
    right: Vec3,
    up: Vec3,
    pub vertical_fov: f64,
}
fn unit(v: Vec3) -> Result<Vec3, GeometryError> {
    if !v.finite() {
        return Err(GeometryError::InvalidInput);
    }
    let n = v.norm();
    if !n.is_finite() || n < 1e-10 {
        return Err(GeometryError::InvalidInput);
    }
    Ok(v.scale(1.0 / n))
}
impl PerspectiveCamera {
    pub fn look_at(
        eye: Vec3,
        target: Vec3,
        up: Vec3,
        vertical_fov: f64,
    ) -> Result<Self, GeometryError> {
        if !eye.finite()
            || !target.finite()
            || !up.finite()
            || !vertical_fov.is_finite()
            || !(0.01..std::f64::consts::PI - 0.01).contains(&vertical_fov)
        {
            return Err(GeometryError::InvalidInput);
        }
        let forward = unit(target.minus(eye))?;
        let right = unit(forward.cross(up))?;
        let actual_up = unit(right.cross(forward))?;
        Ok(Self {
            eye,
            forward,
            right,
            up: actual_up,
            vertical_fov,
        })
    }
    pub fn ray_for_pixel(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> Result<Ray, GeometryError> {
        if width == 0 || height == 0 || x >= width || y >= height {
            return Err(GeometryError::InvalidInput);
        }
        let tan = (self.vertical_fov * 0.5).tan();
        let aspect = (width as f64) / (height as f64);
        let u = (((x as f64) + 0.5) / (width as f64) * 2.0 - 1.0) * tan * aspect;
        let v = (1.0 - ((y as f64) + 0.5) / (height as f64) * 2.0) * tan;
        let direction = unit(
            self.forward
                .plus(self.right.scale(u))
                .plus(self.up.scale(v)),
        )?;
        Ray::new(self.eye, direction)
    }
}
