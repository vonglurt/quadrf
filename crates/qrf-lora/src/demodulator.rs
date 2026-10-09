//! Streaming LoRa receiver: preamble detection, time and carrier-offset estimation from the preamble and the
//! 2¼ downchirps, sync-word check, explicit-header decoding, payload demodulation and decoding
//! (SPEC-003 S-003-1/7/14; SPEC-008 S-008-6).
//!
//! The receiver works at one sample per chip on picks of the oversampled input: the dechirped symbol is a
//! pure tone whose 2^SF-point DFT peaks at the chirp index (S-003-1). A delay of d chips moves the preamble
//! upchirps' peak to −d and the downchirps' peak to +d; a carrier offset of ε bins moves both by +ε
//! (Tapparel et al. 2020 §III, Bernier et al. 2020), so the two together give ε and d. The fractional part
//! of ε comes from the phase advance between consecutive preamble symbols.
//!
//! A window that is misaligned by a fractional chip sees the chirp fold inside it and, at one sample per
//! chip, the two sides of the fold alias onto the same tone with a phase difference of 2π times the
//! fraction: the peak is biased and weakened. The receiver therefore detects on every sample phase of the
//! oversampled stream (one of them is within half a sample of the symbol grid), estimates coarsely from
//! that phase, re-dechirps the preamble and the downchirps aligned to the coarse estimate where the
//! distortion is gone, and then picks symbol samples with cubic interpolation at the fractional timing.
//!
//! Decisions are hard (the oracle's receiver is the same class); the result is deterministic for a given
//! sample stream, whatever the chunking.

use crate::chirp::chirp;
use crate::coding::{self, Header, HeaderBlock};
use crate::params::{Config, ConfigError};
use num_complex::Complex;
use rustfft::{Fft, FftPlanner};
use std::f64::consts::TAU;
use std::sync::Arc;

type C = Complex<f32>;

/// One received frame.
#[derive(Clone, Debug, PartialEq)]
pub struct Frame {
    pub header: Header,
    /// De-whitened payload bytes (`header.payload_len` of them).
    pub payload: Vec<u8>,
    /// `Some(true)` when the header announced a CRC and it matched; `None` without a CRC.
    pub crc_ok: Option<bool>,
    /// The CRC as received, when the header announced one, so that a consumer can re-check the verdict.
    pub crc_received: Option<u16>,
    /// Estimated index, in input samples, of the first preamble sample (assuming the configured preamble length).
    pub start_sample: u64,
    /// Index of the first sample after the last payload symbol.
    pub end_sample: u64,
    /// Carrier offset of the transmitter relative to the receiver, Hz.
    pub cfo_hz: f64,
    /// Signal-to-noise ratio in the signal bandwidth, estimated from the payload symbols' dechirped spectra, dB.
    pub snr_db: f32,
    /// Mean power of the frame's samples relative to a full-scale (unit) sample, dB.
    pub rssi_dbfs: f32,
    /// Codewords the Hamming decoder could not repair (payload still delivered; the CRC says whether it is right).
    pub codeword_errors: u32,
    pub codeword_corrections: u32,
    /// Raw chirp indices: eight of the header block, then the payload blocks.
    pub symbols: Vec<u16>,
}

/// Receiver counters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats {
    pub windows: u64,
    pub detections: u64,
    pub sync_failures: u64,
    pub header_errors: u64,
    pub frames: u64,
    pub crc_failures: u64,
}

const AROUND: usize = 7;
const HALF: isize = 3;

/// The dechirped spectrum of one symbol-length window, reduced to what the receiver uses.
#[derive(Clone, Copy, Debug)]
struct Win {
    start: u64,
    bin: usize,
    /// |Y|² at the peak bin.
    peak: f32,
    /// Mean |Y|² of the other bins.
    floor: f32,
    /// Y at bins `bin − 3 … bin + 3` (circular).
    around: [C; AROUND],
}

impl Win {
    /// Y at an absolute bin, if within the stored neighbourhood.
    fn at(&self, bin: isize, n: usize) -> Option<C> {
        let d = wrap_bins(bin - self.bin as isize, n);
        (d.abs() <= HALF).then(|| self.around[(d + HALF) as usize])
    }

    /// Position of the tone relative to the peak bin, in bins, from the ratio of the larger neighbour to the
    /// peak: for a rectangular window |Y[b ± 1]| / |Y[b]| = |δ| / (1 − |δ|).
    fn frac(&self) -> f64 {
        frac_from(self.around[(HALF - 1) as usize].norm(), self.around[HALF as usize].norm(), self.around[(HALF + 1) as usize].norm())
    }

    /// Tone position in (−N/2, N/2], bins.
    fn position(&self, n: usize) -> f64 {
        wrap_f(self.bin as f64 + self.frac(), n as f64)
    }
}

fn frac_from(lower: f32, peak: f32, upper: f32) -> f64 {
    if peak <= 0.0 {
        return 0.0;
    }
    if upper >= lower {
        let r = f64::from(upper / peak).min(1.0);
        r / (1.0 + r)
    } else {
        let r = f64::from(lower / peak).min(1.0);
        -r / (1.0 + r)
    }
}

/// Signed circular difference in (−n/2, n/2].
fn wrap_bins(d: isize, n: usize) -> isize {
    let n = n as isize;
    let mut d = d.rem_euclid(n);
    if d > n / 2 {
        d -= n;
    }
    d
}

/// Trace of the synchroniser and symbol decisions on stderr when `QRF_LORA_DEBUG` is set.
fn debug() -> bool {
    std::env::var_os("QRF_LORA_DEBUG").is_some()
}

fn wrap_f(x: f64, n: f64) -> f64 {
    let mut y = x.rem_euclid(n);
    if y > n / 2.0 {
        y -= n;
    }
    y
}

/// Cubic Lagrange interpolation coefficients for a fraction `mu` ∈ [0, 1) between samples 0 and 1 of x[−1..=2].
fn lagrange4(mu: f32) -> [f32; 4] {
    let m = mu;
    [
        -m * (m - 1.0) * (m - 2.0) / 6.0,
        (m + 1.0) * (m - 1.0) * (m - 2.0) / 2.0,
        -(m + 1.0) * m * (m - 2.0) / 2.0,
        (m + 1.0) * m * (m - 1.0) / 6.0,
    ]
}

#[derive(Clone, Copy, Debug)]
struct Plan {
    /// Start of the first consistent preamble window (where scanning resumes if this frame fails).
    first: u64,
    /// Estimated start of the first sync symbol, in input samples (real-valued).
    sync_start: f64,
    /// Carrier offset in bins.
    eps: f64,
}

impl Plan {
    fn header_start(&self, sps: usize) -> f64 {
        self.sync_start + 4.25 * sps as f64
    }
}

enum State {
    Detect,
    Sync {
        first: u64,
        /// Grid start of the next window on the chosen sample phase.
        spos: u64,
        ref_peak: f32,
        scanned: usize,
        /// Best downchirp window so far and how many more windows to compare it with.
        cand: Option<(Win, usize)>,
    },
    Header {
        plan: Plan,
    },
    Payload {
        plan: Plan,
        hb: HeaderBlock,
        n_sym: usize,
        symbols: Vec<u16>,
        snr_acc: f64,
        drift: f64,
        drift_n: u32,
        /// Timing correction accumulated by tracking, in samples.
        track: f64,
    },
}

/// A streaming demodulator for one link configuration. Feed samples with [`Demodulator::process`] in any
/// chunking; frames come out as soon as their last symbol is in.
pub struct Demodulator {
    cfg: Config,
    n: usize,
    os: usize,
    sps: usize,
    ldro: bool,
    fft: Arc<dyn Fft<f32>>,
    scratch: Vec<C>,
    work: Vec<C>,
    up_ref: Vec<C>,
    down_ref: Vec<C>,
    buf: Vec<C>,
    base: u64,
    pos: u64,
    /// Sample phases scanned in detection, `phase_step` samples apart.
    phases: usize,
    phase_step: usize,
    histories: Vec<Vec<Win>>,
    history: Vec<Win>,
    state: State,
    out: Vec<Frame>,
    /// Consecutive windows that must agree before a preamble is declared.
    pub n_detect: usize,
    /// Peak-to-floor power ratio a window must reach to count.
    pub detect_threshold: f32,
    pub stats: Stats,
}

impl Demodulator {
    pub fn new(cfg: Config) -> Result<Self, ConfigError> {
        cfg.validate()?;
        let n = cfg.n();
        let os = usize::from(cfg.oversampling);
        let mut planner = FftPlanner::<f32>::new();
        let fft = planner.plan_fft_forward(n);
        let scratch = vec![C::new(0.0, 0.0); fft.get_inplace_scratch_len()];
        let up: Vec<C> = chirp(cfg.sf, 1, 0, true);
        let up_ref: Vec<C> = up.iter().map(|c| c.conj()).collect();
        let down_ref = up;
        let ldro = cfg.ldro_enabled();
        let phases = os.min(8);
        let phase_step = os / phases;
        Ok(Demodulator {
            n,
            os,
            sps: n * os,
            ldro,
            fft,
            scratch,
            work: vec![C::new(0.0, 0.0); n],
            up_ref,
            down_ref,
            buf: Vec::new(),
            base: 0,
            pos: 0,
            phases,
            phase_step,
            histories: vec![Vec::new(); phases],
            history: Vec::new(),
            state: State::Detect,
            out: Vec::new(),
            n_detect: 4,
            detect_threshold: 6.0,
            stats: Stats::default(),
            cfg,
        })
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    /// Consumes samples and returns the frames completed by them.
    pub fn process(&mut self, samples: &[C]) -> Vec<Frame> {
        self.buf.extend_from_slice(samples);
        while self.step() {}
        self.trim();
        std::mem::take(&mut self.out)
    }

    /// Absolute index one past the last buffered sample.
    fn end(&self) -> u64 {
        self.base + self.buf.len() as u64
    }

    /// Earliest sample any state may still need.
    fn keep_from(&self) -> u64 {
        match &self.state {
            State::Detect => self.histories.iter().filter_map(|h| h.first()).map(|w| w.start).min().unwrap_or(self.pos),
            State::Sync { first, .. } => *first,
            State::Header { plan } | State::Payload { plan, .. } => plan.first,
        }
    }

    fn trim(&mut self) {
        // One sample of margin stays before the earliest needed one, so an interpolated pick at that sample
        // always sees its predecessor whatever the chunking.
        let keep = self.keep_from().min(self.end()).saturating_sub(1);
        let cut = (keep.saturating_sub(self.base)) as usize;
        if cut > 4 * self.sps + 65_536 {
            self.buf.drain(..cut);
            self.base += cut as u64;
        }
    }

    fn reset_to(&mut self, pos: u64) {
        for h in &mut self.histories {
            h.clear();
        }
        self.history.clear();
        self.state = State::Detect;
        self.pos = pos;
    }

    /// Dechirps the symbol-length window whose first pick is at absolute sample `start + mu` (cubic
    /// interpolation when `mu` ≠ 0), with the up (or down) reference, after a rotation by −ε bins whose phase
    /// is 0 at `t_ref`. `None` until the samples are buffered.
    fn window(&mut self, start: u64, mu: f64, eps: f64, t_ref: f64, down: bool) -> Option<Win> {
        let i0 = usize::try_from(start.checked_sub(self.base)?).ok()?;
        let interp = mu > 1e-6;
        let last = i0 + (self.n - 1) * self.os + if interp { 2 } else { 0 };
        if last >= self.buf.len() {
            return None;
        }
        let n = self.n;
        let reference = if down { &self.down_ref } else { &self.up_ref };
        let coef = lagrange4(mu as f32);
        let pick = |buf: &[C], idx: usize| -> C {
            if interp {
                let a = if idx == 0 { buf[0] } else { buf[idx - 1] };
                a * coef[0] + buf[idx] * coef[1] + buf[idx + 1] * coef[2] + buf[idx + 2] * coef[3]
            } else {
                buf[idx]
            }
        };
        if eps != 0.0 {
            let cycles0 = -eps * ((start as f64 + mu - t_ref) / self.sps as f64);
            let mut ph = TAU * cycles0.rem_euclid(1.0);
            let step = -TAU * eps / n as f64;
            for (k, (w, r)) in self.work.iter_mut().zip(reference).enumerate() {
                let (im, re) = ph.sin_cos();
                ph += step;
                *w = pick(&self.buf, i0 + k * self.os) * C::new(re as f32, im as f32) * r;
            }
        } else {
            for (k, (w, r)) in self.work.iter_mut().zip(reference).enumerate() {
                *w = pick(&self.buf, i0 + k * self.os) * r;
            }
        }
        self.fft.process_with_scratch(&mut self.work, &mut self.scratch);
        let mut best = 0usize;
        let mut bp = -1f32;
        let mut sum = 0f32;
        for (k, y) in self.work.iter().enumerate() {
            let p = y.norm_sqr();
            sum += p;
            if p > bp {
                bp = p;
                best = k;
            }
        }
        let floor = if n > 1 { ((sum - bp) / (n - 1) as f32).max(1e-30) } else { sum.max(1e-30) };
        let mut around = [C::new(0.0, 0.0); AROUND];
        for (j, a) in around.iter_mut().enumerate() {
            let idx = (best as isize + j as isize - HALF).rem_euclid(n as isize) as usize;
            *a = self.work[idx];
        }
        Some(Win { start, bin: best, peak: bp, floor, around })
    }

    /// A window at a real-valued start: integer part plus interpolation fraction.
    fn window_f(&mut self, start: f64, eps: f64, t_ref: f64, down: bool) -> Option<Win> {
        if start < 0.0 {
            return None;
        }
        let i = start.floor();
        self.window(i as u64, start - i, eps, t_ref, down)
    }

    fn step(&mut self) -> bool {
        let state = std::mem::replace(&mut self.state, State::Detect);
        match state {
            State::Detect => self.step_detect(),
            State::Sync { first, spos, ref_peak, scanned, cand } => self.step_sync(first, spos, ref_peak, scanned, cand),
            State::Header { plan } => self.step_header(plan),
            State::Payload { plan, hb, n_sym, symbols, snr_acc, drift, drift_n, track } => {
                self.step_payload(plan, hb, n_sym, symbols, snr_acc, drift, drift_n, track)
            }
        }
    }

    fn step_detect(&mut self) -> bool {
        // All phases of this grid step must be computable before any is, so that chunking cannot change the outcome.
        let last_start = self.pos + ((self.phases - 1) * self.phase_step) as u64;
        if last_start + self.sps as u64 > self.end() {
            self.state = State::Detect;
            return false;
        }
        let n_detect = self.n_detect;
        let mut winner: Option<(usize, f32)> = None;
        for p in 0..self.phases {
            let start = self.pos + (p * self.phase_step) as u64;
            let w = self.window(start, 0.0, 0.0, 0.0, false).expect("checked above");
            self.stats.windows += 1;
            let strong = w.peak > self.detect_threshold * w.floor;
            let hist = &mut self.histories[p];
            let consistent = hist.first().is_none_or(|h| wrap_bins(w.bin as isize - h.bin as isize, self.n).abs() <= 1);
            if strong && consistent {
                hist.push(w);
            } else {
                hist.clear();
                if strong {
                    hist.push(w);
                }
            }
            if hist.len() >= n_detect {
                let mean = hist.iter().map(|h| h.peak / h.floor).sum::<f32>() / hist.len() as f32;
                if winner.is_none_or(|(_, m)| mean > m) {
                    winner = Some((p, mean));
                }
            }
        }
        self.pos += self.sps as u64;
        match winner {
            Some((p, _)) => {
                self.stats.detections += 1;
                self.history = std::mem::take(&mut self.histories[p]);
                for h in &mut self.histories {
                    h.clear();
                }
                let ref_peak = self.history.iter().map(|h| h.peak).sum::<f32>() / self.history.len() as f32;
                let spos = self.pos + (p * self.phase_step) as u64;
                self.state = State::Sync { first: self.history[0].start, spos, ref_peak, scanned: 0, cand: None };
            }
            None => self.state = State::Detect,
        }
        true
    }

    fn step_sync(&mut self, first: u64, spos: u64, ref_peak: f32, scanned: usize, cand: Option<(Win, usize)>) -> bool {
        if let Some((best, 0)) = cand {
            // The aligned windows of the refinement reach up to two symbols past the downchirp window, whichever
            // symbol it fell in; wait for them so that the outcome does not depend on how samples arrive.
            if self.end() < best.start + 4 * self.sps as u64 + 8 {
                self.state = State::Sync { first, spos, ref_peak, scanned, cand };
                return false;
            }
            match self.finish_sync(first, best) {
                Some(plan) => self.state = State::Header { plan },
                None => {
                    self.stats.sync_failures += 1;
                    self.reset_to(first + self.sps as u64);
                }
            }
            return true;
        }
        let (Some(up), Some(dn)) = (self.window(spos, 0.0, 0.0, 0.0, false), self.window(spos, 0.0, 0.0, 0.0, true)) else {
            self.state = State::Sync { first, spos, ref_peak, scanned, cand };
            return false;
        };
        self.stats.windows += 2;
        let next = spos + self.sps as u64;
        let scanned = scanned + 1;
        let b0 = self.history[0].bin;
        if cand.is_none() && up.peak > dn.peak && up.peak > self.detect_threshold * up.floor && wrap_bins(up.bin as isize - b0 as isize, self.n).abs() <= 1 {
            self.history.push(up);
        }
        let mut cand = cand;
        match cand.as_mut() {
            None => {
                if dn.peak > up.peak && dn.peak > 0.25 * ref_peak {
                    cand = Some((dn, 2));
                }
            }
            Some((best, remaining)) => {
                if dn.peak > best.peak {
                    *best = dn;
                }
                *remaining -= 1;
            }
        }
        if matches!(cand, Some((_, 0))) {
            self.state = State::Sync { first, spos: next, ref_peak, scanned, cand };
            return true;
        }
        if scanned > usize::from(self.cfg.preamble_len) + 6 {
            self.stats.sync_failures += 1;
            self.reset_to(first + self.sps as u64);
            return true;
        }
        self.state = State::Sync { first, spos: next, ref_peak, scanned, cand };
        true
    }

    /// Coarse joint time and carrier-offset estimate from the grid windows, refinement on windows aligned
    /// to it, then the sync-word check that also settles which symbol the downchirp window fell in.
    fn finish_sync(&mut self, first: u64, down: Win) -> Option<Plan> {
        let n = self.n;
        let nf = n as f64;
        let sps = self.sps as f64;
        let hist_len = self.history.len();
        if hist_len < 2 {
            return None;
        }
        // The last consistent window may straddle the first sync symbol; leave it out when there are others.
        let used = if hist_len > 2 { hist_len - 1 } else { hist_len };
        let hist: Vec<Win> = self.history[..used].to_vec();
        let mag = |b: isize| -> f64 { hist.iter().filter_map(|w| w.at(b, n)).map(|c| f64::from(c.norm())).sum() };
        let mut b = hist[0].bin as isize;
        let (m_lo, m_0, m_hi) = (mag(b - 1), mag(b), mag(b + 1));
        if m_hi > m_0 && m_hi >= m_lo {
            b += 1;
        } else if m_lo > m_0 {
            b -= 1;
        }
        let (m_lo, m_0, m_hi) = (mag(b - 1), mag(b), mag(b + 1));
        let b_up = b as f64 + frac_from(m_lo as f32, m_0 as f32, m_hi as f32);
        // Fractional carrier offset: phase advance per symbol at the preamble bin.
        let mut acc = Complex::<f64>::new(0.0, 0.0);
        for pair in hist.windows(2) {
            if pair[1].start - pair[0].start != self.sps as u64 {
                continue;
            }
            if let (Some(a), Some(c)) = (pair[0].at(b, n), pair[1].at(b, n)) {
                let a = Complex::new(f64::from(a.re), f64::from(a.im));
                let c = Complex::new(f64::from(c.re), f64::from(c.im));
                acc += c * a.conj();
            }
        }
        let eps_f = acc.arg() / TAU;
        // Coarse offset from the two chirp directions: 2ε ≡ b_up + b_down (mod N), |ε| ≤ N/4.
        let b_down = down.bin as f64 + down.frac();
        let s = (b_up + b_down).rem_euclid(nf);
        let eps_c = [s / 2.0, s / 2.0 + nf / 2.0]
            .into_iter()
            .map(|c| wrap_f(c, nf))
            .min_by(|a, b| a.abs().total_cmp(&b.abs()))?;
        if eps_c.abs() > nf / 4.0 {
            return None;
        }
        let eps1 = (eps_c - eps_f).round() + eps_f;
        let d1 = (eps1 - b_up).rem_euclid(nf);
        if wrap_f(eps1 + d1 - b_down, nf).abs() > 1.5 {
            return None;
        }
        let (e0, e1) = crate::modulator::Modulator::sync_symbols(self.cfg.sync_word);
        for j in 0..3i64 {
            // Coarse symbol grid: the downchirps start d1 chips after the chosen window, j symbols earlier.
            let tau_d1 = down.start as f64 + (d1 * self.os as f64).round() - j as f64 * sps;
            let sync1 = tau_d1 - 2.0 * sps;
            if sync1 < self.base as f64 {
                continue;
            }
            // Aligned preamble windows (as many as are buffered) and the two full downchirps.
            let mut up_pos: Vec<(f64, f32)> = Vec::new();
            let mut m = 1.0;
            while m <= f64::from(self.cfg.preamble_len) {
                let start = sync1 - m * sps;
                if start < self.base as f64 || start < first as f64 {
                    break;
                }
                let Some(w) = self.window_f(start, eps1, sync1, false) else { break };
                if w.peak > self.detect_threshold * w.floor {
                    up_pos.push((w.position(n), w.peak));
                }
                m += 1.0;
            }
            let dn_wins: Vec<Win> = [tau_d1, tau_d1 + sps].into_iter().filter_map(|t| self.window_f(t, eps1, sync1, true)).collect();
            let Some(dn_best) = dn_wins.iter().max_by(|a, b| a.peak.total_cmp(&b.peak)) else { continue };
            if up_pos.is_empty() {
                continue;
            }
            // Peak-weighted mean of the residual tone positions.
            let wsum: f32 = up_pos.iter().map(|p| p.1).sum();
            let delta_up: f64 = up_pos.iter().map(|p| p.0 * f64::from(p.1 / wsum)).sum();
            let delta_dn = dn_best.position(n);
            let eps_r = (delta_up + delta_dn) / 2.0;
            let d_r = (delta_dn - delta_up) / 2.0;
            if debug() {
                eprintln!("sync pass2: j {j} sync1 {sync1} eps1 {eps1:.3} d1 {d1:.3} | aligned up {} windows mean pos {delta_up:.3}, down pos {delta_dn:.3} -> eps_r {eps_r:.3} d_r {d_r:.3}", up_pos.len());
            }
            if eps_r.abs() > 1.5 || d_r.abs() > 1.5 {
                continue;
            }
            let mut eps = eps1 + eps_r;
            let sync_start = sync1 + d_r * self.os as f64;
            let (Some(q0), Some(q1)) = (self.window_f(sync_start, eps, sync_start, false), self.window_f(sync_start + sps, eps, sync_start, false)) else {
                continue;
            };
            let r0 = wrap_bins(q0.bin as isize - e0 as isize, n);
            let r1 = wrap_bins(q1.bin as isize - e1 as isize, n);
            if debug() {
                eprintln!("sync check: j {j} sync_start {sync_start:.2} eps {eps:.3} q0 {} ({e0}) q1 {} ({e1}) r0 {r0} r1 {r1} frac {:.3} {:.3}", q0.bin, q1.bin, q0.frac(), q1.frac());
            }
            if r0.abs() <= 2 && r1.abs() <= 2 && q0.peak > self.detect_threshold * q0.floor && q1.peak > self.detect_threshold * q1.floor {
                // Both residuals agreeing on a whole bin means the integer part of ε is off by that much.
                if r0 == r1 {
                    eps += r0 as f64;
                }
                return Some(Plan { first, sync_start, eps });
            }
        }
        None
    }

    fn step_header(&mut self, plan: Plan) -> bool {
        let sps = self.sps as f64;
        let h0 = plan.header_start(self.sps);
        let end = (h0 + 8.0 * sps).ceil() as u64 + 3;
        if end > self.end() {
            self.state = State::Header { plan };
            return false;
        }
        let mut symbols = Vec::with_capacity(8);
        for i in 0..8 {
            let w = self.window_f(h0 + i as f64 * sps, plan.eps, plan.sync_start, false).expect("header samples buffered");
            if debug() {
                eprintln!("header sym {i}: start {:.2} bin {} frac {:.3} peak/floor {:.1}", h0 + i as f64 * sps, w.bin, w.frac(), w.peak / w.floor);
            }
            symbols.push(w.bin as u16);
        }
        match coding::decode_header_block(&symbols, self.cfg.sf) {
            None => {
                self.stats.header_errors += 1;
                self.reset_to(h0 as u64);
            }
            Some(hb) => {
                let n_sym = coding::payload_symbol_count(usize::from(hb.header.payload_len), self.cfg.sf, hb.header.cr, hb.header.has_crc, self.ldro);
                self.state = State::Payload { plan, hb, n_sym, symbols, snr_acc: 0.0, drift: 0.0, drift_n: 0, track: 0.0 };
            }
        }
        true
    }

    #[allow(clippy::too_many_arguments)]
    fn step_payload(&mut self, plan: Plan, hb: HeaderBlock, n_sym: usize, mut symbols: Vec<u16>, mut snr_acc: f64, mut drift: f64, mut drift_n: u32, mut track: f64) -> bool {
        let sps = self.sps as f64;
        let h0 = plan.header_start(self.sps);
        while symbols.len() < 8 + n_sym {
            let i = symbols.len() - 8;
            let start = h0 + (8 + i) as f64 * sps + track;
            let Some(w) = self.window_f(start, plan.eps, plan.sync_start, false) else {
                self.state = State::Payload { plan, hb, n_sym, symbols, snr_acc, drift, drift_n, track };
                return false;
            };
            if debug() {
                eprintln!("payload sym {i}: start {start:.2} bin {} frac {:.3} peak/floor {:.1} track {track:.2}", w.bin, w.frac(), w.peak / w.floor);
            }
            symbols.push(w.bin as u16);
            snr_acc += f64::from((w.peak - w.floor).max(0.0) / w.floor) / self.n as f64;
            // Timing tracking: a persistent fractional offset of the tone means the sample timing has drifted
            // (a tone δ bins high means the signal arrives δ chips earlier than assumed).
            drift += w.frac();
            drift_n += 1;
            if drift_n == 8 {
                let avg = drift / 8.0;
                if avg.abs() > 0.1 {
                    track -= avg * self.os as f64;
                }
                drift = 0.0;
                drift_n = 0;
            }
        }
        let end_f = h0 + (8 + n_sym) as f64 * sps + track;
        let end = end_f.round() as u64;
        let decoded = coding::decode_payload(&symbols[8..], self.cfg.sf, self.ldro, &hb);
        let snr_db = if n_sym > 0 { (10.0 * (snr_acc / n_sym as f64).max(1e-12).log10()) as f32 } else { f32::NAN };
        let rssi_dbfs = {
            // From the first sync sample to the last sample the final symbol's window picked (always buffered).
            let a = ((plan.sync_start.max(0.0) as u64).saturating_sub(self.base)) as usize;
            let last_pick = (end_f - self.os as f64).floor().max(0.0) as u64;
            let b = ((last_pick + 1).saturating_sub(self.base) as usize).min(self.buf.len());
            if b > a {
                let p = self.buf[a..b].iter().map(|v| f64::from(v.norm_sqr())).sum::<f64>() / (b - a) as f64;
                (10.0 * p.max(1e-30).log10()) as f32
            } else {
                f32::NAN
            }
        };
        self.stats.frames += 1;
        if decoded.crc_ok == Some(false) {
            self.stats.crc_failures += 1;
        }
        let preamble = f64::from(self.cfg.preamble_len) * sps;
        self.out.push(Frame {
            header: hb.header,
            payload: decoded.payload,
            crc_ok: decoded.crc_ok,
            crc_received: decoded.crc_received,
            start_sample: (plan.sync_start - preamble).round().max(0.0) as u64,
            end_sample: end,
            cfo_hz: plan.eps * self.cfg.bin_hz(),
            snr_db,
            rssi_dbfs,
            codeword_errors: hb.errors + decoded.codeword_errors,
            codeword_corrections: hb.corrected + decoded.codeword_corrections,
            symbols,
        });
        self.reset_to(end);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modulator::Modulator;
    use crate::sim::{Rng, apply_cfo, channel_filter, Fir};

    /// Frames through an AWGN channel with carrier offset, random timing and phase; returns (sent payloads, received frames).
    fn run(cfg: Config, n_frames: usize, snr_db_in_bw: f64, cfo_ppm: f64, seed: u64, chunk: usize, filter: bool) -> (Vec<Vec<u8>>, Vec<Frame>) {
        let modem = Modulator::new(cfg.clone()).unwrap();
        let mut demod = Demodulator::new(cfg.clone()).unwrap();
        let mut rng = Rng::new(seed);
        let fs = cfg.sample_rate_hz();
        let os = f64::from(cfg.oversampling);
        let noise_power = if snr_db_in_bw.is_finite() { os / 10f64.powf(snr_db_in_bw / 10.0) } else { 0.0 };
        let mut fir = filter.then(|| Fir::new(channel_filter(cfg.oversampling as usize)));
        let mut sent = Vec::new();
        let mut stream: Vec<C> = Vec::new();
        let gap = |rng: &mut Rng, n: usize| -> Vec<Complex<f64>> { (0..n).map(|_| rng.complex_noise(noise_power)).collect() };
        for i in 0..n_frames {
            let len = rng.int(8, 32) as usize;
            let mut payload = vec![(i >> 8) as u8, i as u8];
            payload.extend((2..len).map(|_| rng.int(0, 255) as u8));
            let g = (rng.range(0.5, 2.0) * cfg.symbol_samples() as f64) as usize;
            let mut block = gap(&mut rng, g);
            let cfo = rng.range(-cfo_ppm, cfo_ppm) * 1e-6 * 915e6;
            let clean = modem.modulate(&payload);
            let mut x = apply_cfo(&clean, cfo, fs, rng.range(0.0, TAU));
            for v in &mut x {
                *v += rng.complex_noise(noise_power);
            }
            block.extend(x);
            let block = match fir.as_mut() {
                Some(f) => f.process(&block),
                None => block,
            };
            stream.extend(block.iter().map(|v| C::new(v.re as f32, v.im as f32)));
            sent.push(payload);
        }
        stream.extend(gap(&mut rng, 3 * cfg.symbol_samples()).iter().map(|v| C::new(v.re as f32, v.im as f32)));
        let mut frames = Vec::new();
        for c in stream.chunks(chunk.max(1)) {
            frames.extend(demod.process(c));
        }
        (sent, frames)
    }

    fn check_all(cfg: Config, snr: f64, cfo_ppm: f64, seed: u64, chunk: usize, filter: bool, min_ok: usize) {
        let n = 12;
        let (sent, frames) = run(cfg.clone(), n, snr, cfo_ppm, seed, chunk, filter);
        let mut ok = 0;
        for f in &frames {
            if f.crc_ok == Some(false) {
                assert!(snr < 20.0, "a CRC failure in a clean test: {f:?}");
                continue;
            }
            let idx = usize::from(f.payload[0]) << 8 | usize::from(f.payload[1]);
            assert_eq!(f.payload, sent[idx], "sf {} payload mismatch", cfg.sf);
            ok += 1;
        }
        assert!(ok >= min_ok, "sf {} bw {} os {}: {ok} of {n} frames decoded at {snr} dB", cfg.sf, cfg.bw_hz, cfg.oversampling);
    }

    #[test]
    fn clean_loopback_every_preset() {
        for (sf, bw, cr) in [(7u8, 500_000u32, 1u8), (7, 250_000, 1), (9, 250_000, 1), (11, 500_000, 4), (11, 250_000, 1), (11, 125_000, 4), (12, 125_000, 4)] {
            check_all(Config::meshtastic(sf, bw, cr), f64::INFINITY, 2.0, 7, 100_000, false, 12);
        }
    }

    #[test]
    fn loopback_at_other_oversamplings_and_chunkings() {
        for os in [1u8, 2, 8] {
            let mut cfg = Config::meshtastic(8, 250_000, 2);
            cfg.oversampling = os;
            check_all(cfg, 10.0, 2.0, 11, 1, false, 12);
        }
        check_all(Config::meshtastic(7, 500_000, 1), 5.0, 2.0, 3, 1, false, 12);
        check_all(Config::meshtastic(7, 500_000, 1), 5.0, 2.0, 3, 1 << 20, false, 12);
    }

    #[test]
    fn large_carrier_offsets_within_a_quarter_bandwidth() {
        // S-003-14: ±25 % of BW is what the SX1262 tolerates; 915 MHz × 50 ppm = 46 kHz ≈ 18 % of 250 kHz.
        check_all(Config::meshtastic(9, 250_000, 1), f64::INFINITY, 50.0, 5, 65_536, false, 12);
        check_all(Config::meshtastic(12, 125_000, 4), f64::INFINITY, 30.0, 6, 65_536, false, 12);
    }

    #[test]
    fn decodes_near_the_datasheet_threshold() {
        // 2.5 dB above the S-003-2 thresholds, through the corpus channel filter: the oracle decodes ≥ 90 % there.
        check_all(Config::meshtastic(7, 500_000, 1), -5.0, 2.0, 21, 100_000, true, 10);
        check_all(Config::meshtastic(11, 250_000, 1), -15.0, 2.0, 22, 100_000, true, 10);
    }

    #[test]
    fn deterministic_and_chunk_independent() {
        let cfg = Config::meshtastic(9, 250_000, 1);
        let (_, a) = run(cfg.clone(), 6, -10.0, 2.0, 99, 777, true);
        let (_, b) = run(cfg.clone(), 6, -10.0, 2.0, 99, 1 << 16, true);
        let (_, c) = run(cfg, 6, -10.0, 2.0, 99, 1, true);
        for (name, other) in [("65536", &b), ("1", &c)] {
            assert_eq!(a.len(), other.len(), "chunk {name}: frame count");
            for (i, (x, y)) in a.iter().zip(other).enumerate() {
                assert_eq!(x, y, "chunk {name}: frame {i} differs");
            }
        }
    }

    #[test]
    fn noise_alone_yields_no_frames() {
        let cfg = Config::meshtastic(7, 500_000, 1);
        let mut demod = Demodulator::new(cfg.clone()).unwrap();
        let mut rng = Rng::new(4242);
        let noise: Vec<C> = (0..4_000_000).map(|_| { let v = rng.complex_noise(1.0); C::new(v.re as f32, v.im as f32) }).collect();
        let frames = demod.process(&noise);
        assert!(frames.is_empty(), "{} false frames in noise", frames.len());
    }

    #[test]
    fn reports_offsets_and_timing() {
        let cfg = Config::meshtastic(7, 500_000, 1);
        let modem = Modulator::new(cfg.clone()).unwrap();
        let mut demod = Demodulator::new(cfg.clone()).unwrap();
        let lead = 1234usize;
        let cfo = 1500.0;
        let mut x: Vec<C> = vec![C::new(0.0, 0.0); lead];
        x.extend(apply_cfo(&modem.modulate(&[1, 2, 3, 4, 5, 6, 7, 8]), cfo, cfg.sample_rate_hz(), 0.3).iter().map(|v| C::new(v.re as f32, v.im as f32)));
        x.extend(std::iter::repeat_n(C::new(0.0, 0.0), 4 * cfg.symbol_samples()));
        let frames = demod.process(&x);
        assert_eq!(frames.len(), 1);
        let f = &frames[0];
        assert_eq!(f.payload, [1, 2, 3, 4, 5, 6, 7, 8]);
        assert!((f.cfo_hz - cfo).abs() < cfg.bin_hz() * 0.1, "cfo {} vs {cfo}", f.cfo_hz);
        assert!((f.start_sample as i64 - lead as i64).abs() <= 1, "start {} vs {lead}", f.start_sample);
        assert_eq!(f.end_sample as usize, lead + modem.frame_samples(8));
        assert!(f.snr_db > 30.0);
        assert!((f.rssi_dbfs - 0.0).abs() < 0.5, "rssi {}", f.rssi_dbfs);
    }
}
