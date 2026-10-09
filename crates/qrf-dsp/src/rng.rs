//! A seeded generator for tests, benchmarks and the plane-wave simulator:
//! SplitMix64, uniform, Box–Muller Gaussian, circular complex Gaussian.

use num_complex::Complex;

/// SplitMix64.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Uniform in (0, 1].
    pub fn unit(&mut self) -> f64 {
        ((self.next_u64() >> 11) as f64 + 1.0) / 9_007_199_254_740_993.0
    }
    /// Uniform in [lo, hi).
    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * (self.unit() - f64::EPSILON).max(0.0)
    }
    /// Two independent standard normal variates.
    pub fn gaussian_pair(&mut self) -> (f64, f64) {
        let u1 = self.unit();
        let u2 = self.unit();
        let r = (-2.0 * u1.ln()).sqrt();
        let t = 2.0 * std::f64::consts::PI * u2;
        (r * t.cos(), r * t.sin())
    }
    /// Circular complex Gaussian with E|z|² = `power`.
    pub fn complex_gaussian(&mut self, power: f64) -> Complex<f64> {
        let (a, b) = self.gaussian_pair();
        let s = (power / 2.0).sqrt();
        Complex::new(a * s, b * s)
    }
    /// A buffer of circular complex Gaussian samples in f32.
    pub fn complex_gaussian_f32(&mut self, n: usize, power: f64) -> Vec<Complex<f32>> {
        (0..n).map(|_| { let z = self.complex_gaussian(power); Complex::new(z.re as f32, z.im as f32) }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gaussian_power_is_as_declared() {
        let mut r = Rng::new(1);
        let v = r.complex_gaussian_f32(200_000, 0.25);
        let p: f64 = v.iter().map(|z| z.norm_sqr() as f64).sum::<f64>() / v.len() as f64;
        assert!((p - 0.25).abs() < 0.005, "{p}");
    }
}
