//! Persistent GPU session with typed content-stamp invalidation.
use crate::{geometry_id, Device, GeometryId, Output};
use blobin_cuda02_provider::{Request, Sample};
use blobin_cuda03_resident::{cuda, vulkan};
use std::path::Path;

enum Session {
    Cuda(cuda::Resident),
    Vulkan(vulkan::Resident),
}
impl Session {
    fn update(&mut self, req: &Request) -> Result<(), String> {
        match self {
            Self::Cuda(s) => s.update(req),
            Self::Vulkan(s) => s.update(req),
        }
    }
    fn compute(&mut self) -> Result<Vec<Sample>, String> {
        match self {
            Self::Cuda(s) => {
                s.launch()?;
                s.readback()
            }
            Self::Vulkan(s) => {
                s.launch()?;
                s.readback()
            }
        }
    }
    fn bytes(&self) -> u64 {
        match self {
            Self::Cuda(s) => s.resident_bytes(),
            Self::Vulkan(s) => s.resident_bytes(),
        }
    }
}
pub struct ResidentBridge {
    device: Device,
    session: Option<Session>,
    shape: Option<((u32, u32), usize)>,
    stamp: Option<GeometryId>,
    cached: Option<Vec<Sample>>,
}
impl ResidentBridge {
    pub fn new(device: Device) -> Result<Self, String> {
        if device == Device::Cpu {
            return Err("CUDA04B REQUIRES GPU".into());
        }
        Ok(Self {
            device,
            session: None,
            shape: None,
            stamp: None,
            cached: None,
        })
    }
    pub fn invalidate(&mut self) {
        self.session = None;
        self.shape = None;
        self.stamp = None;
        self.cached = None;
    }
    pub fn resident_bytes(&self) -> u64 {
        self.session.as_ref().map_or(0, Session::bytes)
    }
    pub fn evaluate(&mut self, req: &Request, library: Option<&Path>) -> Result<Output, String> {
        let id = geometry_id(req)?;
        let shape = (req.dimensions(), req.primitives().len());
        if self.stamp == Some(id) {
            if let (Some(_), Some(samples)) = (&self.session, &self.cached) {
                return Ok(Output {
                    scene: req.authority(),
                    geometry: id,
                    selected: self.device,
                    samples: samples.clone(),
                    cache_hit: true,
                });
            }
        }
        let replace = self.shape != Some(shape) || self.session.is_none();
        // Revoke the old result before any possibly failing GPU operation.
        self.stamp = None;
        self.cached = None;
        let result = if replace {
            self.invalidate();
            let created = match self.device {
                Device::Cuda => Session::Cuda(cuda::Resident::new(
                    req,
                    library.ok_or("CUDA04B LIBRARY REQUIRED")?,
                )?),
                Device::Vulkan => Session::Vulkan(vulkan::Resident::new(req)?),
                Device::Cpu => return Err("CUDA04B REQUIRES GPU".into()),
            };
            self.session = Some(created);
            self.shape = Some(shape);
            Ok(())
        } else {
            self.session.as_mut().expect("session").update(req)
        };
        if let Err(e) = result {
            self.invalidate();
            return Err(e);
        }
        let computed = self.session.as_mut().expect("session").compute();
        let samples = match computed {
            Ok(s)
                if s.len() == shape.0 .0 as usize * shape.0 .1 as usize
                    && s.iter().all(|p| {
                        [p.distance, p.nx, p.ny, p.coverage]
                            .iter()
                            .all(|v| v.is_finite())
                    }) =>
            {
                s
            }
            Ok(_) => {
                self.invalidate();
                return Err("CUDA04B INVALID OUTPUT".into());
            }
            Err(e) => {
                self.invalidate();
                return Err(e);
            }
        };
        self.cached = Some(samples.clone());
        self.stamp = Some(id);
        Ok(Output {
            scene: req.authority(),
            geometry: id,
            selected: self.device,
            samples,
            cache_hit: false,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use blobin_cuda02_provider::generate_scene;
    use blobin_f2_topology::Scene;
    #[test]
    fn cpu_cannot_masquerade_as_resident_gpu() {
        assert!(ResidentBridge::new(Device::Cpu).is_err());
    }
    #[test]
    fn missing_cuda_library_fails_without_retaining_state() {
        let req = Request::from_f2(Scene::default(), generate_scene(16)).unwrap();
        let mut b = ResidentBridge::new(Device::Cuda).unwrap();
        assert!(b.evaluate(&req, None).is_err());
        assert_eq!(b.resident_bytes(), 0);
        b.invalidate();
        assert_eq!(b.resident_bytes(), 0);
    }
    #[test]
    fn shape_checks_dimensions_instead_of_pixel_count() {
        let a = ((256, 192), 16usize);
        let b = ((192, 256), 16usize);
        assert_ne!(a, b);
    }
}
