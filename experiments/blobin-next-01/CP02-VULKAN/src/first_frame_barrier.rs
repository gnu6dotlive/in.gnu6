//! PACER-06: a bounded protocol-outcome barrier for only the first frame.
//! A wl_surface.frame callback is scheduling credit, NOT proof of presentation.
use std::time::Duration;
const FIRST_OUTCOME_TIMEOUT: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupMode {
    Current,
    AwaitFirstOutcome,
}
impl StartupMode {
    pub fn parse(s: &str) -> Result<Self, &'static str> {
        match s {
            "current" => Ok(Self::Current),
            "await-first-feedback" => Ok(Self::AwaitFirstOutcome),
            _ => Err("GPU05_PACER06_UNSUPPORTED_STARTUP_MODE"),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::AwaitFirstOutcome => "await-first-feedback",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Presented,
    Discarded,
}

#[derive(Debug)]
pub struct FirstFrameBarrier {
    mode: StartupMode,
    submitted_at: Option<Duration>,
    first_outcome: Option<Outcome>,
    wait_iterations: u64,
}
impl FirstFrameBarrier {
    pub fn new(mode: StartupMode) -> Self {
        Self {
            mode,
            submitted_at: None,
            first_outcome: None,
            wait_iterations: 0,
        }
    }
    pub fn submitted(&mut self, seq: u64, at: Duration) -> Result<(), &'static str> {
        if seq == 1 {
            if self.submitted_at.is_some() {
                return Err("GPU05_PACER06_FIRST_FRAME_SUBMITTED_TWICE");
            }
            self.submitted_at = Some(at);
        }
        Ok(())
    }
    pub fn first_feedback(&mut self, seq: u64, outcome: Outcome) -> Result<(), &'static str> {
        if seq != 1 {
            return Ok(());
        }
        if self.submitted_at.is_none() {
            return Err("GPU05_PACER06_FIRST_FEEDBACK_BEFORE_FIRST_SUBMISSION");
        }
        if self.first_outcome.replace(outcome).is_some() {
            return Err("GPU05_PACER06_DUPLICATE_FIRST_FEEDBACK");
        }
        Ok(())
    }
    pub fn permit(&mut self, next_seq: u64, at: Duration) -> Result<bool, &'static str> {
        if self.mode == StartupMode::Current || next_seq <= 1 {
            return Ok(true);
        }
        if self.first_outcome.is_some() {
            return Ok(true);
        }
        let since = self
            .submitted_at
            .ok_or("GPU05_PACER06_SECOND_WITHOUT_FIRST")?;
        if at.saturating_sub(since) > FIRST_OUTCOME_TIMEOUT {
            return Err("GPU05_PACER06_FIRST_FEEDBACK_TIMEOUT_GT_250MS");
        }
        self.wait_iterations += 1;
        Ok(false)
    }
    pub fn mode(&self) -> StartupMode {
        self.mode
    }
    pub fn outcome(&self) -> Option<Outcome> {
        self.first_outcome
    }
    pub fn wait_iterations(&self) -> u64 {
        self.wait_iterations
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn ms(x: u64) -> Duration {
        Duration::from_millis(x)
    }
    #[test]
    fn baseline_current_mode_can_submit_second_after_callback() {
        let mut b = FirstFrameBarrier::new(StartupMode::Current);
        b.submitted(1, ms(0)).unwrap();
        assert_eq!(b.permit(2, ms(10)), Ok(true));
    }
    #[test]
    fn callback_credit_does_not_promote_first_frame() {
        let mut b = FirstFrameBarrier::new(StartupMode::AwaitFirstOutcome);
        b.submitted(1, ms(0)).unwrap();
        assert_eq!(b.permit(2, ms(13)), Ok(false));
        assert_eq!(b.permit(2, ms(20)), Ok(false));
        assert_eq!(b.wait_iterations(), 2);
    }
    #[test]
    fn first_presented_feedback_opens_gate() {
        let mut b = FirstFrameBarrier::new(StartupMode::AwaitFirstOutcome);
        b.submitted(1, ms(0)).unwrap();
        b.first_feedback(1, Outcome::Presented).unwrap();
        assert_eq!(b.permit(2, ms(13)), Ok(true));
        assert_eq!(b.outcome(), Some(Outcome::Presented));
    }
    #[test]
    fn first_discarded_stays_red_but_unblocks_gate() {
        let mut b = FirstFrameBarrier::new(StartupMode::AwaitFirstOutcome);
        b.submitted(1, ms(0)).unwrap();
        b.first_feedback(1, Outcome::Discarded).unwrap();
        assert_eq!(b.permit(2, ms(13)), Ok(true));
        assert_eq!(b.outcome(), Some(Outcome::Discarded));
    }
    #[test]
    fn first_feedback_absent_fails_closed_instead_of_hanging() {
        let mut b = FirstFrameBarrier::new(StartupMode::AwaitFirstOutcome);
        b.submitted(1, ms(0)).unwrap();
        assert_eq!(b.permit(2, ms(250)), Ok(false));
        assert_eq!(
            b.permit(2, ms(251)),
            Err("GPU05_PACER06_FIRST_FEEDBACK_TIMEOUT_GT_250MS")
        );
    }
    #[test]
    fn duplicate_first_or_invalid_mode_rejected() {
        let mut b = FirstFrameBarrier::new(StartupMode::AwaitFirstOutcome);
        assert!(StartupMode::parse("pause600").is_err());
        assert_eq!(
            b.first_feedback(1, Outcome::Presented),
            Err("GPU05_PACER06_FIRST_FEEDBACK_BEFORE_FIRST_SUBMISSION")
        );
        b.submitted(1, ms(0)).unwrap();
        assert_eq!(
            b.submitted(1, ms(1)),
            Err("GPU05_PACER06_FIRST_FRAME_SUBMITTED_TWICE")
        );
        b.first_feedback(1, Outcome::Presented).unwrap();
        assert_eq!(
            b.first_feedback(1, Outcome::Discarded),
            Err("GPU05_PACER06_DUPLICATE_FIRST_FEEDBACK")
        );
    }
    #[test]
    fn foreign_frame_feedback_cannot_open_first_gate() {
        let mut b = FirstFrameBarrier::new(StartupMode::AwaitFirstOutcome);
        b.submitted(1, ms(0)).unwrap();
        b.first_feedback(2, Outcome::Presented).unwrap();
        assert_eq!(b.permit(2, ms(14)), Ok(false));
    }
}
