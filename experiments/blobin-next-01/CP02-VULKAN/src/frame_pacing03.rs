//! GPU05-PACER-03: separate Wayland wl_surface.frame cadence authorization
//! from wp_presentation verification. No clock-domain arithmetic is inferred.
use std::{collections::BTreeSet, time::Duration};
const MAX_PENDING: usize = 2;
const STALL: Duration = Duration::from_millis(250);

#[derive(Debug)]
pub struct FrameCallbackPacing {
    credit: bool,
    outstanding: BTreeSet<u64>,
    last_submitted: Option<u64>,
    last_progress: Duration,
    max_feedback_inflight: usize,
    callbacks: u64,
    watchdog_trips: u64,
}
impl Default for FrameCallbackPacing {
    fn default() -> Self {
        Self {
            credit: true,
            outstanding: BTreeSet::new(),
            last_submitted: None,
            last_progress: Duration::ZERO,
            max_feedback_inflight: 0,
            callbacks: 0,
            watchdog_trips: 0,
        }
    }
}
impl FrameCallbackPacing {
    pub fn new() -> Self {
        Self::default()
    }
    /// Credit is created only by the callback from the exact owned wl_surface,
    /// except for the initial bootstrap commit.
    pub fn on_frame_callback(&mut self, seq: u64, at: Duration) -> Result<(), &'static str> {
        if !self.outstanding.remove(&seq) {
            return Err("GPU05_PACER03_UNKNOWN_OR_DUPLICATE_FRAME_CALLBACK");
        }
        self.credit = true;
        self.callbacks += 1;
        self.last_progress = at;
        Ok(())
    }
    pub fn on_presentation_feedback(&mut self, at: Duration) {
        self.last_progress = at;
    }
    pub fn should_submit(&mut self, wp_pending: usize, at: Duration) -> Result<bool, &'static str> {
        self.max_feedback_inflight = self.max_feedback_inflight.max(wp_pending);
        if wp_pending > MAX_PENDING {
            return Err("GPU05_PACER03_TOO_MANY_WP_FEEDBACKS");
        }
        if at.saturating_sub(self.last_progress) > STALL {
            self.watchdog_trips += 1;
            return Err("GPU05_PACER03_NO_WAYLAND_PROGRESS_GT_250MS");
        }
        if wp_pending >= MAX_PENDING || !self.credit {
            return Ok(false);
        }
        if self.outstanding.len() >= MAX_PENDING {
            return Err("GPU05_PACER03_CALLBACK_QUEUE_SATURATED");
        }
        Ok(true)
    }
    pub fn submitted(&mut self, seq: u64, at: Duration) -> Result<(), &'static str> {
        if !self.credit {
            return Err("GPU05_PACER03_NO_FRAME_CREDIT");
        }
        if self.last_submitted.is_some_and(|last| seq <= last) {
            return Err("GPU05_PACER03_NONINCREASING_SEQUENCE");
        }
        if !self.outstanding.insert(seq) {
            return Err("GPU05_PACER03_DUPLICATE_SUBMISSION");
        }
        self.credit = false;
        self.last_submitted = Some(seq);
        self.last_progress = at;
        Ok(())
    }
    pub fn callbacks(&self) -> u64 {
        self.callbacks
    }
    pub fn max_inflight(&self) -> usize {
        self.max_feedback_inflight
    }
    pub fn watchdog_trips(&self) -> u64 {
        self.watchdog_trips
    }
    pub fn callback_pending(&self) -> usize {
        self.outstanding.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn us(x: u64) -> Duration {
        Duration::from_micros(x)
    }
    #[test]
    fn seventy_five_hertz_one_callback_one_next_commit() {
        let mut p = FrameCallbackPacing::new();
        assert!(p.should_submit(0, Duration::ZERO).unwrap());
        p.submitted(1, Duration::ZERO).unwrap();
        assert!(!p.should_submit(1, us(10_000)).unwrap());
        p.on_frame_callback(1, us(13_340)).unwrap();
        assert!(p.should_submit(1, us(13_340)).unwrap());
        p.submitted(2, us(13_340)).unwrap();
        assert!(!p.should_submit(2, us(26_680)).unwrap());
        p.on_presentation_feedback(us(26_680));
        p.on_frame_callback(2, us(26_681)).unwrap();
        assert!(p.should_submit(1, us(26_682)).unwrap());
        assert_eq!(p.callbacks(), 2);
        assert_eq!(p.max_inflight(), 2);
    }
    #[test]
    fn one_hundred_sixty_five_hertz_callback_does_not_fake_presentation() {
        let mut p = FrameCallbackPacing::new();
        p.submitted(1, Duration::ZERO).unwrap();
        p.on_frame_callback(1, us(6_064)).unwrap();
        assert!(p.should_submit(1, us(6_064)).unwrap());
        assert_eq!(p.callbacks(), 1);
        p.submitted(2, us(6_065)).unwrap();
        // wp_feedback is independent: it may still show two pending frames.
        assert!(!p.should_submit(2, us(12_128)).unwrap());
    }
    #[test]
    fn duplicate_or_foreign_callbacks_fail_closed() {
        let mut p = FrameCallbackPacing::new();
        p.submitted(20, Duration::ZERO).unwrap();
        assert!(p.on_frame_callback(99, us(5)).is_err());
        p.on_frame_callback(20, us(6)).unwrap();
        assert_eq!(
            p.on_frame_callback(20, us(7)),
            Err("GPU05_PACER03_UNKNOWN_OR_DUPLICATE_FRAME_CALLBACK")
        );
    }
    #[test]
    fn callback_credit_cannot_be_spent_twice() {
        let mut p = FrameCallbackPacing::new();
        p.submitted(1, Duration::ZERO).unwrap();
        assert_eq!(p.submitted(2, us(3)), Err("GPU05_PACER03_NO_FRAME_CREDIT"));
        p.on_frame_callback(1, us(4)).unwrap();
        p.submitted(2, us(5)).unwrap();
        assert_eq!(p.callback_pending(), 1);
    }
    #[test]
    fn no_callback_or_presentation_trips_bounded_watchdog() {
        let mut p = FrameCallbackPacing::new();
        p.submitted(1, Duration::ZERO).unwrap();
        assert!(!p.should_submit(1, us(250_000)).unwrap());
        assert_eq!(
            p.should_submit(1, us(250_001)),
            Err("GPU05_PACER03_NO_WAYLAND_PROGRESS_GT_250MS")
        );
        assert_eq!(p.watchdog_trips(), 1);
    }
    #[test]
    fn compositor_feedback_alone_does_not_create_frame_credit() {
        let mut p = FrameCallbackPacing::new();
        p.submitted(1, Duration::ZERO).unwrap();
        p.on_presentation_feedback(us(249_000));
        assert!(!p.should_submit(0, us(250_000)).unwrap());
        assert_eq!(p.callback_pending(), 1);
    }
    #[test]
    fn terminal_callback_after_final_presented_is_accounted_without_submitting_again() {
        let mut p = FrameCallbackPacing::new();
        p.submitted(1, Duration::ZERO).unwrap();
        p.on_presentation_feedback(us(6_065));
        assert_eq!(p.callback_pending(), 1);
        p.on_frame_callback(1, us(9_000)).unwrap();
        assert_eq!(p.callback_pending(), 0);
        assert_eq!(p.callbacks(), 1);
    }
    #[test]
    fn progress_event_can_extend_watchdog_without_inventing_frame() {
        let mut p = FrameCallbackPacing::new();
        p.submitted(1, Duration::ZERO).unwrap();
        p.on_presentation_feedback(us(200_000));
        assert!(!p.should_submit(0, us(280_000)).unwrap());
        p.on_frame_callback(1, us(281_000)).unwrap();
        assert!(p.should_submit(0, us(282_000)).unwrap());
    }
}
