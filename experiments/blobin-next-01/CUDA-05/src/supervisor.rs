//! GPU-resident supervised gate: real dispatch proof before every operation.
//! Probes prove current execution, not ongoing physical liveness or OS privilege.
use crate::{Ingress, Permit, Probe, ProbeState};
use blobin_cuda02_provider::{compare, cpu, generate_scene, Request};
use blobin_cuda04_bridge::{
    provider,
    resident::{ObservedFrame, ResidentBridge},
    Device,
};
use blobin_f2_topology::Scene;
use std::path::PathBuf;

pub struct Policy {
    device: Device,
    cuda_library: Option<PathBuf>,
    approved_hash: Option<[u8; 32]>,
}
impl Policy {
    pub fn new(
        device: Device,
        cuda_library: Option<PathBuf>,
        approved_hash: Option<[u8; 32]>,
    ) -> Result<Self, String> {
        if device == Device::Cpu {
            return Err("CUDA05B GPU ONLY".into());
        }
        if device == Device::Cuda
            && (cuda_library.as_ref().is_none_or(|p| !p.is_absolute()) || approved_hash.is_none())
        {
            return Err("CUDA05B CUDA PATH AND APPROVED SHA REQUIRED".into());
        }
        Ok(Self {
            device,
            cuda_library,
            approved_hash,
        })
    }
}
struct QualifiedReady;
impl Probe for QualifiedReady {
    fn check(&self, _: Device) -> Result<ProbeState, String> {
        Ok(ProbeState::Ready)
    }
}
pub struct Supervisor {
    policy: Policy,
    ingress: Ingress,
    permit: Option<Permit>,
    challenges: u64,
}
impl Supervisor {
    pub fn new(policy: Policy) -> Result<Self, String> {
        let ingress = Ingress::new(
            policy.device,
            policy.cuda_library.clone(),
            policy.approved_hash,
        )?;
        Ok(Self {
            policy,
            ingress,
            permit: None,
            challenges: 0,
        })
    }
    pub fn generation(&self) -> Option<u64> {
        self.ingress.health_generation()
    }
    pub fn challenge_count(&self) -> u64 {
        self.challenges
    }
    pub fn revoke(&mut self) -> Result<(), String> {
        self.permit = None;
        self.ingress.revoke()
    }
    fn challenge(&mut self) -> Result<(), String> {
        if self.challenges == u64::MAX {
            return Err("CUDA05B PROBE COUNTER EXHAUSTED".into());
        }
        if self.policy.device == Device::Cuda {
            let path = self
                .policy
                .cuda_library
                .as_deref()
                .ok_or("CUDA05B MISSING LIBRARY")?;
            let expected = self
                .policy
                .approved_hash
                .ok_or("CUDA05B MISSING APPROVAL")?;
            if provider::inspect(path)?.fingerprint.sha256 != expected {
                return Err("CUDA05B CHALLENGE UNAPPROVED PROVIDER".into());
            }
        }
        let req = Request::from_f2(Scene::default(), generate_scene(16))?;
        let reference = cpu(&req)?;
        // Fresh GPU session proves a real native CUDA/Vulkan dispatch, GPU synchronization
        // and full-frame numerical parity independently of the cached runtime.
        let mut challenger =
            ResidentBridge::new_managed(self.policy.device, self.policy.approved_hash)?;
        challenger.report_device_recovered()?;
        let actual = challenger.evaluate_observed(&req, self.policy.cuda_library.as_deref())?;
        if actual.output.cache_hit || actual.output.selected != self.policy.device {
            return Err("CUDA05B GPU CHALLENGE BYPASSED".into());
        }
        compare(&reference, &actual.output.samples)?;
        self.challenges += 1;
        Ok(())
    }
    pub fn acquire(&mut self) -> Result<u64, String> {
        self.revoke()?;
        if let Err(error) = self.challenge() {
            let _ = self.revoke();
            return Err(error);
        }
        let permit = self.ingress.acquire(&QualifiedReady)?;
        let generation = permit.generation();
        self.permit = Some(permit);
        Ok(generation)
    }
    pub fn render(&mut self, req: &Request) -> Result<ObservedFrame, String> {
        if self.permit.is_none() {
            return Err("CUDA05B NO ACTIVE PERMIT".into());
        }
        // Recheck real device liveness and provider bytes before EACH use, including hits.
        if let Err(error) = self.challenge() {
            let _ = self.revoke();
            return Err(error);
        }
        let result = self
            .ingress
            .evaluate(self.permit.as_ref().expect("checked"), req);
        if result.is_err() {
            let _ = self.revoke();
        }
        result
    }
}
