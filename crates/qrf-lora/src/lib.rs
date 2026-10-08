//! LoRa chirp-spread-spectrum modulator and demodulator
//!
//! Implements: SPEC-003 S-003-1/7/12…14; SPEC-008 S-008-6. Skeleton created 2026-10-08 (backlog R-01); the behaviour
//! statements it must satisfy are in `specs/`, and each public item added here
//! names the statement it implements.
#![forbid(unsafe_code)]

/// The crate's name, for the `Health` message and logs (SPEC-009 S-009-5).
pub const NAME: &str = "qrf-lora";

#[cfg(test)]
mod tests {
    #[test]
    fn name_is_stable() {
        assert_eq!(super::NAME, "qrf-lora");
    }
}
