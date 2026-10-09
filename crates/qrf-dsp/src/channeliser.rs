//! The polyphase channeliser: M bins over the capture band, P taps per phase,
//! critically sampled (SPEC-008 S-008-6; analysis T19, T25).
//!
//! Each block of M input samples yields one output sample in every bin. The
//! filter stage is the polyphase decomposition of the prototype in `filter`:
//! the branch for input residue m (sample `m` of a block) holds the taps
//! h[pM + M − 1 − m] for the block p steps back, which is the full-rate
//! convolution evaluated once per block; an M-point FFT then separates the
//! bins. For an input tone at ω = 2πk/M + δ the output in bin k is
//! e^{jδ(nM + M − 1)}·H(δ) exactly, so a tone at a bin centre passes with the
//! prototype's unit DC gain and a tone off centre is weighted by the
//! prototype's response at the offset (`filter::response_db`).
//!
//! The filter stage is one f32 multiply-add over P blocks of 2M floats; the
//! scalar kernel is the reference and `polyphase_neon` the leaf kernel with
//! `vfmaq_f32` (S-008-5), compared on random data in the tests.

use std::sync::Arc;

use num_complex::Complex;
use rustfft::{Fft, FftPlanner};

use crate::filter::{KAISER_BETA_60DB, prototype};

/// Bins over the 26 MHz capture (T19, T25).
pub const M: usize = 128;
/// Taps per phase (T19, T25).
pub const P: usize = 8;
/// The tile's sample rate per element (T14).
pub const FS_HZ: f64 = 26e6;

/// Which filter kernel `push_block` uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kernel {
    Scalar,
    #[cfg(target_arch = "aarch64")]
    Neon,
}

impl Kernel {
    /// The fastest kernel for this target.
    pub const fn fastest() -> Self {
        #[cfg(target_arch = "aarch64")]
        {
            Kernel::Neon
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            Kernel::Scalar
        }
    }
    pub const fn name(self) -> &'static str {
        match self {
            Kernel::Scalar => "scalar",
            #[cfg(target_arch = "aarch64")]
            Kernel::Neon => "neon",
        }
    }
}

/// One channel's filter bank state.
pub struct Channeliser {
    m: usize,
    p: usize,
    /// Taps as f32 pairs, `p` rows of `2m`: row q, pair i holds h[qM + M − 1 − i] twice (I and Q).
    taps: Vec<f32>,
    /// Block history as f32 pairs, `p` rows of `2m`, a ring over rows.
    hist: Vec<f32>,
    newest: usize,
    v: Vec<f32>,
    fft: Arc<dyn Fft<f32>>,
    scratch: Vec<Complex<f32>>,
    kernel: Kernel,
}

impl Channeliser {
    /// The S-008-6 bank: `m` bins, `p` taps, the 60 dB Kaiser prototype.
    pub fn new(m: usize, p: usize) -> Self {
        Self::with_prototype(m, p, &prototype(m, p, KAISER_BETA_60DB))
    }

    /// A bank with a caller-supplied prototype of `m * p` taps.
    pub fn with_prototype(m: usize, p: usize, h: &[f32]) -> Self {
        assert!(m >= 2 && p >= 1 && h.len() == m * p, "m {m} p {p} taps {}", h.len());
        let mut taps = vec![0f32; p * 2 * m];
        for q in 0..p {
            for i in 0..m {
                let t = h[q * m + m - 1 - i];
                taps[q * 2 * m + 2 * i] = t;
                taps[q * 2 * m + 2 * i + 1] = t;
            }
        }
        let fft = FftPlanner::new().plan_fft_forward(m);
        let scratch = vec![Complex::new(0.0, 0.0); fft.get_inplace_scratch_len()];
        Self { m, p, taps, hist: vec![0f32; p * 2 * m], newest: 0, v: vec![0f32; 2 * m], fft, scratch, kernel: Kernel::fastest() }
    }

    pub fn m(&self) -> usize {
        self.m
    }
    pub fn p(&self) -> usize {
        self.p
    }
    pub fn kernel(&self) -> Kernel {
        self.kernel
    }
    pub fn set_kernel(&mut self, k: Kernel) {
        self.kernel = k;
    }
    /// Bin width in Hz at sample rate `fs`.
    pub fn bin_hz(&self, fs: f64) -> f64 {
        fs / self.m as f64
    }
    /// Frequency offset from the capture centre of FFT bin `k` (bins above m/2 are negative).
    pub fn bin_offset_hz(&self, k: usize, fs: f64) -> f64 {
        let s = if k < self.m / 2 { k as f64 } else { k as f64 - self.m as f64 };
        s * self.bin_hz(fs)
    }
    /// The FFT bin whose centre is nearest to `offset_hz` from the capture centre.
    pub fn bin_for_offset(&self, offset_hz: f64, fs: f64) -> usize {
        let s = (offset_hz / self.bin_hz(fs)).round() as i64;
        s.rem_euclid(self.m as i64) as usize
    }
    /// Forget the history.
    pub fn reset(&mut self) {
        self.hist.iter_mut().for_each(|v| *v = 0.0);
        self.newest = 0;
    }

    /// One block of `m` input samples in, `m` bin outputs out (FFT order: bin k at index k).
    pub fn push_block(&mut self, block: &[Complex<f32>], out: &mut [Complex<f32>]) {
        let (m, p, n2) = (self.m, self.p, 2 * self.m);
        assert!(block.len() == m && out.len() >= m, "block {} out {}", block.len(), out.len());
        self.newest = (self.newest + p - 1) % p;
        let row = &mut self.hist[self.newest * n2..(self.newest + 1) * n2];
        for (pair, z) in row.chunks_exact_mut(2).zip(block) {
            pair[0] = z.re;
            pair[1] = z.im;
        }
        match self.kernel {
            Kernel::Scalar => polyphase_scalar(&self.taps, &self.hist, self.newest, p, n2, &mut self.v),
            #[cfg(target_arch = "aarch64")]
            Kernel::Neon => polyphase_neon(&self.taps, &self.hist, self.newest, p, n2, &mut self.v),
        }
        for (o, pair) in out[..m].iter_mut().zip(self.v.chunks_exact(2)) {
            *o = Complex::new(pair[0], pair[1]);
        }
        self.fft.process_with_scratch(&mut out[..m], &mut self.scratch);
    }

    /// Whole blocks of `x` through the bank; `out` receives `blocks * m` bins, block after block. Returns the block count.
    pub fn process(&mut self, x: &[Complex<f32>], out: &mut [Complex<f32>]) -> usize {
        let blocks = x.len() / self.m;
        assert!(out.len() >= blocks * self.m, "out holds {}, {} needed", out.len(), blocks * self.m);
        for b in 0..blocks {
            let (bx, bo) = (&x[b * self.m..(b + 1) * self.m], &mut out[b * self.m..(b + 1) * self.m]);
            self.push_block(bx, bo);
        }
        blocks
    }
}

/// Reference filter stage: v[i] = Σ_q taps[q][i] · hist[(newest + q) mod p][i] over `n2` floats.
pub fn polyphase_scalar(taps: &[f32], hist: &[f32], newest: usize, p: usize, n2: usize, v: &mut [f32]) {
    assert!(taps.len() >= p * n2 && hist.len() >= p * n2 && v.len() >= n2);
    v[..n2].iter_mut().for_each(|x| *x = 0.0);
    for q in 0..p {
        let row = (newest + q) % p;
        let t = &taps[q * n2..(q + 1) * n2];
        let h = &hist[row * n2..(row + 1) * n2];
        for ((vi, ti), hi) in v[..n2].iter_mut().zip(t).zip(h) {
            *vi += ti * hi;
        }
    }
}

/// NEON filter stage: sixteen floats (four accumulators) per pass over the `p` rows, so that each tap and history float is loaded once and the accumulators stay in registers; `n2` must be a multiple of 16 (M is a power of two ≥ 8).
#[cfg(target_arch = "aarch64")]
pub fn polyphase_neon(taps: &[f32], hist: &[f32], newest: usize, p: usize, n2: usize, v: &mut [f32]) {
    use std::arch::aarch64::{vdupq_n_f32, vfmaq_f32, vld1q_f32, vst1q_f32};
    assert!(taps.len() >= p * n2 && hist.len() >= p * n2 && v.len() >= n2 && n2 % 16 == 0);
    // SAFETY (S-008-5): every load reads 4 floats at offsets below `p * n2`
    // of `taps` and `hist` and every store writes 4 floats below `n2` of `v`,
    // all checked above (i + 12 + 4 ≤ n2 because n2 % 16 == 0); NEON float
    // loads and stores accept unaligned addresses; NEON is mandatory on
    // AArch64 Linux targets. The test `neon_equals_scalar` compares this
    // kernel with `polyphase_scalar` on random taps and histories.
    unsafe {
        let (tp, hp, vp) = (taps.as_ptr(), hist.as_ptr(), v.as_mut_ptr());
        let mut i = 0;
        while i < n2 {
            let mut a0 = vdupq_n_f32(0.0);
            let mut a1 = vdupq_n_f32(0.0);
            let mut a2 = vdupq_n_f32(0.0);
            let mut a3 = vdupq_n_f32(0.0);
            for q in 0..p {
                let t = tp.add(q * n2 + i);
                let h = hp.add(((newest + q) % p) * n2 + i);
                a0 = vfmaq_f32(a0, vld1q_f32(t), vld1q_f32(h));
                a1 = vfmaq_f32(a1, vld1q_f32(t.add(4)), vld1q_f32(h.add(4)));
                a2 = vfmaq_f32(a2, vld1q_f32(t.add(8)), vld1q_f32(h.add(8)));
                a3 = vfmaq_f32(a3, vld1q_f32(t.add(12)), vld1q_f32(h.add(12)));
            }
            vst1q_f32(vp.add(i), a0);
            vst1q_f32(vp.add(i + 4), a1);
            vst1q_f32(vp.add(i + 8), a2);
            vst1q_f32(vp.add(i + 12), a3);
            i += 16;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;
    use std::f64::consts::PI;

    fn tone(n: usize, cycles_per_block: f64, m: usize, amp: f32) -> Vec<Complex<f32>> {
        (0..n).map(|i| { let a = 2.0 * PI * cycles_per_block * i as f64 / m as f64; Complex::new(amp * a.cos() as f32, amp * a.sin() as f32) }).collect()
    }

    /// Mean output power in bin `k` over the blocks after the filter has filled.
    fn bin_power(ch: &mut Channeliser, x: &[Complex<f32>], k: usize) -> f64 {
        let m = ch.m();
        let mut out = vec![Complex::new(0.0, 0.0); x.len()];
        let blocks = ch.process(x, &mut out);
        let skip = ch.p();
        let mut s = 0.0;
        for b in skip..blocks {
            s += out[b * m + k].norm_sqr() as f64;
        }
        s / (blocks - skip) as f64
    }

    #[test]
    fn tone_at_bin_centre_passes_at_unit_gain() {
        let mut ch = Channeliser::new(M, P);
        for k in [0usize, 1, 17, 63, 64, 100, 127] {
            let cycles = if k < M / 2 { k as f64 } else { k as f64 - M as f64 };
            let x = tone(64 * M, cycles, M, 0.5);
            let p = bin_power(&mut ch, &x, k);
            let db = 10.0 * (p / 0.25).log10();
            assert!(db.abs() < 0.01, "bin {k}: {db} dB");
            // leakage into the neighbouring bins is below the 60 dB design (the prototype at ±1 bin)
            let p1 = bin_power(&mut ch, &x, (k + 1) % M);
            assert!(10.0 * (p1 / 0.25).log10() < -50.0);
            ch.reset();
        }
    }

    #[test]
    fn off_centre_tone_follows_the_prototype_response() {
        let mut ch = Channeliser::new(M, P);
        let h = prototype(M, P, KAISER_BETA_60DB);
        for off in [0.1f64, 0.25, 0.5] {
            let x = tone(64 * M, 10.0 + off, M, 0.5);
            let db = 10.0 * (bin_power(&mut ch, &x, 10) / 0.25).log10();
            let want = crate::filter::response_db(&h, M, off);
            assert!((db - want).abs() < 0.05, "offset {off}: {db} vs {want}");
            ch.reset();
        }
    }

    #[test]
    fn scalar_and_fastest_kernels_agree() {
        let mut r = Rng::new(7);
        let x = r.complex_gaussian_f32(32 * M, 1.0);
        let mut a = Channeliser::new(M, P);
        a.set_kernel(Kernel::Scalar);
        let mut b = Channeliser::new(M, P);
        let (mut oa, mut ob) = (vec![Complex::new(0.0, 0.0); x.len()], vec![Complex::new(0.0, 0.0); x.len()]);
        a.process(&x, &mut oa);
        b.process(&x, &mut ob);
        let worst = oa.iter().zip(&ob).map(|(p, q)| (p - q).norm()).fold(0.0f32, f32::max);
        let scale = oa.iter().map(|z| z.norm()).fold(0.0f32, f32::max);
        assert!(worst <= 4.0 * f32::EPSILON * scale, "worst {worst} of {scale}");
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn neon_equals_scalar() {
        let mut r = Rng::new(3);
        for trial in 0..2000 {
            let (p, n2) = (1 + (r.next_u64() % 12) as usize, 16 * (1 + (r.next_u64() % 20) as usize));
            let taps: Vec<f32> = (0..p * n2).map(|_| (r.unit() * 2.0 - 1.0) as f32).collect();
            let hist: Vec<f32> = (0..p * n2).map(|_| (r.unit() * 2.0 - 1.0) as f32).collect();
            let newest = (r.next_u64() % p as u64) as usize;
            let (mut vs, mut vn) = (vec![0f32; n2], vec![0f32; n2]);
            polyphase_scalar(&taps, &hist, newest, p, n2, &mut vs);
            polyphase_neon(&taps, &hist, newest, p, n2, &mut vn);
            for i in 0..n2 {
                // fused and separate rounding: within p half-ulps of the partial sums' scale
                assert!((vs[i] - vn[i]).abs() <= p as f32 * f32::EPSILON * (vs[i].abs() + 1.0), "trial {trial} i {i}: {} vs {}", vs[i], vn[i]);
            }
        }
    }

    #[test]
    fn bin_indexing() {
        let ch = Channeliser::new(M, P);
        assert_eq!(ch.bin_hz(FS_HZ), 203_125.0);
        assert_eq!(ch.bin_offset_hz(1, FS_HZ), 203_125.0);
        assert_eq!(ch.bin_offset_hz(127, FS_HZ), -203_125.0);
        assert_eq!(ch.bin_for_offset(-203_125.0, FS_HZ), 127);
        assert_eq!(ch.bin_for_offset(12.8e6, FS_HZ), 63);
    }
}
