//! ORACLE-04 read-only stall timing: two *local* monotonic domains.
#![allow(dead_code)]
use std::time::Duration;

/// This is a loop scheduling diagnostic, not a Wayland presentation clock,
/// a physical photodiode sample, or an input-to-photon claim.
pub const LONG_LOOP_GAP: Duration = Duration::from_micros(33_400);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopGap {
    pub wall_gap_ns: u64,
    pub thread_cpu_gap_ns: u64,
    pub wall_minus_cpu_ns: u64,
    pub exceeds_33_4ms: bool,
}

#[derive(Default, Debug)]
pub struct HostStallProbe {
    previous_wall_ns: Option<u64>,
    previous_thread_cpu_ns: Option<u64>,
    pub iterations: u64,
    pub outliers: u64,
    pub max_loop_gap_ns: u64,
    pub max_thread_cpu_gap_ns: u64,
}

impl HostStallProbe {
    pub fn new() -> Self {
        Self::default()
    }

    /// The input clock values each belong to their OWN monotonic domain.
    /// Only consecutive deltas *within* a domain are subtracted.
    /// Wall minus thread CPU is not conclusive evidence of OS preemption:
    /// the thread may also be in a blocking syscall.
    pub fn sample(&mut self, wall_ns: u64, cpu_ns: u64) -> Result<Option<LoopGap>, &'static str> {
        self.iterations = self
            .iterations
            .checked_add(1)
            .ok_or("ORACLE04_SAMPLE_COUNTER_EXHAUSTED")?;
        let result =
            if let (Some(wall), Some(cpu)) = (self.previous_wall_ns, self.previous_thread_cpu_ns) {
                if wall_ns < wall || cpu_ns < cpu {
                    return Err("ORACLE04_NONMONOTONIC_HOST_CLOCK");
                }
                let wall_gap_ns = wall_ns - wall;
                let thread_cpu_gap_ns = cpu_ns - cpu;
                // Consecutive CPU time cannot exceed wall time for a single
                // thread, allowing 2ms for timer quantization and clock error.
                if thread_cpu_gap_ns > wall_gap_ns.saturating_add(2_000_000) {
                    return Err("ORACLE04_THREAD_CPU_CLOCK_INCONSISTENT");
                }
                self.max_loop_gap_ns = self.max_loop_gap_ns.max(wall_gap_ns);
                self.max_thread_cpu_gap_ns = self.max_thread_cpu_gap_ns.max(thread_cpu_gap_ns);
                let long = wall_gap_ns > LONG_LOOP_GAP.as_nanos() as u64;
                if long {
                    self.outliers += 1;
                }
                Some(LoopGap {
                    wall_gap_ns,
                    thread_cpu_gap_ns,
                    wall_minus_cpu_ns: wall_gap_ns.saturating_sub(thread_cpu_gap_ns),
                    exceeds_33_4ms: long,
                })
            } else {
                None
            };
        self.previous_wall_ns = Some(wall_ns);
        self.previous_thread_cpu_ns = Some(cpu_ns);
        Ok(result)
    }
}

/// Linux per-thread *CPU* time. This is not CLOCK_MONOTONIC and MUST NOT
/// be subtracted directly from compositor timestamps or wall clock epochs.
pub fn thread_cpu_time_ns() -> Result<u64, String> {
    let mut ts = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: own writable stack timespec, valid CLOCK_THREAD_CPUTIME_ID.
    if unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &raw mut ts) } != 0 {
        return Err(format!(
            "ORACLE04_THREAD_CLOCK_FAILED {}",
            std::io::Error::last_os_error()
        ));
    }
    if ts.tv_sec < 0 || !(0..1_000_000_000).contains(&ts.tv_nsec) {
        return Err("ORACLE04_THREAD_CPU_CLOCK_INVALID".into());
    }
    let ns = (ts.tv_sec as u64)
        .checked_mul(1_000_000_000)
        .and_then(|n| n.checked_add(ts.tv_nsec as u64))
        .ok_or("ORACLE04_THREAD_CPU_CLOCK_OVERFLOW")?;
    Ok(ns)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordinary_loop_is_not_outlier() {
        let mut p = HostStallProbe::new();
        assert_eq!(p.sample(0, 100).unwrap(), None);
        let x = p.sample(13_300_000, 100_000).unwrap().unwrap();
        assert!(!x.exceeds_33_4ms);
        assert_eq!(p.outliers, 0);
    }
    #[test]
    fn large_wall_gap_survives_callback_resumption() {
        let mut p = HostStallProbe::new();
        p.sample(0, 100).unwrap();
        let x = p.sample(435_000_000, 350_100).unwrap().unwrap();
        assert!(x.exceeds_33_4ms);
        assert_eq!(x.wall_gap_ns, 435_000_000);
        assert_eq!(p.max_loop_gap_ns, 435_000_000);
        assert_eq!(p.outliers, 1);
    }
    #[test]
    fn long_cpu_work_distinguished_from_wait_or_deschedule() {
        let mut p = HostStallProbe::new();
        p.sample(0, 0).unwrap();
        let busy = p.sample(52_000_000, 48_000_000).unwrap().unwrap();
        assert!(busy.exceeds_33_4ms);
        assert_eq!(busy.wall_minus_cpu_ns, 4_000_000);
    }
    #[test]
    fn bad_clock_regression_is_rejected() {
        let mut p = HostStallProbe::new();
        p.sample(10, 10).unwrap();
        assert_eq!(p.sample(9, 11), Err("ORACLE04_NONMONOTONIC_HOST_CLOCK"));
    }
    #[test]
    fn inconsistent_single_thread_cpu_delta_is_rejected() {
        let mut p = HostStallProbe::new();
        p.sample(0, 0).unwrap();
        assert_eq!(
            p.sample(5_000_000, 15_000_000),
            Err("ORACLE04_THREAD_CPU_CLOCK_INCONSISTENT")
        );
    }
    #[test]
    fn exact_boundary_is_not_automatically_violated() {
        let mut p = HostStallProbe::new();
        p.sample(0, 0).unwrap();
        assert!(!p.sample(33_400_000, 100).unwrap().unwrap().exceeds_33_4ms);
    }
}
