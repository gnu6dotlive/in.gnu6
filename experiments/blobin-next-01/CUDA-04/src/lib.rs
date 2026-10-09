//! CUDA-04: geometry identity and bounded runtime cache, independent of F2 identity.
pub mod resident;
use blobin_cuda02_provider::{Backend, Request, Sample, SceneAuthority};
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, VecDeque},
    path::Path,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GeometryId(pub [u8; 32]);
pub fn geometry_id(req: &Request) -> Result<GeometryId, String> {
    req.validate()?;
    let mut h = Sha256::new();
    h.update(b"blobin/geometry/v1\0");
    let (w, ht) = req.dimensions();
    h.update(w.to_le_bytes());
    h.update(ht.to_le_bytes());
    h.update((req.primitives().len() as u32).to_le_bytes());
    for p in req.primitives() {
        h.update(p.x.to_bits().to_le_bytes());
        h.update(p.y.to_bits().to_le_bytes());
        h.update(p.radius.to_bits().to_le_bytes());
        h.update(p.op.to_le_bytes());
    }
    Ok(GeometryId(h.finalize().into()))
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Device {
    Cpu,
    Vulkan,
    Cuda,
}
#[derive(Clone, Copy, Debug)]
pub enum Policy {
    Strict(Device),
    PreferCuda,
    PreferVulkan,
}
#[derive(Clone, Copy, Debug)]
pub struct Available {
    pub cuda: bool,
    pub vulkan: bool,
}
#[derive(Debug)]
pub struct Output {
    pub scene: SceneAuthority,
    pub geometry: GeometryId,
    pub selected: Device,
    pub samples: Vec<Sample>,
    pub cache_hit: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Key {
    geometry: GeometryId,
    device: Device,
}
pub struct Bridge {
    limit: usize,
    cache: HashMap<Key, Vec<Sample>>,
    lru: VecDeque<Key>,
}
impl Bridge {
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            cache: HashMap::new(),
            lru: VecDeque::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.cache.len()
    }
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
    pub fn clear(&mut self) {
        self.cache.clear();
        self.lru.clear();
    }
    fn touch(&mut self, key: Key) {
        self.lru.retain(|v| *v != key);
        self.lru.push_back(key);
    }
    pub fn evaluate(
        &mut self,
        req: &Request,
        policy: Policy,
        available: Available,
        cuda_library: Option<&Path>,
    ) -> Result<Output, String> {
        let geometry = geometry_id(req)?;
        let chain: &[Device] = match policy {
            Policy::Strict(Device::Cpu) => &[Device::Cpu],
            Policy::Strict(Device::Vulkan) => &[Device::Vulkan],
            Policy::Strict(Device::Cuda) => &[Device::Cuda],
            Policy::PreferCuda => &[Device::Cuda, Device::Vulkan, Device::Cpu],
            Policy::PreferVulkan => &[Device::Vulkan, Device::Cpu],
        };
        let mut errors = Vec::new();
        for &device in chain {
            if (device == Device::Cuda && (!available.cuda || cuda_library.is_none()))
                || (device == Device::Vulkan && !available.vulkan)
            {
                errors.push(format!("{device:?}: unavailable"));
                continue;
            }
            let key = Key { geometry, device };
            if let Some(samples) = self.cache.get(&key).cloned() {
                self.touch(key);
                return Ok(Output {
                    scene: req.authority(),
                    geometry,
                    selected: device,
                    samples,
                    cache_hit: true,
                });
            }
            let backend = match device {
                Device::Cpu => Backend::Cpu,
                Device::Vulkan => Backend::Vulkan,
                Device::Cuda => Backend::Cuda,
            };
            match blobin_cuda02_provider::evaluate(req, backend, cuda_library) {
                Ok(frame) if frame.backend == backend => {
                    let samples = frame.samples;
                    if self.limit > 0 {
                        if self.cache.len() >= self.limit {
                            if let Some(old) = self.lru.pop_front() {
                                self.cache.remove(&old);
                            }
                        }
                        self.cache.insert(key, samples.clone());
                        self.touch(key);
                    }
                    return Ok(Output {
                        scene: req.authority(),
                        geometry,
                        selected: device,
                        samples,
                        cache_hit: false,
                    });
                }
                Ok(_) => errors.push(format!("{device:?}: provider returned unexpected backend")),
                Err(e) => errors.push(format!("{device:?}: {e}")),
            }
        }
        Err(format!("CUDA04 NO BACKEND: {}", errors.join("; ")))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use blobin_cuda02_provider::generate_scene;
    use blobin_f2_topology::Scene;
    fn req(radius_delta: f32) -> Request {
        let mut p = generate_scene(16);
        p[0].radius += radius_delta;
        Request::from_f2(Scene::default(), p).unwrap()
    }
    #[test]
    fn digest_determinism_mutation_and_f2_separation() {
        let (a, b, c) = (req(0.0), req(0.0), req(0.08));
        assert_eq!(geometry_id(&a).unwrap(), geometry_id(&b).unwrap());
        assert_ne!(geometry_id(&a).unwrap(), geometry_id(&c).unwrap());
        assert_eq!(a.authority(), c.authority());
    }
    #[test]
    fn cpu_cache_hit_miss_eviction() {
        let mut b = Bridge::new(1);
        let avail = Available {
            cuda: false,
            vulkan: false,
        };
        assert!(
            !b.evaluate(&req(0.0), Policy::Strict(Device::Cpu), avail, None)
                .unwrap()
                .cache_hit
        );
        assert!(
            b.evaluate(&req(0.0), Policy::Strict(Device::Cpu), avail, None)
                .unwrap()
                .cache_hit
        );
        assert!(
            !b.evaluate(&req(0.08), Policy::Strict(Device::Cpu), avail, None)
                .unwrap()
                .cache_hit
        );
        assert!(
            !b.evaluate(&req(0.0), Policy::Strict(Device::Cpu), avail, None)
                .unwrap()
                .cache_hit
        );
        assert_eq!(b.len(), 1);
    }
    #[test]
    fn strict_fails_closed_auto_reports_fallback() {
        let mut b = Bridge::new(0);
        let avail = Available {
            cuda: false,
            vulkan: false,
        };
        assert!(b
            .evaluate(&req(0.0), Policy::Strict(Device::Cuda), avail, None)
            .is_err());
        assert!(b
            .evaluate(&req(0.0), Policy::Strict(Device::Vulkan), avail, None)
            .is_err());
        let out = b
            .evaluate(&req(0.0), Policy::PreferCuda, avail, None)
            .unwrap();
        assert_eq!(out.selected, Device::Cpu);
        assert!(!out.cache_hit);
    }
}
