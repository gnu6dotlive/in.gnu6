//! Persistent GPU session with typed content-stamp invalidation.
use crate::{
    geometry_id,
    health::{DeviceHealth, HealthView},
    provider::{self, ProviderFingerprint},
    Device, GeometryId, Output,
};
use blobin_cuda02_provider::{Request, Sample};
use blobin_cuda03_resident::{cuda, vulkan};
use std::path::{Path, PathBuf};

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
/// Provenance snapshot tied to the same synchronous evaluation.
#[derive(Debug)]
pub struct ObservedFrame {
    pub output: Output,
    pub health: HealthView,
    pub provider: Option<ProviderFingerprint>,
}
pub struct ResidentBridge {
    device: Device,
    session: Option<Session>,
    shape: Option<((u32, u32), usize)>,
    stamp: Option<GeometryId>,
    cached: Option<Vec<Sample>>,
    cuda_path: Option<PathBuf>,
    provider: Option<ProviderFingerprint>,
    approved_sha256: Option<[u8; 32]>,
    health: DeviceHealth,
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
            cuda_path: None,
            provider: None,
            approved_sha256: None,
            health: DeviceHealth::legacy(),
        })
    }
    /// Managed sessions begin unverified and require explicit recovery.
    /// An approved SHA-256 digest is mandatory for CUDA native execution.
    pub fn new_managed(device: Device, approved_sha256: Option<[u8; 32]>) -> Result<Self, String> {
        if device == Device::Cuda && approved_sha256.is_none() {
            return Err("CUDA04D CUDA APPROVED HASH REQUIRED".into());
        }
        let mut bridge = Self::new(device)?;
        bridge.approved_sha256 = approved_sha256;
        bridge.health = DeviceHealth::managed();
        Ok(bridge)
    }
    pub fn health(&self) -> HealthView {
        self.health.view()
    }
    pub fn provider(&self) -> Option<ProviderFingerprint> {
        self.provider
    }
    pub fn report_device_lost(&mut self) -> Result<u64, String> {
        self.invalidate();
        self.health.lost()
    }
    /// Caller must have re-probed the backend. This API does not probe hardware itself.
    pub fn report_device_recovered(&mut self) -> Result<u64, String> {
        self.invalidate();
        self.health.recovered()
    }
    pub fn invalidate(&mut self) {
        self.session = None;
        self.shape = None;
        self.stamp = None;
        self.cached = None;
        self.cuda_path = None;
        self.provider = None;
    }
    pub fn resident_bytes(&self) -> u64 {
        self.session.as_ref().map_or(0, Session::bytes)
    }
    pub fn evaluate_observed(
        &mut self,
        req: &Request,
        library: Option<&Path>,
    ) -> Result<ObservedFrame, String> {
        let output = self.evaluate(req, library)?;
        Ok(ObservedFrame {
            output,
            health: self.health(),
            provider: self.provider(),
        })
    }
    pub fn evaluate(&mut self, req: &Request, library: Option<&Path>) -> Result<Output, String> {
        // A caller-operated health gate is checked before ANY cache hit.
        self.health.require_ready()?;
        let id = geometry_id(req)?;
        let shape = (req.dimensions(), req.primitives().len());
        let pinned = if self.device == Device::Cuda {
            let path = match library {
                Some(path) => path,
                None => {
                    self.invalidate();
                    return Err("CUDA04D CUDA PROVIDER REQUIRED".into());
                }
            };
            let checked = match provider::inspect(path) {
                Ok(p) => p,
                Err(e) => {
                    self.invalidate();
                    return Err(e);
                }
            };
            if self
                .approved_sha256
                .is_some_and(|expected| expected != checked.fingerprint.sha256)
            {
                self.invalidate();
                return Err("CUDA04D PROVIDER HASH NOT APPROVED".into());
            }
            let path_changed = self.cuda_path.as_deref() != Some(path);
            let bytes_changed = self.provider.is_some_and(|old| old != checked.fingerprint);
            if bytes_changed && !path_changed && self.approved_sha256.is_none() {
                self.invalidate();
                return Err("CUDA04D PROVIDER REPLACED WITHOUT APPROVAL".into());
            }
            if path_changed || bytes_changed {
                self.invalidate();
            }
            Some(checked)
        } else {
            None
        };
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
                Device::Cuda => {
                    let verified = pinned
                        .as_ref()
                        .ok_or("CUDA04D VERIFIED PROVIDER REQUIRED")?;
                    cuda::Resident::new(req, &verified.loader_path()).map(Session::Cuda)
                }
                Device::Vulkan => vulkan::Resident::new(req).map(Session::Vulkan),
                Device::Cpu => return Err("CUDA04B REQUIRES GPU".into()),
            };
            self.session = Some(match created {
                Ok(session) => session,
                Err(error) => {
                    let _ = self.report_device_lost();
                    return Err(error);
                }
            });
            self.shape = Some(shape);
            self.cuda_path = if self.device == Device::Cuda {
                library.map(Path::to_path_buf)
            } else {
                None
            };
            self.provider = pinned.as_ref().map(|p| p.fingerprint);
            Ok(())
        } else {
            self.session.as_mut().expect("session").update(req)
        };
        if let Err(e) = result {
            let _ = self.report_device_lost();
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
                let _ = self.report_device_lost();
                return Err("CUDA04D INVALID RESIDENT OUTPUT".into());
            }
            Err(e) => {
                let _ = self.report_device_lost();
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
