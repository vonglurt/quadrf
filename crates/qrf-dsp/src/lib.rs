//! Channeliser, CFAR detector, bearing estimation and SIMD kernels
//!
//! Implements: SPEC-008 S-008-6; analysis T16, T19. Skeleton created 2026-10-08 (backlog R-01); the behaviour
//! statements it must satisfy are in `specs/`, and each public item added here
//! names the statement it implements.
// `unsafe` is permitted here only in leaf kernels/ioctl wrappers with a safety comment, a scalar reference and a property test (SPEC-008 S-008-5).

/// The crate's name, for the `Health` message and logs (SPEC-009 S-009-5).
pub const NAME: &str = "qrf-dsp";

#[cfg(test)]
mod tests {
    #[test]
    fn name_is_stable() {
        assert_eq!(super::NAME, "qrf-dsp");
    }
}
