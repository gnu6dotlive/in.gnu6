//! CUDA-05 bounded ingress: sealed policy, supervised device probe and revocable grants.
//! No process credential, OS hardware attestation or sandbox claim is implied.
use blobin_cuda02_provider::Request;
use blobin_cuda04_bridge::{
    provider,
    resident::{ObservedFrame, ResidentBridge},
    Device,
};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProbeState {
    Ready,
    Unavailable,
}
/// This trait is a test seam. A caller-provided implementation is NOT a trusted probe.
pub trait Probe {
    fn check(&self, device: Device) -> Result<ProbeState, String>;
}

pub struct Permit {
    generation: u64,
    device: Device,
}
impl Permit {
    pub fn generation(&self) -> u64 {
        self.generation
    }
}

pub struct Ingress {
    bridge: ResidentBridge,
    device: Device,
    provider_path: Option<PathBuf>,
    expected_hash: Option<[u8; 32]>,
    generation: Option<u64>,
}
impl Ingress {
    pub fn new(
        device: Device,
        provider_path: Option<PathBuf>,
        expected_hash: Option<[u8; 32]>,
    ) -> Result<Self, String> {
        if device == Device::Cpu {
            return Err("CUDA05 GPU ONLY".into());
        }
        if device == Device::Cuda
            && (provider_path.as_ref().is_none_or(|p| !p.is_absolute()) || expected_hash.is_none())
        {
            return Err("CUDA05 APPROVED CUDA PATH AND HASH REQUIRED".into());
        }
        Ok(Self {
            bridge: ResidentBridge::new_managed(device, expected_hash)?,
            device,
            provider_path,
            expected_hash,
            generation: None,
        })
    }
    pub fn revoke(&mut self) -> Result<(), String> {
        self.generation = None;
        self.bridge.report_device_lost()?;
        Ok(())
    }
    pub fn acquire(&mut self, probe: &impl Probe) -> Result<Permit, String> {
        self.revoke()?;
        if probe.check(self.device)? != ProbeState::Ready {
            return Err("CUDA05 DEVICE UNAVAILABLE".into());
        }
        if self.device == Device::Cuda {
            self.verify_provider()?;
        }
        let generation = self.bridge.report_device_recovered()?;
        self.generation = Some(generation);
        Ok(Permit {
            generation,
            device: self.device,
        })
    }
    fn verify_provider(&self) -> Result<(), String> {
        let path = self.provider_path.as_deref().ok_or("CUDA05 NO PATH")?;
        let expected = self.expected_hash.ok_or("CUDA05 NO HASH")?;
        if provider::inspect(path)?.fingerprint.sha256 != expected {
            return Err("CUDA05 UNAPPROVED PROVIDER".into());
        }
        Ok(())
    }
    pub fn evaluate(&mut self, permit: &Permit, req: &Request) -> Result<ObservedFrame, String> {
        if self.generation != Some(permit.generation) || permit.device != self.device {
            return Err("CUDA05 STALE OR FOREIGN PERMIT".into());
        }
        if self.device == Device::Cuda {
            if let Err(err) = self.verify_provider() {
                self.revoke()?;
                return Err(err);
            }
        }
        match self
            .bridge
            .evaluate_observed(req, self.provider_path.as_deref())
        {
            Ok(out) if out.health.generation == permit.generation => Ok(out),
            Ok(_) => {
                self.revoke()?;
                Err("CUDA05 HEALTH GENERATION DRIFT".into())
            }
            Err(e) => {
                let _ = self.revoke();
                Err(e)
            }
        }
    }
    pub fn health_generation(&self) -> Option<u64> {
        self.generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixed(ProbeState);
    impl Probe for Fixed {
        fn check(&self, _: Device) -> Result<ProbeState, String> {
            Ok(self.0)
        }
    }
    #[test]
    fn denied_and_revoked_generations() {
        let mut i = Ingress::new(Device::Vulkan, None, None).unwrap();
        assert!(i.acquire(&Fixed(ProbeState::Unavailable)).is_err());
        let old = i.acquire(&Fixed(ProbeState::Ready)).unwrap();
        assert_eq!(i.health_generation(), Some(old.generation()));
        i.revoke().unwrap();
        assert_eq!(i.health_generation(), None);
        let newer = i.acquire(&Fixed(ProbeState::Ready)).unwrap();
        assert!(newer.generation() > old.generation());
        let req = Request::from_f2(
            blobin_f2_topology::Scene::default(),
            blobin_cuda02_provider::generate_scene(16),
        )
        .unwrap();
        assert!(i.evaluate(&old, &req).is_err());
    }
    #[test]
    fn denied_missing_cuda_policy() {
        assert!(Ingress::new(Device::Cuda, None, None).is_err());
        assert!(Ingress::new(Device::Cpu, None, None).is_err());
    }
}
