//! Encoder conformance against the oracle transmitter's stage outputs (backlog R-08; docs/lora-corpus.md).
//!
//! `tests/data/lora-tx-vectors.json` is written by `scripts/oracle-vectors.py`: for each case it records what
//! every transmitter stage of the oracle emitted for a payload (whitened nibbles, header nibbles, CRC
//! nibbles, Hamming codewords, interleaved words, chirp indices) and the first samples of the modulated frame.
//! `qrf-lora` must produce the same chirp indices for the same bytes, the same waveform, and decode them back.

use num_complex::Complex;
use qrf_lora::coding::{self, bytes_to_nibbles, hamming_encode, payload_crc, whiten, Header};
use qrf_lora::{Config, Ldro, Modulator};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize)]
struct Vectors {
    sync_word: u8,
    preamble_len: u16,
    cases: BTreeMap<String, Case>,
}

#[derive(Deserialize)]
struct Case {
    bw_hz: u32,
    sf: u8,
    cr: u8,
    ldro: bool,
    payload_hex: String,
    oversampling: u8,
    whitening: Vec<u8>,
    header: Vec<u8>,
    add_crc: Vec<u8>,
    hamming_enc: Vec<u8>,
    gray_demap: Vec<u16>,
    iq_head: Vec<[f32; 2]>,
    n_symbols_incl_quarter: f64,
}

fn load() -> Vectors {
    let text = include_str!("data/lora-tx-vectors.json");
    serde_json::from_str(text).expect("vectors parse")
}

fn hex(s: &str) -> Vec<u8> {
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()).collect()
}

fn config(v: &Vectors, c: &Case) -> Config {
    let mut cfg = Config::meshtastic(c.sf, c.bw_hz, c.cr);
    cfg.sync_word = v.sync_word;
    cfg.preamble_len = v.preamble_len;
    cfg.oversampling = c.oversampling;
    cfg.ldro = if c.ldro { Ldro::On } else { Ldro::Off };
    cfg
}

#[test]
fn every_stage_matches_the_oracle() {
    let v = load();
    assert!(v.cases.len() >= 29);
    for (name, c) in &v.cases {
        let payload = hex(&c.payload_hex);
        // Whitening, low nibble first.
        let mut white = payload.clone();
        whiten(&mut white);
        assert_eq!(bytes_to_nibbles(&white), c.whitening, "{name}: whitening");
        // Header nibbles then the whitened payload nibbles.
        let header = Header { payload_len: payload.len() as u8, cr: c.cr, has_crc: true };
        let mut nibbles = header.to_nibbles().to_vec();
        nibbles.extend(bytes_to_nibbles(&white));
        assert_eq!(nibbles, c.header, "{name}: header stage");
        // CRC: low byte first, each low nibble first, not whitened.
        let crc = payload_crc(&payload);
        nibbles.extend(bytes_to_nibbles(&[(crc & 0xFF) as u8, (crc >> 8) as u8]));
        assert_eq!(nibbles, c.add_crc, "{name}: CRC stage");
        // Hamming: the first SF − 2 codewords at 4/8, the rest at the frame's rate.
        let rows_h = usize::from(c.sf) - 2;
        let cws: Vec<u8> = nibbles.iter().enumerate().map(|(i, &nb)| hamming_encode(nb, if i < rows_h { 4 } else { c.cr })).collect();
        assert_eq!(cws, c.hamming_enc, "{name}: Hamming stage");
        // Interleaver + Gray mapping: the chirp indices.
        let cfg = config(&v, c);
        let modem = Modulator::new(cfg.clone()).unwrap();
        assert_eq!(modem.symbols(&payload), c.gray_demap, "{name}: chirp indices");
        // Frame length in symbols, quarter downchirp included.
        let sps = cfg.symbol_samples();
        assert_eq!(modem.frame_samples(payload.len()) as f64 / sps as f64, c.n_symbols_incl_quarter, "{name}: frame length");
        // The waveform itself.
        let x = modem.modulate(&payload);
        for (i, want) in c.iq_head.iter().enumerate() {
            let got = x[i];
            let d = (got - Complex::new(want[0], want[1])).norm();
            assert!(d < 2e-3, "{name}: sample {i} differs by {d}: {got} vs {want:?}");
        }
        // And the decoder inverts the chirp indices.
        let hb = coding::decode_header_block(&c.gray_demap[..8], c.sf).expect("header block");
        assert_eq!(hb.header, header, "{name}: header");
        let d = coding::decode_payload(&c.gray_demap[8..], c.sf, c.ldro, &hb);
        assert_eq!(d.payload, payload, "{name}: payload");
        assert_eq!(d.crc_ok, Some(true), "{name}: CRC");
        assert_eq!(d.codeword_errors, 0);
    }
}

#[test]
fn whitening_sequence_is_what_the_oracle_applies_to_zeros() {
    let v = load();
    let c = &v.cases["SHORT_TURBO_zeros32"];
    let seq = coding::whitening_sequence(32);
    assert_eq!(bytes_to_nibbles(&seq), c.whitening);
}
