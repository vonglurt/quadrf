//! qrf core types, configuration and time base
//!
//! Implements: SPEC-008 S-008-12; SPEC-009 S-009-4/6 (TAI timestamps, clock
//! quality). Each public item names the statement it implements.
#![forbid(unsafe_code)]

/// The crate's name, for the `Health` message and logs (SPEC-009 S-009-5).
pub const NAME: &str = "qrf-core";

/// The two clocks every bus message carries (SPEC-009 S-009-4).
///
/// `t_tai_ns` is CLOCK_TAI, the clock the overlay associates sensors by; Linux
/// derives it from CLOCK_REALTIME plus the kernel's TAI offset, which stays
/// zero until a leap-second-aware daemon sets it (S-009-6: chrony or gpsd), so
/// a message's `clock_quality` says how far to trust it. `t_mono_ns` is
/// CLOCK_MONOTONIC_RAW, unaffected by NTP slewing, for intervals and rates.
pub mod time {
    use nix::sys::time::TimeSpec;
    use nix::time::{ClockId, clock_gettime};

    fn ns(t: TimeSpec) -> i64 {
        i64::from(t.tv_sec()) * 1_000_000_000 + i64::from(t.tv_nsec())
    }

    /// Nanoseconds of CLOCK_TAI (S-009-4 `t_tai_ns`).
    pub fn now_tai_ns() -> i64 {
        ns(clock_gettime(ClockId::CLOCK_TAI).expect("CLOCK_TAI is readable on every Linux since 3.10"))
    }

    /// Nanoseconds of CLOCK_MONOTONIC_RAW (S-009-4 `t_mono_ns`).
    pub fn now_mono_ns() -> i64 {
        ns(clock_gettime(ClockId::CLOCK_MONOTONIC_RAW).expect("CLOCK_MONOTONIC_RAW is readable on every Linux since 2.6.28"))
    }

    /// Both clocks read back to back; what a `Header` carries.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Stamp {
        pub tai_ns: i64,
        pub mono_ns: i64,
    }

    impl Stamp {
        pub fn now() -> Self {
            Self { tai_ns: now_tai_ns(), mono_ns: now_mono_ns() }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::time::{Stamp, now_mono_ns, now_tai_ns};

    #[test]
    fn name_is_stable() {
        assert_eq!(super::NAME, "qrf-core");
    }

    #[test]
    fn tai_is_after_2020() {
        // 2020-01-01T00:00:00Z is 1 577 836 800 s after the epoch; TAI runs ahead of UTC, never behind.
        assert!(now_tai_ns() > 1_577_836_800 * 1_000_000_000);
    }

    #[test]
    fn mono_raw_is_monotonic() {
        let a = now_mono_ns();
        let b = now_mono_ns();
        assert!(b >= a);
        assert!(b - a < 1_000_000_000, "two reads should be well under a second apart");
    }

    #[test]
    fn stamp_reads_both() {
        let s = Stamp::now();
        assert!(s.tai_ns > 0 && s.mono_ns > 0);
    }
}
