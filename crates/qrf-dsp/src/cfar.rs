//! Per-slot energy detector with a cell-averaging CFAR threshold and a
//! duty-cycle estimator (SPEC-008 S-008-6; SPEC-003 S-003-8 slot plan;
//! analysis T25). Powers are in the units of the input samples squared
//! (dBFS when the input was scaled by `convert::CS8_SCALE`).

use num_complex::Complex;
use serde::Serialize;

use crate::stat::cfar_alpha_ca;

/// A channel plan: `slots` channels of `width_hz`, centres `first_centre_hz + i * step_hz`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlotPlan {
    pub name: &'static str,
    pub first_centre_hz: f64,
    pub step_hz: f64,
    pub width_hz: f64,
    pub slots: usize,
}

impl SlotPlan {
    /// The US 104-slot plan: 250 kHz slots across 902–928 MHz; index 19 (Meshtastic's one-based slot 20) at 906.875 MHz (S-003-8).
    pub const US_104: SlotPlan = SlotPlan { name: "US-104", first_centre_hz: 902.125e6, step_hz: 250e3, width_hz: 250e3, slots: 104 };

    /// Centre of zero-based slot `i`.
    pub fn centre_hz(&self, i: usize) -> f64 {
        self.first_centre_hz + self.step_hz * i as f64
    }
    /// Meshtastic's one-based channel number of zero-based slot `i`.
    pub fn meshtastic_channel(i: usize) -> usize {
        i + 1
    }
}

/// Which FFT bins (indices in `Channeliser` output order) fall in each slot: a bin belongs to the slot whose half-open width contains its centre.
#[derive(Clone, Debug)]
pub struct SlotMap {
    pub plan: SlotPlan,
    pub bins: Vec<Vec<usize>>,
}

impl SlotMap {
    /// Map for a capture centred on `centre_hz` at `fs` through an `m`-bin channeliser.
    pub fn new(plan: SlotPlan, centre_hz: f64, fs: f64, m: usize) -> Self {
        let bin_hz = fs / m as f64;
        let bins = (0..plan.slots)
            .map(|i| {
                let lo = plan.centre_hz(i) - plan.width_hz / 2.0 - centre_hz;
                let hi = lo + plan.width_hz;
                (0..m)
                    .filter(|&k| {
                        let s = if k < m / 2 { k as f64 } else { k as f64 - m as f64 };
                        let f = s * bin_hz;
                        lo <= f && f < hi
                    })
                    .collect()
            })
            .collect();
        Self { plan, bins }
    }

    /// The slot holding a frequency offset from the capture centre, if any.
    pub fn slot_for_offset(&self, offset_hz: f64, centre_hz: f64) -> Option<usize> {
        let f = offset_hz + centre_hz - (self.plan.first_centre_hz - self.plan.width_hz / 2.0);
        if f < 0.0 {
            return None;
        }
        let i = (f / self.plan.step_hz) as usize;
        (i < self.plan.slots).then_some(i)
    }
}

/// Independent looks equivalent to `n_avg` block outputs of `nbins` adjacent bins of a critically sampled bank with prototype `h` on white noise (T25): (n_avg·nbins)² / Σ over bin pairs and block lags of (n_avg − |τ|)|ρ|², ρ(dk, τ) = Σ h[n] h[n + τm] e^{j2π dk n/m} / Σ h².
pub fn effective_looks(h: &[f32], m: usize, p: usize, nbins: usize, n_avg: usize) -> f64 {
    let h: Vec<f64> = h.iter().map(|&v| v as f64).collect();
    let e2: f64 = h.iter().map(|v| v * v).sum();
    let mut tot = 0.0;
    for dk in -(nbins as i64 - 1)..nbins as i64 {
        let cnt = (nbins as i64 - dk.abs()) as f64;
        for tau in -(p as i64 - 1)..p as i64 {
            let (mut re, mut im) = (0.0f64, 0.0f64);
            for n in 0..h.len() {
                let j = n as i64 + tau * m as i64;
                if j >= 0 && (j as usize) < h.len() {
                    let a = 2.0 * std::f64::consts::PI * dk as f64 * n as f64 / m as f64;
                    re += h[n] * h[j as usize] * a.cos();
                    im += h[n] * h[j as usize] * a.sin();
                }
            }
            tot += cnt * (n_avg as f64 - tau.abs() as f64) * (re * re + im * im) / (e2 * e2);
        }
    }
    ((n_avg * nbins) as f64).powi(2) / tot
}

/// Detector parameters.
#[derive(Clone, Copy, Debug)]
pub struct DetectorConfig {
    /// Blocks averaged per look (203 blocks ≈ 1.0 ms at M = 128, 26 MSPS; T25).
    pub n_avg: usize,
    /// Independent power contributions per block per bin (1 for one element, 4 when the four elements' powers are summed).
    pub looks_per_block: usize,
    /// Training slots in total, half on each side beyond the guard.
    pub train: usize,
    /// Guard slots on each side.
    pub guard: usize,
    /// Design false-alarm probability per slot per look.
    pub pfa: f64,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self { n_avg: 203, looks_per_block: 1, train: 16, guard: 1, pfa: 1e-3 }
    }
}

/// One look: per-slot mean power, the CFAR noise estimate and the decision.
#[derive(Clone, Debug, Serialize)]
pub struct Look {
    pub seq: u64,
    pub power: Vec<f32>,
    pub noise: Vec<f32>,
    pub detected: Vec<bool>,
}

/// Accumulates bin powers over `n_avg` blocks, then decides every slot against its CA-CFAR threshold.
pub struct Detector {
    map: SlotMap,
    cfg: DetectorConfig,
    alpha: Vec<f32>,
    looks: Vec<(usize, usize)>,
    acc: Vec<f32>,
    count: usize,
    seq: u64,
}

/// The training cells of slot `i` among `n`: `half` on each side beyond `guard`, clipped to the plan.
fn training_cells(i: usize, n: usize, guard: usize, half: usize) -> impl Iterator<Item = usize> {
    (i.saturating_sub(guard + half)..i.saturating_sub(guard)).chain((i + guard + 1).min(n)..(i + guard + half + 1).min(n))
}

impl Detector {
    /// Thresholds for independent looks (k = n_avg · bins · looks_per_block); use `with_prototype` for the bank's real statistics.
    pub fn new(map: SlotMap, cfg: DetectorConfig) -> Self {
        let keff: Vec<f64> = map.bins.iter().map(|b| (cfg.n_avg * b.len().max(1)) as f64).collect();
        Self::build(map, cfg, &keff)
    }

    /// Thresholds from the ratio law with the effective looks of a bank whose prototype is `h` (`m` bins, `p` taps): T25.
    pub fn with_prototype(map: SlotMap, cfg: DetectorConfig, h: &[f32], m: usize, p: usize) -> Self {
        let mut by_bins: std::collections::HashMap<usize, f64> = std::collections::HashMap::new();
        let keff: Vec<f64> = map.bins.iter().map(|b| *by_bins.entry(b.len().max(1)).or_insert_with(|| effective_looks(h, m, p, b.len().max(1), cfg.n_avg))).collect();
        Self::build(map, cfg, &keff)
    }

    fn build(map: SlotMap, cfg: DetectorConfig, keff: &[f64]) -> Self {
        assert!(cfg.n_avg >= 1 && cfg.looks_per_block >= 1 && cfg.train >= 2 && cfg.pfa > 0.0 && cfg.pfa < 1.0);
        let m = map.bins.iter().flatten().copied().max().map_or(0, |k| k + 1);
        let n = map.bins.len();
        let e = cfg.looks_per_block as f64;
        let looks: Vec<(usize, usize)> = (0..n)
            .map(|i| {
                let cells: Vec<usize> = training_cells(i, n, cfg.guard, cfg.train / 2).collect();
                let nc = cells.len() as f64;
                let kk = nc * nc / cells.iter().map(|&j| 1.0 / (keff[j] * e)).sum::<f64>();
                ((keff[i] * e).round() as usize, kk.round() as usize)
            })
            .collect();
        let alpha = looks.iter().map(|&(k, kk)| cfar_alpha_ca(k, kk, cfg.pfa) as f32).collect();
        Self { map, cfg, alpha, looks, acc: vec![0.0; m], count: 0, seq: 0 }
    }

    pub fn map(&self) -> &SlotMap {
        &self.map
    }
    pub fn config(&self) -> &DetectorConfig {
        &self.cfg
    }
    /// The threshold multiplier of slot `i` over its training mean.
    pub fn alpha(&self, i: usize) -> f32 {
        self.alpha[i]
    }
    /// (k, K) of slot `i`: the cell's independent looks and the training mean's Gamma shape.
    pub fn looks(&self, i: usize) -> (usize, usize) {
        self.looks[i]
    }

    /// One block of channeliser bins; returns a look every `n_avg` blocks.
    pub fn push_bins(&mut self, bins: &[Complex<f32>]) -> Option<Look> {
        assert!(bins.len() >= self.acc.len());
        for (a, z) in self.acc.iter_mut().zip(bins) {
            *a += z.norm_sqr();
        }
        self.step()
    }

    /// One block of per-bin powers already summed by the caller (for instance over the four elements).
    pub fn push_power(&mut self, power: &[f32]) -> Option<Look> {
        assert!(power.len() >= self.acc.len());
        for (a, p) in self.acc.iter_mut().zip(power) {
            *a += p;
        }
        self.step()
    }

    fn step(&mut self) -> Option<Look> {
        self.count += 1;
        if self.count < self.cfg.n_avg {
            return None;
        }
        let norm = 1.0 / (self.cfg.n_avg * self.cfg.looks_per_block) as f32;
        let power: Vec<f32> = self.map.bins.iter().map(|b| if b.is_empty() { 0.0 } else { b.iter().map(|&k| self.acc[k]).sum::<f32>() * norm / b.len() as f32 }).collect();
        let n = power.len();
        let (g, half) = (self.cfg.guard, self.cfg.train / 2);
        let noise: Vec<f32> = (0..n)
            .map(|i| {
                let (mut s, mut c) = (0.0f32, 0usize);
                for j in training_cells(i, n, g, half) {
                    s += power[j];
                    c += 1;
                }
                if c == 0 { f32::INFINITY } else { s / c as f32 }
            })
            .collect();
        let detected = (0..n).map(|i| power[i] > self.alpha[i] * noise[i]).collect();
        self.acc.iter_mut().for_each(|a| *a = 0.0);
        self.count = 0;
        self.seq += 1;
        Some(Look { seq: self.seq, power, noise, detected })
    }
}

/// One slot's occupancy over a window (SPEC-009 S-009-5 `Occupancy.slots` without the bearing fields).
#[derive(Clone, Copy, Debug, Serialize)]
pub struct SlotOccupancy {
    /// 10 log10 of the mean power over the window, in the input's units (dBFS for CS8-scaled input).
    pub power_db: f32,
    /// Fraction of looks in the window that exceeded the CFAR threshold.
    pub duty: f32,
}

/// Occupancy of every slot over one window.
#[derive(Clone, Debug, Serialize)]
pub struct Occupancy {
    pub plan: &'static str,
    pub looks: usize,
    pub slots: Vec<SlotOccupancy>,
}

/// Duty-cycle estimator: counts detections and sums power over `window` looks.
pub struct Duty {
    window: usize,
    plan: &'static str,
    det: Vec<u32>,
    pw: Vec<f64>,
    looks: usize,
}

impl Duty {
    pub fn new(plan: &SlotPlan, window: usize) -> Self {
        assert!(window >= 1);
        Self { window, plan: plan.name, det: vec![0; plan.slots], pw: vec![0.0; plan.slots], looks: 0 }
    }

    /// Fold one look in; returns the window's occupancy when it completes.
    pub fn update(&mut self, look: &Look) -> Option<Occupancy> {
        for i in 0..self.det.len() {
            self.det[i] += u32::from(look.detected[i]);
            self.pw[i] += f64::from(look.power[i]);
        }
        self.looks += 1;
        if self.looks < self.window {
            return None;
        }
        let n = self.looks as f64;
        let slots = (0..self.det.len()).map(|i| SlotOccupancy { power_db: (10.0 * (self.pw[i] / n).max(1e-30).log10()) as f32, duty: self.det[i] as f32 / self.looks as f32 }).collect();
        let occ = Occupancy { plan: self.plan, looks: self.looks, slots };
        self.det.iter_mut().for_each(|d| *d = 0);
        self.pw.iter_mut().for_each(|p| *p = 0.0);
        self.looks = 0;
        Some(occ)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn us_plan_maps_every_slot_to_at_least_one_bin() {
        let map = SlotMap::new(SlotPlan::US_104, 915e6, 26e6, 128);
        let ones = map.bins.iter().filter(|b| b.len() == 1).count();
        let twos = map.bins.iter().filter(|b| b.len() == 2).count();
        assert_eq!((ones, twos), (80, 24), "T25");
        assert!(map.bins.iter().all(|b| !b.is_empty()));
        assert_eq!(SlotPlan::US_104.centre_hz(19), 906.875e6, "S-003-8: Meshtastic slot 20");
        assert_eq!(SlotPlan::meshtastic_channel(19), 20);
        assert_eq!(map.slot_for_offset(906.875e6 - 915e6, 915e6), Some(19));
        assert_eq!(map.slot_for_offset(-13e6, 915e6), Some(0));
        assert_eq!(map.slot_for_offset(-13.1e6, 915e6), None);
        assert_eq!(map.slot_for_offset(12.99e6, 915e6), Some(103));
    }

    #[test]
    fn thresholds_follow_t25() {
        use crate::filter::{KAISER_BETA_60DB, prototype};
        let map = SlotMap::new(SlotPlan::US_104, 915e6, 26e6, 128);
        let h = prototype(128, 8, KAISER_BETA_60DB);
        assert!((effective_looks(&h, 128, 8, 1, 203) - 196.5).abs() < 0.1, "T25");
        assert!((effective_looks(&h, 128, 8, 2, 203) - 388.7).abs() < 0.1, "T25");
        let d = Detector::with_prototype(map.clone(), DetectorConfig::default(), &h, 128, 8);
        assert_eq!(d.looks(50), (197, 3588), "T25 interior 1-bin slot 50");
        assert!((d.alpha(50) - 1.243).abs() < 1e-3);
        let two = (20..80).find(|&i| map.bins[i].len() == 2).unwrap();
        assert_eq!(d.looks(two), (389, 3588));
        assert!((d.alpha(two) - 1.174).abs() < 1e-3);
        assert_eq!(d.looks(0), (389, 1794), "T25 edge slot 0");
        assert!((d.alpha(0) - 1.184).abs() < 1e-3);
        // independent looks, large training mean: the gamma tail of T25 (k=203: 1.231) within the K effect
        let d = Detector::new(map, DetectorConfig::default());
        assert!(d.alpha(50) > 1.231 && d.alpha(50) < 1.25);
    }
}
