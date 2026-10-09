//! Channel simulation helpers for tests and corpus generation: a deterministic generator, white Gaussian
//! noise, carrier offset, the corpus channel filter and CS8 quantisation (docs/lora-corpus.md §2).

use num_complex::Complex;
use std::f64::consts::{PI, TAU};

/// xorshift64* generator: deterministic, dependency-free, adequate for test noise.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in [0, 1).
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in [lo, hi).
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.uniform()
    }

    /// Uniform integer in lo..=hi.
    pub fn int(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next_u64() % (hi - lo + 1)
    }

    /// Two independent standard normal samples (Box–Muller).
    pub fn gaussian_pair(&mut self) -> (f64, f64) {
        let u1 = (1.0 - self.uniform()).max(1e-300);
        let u2 = self.uniform();
        let r = (-2.0 * u1.ln()).sqrt();
        let (s, c) = (TAU * u2).sin_cos();
        (r * c, r * s)
    }

    /// Complex white Gaussian noise with total power `power` per sample.
    pub fn complex_noise(&mut self, power: f64) -> Complex<f64> {
        let (a, b) = self.gaussian_pair();
        let s = (power / 2.0).sqrt();
        Complex::new(a * s, b * s)
    }
}

/// Modified Bessel function of the first kind, order 0 (power series).
pub fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let q = x * x / 4.0;
    for k in 1..200 {
        term *= q / (k as f64 * k as f64);
        sum += term;
        if term < 1e-17 * sum {
            break;
        }
    }
    sum
}

/// The corpus channel filter (docs/lora-corpus.md §2): a Kaiser-window low-pass with unity DC gain, −6 dB at
/// 0.6 BW and a 60 dB stopband from 0.7 BW, designed for `os` samples per chip (75 taps at 4 × BW).
pub fn channel_filter(os: usize) -> Vec<f64> {
    let fs_bw = os as f64;
    let delta = 0.2 / fs_bw;
    let mut n = (52.0 / (2.285 * 2.0 * PI * delta)).ceil() as usize + 1;
    n |= 1;
    let m = (n - 1) as f64 / 2.0;
    let fc = 0.6 / fs_bw;
    let beta = 5.65;
    let i0b = bessel_i0(beta);
    let mut h: Vec<f64> = (0..n)
        .map(|k| {
            let x = k as f64 - m;
            let sinc = if x == 0.0 { 1.0 } else { (PI * 2.0 * fc * x).sin() / (PI * 2.0 * fc * x) };
            let r = 2.0 * k as f64 / (n as f64 - 1.0) - 1.0;
            let w = bessel_i0(beta * (1.0 - r * r).max(0.0).sqrt()) / i0b;
            2.0 * fc * sinc * w
        })
        .collect();
    let sum: f64 = h.iter().sum();
    for v in &mut h {
        *v /= sum;
    }
    h
}

/// A streaming FIR filter (direct form, history carried between calls).
pub struct Fir {
    taps: Vec<f64>,
    hist: Vec<Complex<f64>>,
}

impl Fir {
    pub fn new(taps: Vec<f64>) -> Self {
        let hist = vec![Complex::new(0.0, 0.0); taps.len().saturating_sub(1)];
        Fir { taps, hist }
    }

    pub fn group_delay(&self) -> usize {
        (self.taps.len() - 1) / 2
    }

    /// Filters `x`, continuing from the previous call; output length equals input length.
    pub fn process(&mut self, x: &[Complex<f64>]) -> Vec<Complex<f64>> {
        let l = self.hist.len();
        let mut buf = Vec::with_capacity(l + x.len());
        buf.extend_from_slice(&self.hist);
        buf.extend_from_slice(x);
        let mut y = Vec::with_capacity(x.len());
        for i in 0..x.len() {
            let mut acc = Complex::new(0.0, 0.0);
            for (k, &t) in self.taps.iter().enumerate() {
                acc += buf[i + l - k] * t;
            }
            y.push(acc);
        }
        if l > 0 {
            self.hist.copy_from_slice(&buf[buf.len() - l..]);
        }
        y
    }
}

/// Multiplies `x` by exp(j(2π·cfo_hz/fs·n + phase)).
pub fn apply_cfo(x: &[Complex<f32>], cfo_hz: f64, fs: f64, phase: f64) -> Vec<Complex<f64>> {
    let step = TAU * cfo_hz / fs;
    x.iter()
        .enumerate()
        .map(|(n, v)| {
            let (s, c) = (step * n as f64 + phase).sin_cos();
            Complex::new(f64::from(v.re), f64::from(v.im)) * Complex::new(c, s)
        })
        .collect()
}

/// Rounds, clips to int8 and interleaves I, Q; returns the bytes and the number of clipped values.
pub fn to_cs8(x: &[Complex<f64>], scale: f64) -> (Vec<u8>, usize) {
    let mut out = Vec::with_capacity(2 * x.len());
    let mut clipped = 0;
    for v in x {
        for c in [v.re * scale, v.im * scale] {
            let r = c.round();
            if !(-128.0..=127.0).contains(&r) {
                clipped += 1;
            }
            out.push(r.clamp(-128.0, 127.0) as i8 as u8);
        }
    }
    (out, clipped)
}

/// CS8 bytes to complex samples scaled by `1/scale`.
pub fn from_cs8(bytes: &[u8], scale: f32) -> Vec<Complex<f32>> {
    bytes.chunks_exact(2).map(|p| Complex::new(f32::from(p[0] as i8) / scale, f32::from(p[1] as i8) / scale)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_matches_the_corpus_design() {
        let h = channel_filter(4);
        assert_eq!(h.len(), 75, "docs/lora-corpus.md: 75 taps at 4 × BW");
        assert!((h.iter().sum::<f64>() - 1.0).abs() < 1e-12);
        let sum_sq: f64 = h.iter().map(|v| v * v).sum();
        let enbw_bw = 4.0 * sum_sq;
        assert!((enbw_bw - 1.1548846).abs() < 1e-4, "equivalent noise bandwidth {enbw_bw} BW (sidecar: 1.15488)");
    }

    #[test]
    fn noise_power_is_calibrated() {
        let mut rng = Rng::new(1);
        let n = 200_000;
        let p: f64 = (0..n).map(|_| rng.complex_noise(2.0).norm_sqr()).sum::<f64>() / n as f64;
        assert!((p - 2.0).abs() < 0.03, "{p}");
    }

    #[test]
    fn cs8_round_trip() {
        let x = vec![Complex::new(0.5, -0.25), Complex::new(-1.0, 1.0), Complex::new(2.0, -2.0)];
        let (bytes, clipped) = to_cs8(&x, 100.0);
        assert_eq!(clipped, 2);
        assert_eq!(bytes, [50, 231, 156, 100, 127, 128]);
        let back = from_cs8(&bytes, 100.0);
        assert!((back[0].re - 0.5).abs() < 1e-6 && (back[0].im + 0.25).abs() < 1e-6);
    }
}
