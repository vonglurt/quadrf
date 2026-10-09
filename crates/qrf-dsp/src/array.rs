//! Element geometry, direction cosines, steering vectors and a plane-wave
//! simulator (SPEC-009 S-009-8: true wavelength and surveyed positions;
//! analysis T9, T16).
//!
//! Array frame: the elements lie in the x–y plane and the array normal is
//! +z (the tile's boresight; the zenith for the horizontal 915 MHz aperture).
//! A direction is (θ, φ): θ from the normal, φ in the plane from +x toward
//! +y. A plane wave from (θ, φ) reaches element i with phase +k·(r_i · u),
//! u the unit vector toward the source. The mapping to true-north azimuth and
//! elevation is the sensor's mounting transform (S-009-7); `Bearing` offers
//! the two standard mountings.

use num_complex::Complex;

use crate::rng::Rng;

/// Speed of light, m/s.
pub const C: f64 = 299_792_458.0;

/// Four element positions in metres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Array {
    pub pos_m: [[f64; 3]; 4],
    /// Pitch of a square array, if it is one (`square`); the phase-difference estimator needs it.
    pub pitch_m: Option<f64>,
}

impl Array {
    /// A 2 × 2 square of `pitch_m` in the x–y plane: 0 (−, −), 1 (+, −), 2 (−, +), 3 (+, +); the x pairs are (0, 1), (2, 3) and the y pairs (0, 2), (1, 3).
    pub fn square(pitch_m: f64) -> Self {
        let h = pitch_m / 2.0;
        Self { pos_m: [[-h, -h, 0.0], [h, -h, 0.0], [-h, h, 0.0], [h, h, 0.0]], pitch_m: Some(pitch_m) }
    }
    /// The tile's antenna module, 45.5 mm pitch (SPEC-001 S-001-12).
    pub fn tile() -> Self {
        Self::square(0.0455)
    }
    /// The revision 0 FTFE aperture, 164 mm pitch (SPEC-007 S-007-10).
    pub fn ftfe() -> Self {
        Self::square(0.164)
    }
    /// Half-wave pitch at `f_hz`.
    pub fn half_wave(f_hz: f64) -> Self {
        Self::square(C / f_hz / 2.0)
    }
    pub const X_PAIRS: [(usize, usize); 2] = [(0, 1), (2, 3)];
    pub const Y_PAIRS: [(usize, usize); 2] = [(0, 2), (1, 3)];

    /// Pitch over wavelength at `f_hz` (T9).
    pub fn d_over_lambda(&self, f_hz: f64) -> Option<f64> {
        self.pitch_m.map(|d| d * f_hz / C)
    }
    /// The unambiguous half-cone of a two-element phase difference, degrees from the normal: asin(λ/2d) (T9).
    pub fn unambiguous_deg(&self, f_hz: f64) -> Option<f64> {
        self.d_over_lambda(f_hz).map(|dl| { let s = 1.0 / (2.0 * dl); if s >= 1.0 { 90.0 } else { s.asin().to_degrees() } })
    }

    /// Unit vector toward (θ, φ) in degrees.
    pub fn direction(theta_deg: f64, phi_deg: f64) -> [f64; 3] {
        let (t, p) = (theta_deg.to_radians(), phi_deg.to_radians());
        [t.sin() * p.cos(), t.sin() * p.sin(), t.cos()]
    }

    /// Angle in degrees between two directions.
    pub fn separation_deg(a: (f64, f64), b: (f64, f64)) -> f64 {
        let (u, v) = (Self::direction(a.0, a.1), Self::direction(b.0, b.1));
        (u[0] * v[0] + u[1] * v[1] + u[2] * v[2]).clamp(-1.0, 1.0).acos().to_degrees()
    }

    /// Steering vector at `f_hz` toward (θ, φ): exp(+j k r_i · u).
    pub fn steering(&self, f_hz: f64, theta_deg: f64, phi_deg: f64) -> [Complex<f64>; 4] {
        let u = Self::direction(theta_deg, phi_deg);
        let k = 2.0 * std::f64::consts::PI * f_hz / C;
        self.pos_m.map(|r| Complex::from_polar(1.0, k * (r[0] * u[0] + r[1] * u[1] + r[2] * u[2])))
    }

    /// `n` samples per element of plane waves from `sources` (θ, φ, power) in circular complex Gaussian noise of `noise_power` per element; each source is a unit-variance complex Gaussian process.
    pub fn plane_waves(&self, f_hz: f64, sources: &[(f64, f64, f64)], n: usize, noise_power: f64, rng: &mut Rng) -> [Vec<Complex<f32>>; 4] {
        let mut out: [Vec<Complex<f32>>; 4] = std::array::from_fn(|_| vec![Complex::new(0.0, 0.0); n]);
        let steer: Vec<[Complex<f64>; 4]> = sources.iter().map(|&(t, p, _)| self.steering(f_hz, t, p)).collect();
        for i in 0..n {
            let mut v = [Complex::new(0.0f64, 0.0); 4];
            for (s, &(_, _, p)) in steer.iter().zip(sources) {
                let z = rng.complex_gaussian(p);
                for e in 0..4 {
                    v[e] += z * s[e];
                }
            }
            for e in 0..4 {
                let w = v[e] + rng.complex_gaussian(noise_power);
                out[e][i] = Complex::new(w.re as f32, w.im as f32);
            }
        }
        out
    }

    /// One unit-power source at (θ, φ) at `snr_db` per element.
    pub fn plane_wave(&self, f_hz: f64, theta_deg: f64, phi_deg: f64, n: usize, snr_db: f64, rng: &mut Rng) -> [Vec<Complex<f32>>; 4] {
        self.plane_waves(f_hz, &[(theta_deg, phi_deg, 1.0)], n, 10f64.powf(-snr_db / 10.0), rng)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_and_steering() {
        let a = Array::half_wave(915e6);
        assert!((a.pitch_m.unwrap() - 0.1638).abs() < 1e-3, "T9 half-wave pitch 164 mm");
        assert!((a.unambiguous_deg(915e6).unwrap() - 90.0).abs() < 1e-9);
        assert!((Array::tile().unambiguous_deg(5.8e9).unwrap() - 34.6).abs() < 0.05, "S-001-14");
        let s = a.steering(915e6, 90.0, 0.0); // from +x at the horizon: the +x elements lead by k·d = π
        let d = (s[1] / s[0]).arg();
        assert!((d.abs() - std::f64::consts::PI).abs() < 1e-9, "{d}");
        let s = a.steering(915e6, 0.0, 0.0); // boresight: all equal
        assert!(s.iter().all(|z| (z - s[0]).norm() < 1e-12));
        assert!((Array::separation_deg((10.0, 0.0), (10.0, 180.0)) - 20.0).abs() < 1e-9);
    }
}
