//! Two-element phase-difference bearing on the 2 × 2 square (SPEC-008
//! S-008-6; analysis T16, T25): the cross products of the two x pairs are
//! summed, likewise the two y pairs; the two phase differences
//! Δx = k d sin θ cos φ and Δy = k d sin θ sin φ give φ by their ratio and θ
//! by their magnitude. σ is the CRLB at the SNR estimated from the pairs'
//! coherence.

use num_complex::Complex;
use serde::Serialize;

use crate::array::{Array, C};

/// A bearing in the array frame (`array` module), with the two standard mountings to SPEC-009 S-009-5 azimuth and elevation.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Bearing {
    /// Angle from the array normal, degrees (0 = boresight).
    pub theta_deg: f64,
    /// In-plane angle from +x toward +y, degrees, in (−180, 180].
    pub phi_deg: f64,
    /// One standard deviation of `theta_deg` (the T16 form, 1/(√(N·SNR)·k d cos θ) at high SNR, halved in variance by the two parallel pairs).
    pub sigma_theta_deg: f64,
    /// One standard deviation of `phi_deg` (σ_Δ/(k d sin θ); unbounded at boresight, where φ is undefined).
    pub sigma_phi_deg: f64,
    /// SNR per element estimated from the pairs' coherence, dB.
    pub snr_db: f64,
    /// Samples used.
    pub n: usize,
}

impl Bearing {
    /// Horizontal array, normal up, +x east, +y north: azimuth clockwise from north, elevation above the horizon.
    pub fn horizontal_mount(&self) -> (f64, f64) {
        (wrap_deg(90.0 - self.phi_deg), 90.0 - self.theta_deg)
    }
    /// Vertical array facing forward: +x to the right, +y up, normal forward: azimuth to the right of boresight, elevation above it.
    pub fn forward_mount(&self) -> (f64, f64) {
        let u = Array::direction(self.theta_deg, self.phi_deg);
        (u[0].atan2(u[2]).to_degrees(), u[1].clamp(-1.0, 1.0).asin().to_degrees())
    }
}

/// Into (−180, 180].
pub fn wrap_deg(a: f64) -> f64 {
    let w = (a + 180.0).rem_euclid(360.0) - 180.0;
    if w == -180.0 { 180.0 } else { w }
}

/// Σ x_i x_j* and the two energies.
pub fn cross(xi: &[Complex<f32>], xj: &[Complex<f32>]) -> (Complex<f64>, f64, f64) {
    let (mut c, mut ei, mut ej) = (Complex::new(0.0f64, 0.0), 0.0f64, 0.0f64);
    for (a, b) in xi.iter().zip(xj) {
        let (a, b) = (Complex::new(a.re as f64, a.im as f64), Complex::new(b.re as f64, b.im as f64));
        c += a * b.conj();
        ei += a.norm_sqr();
        ej += b.norm_sqr();
    }
    (c, ei, ej)
}

/// SNR per element from the mean magnitude coherence γ of the pairs: γ = ρ/(1 + ρ).
pub fn snr_from_coherence(gamma: f64) -> f64 {
    let g = gamma.clamp(0.0, 1.0 - 1e-9);
    g / (1.0 - g)
}

/// Variance of a pair's phase difference from `n` samples at SNR ρ: (1 + 2ρ)/(2 n ρ²) (→ 1/(nρ) at high SNR, the T16 form).
pub fn phase_variance(n: usize, snr: f64) -> f64 {
    (1.0 + 2.0 * snr) / (2.0 * n as f64 * snr * snr)
}

/// Mean magnitude coherence over the four pairs and the two summed cross products (x pairs, y pairs).
pub fn pair_sums(x: [&[Complex<f32>]; 4], n: usize) -> (Complex<f64>, Complex<f64>, f64) {
    let mut sum_x = Complex::new(0.0f64, 0.0);
    let mut sum_y = Complex::new(0.0f64, 0.0);
    let mut gamma = 0.0;
    for (i, j) in Array::X_PAIRS {
        let (c, ei, ej) = cross(&x[j][..n], &x[i][..n]);
        sum_x += c;
        gamma += c.norm() / (ei * ej).sqrt().max(1e-300);
    }
    for (i, j) in Array::Y_PAIRS {
        let (c, ei, ej) = cross(&x[j][..n], &x[i][..n]);
        sum_y += c;
        gamma += c.norm() / (ei * ej).sqrt().max(1e-300);
    }
    (sum_x, sum_y, gamma / 4.0)
}

/// The σ pair (θ, φ) in degrees for `n` samples at `snr` on a square of `kd`, at `theta` radians.
pub fn sigmas_deg(n: usize, snr: f64, kd: f64, theta: f64) -> (f64, f64) {
    // each difference is the average of two pairs: half the single-pair variance
    let sigma_d = (phase_variance(n, snr) / 2.0).sqrt();
    ((sigma_d / (kd * theta.cos().max(1e-9))).to_degrees(), (sigma_d / (kd * theta.sin().max(1e-9))).to_degrees())
}

/// The estimator. `x` holds the four elements' samples of one bin over one burst; `f_hz` is the true carrier (S-009-8).
pub fn phase_difference(array: &Array, f_hz: f64, x: [&[Complex<f32>]; 4]) -> Bearing {
    let d = array.pitch_m.expect("phase_difference needs a square array");
    let n = x.iter().map(|v| v.len()).min().unwrap_or(0);
    let kd = 2.0 * std::f64::consts::PI * f_hz / C * d;
    let (sum_x, sum_y, gamma) = pair_sums(x, n);
    let (dx, dy) = (sum_x.arg(), sum_y.arg());
    let phi = dy.atan2(dx);
    let theta = ((dx * dx + dy * dy).sqrt() / kd).min(1.0).asin();
    let snr = snr_from_coherence(gamma);
    let (st, sp) = sigmas_deg(n, snr, kd, theta);
    Bearing { theta_deg: theta.to_degrees(), phi_deg: wrap_deg(phi.to_degrees()), sigma_theta_deg: st, sigma_phi_deg: sp, snr_db: 10.0 * snr.log10(), n }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::Rng;

    #[test]
    fn recovers_a_noiseless_plane_wave() {
        let a = Array::half_wave(915e6);
        let mut r = Rng::new(5);
        for (t, p) in [(10.0, 0.0), (25.0, 40.0), (60.0, -120.0), (85.0, 45.0)] {
            let x = a.plane_wave(915e6, t, p, 512, 80.0, &mut r);
            let b = phase_difference(&a, 915e6, [&x[0], &x[1], &x[2], &x[3]]);
            assert!(Array::separation_deg((b.theta_deg, b.phi_deg), (t, p)) < 1e-2, "{b:?}");
        }
        let x = a.plane_wave(915e6, 0.0, 0.0, 512, 80.0, &mut r);
        let b = phase_difference(&a, 915e6, [&x[0], &x[1], &x[2], &x[3]]);
        assert!(b.theta_deg < 0.01);
    }

    #[test]
    fn mountings() {
        let b = Bearing { theta_deg: 60.0, phi_deg: 90.0, sigma_theta_deg: 0.0, sigma_phi_deg: 0.0, snr_db: 0.0, n: 0 };
        let (az, el) = b.horizontal_mount(); // from +y = north, 30° up
        assert!((az - 0.0).abs() < 1e-9 && (el - 30.0).abs() < 1e-9);
        let (az, el) = b.forward_mount(); // from +y = up: straight above boresight
        assert!(az.abs() < 1e-9 && (el - 60.0).abs() < 1e-9);
        let b = Bearing { theta_deg: 20.0, phi_deg: 0.0, ..b };
        let (az, el) = b.forward_mount(); // from +x = right
        assert!((az - 20.0).abs() < 1e-9 && el.abs() < 1e-9);
    }

    #[test]
    fn rms_error_at_20_db_is_within_the_t25_bound() {
        let a = Array::half_wave(915e6);
        let mut r = Rng::new(11);
        let (trials, n) = (300, 1024);
        // forward-facing use: sources within 30° of boresight; the error is the angle between the true and estimated directions
        let (mut se, mut st) = (0.0, 0.0);
        for t in 0..trials {
            let (theta, phi) = (30.0 * (t as f64 + 0.5) / trials as f64, r.range(-180.0, 180.0));
            let x = a.plane_wave(915e6, theta, phi, n, 20.0, &mut r);
            let b = phase_difference(&a, 915e6, [&x[0], &x[1], &x[2], &x[3]]);
            se += Array::separation_deg((b.theta_deg, b.phi_deg), (theta, phi)).powi(2);
            st += b.sigma_theta_deg;
        }
        let rms = (se / trials as f64).sqrt();
        let sigma = st / trials as f64;
        assert!(rms <= 0.1, "rms {rms}");
        // the two-axis error against the per-axis σ: √2 within a tolerance
        assert!(rms > 1.0 * sigma && rms < 2.0 * sigma, "rms {rms} vs reported sigma_theta {sigma}");
        // horizontal use: sources at the horizon, φ in the unambiguous middle of a quadrant
        let (mut se, mut sp) = (0.0, 0.0);
        for t in 0..trials {
            let phi = 20.0 + 50.0 * (t as f64 + 0.5) / trials as f64;
            let x = a.plane_wave(915e6, 90.0, phi, n, 20.0, &mut r);
            let b = phase_difference(&a, 915e6, [&x[0], &x[1], &x[2], &x[3]]);
            se += wrap_deg(b.phi_deg - phi).powi(2);
            sp += b.sigma_phi_deg;
        }
        let rms = (se / trials as f64).sqrt();
        let sigma = sp / trials as f64;
        assert!(rms <= 0.1, "rms phi {rms}");
        assert!(rms > 0.6 * sigma && rms < 1.6 * sigma, "rms {rms} vs reported sigma_phi {sigma}");
    }
}
