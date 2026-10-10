//! Same-clock startup presentation latency, NOT input-to-photon latency.
//! A wp_presentation.clock_id mismatch disables cross-clock arithmetic.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartupAnchors {
    pub app_ready_ns: u128,
    pub first_submit_begin_ns: u128,
    pub first_submit_return_ns: u128,
    pub first_compositor_present_ns: u128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartupLatency {
    pub ready_to_present_ns: u128,
    pub submit_begin_to_present_ns: u128,
    pub submit_return_to_present_ns: Option<u128>,
}

pub fn monotonic_now_ns() -> Result<u128, String> {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: stack-allocated writable libc timespec; clock_gettime fills it.
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &raw mut ts) } != 0 {
        return Err(format!(
            "GPU05_CLOCK_GETTIME_FAILED {}",
            std::io::Error::last_os_error()
        ));
    }
    if ts.tv_sec < 0 || !(0..1_000_000_000).contains(&ts.tv_nsec) {
        return Err("GPU05_CLOCK_GETTIME_INVALID".into());
    }
    Ok(ts.tv_sec as u128 * 1_000_000_000 + ts.tv_nsec as u128)
}

pub fn calculate(wayland_clock_id: u32, s: StartupAnchors) -> Result<StartupLatency, &'static str> {
    if wayland_clock_id != libc::CLOCK_MONOTONIC as u32 {
        return Err("GPU05_WP_CLOCK_NOT_MONOTONIC");
    }
    if s.app_ready_ns > s.first_submit_begin_ns
        || s.first_submit_begin_ns > s.first_submit_return_ns
    {
        return Err("GPU05_NONMONOTONIC_CPU_ANCHORS");
    }
    if s.first_compositor_present_ns < s.first_submit_begin_ns {
        return Err("GPU05_PRESENTATION_BEFORE_FIRST_SUBMIT");
    }
    Ok(StartupLatency {
        ready_to_present_ns: s.first_compositor_present_ns - s.app_ready_ns,
        submit_begin_to_present_ns: s.first_compositor_present_ns - s.first_submit_begin_ns,
        submit_return_to_present_ns: s
            .first_compositor_present_ns
            .checked_sub(s.first_submit_return_ns),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn anchors() -> StartupAnchors {
        StartupAnchors {
            app_ready_ns: 100_000_000,
            first_submit_begin_ns: 101_000_000,
            first_submit_return_ns: 102_000_000,
            first_compositor_present_ns: 115_000_000,
        }
    }
    #[test]
    fn same_clock_ms_deltas_are_exact() {
        assert_eq!(
            calculate(libc::CLOCK_MONOTONIC as u32, anchors()),
            Ok(StartupLatency {
                ready_to_present_ns: 15_000_000,
                submit_begin_to_present_ns: 14_000_000,
                submit_return_to_present_ns: Some(13_000_000),
            })
        );
    }
    #[test]
    fn arbitrary_wayland_clock_is_rejected() {
        assert_eq!(
            calculate(99, anchors()),
            Err("GPU05_WP_CLOCK_NOT_MONOTONIC")
        );
    }
    #[test]
    fn startup_cpu_clock_reversal_rejected() {
        let mut s = anchors();
        s.first_submit_begin_ns = 99_000_000;
        assert_eq!(
            calculate(libc::CLOCK_MONOTONIC as u32, s),
            Err("GPU05_NONMONOTONIC_CPU_ANCHORS")
        );
    }
    #[test]
    fn presentation_before_submission_rejected() {
        let mut s = anchors();
        s.first_compositor_present_ns = 100_500_000;
        assert_eq!(
            calculate(libc::CLOCK_MONOTONIC as u32, s),
            Err("GPU05_PRESENTATION_BEFORE_FIRST_SUBMIT")
        );
    }
    #[test]
    fn blocking_present_call_can_return_after_real_display() {
        let mut s = anchors();
        s.first_submit_return_ns = 117_000_000;
        let v = calculate(libc::CLOCK_MONOTONIC as u32, s).unwrap();
        assert_eq!(v.submit_return_to_present_ns, None);
        assert_eq!(v.submit_begin_to_present_ns, 14_000_000);
    }
}
