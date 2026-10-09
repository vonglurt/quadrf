//! LoRa chirp-spread-spectrum modulator and demodulator (SPEC-003 S-003-1/7/12…14; SPEC-008 S-008-6).
//!
//! Written from the public descriptions of the LoRa physical layer (Tapparel et al. 2020,
//! `resources/lora/lora-phy-paper-tapparel.pdf`; Robyns et al. 2018; Knight & Seeber 2016) and checked
//! against the oracle's observable behaviour (docs/lora-corpus.md): the transmitter stage vectors in
//! `tests/data/lora-tx-vectors.json` fix every coding convention ([`coding`]), the I/Q corpus measures the
//! receiver ([`Demodulator`]). No oracle code is used; the oracle is a separate process in `make corpus`.
//!
//! - [`params::Config`]: spreading factor, bandwidth, coding rate, LDRO rule, sync word, preamble length, oversampling.
//! - [`Modulator`]: payload bytes → unit-amplitude baseband samples (preamble, sync, 2¼ downchirps, header block, payload blocks).
//! - [`Demodulator`]: a stream of samples at `oversampling × BW` → [`Frame`]s with payload, CRC verdict,
//!   carrier offset, SNR and timing; the check against the corpus is `examples/corpus_check.rs` (backlog R-08).
//! - [`sim`]: deterministic noise, carrier offset, the corpus channel filter and CS8 conversion.
#![forbid(unsafe_code)]

pub mod chirp;
pub mod coding;
pub mod demodulator;
pub mod modulator;
pub mod params;
pub mod sim;

pub use demodulator::{Demodulator, Frame, Stats};
pub use modulator::Modulator;
pub use params::{Config, ConfigError, Ldro};

/// The crate's name, for the `Health` message and logs (SPEC-009 S-009-5).
pub const NAME: &str = "qrf-lora";

#[cfg(test)]
mod tests {
    #[test]
    fn name_is_stable() {
        assert_eq!(super::NAME, "qrf-lora");
    }
}
