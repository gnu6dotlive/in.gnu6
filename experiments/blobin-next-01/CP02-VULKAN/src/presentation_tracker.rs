//! GPU-05: pure per-wl_surface presentation-feedback correlation state machine.
//! SOURCE-ONLY until local cargo test and native owned-event-loop integration.
//! No Wayland calls live in this module; events MUST be supplied by the owner
//! of the same wl_surface and its actual wp_presentation_feedback proxies.
use std::collections::HashMap;

const MAX_PENDING: usize = 512;
const MAX_UNDRAINED_RECEIPTS: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedbackOutcome {
    Discarded,
    Presented {
        tv_sec_hi: u32,
        tv_sec_lo: u32,
        tv_nsec: u32,
        refresh_ns: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PresentationReceipt {
    pub feedback_id: u32,
    pub frame_sequence: u64,
    pub surface_generation: u64,
    pub timestamp_ns: Option<u128>,
    pub refresh_ns: Option<u32>,
    pub discarded: bool,
}
#[derive(Debug, Clone, Copy)]
struct Pending {
    frame_sequence: u64,
    surface_generation: u64,
}

#[derive(Debug)]
pub struct FeedbackTracker {
    clock_id: Option<u32>,
    current_surface_generation: Option<u64>,
    last_requested_sequence: Option<u64>,
    last_presented_timestamp_ns: Option<u128>,
    pending: HashMap<u32, Pending>,
    last_feedback_id: u32,
    receipts: Vec<PresentationReceipt>,
}

impl FeedbackTracker {
    pub fn new() -> Self {
        Self {
            clock_id: None,
            current_surface_generation: None,
            last_requested_sequence: None,
            last_presented_timestamp_ns: None,
            pending: HashMap::new(),
            last_feedback_id: 0,
            receipts: Vec::new(),
        }
    }

    /// Binding wp_presentation must actually deliver this event.
    pub fn clock_id(&mut self, clock_id: u32) -> Result<(), &'static str> {
        if self.clock_id.is_some() {
            return Err("GPU05_CLOCK_REDEFINED");
        }
        self.clock_id = Some(clock_id);
        Ok(())
    }

    pub fn begin_surface(&mut self, generation: u64) -> Result<(), &'static str> {
        if self.clock_id.is_none() {
            return Err("GPU05_CLOCK_NOT_ANNOUNCED");
        }
        if !self.pending.is_empty() {
            return Err("GPU05_PENDING_FEEDBACK_ON_RECREATE");
        }
        if self
            .current_surface_generation
            .is_some_and(|old| generation <= old)
        {
            return Err("GPU05_SURFACE_GENERATION_REPLAY");
        }
        self.current_surface_generation = Some(generation);
        self.last_requested_sequence = None;
        self.last_presented_timestamp_ns = None;
        Ok(())
    }

    /// Call immediately after wp_presentation.feedback(surface) succeeds and
    /// before the corresponding actual wl_surface.commit is submitted.
    pub fn requested(&mut self, feedback_id: u32, sequence: u64) -> Result<(), &'static str> {
        let generation = self
            .current_surface_generation
            .ok_or("GPU05_NO_OWNED_SURFACE")?;
        if self.pending.len() >= MAX_PENDING {
            return Err("GPU05_TOO_MANY_PENDING");
        }
        if feedback_id == 0 || feedback_id <= self.last_feedback_id {
            return Err("GPU05_DUPLICATE_FEEDBACK_ID");
        }
        if self
            .last_requested_sequence
            .is_some_and(|old| sequence <= old)
        {
            return Err("GPU05_NONINCREASING_FRAME_SEQUENCE");
        }
        self.last_feedback_id = feedback_id;
        self.pending.insert(
            feedback_id,
            Pending {
                frame_sequence: sequence,
                surface_generation: generation,
            },
        );
        self.last_requested_sequence = Some(sequence);
        Ok(())
    }

    /// Called only by the *same* connection's Wayland event queue.
    pub fn resolved(
        &mut self,
        feedback_id: u32,
        outcome: FeedbackOutcome,
    ) -> Result<PresentationReceipt, &'static str> {
        if self.receipts.len() >= MAX_UNDRAINED_RECEIPTS {
            return Err("GPU05_RECEIPTS_NOT_DRAINED");
        }
        let pending = *self
            .pending
            .get(&feedback_id)
            .ok_or("GPU05_UNSOLICITED_OR_REPEATED_FEEDBACK")?;
        if self.current_surface_generation != Some(pending.surface_generation) {
            return Err("GPU05_DIFFERENT_SURFACE_GENERATION");
        }
        let (timestamp_ns, refresh_ns, discarded) = match outcome {
            FeedbackOutcome::Discarded => (None, None, true),
            FeedbackOutcome::Presented {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
                refresh_ns,
            } => {
                if tv_nsec >= 1_000_000_000 {
                    return Err("GPU05_INVALID_NANOSECOND");
                }
                if refresh_ns > 500_000_000 {
                    return Err("GPU05_REFRESH_OUT_OF_RANGE");
                }
                // wp_presentation timestamps share the announced clock_id.
                let sec = (u128::from(tv_sec_hi) << 32) | u128::from(tv_sec_lo);
                let at = sec * 1_000_000_000 + u128::from(tv_nsec);
                if self
                    .last_presented_timestamp_ns
                    .is_some_and(|old| at <= old)
                {
                    return Err("GPU05_NONMONOTONIC_PRESENTATION");
                }
                (Some(at), Some(refresh_ns), false)
            }
        };
        self.pending.remove(&feedback_id);
        if let Some(at) = timestamp_ns {
            self.last_presented_timestamp_ns = Some(at);
        }
        let receipt = PresentationReceipt {
            feedback_id,
            frame_sequence: pending.frame_sequence,
            surface_generation: pending.surface_generation,
            timestamp_ns,
            refresh_ns,
            discarded,
        };
        self.receipts.push(receipt);
        Ok(receipt)
    }

    pub fn unresolved(&self) -> usize {
        self.pending.len()
    }
    pub fn receipts(&self) -> &[PresentationReceipt] {
        &self.receipts
    }
    /// Owned event loop drains to an append-only trace; never retain unlimited receipts.
    pub fn drain_receipts(&mut self) -> Vec<PresentationReceipt> {
        std::mem::take(&mut self.receipts)
    }
    pub fn clock(&self) -> Option<u32> {
        self.clock_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tracker() -> FeedbackTracker {
        let mut t = FeedbackTracker::new();
        t.clock_id(1).unwrap();
        t.begin_surface(42).unwrap();
        t
    }

    #[test]
    fn pending_capacity_and_receipt_drain_are_bounded() {
        let mut t = tracker();
        for id in 1..=512 {
            t.requested(id, u64::from(id)).unwrap();
        }
        assert_eq!(t.unresolved(), 512);
        assert_eq!(t.requested(513, 513).unwrap_err(), "GPU05_TOO_MANY_PENDING");
        for id in 1..=512 {
            t.resolved(id, FeedbackOutcome::Discarded).unwrap();
        }
        assert_eq!(t.unresolved(), 0);
        assert_eq!(t.drain_receipts().len(), 512);
        assert!(t.receipts().is_empty());
        t.requested(513, 513).unwrap();
        assert_eq!(t.unresolved(), 1);
    }

    #[test]
    fn invalid_refresh_does_not_consume_pending_feedback() {
        let mut t = tracker();
        t.requested(1, 1).unwrap();
        assert_eq!(
            t.resolved(
                1,
                FeedbackOutcome::Presented {
                    tv_sec_hi: 0,
                    tv_sec_lo: 8,
                    tv_nsec: 1,
                    refresh_ns: 500_000_001,
                }
            )
            .unwrap_err(),
            "GPU05_REFRESH_OUT_OF_RANGE"
        );
        assert_eq!(t.unresolved(), 1);
        t.resolved(1, FeedbackOutcome::Discarded).unwrap();
    }

    #[test]
    fn same_surface_commit_produces_one_correlated_receipt() {
        let mut t = tracker();
        t.requested(100, 44).unwrap();
        let receipt = t
            .resolved(
                100,
                FeedbackOutcome::Presented {
                    tv_sec_hi: 0,
                    tv_sec_lo: 10,
                    tv_nsec: 16_666_667,
                    refresh_ns: 16_666_667,
                },
            )
            .unwrap();
        assert_eq!(receipt.frame_sequence, 44);
        assert_eq!(receipt.surface_generation, 42);
        assert_eq!(receipt.timestamp_ns, Some(10_016_666_667));
        assert_eq!(t.unresolved(), 0);
        assert_eq!(t.receipts().len(), 1);
    }

    #[test]
    fn rejected_invalid_events_cannot_consume_valid_pending_feedback() {
        let mut t = tracker();
        t.requested(3, 1).unwrap();
        assert_eq!(
            t.resolved(
                3,
                FeedbackOutcome::Presented {
                    tv_sec_hi: 0,
                    tv_sec_lo: 2,
                    tv_nsec: 1_000_000_000,
                    refresh_ns: 0,
                }
            )
            .unwrap_err(),
            "GPU05_INVALID_NANOSECOND"
        );
        assert_eq!(t.unresolved(), 1);
        t.resolved(3, FeedbackOutcome::Discarded).unwrap();
        assert_eq!(t.unresolved(), 0);
        assert_eq!(
            t.resolved(3, FeedbackOutcome::Discarded).unwrap_err(),
            "GPU05_UNSOLICITED_OR_REPEATED_FEEDBACK"
        );
    }

    #[test]
    fn rejects_surface_generation_replay_and_unknown_feedback() {
        let mut t = tracker();
        t.requested(20, 1).unwrap();
        assert_eq!(
            t.begin_surface(43).unwrap_err(),
            "GPU05_PENDING_FEEDBACK_ON_RECREATE"
        );
        assert_eq!(
            t.resolved(21, FeedbackOutcome::Discarded).unwrap_err(),
            "GPU05_UNSOLICITED_OR_REPEATED_FEEDBACK"
        );
        t.resolved(20, FeedbackOutcome::Discarded).unwrap();
        t.begin_surface(43).unwrap();
        assert_eq!(
            t.begin_surface(43).unwrap_err(),
            "GPU05_SURFACE_GENERATION_REPLAY"
        );
        assert_eq!(
            t.requested(20, 2).unwrap_err(),
            "GPU05_DUPLICATE_FEEDBACK_ID"
        );
    }

    #[test]
    fn rejects_nonmonotonic_frames_and_timestamps() {
        let mut t = tracker();
        t.requested(1, 10).unwrap();
        assert_eq!(
            t.requested(2, 10).unwrap_err(),
            "GPU05_NONINCREASING_FRAME_SEQUENCE"
        );
        t.resolved(
            1,
            FeedbackOutcome::Presented {
                tv_sec_hi: 0,
                tv_sec_lo: 10,
                tv_nsec: 0,
                refresh_ns: 0,
            },
        )
        .unwrap();
        t.requested(2, 11).unwrap();
        assert_eq!(
            t.resolved(
                2,
                FeedbackOutcome::Presented {
                    tv_sec_hi: 0,
                    tv_sec_lo: 9,
                    tv_nsec: 999,
                    refresh_ns: 0,
                }
            )
            .unwrap_err(),
            "GPU05_NONMONOTONIC_PRESENTATION"
        );
        assert_eq!(t.unresolved(), 1);
    }
}
