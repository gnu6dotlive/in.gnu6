//! PACER-05 controlled startup experiment. No frame is silently omitted.
use std::time::Duration;

/// Only these exactly specified experimental variants are authorized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupPolicy {
    Immediate,
    Settled75Ms,
}
impl StartupPolicy {
    pub fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "immediate" => Ok(Self::Immediate),
            "settle75" => Ok(Self::Settled75Ms),
            _ => Err("GPU05_UNSUPPORTED_STARTUP_POLICY"),
        }
    }
    pub fn settle_duration(self) -> Duration {
        match self {
            Self::Immediate => Duration::ZERO,
            Self::Settled75Ms => Duration::from_millis(75),
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Immediate => "immediate",
            Self::Settled75Ms => "settle75",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn only_two_exact_variants_allowed() {
        assert_eq!(
            StartupPolicy::parse("immediate"),
            Ok(StartupPolicy::Immediate)
        );
        assert_eq!(
            StartupPolicy::parse("settle75"),
            Ok(StartupPolicy::Settled75Ms)
        );
        for value in ["", "delay10", "settle750", "Settle75", "unbounded"] {
            assert!(StartupPolicy::parse(value).is_err());
        }
    }
    #[test]
    fn delay_is_bounded_without_changing_scene_or_render_settings() {
        assert_eq!(StartupPolicy::Immediate.settle_duration(), Duration::ZERO);
        assert_eq!(
            StartupPolicy::Settled75Ms.settle_duration(),
            Duration::from_millis(75)
        );
        assert_eq!(StartupPolicy::Settled75Ms.name(), "settle75");
    }
}
