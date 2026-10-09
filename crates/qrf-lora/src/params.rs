//! Air-interface parameters of a LoRa link (SPEC-003 S-003-1, S-003-7, S-003-12).

use std::fmt;

/// Lowest spreading factor the SX1261/2 family offers (S-003-12).
pub const SF_MIN: u8 = 5;
/// Highest spreading factor (S-003-12).
pub const SF_MAX: u8 = 12;
/// Symbol duration above which Meshtastic (and the oracle's AUTO rule) enable low-data-rate optimisation (S-003-7, S-003-14).
pub const LDRO_THRESHOLD_MS: f64 = 16.0;

/// Low-data-rate optimisation: payload symbols carry SF − 2 bits instead of SF.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ldro {
    Off,
    On,
    /// On when the symbol lasts more than [`LDRO_THRESHOLD_MS`] (2^SF / BW > 16 ms).
    Auto,
}

/// Everything both ends of a link agree on before a frame is sent.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// Spreading factor, [`SF_MIN`]..=[`SF_MAX`].
    pub sf: u8,
    /// Signal bandwidth in Hz (125 000, 250 000 or 500 000 for the Meshtastic presets, S-003-6).
    pub bw_hz: u32,
    /// Coding rate 1..=4, meaning 4/5..4/8. The demodulator reads it from the explicit header; the modulator writes it there.
    pub cr: u8,
    pub ldro: Ldro,
    /// Network identifier, 0x2B for Meshtastic (S-003-7).
    pub sync_word: u8,
    /// Number of preamble upchirps, 16 for Meshtastic (S-003-7).
    pub preamble_len: u16,
    /// Whether the modulator appends the 16-bit payload CRC (always on for Meshtastic, S-003-7).
    pub has_crc: bool,
    /// Samples per chip: the sample rate is `oversampling × bw_hz`.
    pub oversampling: u8,
}

/// A parameter outside the range this crate supports.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigError {
    SpreadingFactor(u8),
    CodingRate(u8),
    Bandwidth(u32),
    Oversampling(u8),
    PreambleLength(u16),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::SpreadingFactor(v) => write!(f, "spreading factor {v} outside {SF_MIN}..={SF_MAX}"),
            ConfigError::CodingRate(v) => write!(f, "coding rate {v} outside 1..=4"),
            ConfigError::Bandwidth(v) => write!(f, "bandwidth {v} Hz is not positive"),
            ConfigError::Oversampling(v) => write!(f, "oversampling {v} outside 1..=16"),
            ConfigError::PreambleLength(v) => write!(f, "preamble of {v} symbols is shorter than 6"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl Config {
    /// A Meshtastic link (S-003-7): sync word 0x2B, 16-symbol preamble, CRC on, LDRO by the 16 ms rule, 4 samples per chip.
    pub fn meshtastic(sf: u8, bw_hz: u32, cr: u8) -> Self {
        Config { sf, bw_hz, cr, ldro: Ldro::Auto, sync_word: 0x2B, preamble_len: 16, has_crc: true, oversampling: 4 }
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if !(SF_MIN..=SF_MAX).contains(&self.sf) {
            return Err(ConfigError::SpreadingFactor(self.sf));
        }
        if !(1..=4).contains(&self.cr) {
            return Err(ConfigError::CodingRate(self.cr));
        }
        if self.bw_hz == 0 {
            return Err(ConfigError::Bandwidth(self.bw_hz));
        }
        if !(1..=16).contains(&self.oversampling) {
            return Err(ConfigError::Oversampling(self.oversampling));
        }
        if self.preamble_len < 6 {
            return Err(ConfigError::PreambleLength(self.preamble_len));
        }
        Ok(())
    }

    /// Chips per symbol, 2^SF.
    pub fn n(&self) -> usize {
        1usize << self.sf
    }

    /// Samples per symbol at the configured oversampling.
    pub fn symbol_samples(&self) -> usize {
        self.n() * self.oversampling as usize
    }

    pub fn sample_rate_hz(&self) -> f64 {
        f64::from(self.bw_hz) * f64::from(self.oversampling)
    }

    /// Symbol duration in milliseconds, 2^SF / BW.
    pub fn symbol_ms(&self) -> f64 {
        self.n() as f64 * 1e3 / f64::from(self.bw_hz)
    }

    /// Whether payload symbols are sent at the reduced rate (S-003-7: Meshtastic's rule is the 16 ms threshold).
    pub fn ldro_enabled(&self) -> bool {
        match self.ldro {
            Ldro::Off => false,
            Ldro::On => true,
            Ldro::Auto => self.symbol_ms() > LDRO_THRESHOLD_MS,
        }
    }

    /// Width of one DFT bin in Hz, BW / 2^SF (S-003-1).
    pub fn bin_hz(&self) -> f64 {
        f64::from(self.bw_hz) / self.n() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ldro_rule_matches_the_presets() {
        // S-003-6 presets: LDRO only for LONG_MODERATE (125 k/SF11, 16.4 ms) and LONG_SLOW (125 k/SF12, 32.8 ms).
        assert!(!Config::meshtastic(7, 500_000, 1).ldro_enabled());
        assert!(!Config::meshtastic(11, 250_000, 1).ldro_enabled());
        assert!(!Config::meshtastic(11, 500_000, 4).ldro_enabled());
        assert!(Config::meshtastic(11, 125_000, 4).ldro_enabled());
        assert!(Config::meshtastic(12, 125_000, 4).ldro_enabled());
    }

    #[test]
    fn validation() {
        assert!(Config::meshtastic(7, 500_000, 1).validate().is_ok());
        assert_eq!(Config::meshtastic(13, 500_000, 1).validate(), Err(ConfigError::SpreadingFactor(13)));
        assert_eq!(Config::meshtastic(7, 500_000, 0).validate(), Err(ConfigError::CodingRate(0)));
        assert_eq!(Config::meshtastic(7, 0, 1).validate(), Err(ConfigError::Bandwidth(0)));
    }
}
