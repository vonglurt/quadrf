//! Chirp waveforms (SPEC-003 S-003-1).

use num_complex::Complex;
use std::f64::consts::TAU;

/// One symbol of 2^SF chips at `os` samples per chip: an upchirp that starts at frequency
/// (s / 2^SF − ½)·BW, rises by BW / 2^SF per chip and folds to −BW/2 when it reaches +BW/2, with phase 0 at
/// the first sample; in chips, the phase is 2π·(t² / 2N + (s/N − ½)·t) before the fold and
/// 2π·(t² / 2N + (s/N − 3/2)·t) after it, both continuous at the fold and ending at phase 0 modulo 2π,
/// so consecutive symbols are phase-continuous. `up == false` gives the complex conjugate (the downchirp).
pub fn chirp(sf: u8, os: usize, symbol: u16, up: bool) -> Vec<Complex<f32>> {
    let n = 1usize << sf;
    let total = n * os;
    let nf = n as f64;
    let s = f64::from(symbol & (n as u16 - 1));
    let fold = (n - usize::from(symbol & (n as u16 - 1))) * os;
    (0..total)
        .map(|m| {
            let t = m as f64 / os as f64;
            let offset = if m < fold { 0.5 } else { 1.5 };
            let cycles = t * t / (2.0 * nf) + (s / nf - offset) * t;
            let (im, re) = (TAU * cycles.rem_euclid(1.0)).sin_cos();
            let c = Complex::new(re as f32, im as f32);
            if up { c } else { c.conj() }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_magnitude_and_continuity() {
        for (sf, os) in [(7u8, 1usize), (7, 4), (9, 2), (12, 4)] {
            for s in [0u16, 1, 37, (1 << sf) - 1] {
                let c = chirp(sf, os, s, true);
                assert_eq!(c.len(), (1 << sf) * os);
                assert!(c.iter().all(|x| (x.norm() - 1.0).abs() < 1e-5));
                assert!((c[0] - Complex::new(1.0, 0.0)).norm() < 1e-6, "phase 0 at the first sample");
                // The phase increment never jumps by more than the chirp's sweep allows (no discontinuity at the fold).
                let max_step = c.windows(2).map(|w| (w[1] * w[0].conj()).arg().abs()).fold(0f32, f32::max);
                assert!(max_step <= std::f32::consts::PI / os as f32 + 1e-3, "sf {sf} os {os} s {s}: step {max_step}");
            }
        }
    }

    #[test]
    fn dechirp_puts_symbol_s_in_bin_s() {
        // At one sample per chip the dechirped symbol is a pure tone whose DFT peaks at bin s (S-003-1).
        let sf = 8u8;
        let n = 1usize << sf;
        let base = chirp(sf, 1, 0, true);
        for s in [0u16, 1, 100, 255] {
            let x = chirp(sf, 1, s, true);
            let y: Vec<Complex<f64>> = x.iter().zip(&base).map(|(a, b)| {
                let p = a * b.conj();
                Complex::new(f64::from(p.re), f64::from(p.im))
            }).collect();
            let mut best = (0usize, 0f64);
            for k in 0..n {
                let mut acc = Complex::new(0.0, 0.0);
                for (m, v) in y.iter().enumerate() {
                    let ph = -TAU * (k * m) as f64 / n as f64;
                    acc += v * Complex::new(ph.cos(), ph.sin());
                }
                if acc.norm() > best.1 {
                    best = (k, acc.norm());
                }
            }
            assert_eq!(best.0, usize::from(s));
            assert!((best.1 - n as f64).abs() < 1e-6 * n as f64, "all energy in one bin");
        }
    }
}
