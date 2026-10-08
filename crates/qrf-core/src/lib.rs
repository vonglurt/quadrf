//! qrf core types, configuration and time base
//!
//! Implements: SPEC-008 S-008-12; SPEC-009 S-009-4/6 (TAI timestamps, clock quality). Skeleton created 2026-10-08 (backlog R-01); the behaviour
//! statements it must satisfy are in `specs/`, and each public item added here
//! names the statement it implements.
#![forbid(unsafe_code)]

/// The crate's name, for the `Health` message and logs (SPEC-009 S-009-5).
pub const NAME: &str = "qrf-core";

#[cfg(test)]
mod tests {
    #[test]
    fn name_is_stable() {
        assert_eq!(super::NAME, "qrf-core");
    }
}
