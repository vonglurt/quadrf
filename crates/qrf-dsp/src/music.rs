//! MUSIC on the 4 × 4 covariance (SPEC-008 S-008-6 "optionally MUSIC"):
//! sample covariance, Hermitian Jacobi eigendecomposition, the noise
//! subspace of the K smallest eigenvalues, a coarse grid over (θ, φ) in the
//! array frame, local maxima refined on a fine grid with a parabolic step.

use num_complex::Complex;

use crate::array::Array;
use crate::bearing::{Bearing, pair_sums, sigmas_deg, snr_from_coherence, wrap_deg};

type C64 = Complex<f64>;
type Mat = [[C64; 4]; 4];

/// (1/N) Σ x xᴴ.
pub fn covariance(x: [&[Complex<f32>]; 4]) -> Mat {
    let n = x.iter().map(|v| v.len()).min().unwrap_or(0);
    let mut r = [[C64::new(0.0, 0.0); 4]; 4];
    for t in 0..n {
        let v: [C64; 4] = std::array::from_fn(|e| C64::new(x[e][t].re as f64, x[e][t].im as f64));
        for i in 0..4 {
            for j in 0..4 {
                r[i][j] += v[i] * v[j].conj();
            }
        }
    }
    let s = 1.0 / n.max(1) as f64;
    for row in r.iter_mut() {
        for z in row.iter_mut() {
            *z *= s;
        }
    }
    r
}

fn matmul(a: &Mat, b: &Mat) -> Mat {
    let mut c = [[C64::new(0.0, 0.0); 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    c
}

fn adjoint(a: &Mat) -> Mat {
    let mut c = [[C64::new(0.0, 0.0); 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            c[i][j] = a[j][i].conj();
        }
    }
    c
}

fn identity() -> Mat {
    let mut c = [[C64::new(0.0, 0.0); 4]; 4];
    for (i, row) in c.iter_mut().enumerate() {
        row[i] = C64::new(1.0, 0.0);
    }
    c
}

/// Eigenvalues (descending) and the matching eigenvectors as columns, by cyclic Jacobi rotations on a Hermitian matrix.
pub fn eigh4(a: &Mat) -> ([f64; 4], Mat) {
    let mut a = *a;
    let mut v = identity();
    let trace: f64 = (0..4).map(|i| a[i][i].re).sum();
    for _ in 0..60 {
        let off: f64 = (0..4).flat_map(|p| ((p + 1)..4).map(move |q| (p, q))).map(|(p, q)| a[p][q].norm_sqr()).sum();
        if off <= 1e-30 * trace * trace {
            break;
        }
        for p in 0..4 {
            for q in (p + 1)..4 {
                let apq = a[p][q];
                let mag = apq.norm();
                if mag < 1e-300 {
                    continue;
                }
                let phi = apq.arg();
                let theta = 0.5 * (2.0 * mag).atan2(a[p][p].re - a[q][q].re);
                let (c, s) = (theta.cos(), theta.sin());
                // U = D R: D phases a[p][q] to real, R is the real Jacobi rotation
                let mut u = identity();
                u[p][p] = C64::new(c, 0.0);
                u[p][q] = C64::new(-s, 0.0);
                u[q][p] = C64::from_polar(s, -phi);
                u[q][q] = C64::from_polar(c, -phi);
                a = matmul(&matmul(&adjoint(&u), &a), &u);
                v = matmul(&v, &u);
            }
        }
    }
    let mut order = [0usize, 1, 2, 3];
    order.sort_by(|&i, &j| a[j][j].re.partial_cmp(&a[i][i].re).unwrap());
    let vals = order.map(|i| a[i][i].re);
    let mut vecs = [[C64::new(0.0, 0.0); 4]; 4];
    for (col, &i) in order.iter().enumerate() {
        for row in 0..4 {
            vecs[row][col] = v[row][i];
        }
    }
    (vals, vecs)
}

/// The MUSIC estimator for a given array, carrier and source count.
#[derive(Clone, Copy, Debug)]
pub struct Music {
    pub array: Array,
    pub f_hz: f64,
    /// Sources assumed (1–3).
    pub sources: usize,
    /// Coarse grid step, degrees.
    pub coarse_deg: f64,
    /// Fine grid step, degrees.
    pub fine_deg: f64,
    /// Largest angle from the normal searched, degrees (0–90).
    pub theta_max_deg: f64,
}

impl Music {
    pub fn new(array: Array, f_hz: f64, sources: usize) -> Self {
        assert!((1..=3).contains(&sources));
        Self { array, f_hz, sources, coarse_deg: 1.0, fine_deg: 0.05, theta_max_deg: 90.0 }
    }

    /// Noise-subspace columns of the covariance of `x`.
    pub fn noise_subspace(&self, x: [&[Complex<f32>]; 4]) -> Vec<[C64; 4]> {
        let (_, vecs) = eigh4(&covariance(x));
        (self.sources..4).map(|col| std::array::from_fn(|row| vecs[row][col])).collect()
    }

    /// MUSIC pseudo-spectrum 1/‖Eₙᴴ a‖² at (θ, φ).
    pub fn spectrum_at(&self, en: &[[C64; 4]], theta_deg: f64, phi_deg: f64) -> f64 {
        let a = self.array.steering(self.f_hz, theta_deg, phi_deg);
        let mut d = 0.0;
        for e in en {
            let mut p = C64::new(0.0, 0.0);
            for i in 0..4 {
                p += e[i].conj() * a[i];
            }
            d += p.norm_sqr();
        }
        1.0 / d.max(1e-18)
    }

    /// The pseudo-spectrum along φ at a fixed θ, in dB, for plots.
    pub fn spectrum_phi(&self, x: [&[Complex<f32>]; 4], theta_deg: f64, step_deg: f64) -> Vec<(f64, f64)> {
        let en = self.noise_subspace(x);
        let n = (360.0 / step_deg).round() as usize;
        (0..=n).map(|i| { let phi = -180.0 + i as f64 * step_deg; (phi, 10.0 * self.spectrum_at(&en, theta_deg, phi).log10()) }).collect()
    }

    /// The pseudo-spectrum along θ at a fixed φ (θ ≤ 0 means the opposite φ), in dB, for plots.
    pub fn spectrum_theta(&self, x: [&[Complex<f32>]; 4], phi_deg: f64, step_deg: f64) -> Vec<(f64, f64)> {
        let en = self.noise_subspace(x);
        let n = (2.0 * self.theta_max_deg / step_deg).round() as usize;
        (0..=n)
            .map(|i| {
                let t = -self.theta_max_deg + i as f64 * step_deg;
                let p = if t < 0.0 { self.spectrum_at(&en, -t, phi_deg + 180.0) } else { self.spectrum_at(&en, t, phi_deg) };
                (t, 10.0 * p.log10())
            })
            .collect()
    }

    /// Up to `sources` bearings, strongest peak first.
    pub fn estimate(&self, x: [&[Complex<f32>]; 4]) -> Vec<Bearing> {
        let n = x.iter().map(|v| v.len()).min().unwrap_or(0);
        let en = self.noise_subspace(x);
        let nphi = (360.0 / self.coarse_deg).round() as usize;
        let ntheta = (self.theta_max_deg / self.coarse_deg).round() as usize + 1;
        let grid: Vec<f64> = (0..ntheta).flat_map(|it| (0..nphi).map(move |ip| (ip, it))).map(|(ip, it)| self.spectrum_at(&en, it as f64 * self.coarse_deg, -180.0 + ip as f64 * self.coarse_deg)).collect();
        let at = |ip: isize, it: isize| -> f64 {
            // θ below 0 is the opposite φ; θ beyond the limit is outside the search
            let (ip, it) = if it < 0 { (ip + nphi as isize / 2, -it) } else { (ip, it) };
            let ip = ip.rem_euclid(nphi as isize) as usize;
            if it >= ntheta as isize { 0.0 } else { grid[it as usize * nphi + ip] }
        };
        let mut peaks: Vec<(f64, usize, usize)> = Vec::new();
        for it in 0..ntheta {
            for ip in 0..nphi {
                let p = grid[it * nphi + ip];
                let mut is_max = true;
                for dt in -1..=1isize {
                    for dp in -1..=1isize {
                        if (dp, dt) != (0, 0) && at(ip as isize + dp, it as isize + dt) > p {
                            is_max = false;
                        }
                    }
                }
                if is_max && (it > 0 || ip == 0) {
                    peaks.push((p, ip, it));
                }
            }
        }
        peaks.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        // the SNR and σ come from the pairs' coherence, as in `phase_difference`
        let (_, _, gamma) = pair_sums(x, n);
        let snr = snr_from_coherence(gamma);
        let kd = 2.0 * std::f64::consts::PI * self.f_hz / crate::array::C * self.array.pitch_m.unwrap_or(0.1);
        let mut out = Vec::new();
        let mut used: Vec<(f64, f64)> = Vec::new();
        for (_, ip, it) in peaks {
            if out.len() >= self.sources {
                break;
            }
            let (theta0, phi0) = (it as f64 * self.coarse_deg, -180.0 + ip as f64 * self.coarse_deg);
            if used.iter().any(|&u| Array::separation_deg(u, (theta0, phi0)) < 3.0 * self.coarse_deg) {
                continue;
            }
            let (theta, phi) = self.refine(&en, theta0, phi0);
            used.push((theta0, phi0));
            let (st, sp) = sigmas_deg(n, snr, kd, theta.to_radians());
            out.push(Bearing { theta_deg: theta, phi_deg: wrap_deg(phi), sigma_theta_deg: st, sigma_phi_deg: sp, snr_db: 10.0 * snr.log10(), n });
        }
        out
    }

    /// Fine grid of ±1.5 coarse steps around (θ0, φ0) at `fine_deg`, then a parabolic step in each axis (θ through the normal is the opposite φ).
    fn refine(&self, en: &[[C64; 4]], theta0: f64, phi0: f64) -> (f64, f64) {
        let half = (1.5 * self.coarse_deg / self.fine_deg).round() as isize;
        let (mut best, mut bi, mut bj) = (f64::MIN, 0isize, 0isize);
        let mut cache = std::collections::HashMap::new();
        let eval = |i: isize, j: isize, cache: &mut std::collections::HashMap<(isize, isize), f64>| -> f64 {
            *cache.entry((i, j)).or_insert_with(|| {
                let (t, p) = (theta0 + j as f64 * self.fine_deg, phi0 + i as f64 * self.fine_deg);
                if t < 0.0 { self.spectrum_at(en, -t, p + 180.0) } else { self.spectrum_at(en, t.min(self.theta_max_deg), p) }
            })
        };
        for j in -half..=half {
            for i in -half..=half {
                let p = eval(i, j, &mut cache);
                if p > best {
                    (best, bi, bj) = (p, i, j);
                }
            }
        }
        let step = |m: f64, c: f64, p: f64| -> f64 {
            let den = m - 2.0 * c + p;
            if den.abs() < 1e-300 { 0.0 } else { (0.5 * (m - p) / den).clamp(-0.5, 0.5) }
        };
        let lc = best.ln();
        let (lm, lp) = (eval(bi - 1, bj, &mut cache).ln(), eval(bi + 1, bj, &mut cache).ln());
        let phi = phi0 + (bi as f64 + step(lm, lc, lp)) * self.fine_deg;
        let (tm, tp) = (eval(bi, bj - 1, &mut cache).ln(), eval(bi, bj + 1, &mut cache).ln());
        let theta = theta0 + (bj as f64 + step(tm, lc, tp)) * self.fine_deg;
        if theta < 0.0 { (-theta, phi + 180.0) } else { (theta.min(self.theta_max_deg), phi) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    #[test]
    fn jacobi_diagonalises_a_hermitian_matrix() {
        let mut r = Rng::new(9);
        let mut b = [[C64::new(0.0, 0.0); 4]; 4];
        for row in b.iter_mut() {
            for z in row.iter_mut() {
                *z = r.complex_gaussian(1.0);
            }
        }
        let h = matmul(&b, &adjoint(&b));
        let (vals, vecs) = eigh4(&h);
        assert!(vals[0] >= vals[1] && vals[1] >= vals[2] && vals[2] >= vals[3] && vals[3] >= -1e-12);
        let tr: f64 = (0..4).map(|i| h[i][i].re).sum();
        assert!((vals.iter().sum::<f64>() - tr).abs() < 1e-9);
        for col in 0..4 {
            let v: [C64; 4] = std::array::from_fn(|i| vecs[i][col]);
            for i in 0..4 {
                let hv: C64 = (0..4).map(|k| h[i][k] * v[k]).sum();
                assert!((hv - v[i] * vals[col]).norm() < 1e-9, "col {col} row {i}");
            }
            for other in 0..4 {
                let dot: C64 = (0..4).map(|i| vecs[i][col].conj() * vecs[i][other]).sum();
                let want = if other == col { 1.0 } else { 0.0 };
                assert!((dot.norm() - want).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn one_source_at_20_db() {
        let a = Array::half_wave(915e6);
        let m = Music::new(a, 915e6, 1);
        let mut r = Rng::new(21);
        let (trials, n) = (60, 1024);
        let mut se = 0.0;
        for t in 0..trials {
            let (theta, phi) = (30.0 * (t as f64 + 0.5) / trials as f64, r.range(-180.0, 180.0));
            let x = a.plane_wave(915e6, theta, phi, n, 20.0, &mut r);
            let b = m.estimate([&x[0], &x[1], &x[2], &x[3]]);
            assert_eq!(b.len(), 1);
            se += Array::separation_deg((b[0].theta_deg, b[0].phi_deg), (theta, phi)).powi(2);
        }
        let rms = (se / trials as f64).sqrt();
        assert!(rms <= 0.1, "rms {rms}");
    }

    #[test]
    fn two_sources_are_resolved() {
        let a = Array::half_wave(915e6);
        let m = Music::new(a, 915e6, 2);
        let mut r = Rng::new(22);
        let x = a.plane_waves(915e6, &[(20.0, -60.0, 1.0), (25.0, 100.0, 1.0)], 4096, 0.01, &mut r);
        let b = m.estimate([&x[0], &x[1], &x[2], &x[3]]);
        assert_eq!(b.len(), 2);
        let near = |t: f64, p: f64| b.iter().any(|q| Array::separation_deg((q.theta_deg, q.phi_deg), (t, p)) < 1.0);
        assert!(near(20.0, -60.0) && near(25.0, 100.0), "{b:?}");
    }
}
