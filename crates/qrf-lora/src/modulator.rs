//! Baseband LoRa frame synthesis (SPEC-003 S-003-1/7). This produces samples for files, tests and the
//! oracle check; nothing here touches a radio (docs/00-process.md §6 transmit safety).

use crate::chirp::chirp;
use crate::coding;
use crate::params::{Config, ConfigError};
use num_complex::Complex;

/// Turns payloads into frames of unit-amplitude complex samples at `oversampling × bw_hz`.
pub struct Modulator {
    cfg: Config,
    ldro: bool,
    up0: Vec<Complex<f32>>,
    down0: Vec<Complex<f32>>,
    sync: [Vec<Complex<f32>>; 2],
}

impl Modulator {
    pub fn new(cfg: Config) -> Result<Self, ConfigError> {
        cfg.validate()?;
        let os = usize::from(cfg.oversampling);
        let ldro = cfg.ldro_enabled();
        let up0 = chirp(cfg.sf, os, 0, true);
        let down0 = chirp(cfg.sf, os, 0, false);
        let sync = [chirp(cfg.sf, os, Self::sync_symbols(cfg.sync_word).0, true), chirp(cfg.sf, os, Self::sync_symbols(cfg.sync_word).1, true)];
        Ok(Modulator { cfg, ldro, up0, down0, sync })
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    /// The two network-identifier chirps: each nibble of the sync word times eight (16 and 88 for 0x2B).
    pub fn sync_symbols(sync_word: u8) -> (u16, u16) {
        (u16::from(sync_word >> 4) << 3, u16::from(sync_word & 0xF) << 3)
    }

    /// Chirp indices of the header block and payload blocks for `payload` (at most 255 bytes).
    pub fn symbols(&self, payload: &[u8]) -> Vec<u16> {
        coding::encode_frame(self.cfg.sf, self.cfg.cr, self.ldro, self.cfg.has_crc, payload)
    }

    /// Samples in a frame carrying `payload_len` bytes: preamble, two sync symbols, 2¼ downchirps, header block, payload blocks.
    pub fn frame_samples(&self, payload_len: usize) -> usize {
        let sps = self.cfg.symbol_samples();
        let n_sym = usize::from(self.cfg.preamble_len) + 2 + 8 + coding::payload_symbol_count(payload_len, self.cfg.sf, self.cfg.cr, self.cfg.has_crc, self.ldro);
        n_sym * sps + 2 * sps + sps / 4
    }

    /// The complete frame, phase 0 at its first sample, amplitude 1.
    pub fn modulate(&self, payload: &[u8]) -> Vec<Complex<f32>> {
        let mut out = Vec::with_capacity(self.frame_samples(payload.len()));
        self.modulate_into(payload, &mut out);
        out
    }

    pub fn modulate_into(&self, payload: &[u8], out: &mut Vec<Complex<f32>>) {
        assert!(payload.len() <= 255, "LoRa payloads are at most 255 bytes");
        let sps = self.cfg.symbol_samples();
        for _ in 0..self.cfg.preamble_len {
            out.extend_from_slice(&self.up0);
        }
        out.extend_from_slice(&self.sync[0]);
        out.extend_from_slice(&self.sync[1]);
        out.extend_from_slice(&self.down0);
        out.extend_from_slice(&self.down0);
        out.extend_from_slice(&self.down0[..sps / 4]);
        let os = usize::from(self.cfg.oversampling);
        for s in self.symbols(payload) {
            out.extend(chirp(self.cfg.sf, os, s, true));
        }
    }
}
