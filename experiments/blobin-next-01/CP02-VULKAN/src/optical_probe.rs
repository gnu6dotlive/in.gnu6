//! GPU05-ORACLE-02: opt-in renderer-local optical witness patch.
//! This state is NEVER authority over in.gnu6 semantic scene identity.
//! A real compositor-delivered wl_pointer button press triggers a bounded
//! solid-bright ROI pulse; no clock from here is an input-to-photon oracle.
use std::time::Duration;
pub const PULSE: Duration = Duration::from_millis(350);
const MIN_RESET: Duration = Duration::from_millis(900);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Bright(u64),
    Dark(u64),
}
#[derive(Debug, Default)]
pub struct OpticalProbe {
    active_trial: Option<u64>,
    last_trial: u64,
    bright_since: Option<Duration>,
    last_start: Option<Duration>,
    physical_pressed: bool,
}
impl OpticalProbe {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn on_button(
        &mut self,
        pressed: bool,
        now: Duration,
    ) -> Result<Option<Transition>, &'static str> {
        if !pressed {
            self.physical_pressed = false;
            return Ok(None);
        }
        if self.physical_pressed {
            return Ok(None);
        }
        self.physical_pressed = true;
        if self.active_trial.is_some() {
            return Ok(None);
        }
        if self
            .last_start
            .is_some_and(|prior| now.saturating_sub(prior) < MIN_RESET)
        {
            return Ok(None);
        }
        let seq = self
            .last_trial
            .checked_add(1)
            .ok_or("ORACLE02_TRIAL_ID_EXHAUSTED")?;
        self.last_trial = seq;
        self.active_trial = Some(seq);
        self.bright_since = Some(now);
        self.last_start = Some(now);
        Ok(Some(Transition::Bright(seq)))
    }
    pub fn tick(&mut self, now: Duration) -> Option<Transition> {
        let start = self.bright_since?;
        if now.saturating_sub(start) < PULSE {
            return None;
        }
        self.bright_since = None;
        let seq = self.active_trial.take()?;
        Some(Transition::Dark(seq))
    }
    pub fn is_bright(&self) -> bool {
        self.active_trial.is_some()
    }
    pub fn active_trial(&self) -> Option<u64> {
        self.active_trial
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn ms(v: u64) -> Duration {
        Duration::from_millis(v)
    }
    #[test]
    fn real_press_creates_one_bright_pulse() {
        let mut s = OpticalProbe::new();
        assert_eq!(s.on_button(true, ms(0)), Ok(Some(Transition::Bright(1))));
        assert!(s.is_bright());
        assert_eq!(s.tick(ms(349)), None);
        assert_eq!(s.tick(ms(350)), Some(Transition::Dark(1)));
        assert!(!s.is_bright());
    }
    #[test]
    fn hold_repeat_never_creates_a_second_trial() {
        let mut s = OpticalProbe::new();
        s.on_button(true, ms(0)).unwrap();
        assert_eq!(s.on_button(true, ms(11)), Ok(None));
        s.tick(ms(350));
        assert_eq!(s.on_button(true, ms(950)), Ok(None));
        assert_eq!(s.active_trial(), None);
    }
    #[test]
    fn false_release_and_refractory_gate_do_not_pulse() {
        let mut s = OpticalProbe::new();
        assert_eq!(s.on_button(false, ms(0)), Ok(None));
        s.on_button(true, ms(1)).unwrap();
        s.on_button(false, ms(100)).unwrap();
        s.tick(ms(351));
        assert_eq!(s.on_button(true, ms(400)), Ok(None));
        s.on_button(false, ms(450)).unwrap();
        assert_eq!(s.on_button(true, ms(902)), Ok(Some(Transition::Bright(2))));
    }
    #[test]
    fn release_does_not_turn_off_before_optical_window() {
        let mut s = OpticalProbe::new();
        s.on_button(true, ms(0)).unwrap();
        s.on_button(false, ms(10)).unwrap();
        assert!(s.is_bright());
        assert_eq!(s.tick(ms(300)), None);
        assert_eq!(s.tick(ms(400)), Some(Transition::Dark(1)));
    }
    #[test]
    fn no_bright_without_real_press() {
        let mut s = OpticalProbe::new();
        for tick in [1, 200, 399, 1000] {
            assert_eq!(s.tick(ms(tick)), None);
        }
        assert!(!s.is_bright());
    }
    #[test]
    fn clock_rollback_cannot_prematurely_end_pulse() {
        let mut s = OpticalProbe::new();
        s.on_button(true, ms(500)).unwrap();
        assert_eq!(s.tick(ms(2)), None);
    }
}
