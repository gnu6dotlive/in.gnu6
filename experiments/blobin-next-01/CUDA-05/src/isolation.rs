//! CUDA-05C process boundary. No GPU allocations live in this supervisor.
use blobin_cuda04_bridge::{provider, Device};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Injection {
    None,
    Crash,
    Hang,
}
pub struct Policy {
    device: Device,
    worker: PathBuf,
    worker_sha256: [u8; 32],
    native: Option<PathBuf>,
    approved: Option<[u8; 32]>,
    timeout: Duration,
}
impl Policy {
    pub fn new(
        device: Device,
        worker: PathBuf,
        worker_sha256: [u8; 32],
        native: Option<PathBuf>,
        approved: Option<[u8; 32]>,
        timeout: Duration,
    ) -> Result<Self, String> {
        if device == Device::Cpu || !worker.is_absolute() || !worker.is_file() {
            return Err("CUDA05C INVALID GPU WORKER POLICY".into());
        }
        if device == Device::Cuda
            && (native.as_ref().is_none_or(|p| !p.is_absolute()) || approved.is_none())
        {
            return Err("CUDA05C MISSING CUDA APPROVAL".into());
        }
        if timeout < Duration::from_millis(200) || timeout > Duration::from_secs(20) {
            return Err("CUDA05C TIMEOUT POLICY OUT OF RANGE".into());
        }
        Ok(Self {
            device,
            worker,
            worker_sha256,
            native,
            approved,
            timeout,
        })
    }
}
pub struct IsolatedSupervisor {
    policy: Policy,
    generation: u64,
    active: bool,
    launches: u64,
}
impl IsolatedSupervisor {
    pub fn new(policy: Policy) -> Self {
        Self {
            policy,
            generation: 0,
            active: false,
            launches: 0,
        }
    }
    pub fn generation(&self) -> Option<u64> {
        self.active.then_some(self.generation)
    }
    pub fn launches(&self) -> u64 {
        self.launches
    }
    pub fn revoke(&mut self) {
        self.active = false;
    }
    fn advance(&mut self) -> Result<u64, String> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or("CUDA05C GENERATION EXHAUSTED")?;
        Ok(self.generation)
    }
    fn invoke(&mut self, injection: Injection) -> Result<(), String> {
        if provider::inspect(&self.policy.worker)?.fingerprint.sha256 != self.policy.worker_sha256 {
            return Err("CUDA05C WORKER HASH NOT APPROVED".into());
        }
        self.launches = self
            .launches
            .checked_add(1)
            .ok_or("CUDA05C LAUNCH COUNTER EXHAUSTED")?;
        let mut command = Command::new(&self.policy.worker);
        command.arg("--backend").arg(match self.policy.device {
            Device::Cuda => "cuda",
            Device::Vulkan => "vulkan",
            Device::Cpu => return Err("CUDA05C CPU DENIED".into()),
        });
        if let Some(path) = &self.policy.native {
            if self.policy.device == Device::Cuda {
                command.arg("--cuda-lib").arg(path);
            }
        }
        if let Some(hash) = self.policy.approved {
            if self.policy.device == Device::Cuda {
                let digest = hash.iter().map(|b| format!("{b:02x}")).collect::<String>();
                command.arg("--sha256").arg(digest);
            }
        }
        match injection {
            Injection::Crash => {
                command.args(["--inject-fault", "crash"]);
            }
            Injection::Hang => {
                command.args(["--inject-fault", "hang"]);
            }
            Injection::None => {}
        }
        // No shell, caller commands or inherited STDIN. Child is a disposable context.
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("CUDA05C SPAWN {e}"))?;
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let output = child
                        .wait_with_output()
                        .map_err(|e| format!("CUDA05C REAP {e}"))?;
                    if !status.success() {
                        return Err("CUDA05C WORKER CRASH OR ERROR".into());
                    }
                    let expected = match self.policy.device {
                        Device::Cuda => "CUDA05C_WORKER_PASS backend=Cuda ",
                        Device::Vulkan => "CUDA05C_WORKER_PASS backend=Vulkan ",
                        Device::Cpu => unreachable!(),
                    };
                    let line = std::str::from_utf8(&output.stdout)
                        .map_err(|_| "CUDA05C NON-UTF8 WORKER OUTPUT")?;
                    if !line.starts_with(expected) || !line.contains("parity=PASS cache_miss=PASS")
                    {
                        return Err("CUDA05C UNQUALIFIED WORKER RESPONSE".into());
                    }
                    return Ok(());
                }
                Ok(None) if started.elapsed() >= self.policy.timeout => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err("CUDA05C WORKER TIMEOUT KILLED".into());
                }
                Ok(None) => thread::sleep(Duration::from_millis(10)),
                Err(e) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("CUDA05C WAIT ERROR {e}"));
                }
            }
        }
    }
    pub fn acquire(&mut self, injection: Injection) -> Result<u64, String> {
        self.revoke();
        if let Err(e) = self.invoke(injection) {
            self.revoke();
            return Err(e);
        }
        let generation = self.advance()?;
        self.active = true;
        Ok(generation)
    }
    pub fn render(&mut self, injection: Injection) -> Result<u64, String> {
        if !self.active {
            return Err("CUDA05C NO ACTIVE GRANT".into());
        }
        if let Err(e) = self.invoke(injection) {
            self.revoke();
            return Err(e);
        }
        Ok(self.generation)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn policy_rejects_cpu_nonabsolute_worker_and_bad_deadlines() {
        assert!(Policy::new(
            Device::Cpu,
            PathBuf::from("/bin/true"),
            [0; 32],
            None,
            None,
            Duration::from_secs(1)
        )
        .is_err());
        assert!(Policy::new(
            Device::Vulkan,
            PathBuf::from("relative"),
            [0; 32],
            None,
            None,
            Duration::from_secs(1)
        )
        .is_err());
        assert!(Policy::new(
            Device::Vulkan,
            PathBuf::from("/bin/true"),
            [0; 32],
            None,
            None,
            Duration::from_millis(10)
        )
        .is_err());
        assert!(Policy::new(
            Device::Cuda,
            PathBuf::from("/bin/true"),
            [0; 32],
            None,
            None,
            Duration::from_secs(1)
        )
        .is_err());
    }
    #[test]
    fn revocation_denies_render_without_worker_execution() {
        let policy = Policy::new(
            Device::Vulkan,
            PathBuf::from("/bin/true"),
            [0; 32],
            None,
            None,
            Duration::from_secs(1),
        )
        .unwrap();
        let mut s = IsolatedSupervisor::new(policy);
        assert!(s.render(Injection::None).is_err());
        assert_eq!(s.launches(), 0);
        s.revoke();
        assert_eq!(s.generation(), None);
    }
}
