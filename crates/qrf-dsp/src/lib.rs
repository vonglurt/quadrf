//! Channeliser, CFAR detector, bearing estimation and SIMD kernels
//!
//! Implements: SPEC-008 S-008-6 (the parts that are not the LoRa demodulator,
//! which is `qrf-lora`); S-008-5 (f32 DSP; NEON confined to a leaf kernel with
//! a scalar reference and a property test); analysis T16, T19, T25. Backlog
//! R-07. Each public item names the statement it implements.
//!
//! Layout: `convert` (CS8 to `Complex<f32>`, once), `filter` (the prototype
//! low-pass), `channeliser` (the M = 128, P = 8 polyphase filter bank with a
//! scalar and a NEON filter kernel), `cfar` (the 104-slot map, the
//! cell-averaging detector and the duty-cycle estimator), `stat` (the
//! Gamma-tail threshold), `array` (element geometry, steering vectors and a
//! plane-wave simulator), `bearing` (two-element phase differences on the
//! 2 × 2 square), `music` (4 × 4 covariance, Hermitian Jacobi, MUSIC peaks),
//! `rng` (a seeded generator for tests and simulation).
//
// `unsafe` is permitted here only in leaf kernels with a safety comment, a
// scalar reference and a property test (SPEC-008 S-008-5): today only
// `channeliser::polyphase_neon`.

pub mod array;
pub mod bearing;
pub mod cfar;
pub mod channeliser;
pub mod convert;
pub mod filter;
pub mod music;
pub mod rng;
pub mod stat;

pub use array::Array;
pub use bearing::{Bearing, phase_difference};
pub use cfar::{Detector, DetectorConfig, Duty, Look, Occupancy, SlotMap, SlotPlan};
pub use channeliser::{Channeliser, Kernel};
pub use music::Music;

/// The crate's name, for the `Health` message and logs (SPEC-009 S-009-5).
pub const NAME: &str = "qrf-dsp";

#[cfg(test)]
mod tests {
    #[test]
    fn name_is_stable() {
        assert_eq!(super::NAME, "qrf-dsp");
    }
}
