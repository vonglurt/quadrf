//! QuadRF receive ring and DSI staging access: mmap, ioctls, de-interleave
//!
//! Implements: SPEC-002 S-002-14…17; SPEC-008 S-008-2/3/5. Skeleton created 2026-10-08 (backlog R-01); the behaviour
//! statements it must satisfy are in `specs/`, and each public item added here
//! names the statement it implements.
// `unsafe` is permitted here only in leaf kernels/ioctl wrappers with a safety comment, a scalar reference and a property test (SPEC-008 S-008-5).

/// The crate's name, for the `Health` message and logs (SPEC-009 S-009-5).
pub const NAME: &str = "qrf-mipi";

#[cfg(test)]
mod tests {
    #[test]
    fn name_is_stable() {
        assert_eq!(super::NAME, "qrf-mipi");
    }
}
