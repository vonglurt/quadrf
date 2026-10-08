//! QuadRF transceiver control through the CSI node's JTAG ioctls
//!
//! Implements: SPEC-002 S-002-16; SPEC-008 S-008-15 (vendor CLI as child until the register map is observed, R-04). Skeleton created 2026-10-08 (backlog R-01); the behaviour
//! statements it must satisfy are in `specs/`, and each public item added here
//! names the statement it implements.
#![forbid(unsafe_code)]

/// The crate's name, for the `Health` message and logs (SPEC-009 S-009-5).
pub const NAME: &str = "qrf-jtag";

#[cfg(test)]
mod tests {
    #[test]
    fn name_is_stable() {
        assert_eq!(super::NAME, "qrf-jtag");
    }
}
