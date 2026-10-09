//! Caller-reported device-health generation. Never a hardware watchdog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HealthState {
    Unverified,
    Ready,
    Lost,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HealthView {
    pub generation: u64,
    pub state: HealthState,
}
pub struct DeviceHealth {
    view: HealthView,
}
impl DeviceHealth {
    pub fn legacy() -> Self {
        Self {
            view: HealthView {
                generation: 1,
                state: HealthState::Ready,
            },
        }
    }
    pub fn managed() -> Self {
        Self {
            view: HealthView {
                generation: 0,
                state: HealthState::Unverified,
            },
        }
    }
    pub fn view(&self) -> HealthView {
        self.view
    }
    pub fn require_ready(&self) -> Result<u64, String> {
        if self.view.state != HealthState::Ready {
            return Err("CUDA04D DEVICE NOT VERIFIED READY".into());
        }
        Ok(self.view.generation)
    }
    fn transition(&mut self, state: HealthState) -> Result<u64, String> {
        if self.view.generation == u64::MAX {
            self.view.state = HealthState::Lost;
            return Err("CUDA04D GENERATION EXHAUSTED".into());
        }
        self.view.generation += 1;
        self.view.state = state;
        Ok(self.view.generation)
    }
    pub fn lost(&mut self) -> Result<u64, String> {
        self.transition(HealthState::Lost)
    }
    /// 'recovered' means the caller has re-probed/re-authorized the device.
    /// This type does not itself establish driver health.
    pub fn recovered(&mut self) -> Result<u64, String> {
        self.transition(HealthState::Ready)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn managed_is_fail_closed_until_explicit_recovery() {
        let mut h = DeviceHealth::managed();
        assert!(h.require_ready().is_err());
        assert_eq!(h.recovered().unwrap(), 1);
        assert_eq!(h.require_ready().unwrap(), 1);
        assert_eq!(h.lost().unwrap(), 2);
        assert!(h.require_ready().is_err());
        assert_eq!(h.recovered().unwrap(), 3);
    }
    #[test]
    fn generation_overflow_is_fail_closed() {
        let mut h = DeviceHealth {
            view: HealthView {
                generation: u64::MAX,
                state: HealthState::Ready,
            },
        };
        assert!(h.recovered().is_err());
        assert_eq!(h.view().state, HealthState::Lost);
    }
}
