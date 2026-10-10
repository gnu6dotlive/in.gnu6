//! Pure pacing gate between Vulkan queue submissions and compositor outcomes.
use std::time::Duration;
const MAX_WAIT: Duration = Duration::from_millis(250);

#[derive(Debug, Default)]
pub struct FramePacing {
    pending_since: Option<Duration>,
    max_inflight: usize,
}
impl FramePacing {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn allow_submit(&mut self, pending: usize, at: Duration) -> Result<bool, &'static str> {
        self.max_inflight = self.max_inflight.max(pending);
        if pending > 1 {
            return Err("GPU05_MULTI_FRAME_IN_FLIGHT");
        }
        if pending == 0 {
            self.pending_since = None;
            return Ok(true);
        }
        let since = self.pending_since.get_or_insert(at);
        if at.saturating_sub(*since) > MAX_WAIT {
            return Err("GPU05_PRESENTATION_STALL_GT_250MS");
        }
        Ok(false)
    }
    pub fn max_inflight(&self) -> usize {
        self.max_inflight
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn must_wait_until_compositor_settles_one_feedback() {
        let mut p = FramePacing::new();
        assert_eq!(p.allow_submit(0, Duration::ZERO), Ok(true));
        assert_eq!(p.allow_submit(1, Duration::from_millis(1)), Ok(false));
        assert_eq!(p.allow_submit(1, Duration::from_millis(200)), Ok(false));
        assert_eq!(p.allow_submit(0, Duration::from_millis(210)), Ok(true));
        assert_eq!(p.allow_submit(1, Duration::from_millis(230)), Ok(false));
        assert_eq!(p.max_inflight(), 1);
    }
    #[test]
    fn no_second_frame_until_feedback_and_fail_closed_after_stall() {
        let mut p = FramePacing::new();
        assert_eq!(p.allow_submit(1, Duration::from_millis(10)), Ok(false));
        assert_eq!(p.allow_submit(1, Duration::from_millis(260)), Ok(false));
        assert_eq!(
            p.allow_submit(1, Duration::from_millis(261)),
            Err("GPU05_PRESENTATION_STALL_GT_250MS")
        );
    }
    #[test]
    fn multi_frame_pending_is_immediate_invariant_violation() {
        let mut p = FramePacing::new();
        assert_eq!(
            p.allow_submit(2, Duration::ZERO),
            Err("GPU05_MULTI_FRAME_IN_FLIGHT")
        );
        assert_eq!(p.max_inflight(), 2);
    }
    #[test]
    fn monotonic_elapsed_time_is_saturating_and_cannot_wrap() {
        let mut p = FramePacing::new();
        assert_eq!(p.allow_submit(1, Duration::from_millis(150)), Ok(false));
        assert_eq!(p.allow_submit(1, Duration::from_millis(140)), Ok(false));
    }
}
