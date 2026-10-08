//! qrf.v1 feed-bus schema and ZeroMQ transport
//!
//! Implements: SPEC-009 S-009-4/5/9/10; SPEC-008 S-008-7. Skeleton created 2026-10-08 (backlog R-01); the behaviour
//! statements it must satisfy are in `specs/`, and each public item added here
//! names the statement it implements.
#![forbid(unsafe_code)]

/// The crate's name, for the `Health` message and logs (SPEC-009 S-009-5).
pub const NAME: &str = "qrf-bus";

#[cfg(test)]
mod tests {
    #[test]
    fn name_is_stable() {
        assert_eq!(super::NAME, "qrf-bus");
    }
}
