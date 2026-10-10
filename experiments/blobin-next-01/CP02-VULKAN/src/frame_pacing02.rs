//! GPU05-PACER-02: bounded two-feedback-in-flight presentation-informed gate.
//! Pure policy. Caller must invoke submitted only after queue.present
//! and feedback_observed for real compositor presented/discarded receipt.
use std::time::Duration;

const STALL: Duration = Duration::from_millis(250);
const MAX_PENDING: usize = 2;
const FALLBACK_REFRESH_NS: u64 = 13_333_333; // ~75Hz until real feedback
const MIN_REFRESH_NS: u64 = 4_000_000;
const MAX_REFRESH_NS: u64 = 100_000_000;

#[derive(Debug)]
pub struct AdaptiveFramePacing {
    refresh_ns: u64,
    saw_refresh: bool,
    last_submit: Option<Duration>,
    last_feedback: Option<Duration>,
    max_inflight: usize,
    stalled_events: u64,
}
impl Default for AdaptiveFramePacing {
    fn default() -> Self {
        Self {
            refresh_ns: FALLBACK_REFRESH_NS,
            saw_refresh: false,
            last_submit: None,
            last_feedback: None,
            max_inflight: 0,
            stalled_events: 0,
        }
    }
}
impl AdaptiveFramePacing {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn observe_refresh(&mut self, ns: u32) {
        if (MIN_REFRESH_NS..=MAX_REFRESH_NS).contains(&u64::from(ns)) {
            self.refresh_ns = u64::from(ns);
            self.saw_refresh = true;
        }
    }
    pub fn feedback_observed(&mut self, at: Duration) {
        self.last_feedback = Some(at);
    }
    pub fn submitted(&mut self, at: Duration) {
        self.last_submit = Some(at);
    }
    pub fn should_submit(&mut self, pending: usize, at: Duration) -> Result<bool, &'static str> {
        self.max_inflight = self.max_inflight.max(pending);
        if pending > MAX_PENDING {
            return Err("GPU05_PACER02_TOO_MANY_INFLIGHT");
        }
        if pending > 0 {
            let last_progress = self
                .last_submit
                .unwrap_or(Duration::ZERO)
                .max(self.last_feedback.unwrap_or(Duration::ZERO));
            if at.saturating_sub(last_progress) > STALL {
                self.stalled_events += 1;
                return Err("GPU05_PACER02_FEEDBACK_STALL_GT_250MS");
            }
        }
        if pending == MAX_PENDING {
            return Ok(false);
        }
        if let Some(prior) = self.last_submit {
            if at.saturating_sub(prior) < Duration::from_nanos(self.refresh_ns) {
                return Ok(false);
            }
        }
        Ok(true)
    }
    pub fn max_inflight(&self) -> usize {
        self.max_inflight
    }
    pub fn refresh_ns(&self) -> u64 {
        self.refresh_ns
    }
    pub fn learned_refresh(&self) -> bool {
        self.saw_refresh
    }
    pub fn stalled_events(&self) -> u64 {
        self.stalled_events
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn ms(v: u64) -> Duration {
        Duration::from_millis(v)
    }
    fn us(v: u64) -> Duration {
        Duration::from_micros(v)
    }
    #[test]
    fn seventy_five_hertz_can_schedule_second_frame_ahead_of_first_feedback() {
        let mut p = AdaptiveFramePacing::new();
        p.observe_refresh(13_339_736);
        assert!(p.should_submit(0, Duration::ZERO).unwrap());
        p.submitted(Duration::ZERO);
        assert!(!p.should_submit(1, us(13_000)).unwrap());
        assert!(p.should_submit(1, us(13_340)).unwrap());
        p.submitted(us(13_340));
        assert!(!p.should_submit(2, us(27_000)).unwrap());
        p.feedback_observed(us(27_000));
        assert!(p.should_submit(1, us(27_000)).unwrap());
        assert_eq!(p.max_inflight(), 2);
    }
    #[test]
    fn one_hundred_sixty_five_hertz_cadence_uses_actual_feedback_refresh() {
        let mut p = AdaptiveFramePacing::new();
        p.observe_refresh(6_063_628);
        p.submitted(Duration::ZERO);
        assert!(!p.should_submit(1, us(6_000)).unwrap());
        assert!(p.should_submit(1, us(6_065)).unwrap());
        assert_eq!(p.refresh_ns(), 6_063_628);
    }
    #[test]
    fn saturation_and_old_feedback_fail_closed() {
        let mut p = AdaptiveFramePacing::new();
        p.submitted(ms(1));
        assert!(!p.should_submit(2, ms(15)).unwrap());
        assert_eq!(
            p.should_submit(3, ms(15)),
            Err("GPU05_PACER02_TOO_MANY_INFLIGHT")
        );
        assert_eq!(
            p.should_submit(1, ms(252)),
            Err("GPU05_PACER02_FEEDBACK_STALL_GT_250MS")
        );
        assert_eq!(p.stalled_events(), 1);
    }
    #[test]
    fn receipt_resets_watchdog_but_not_minimum_interval() {
        let mut p = AdaptiveFramePacing::new();
        p.observe_refresh(13_339_736);
        p.submitted(ms(1));
        p.feedback_observed(ms(240));
        assert!(p.should_submit(1, ms(245)).unwrap());
        p.submitted(ms(245));
        assert!(!p.should_submit(1, ms(252)).unwrap());
        assert!(p.should_submit(1, ms(259)).unwrap());
    }
    #[test]
    fn invalid_refresh_and_unproven_refresh_leave_default() {
        let mut p = AdaptiveFramePacing::new();
        p.observe_refresh(0);
        p.observe_refresh(1_000_000);
        assert!(!p.learned_refresh());
        assert_eq!(p.refresh_ns(), FALLBACK_REFRESH_NS);
    }
    #[test]
    fn logical_clock_rollback_never_enables_early_submit() {
        let mut p = AdaptiveFramePacing::new();
        p.submitted(ms(30));
        assert!(!p.should_submit(1, ms(5)).unwrap());
    }
}
