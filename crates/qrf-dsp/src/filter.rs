//! The channeliser's prototype low-pass: a Kaiser-windowed sinc of M·P taps
//! with cutoff fs/(2M), unit DC gain (analysis T25).

use std::f64::consts::PI;

/// Kaiser β for 60 dB sidelobes: 0.1102 (A − 8.7) (T25).
pub const KAISER_BETA_60DB: f64 = 0.1102 * (60.0 - 8.7);

/// Modified Bessel function of the first kind, order 0 (power series).
pub fn bessel_i0(x: f64) -> f64 {
    let mut sum = 1.0;
    let mut term = 1.0;
    let q = x * x / 4.0;
    for k in 1..200 {
        term *= q / (k as f64 * k as f64);
        sum += term;
        if term < sum * 1e-17 {
            break;
        }
    }
    sum
}

/// Kaiser window value at tap `n` of `l`.
pub fn kaiser(n: usize, l: usize, beta: f64) -> f64 {
    let r = 2.0 * n as f64 / (l as f64 - 1.0) - 1.0;
    bessel_i0(beta * (1.0 - r * r).max(0.0).sqrt()) / bessel_i0(beta)
}

/// The prototype: `m * p` taps, sinc with cutoff fs/(2m), Kaiser window, Σh = 1.
pub fn prototype(m: usize, p: usize, beta: f64) -> Vec<f32> {
    let l = m * p;
    let c = (l as f64 - 1.0) / 2.0;
    let mut h: Vec<f64> = (0..l)
        .map(|n| {
            let x = (n as f64 - c) / m as f64;
            let s = if x.abs() < 1e-12 { 1.0 } else { (PI * x).sin() / (PI * x) };
            s * kaiser(n, l, beta)
        })
        .collect();
    let sum: f64 = h.iter().sum();
    h.iter_mut().for_each(|v| *v /= sum);
    h.iter().map(|&v| v as f32).collect()
}

/// Power response of the prototype at a frequency offset of `offset_bins` bins (|H(δ)|², δ = 2π·offset/m), in dB.
pub fn response_db(h: &[f32], m: usize, offset_bins: f64) -> f64 {
    let d = 2.0 * PI * offset_bins / m as f64;
    let (mut re, mut im) = (0.0f64, 0.0f64);
    for (n, &v) in h.iter().enumerate() {
        let a = d * n as f64;
        re += v as f64 * a.cos();
        im -= v as f64 * a.sin();
    }
    10.0 * (re * re + im * im).log10()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_dc_gain_and_symmetry() {
        let h = prototype(128, 8, KAISER_BETA_60DB);
        assert_eq!(h.len(), 1024);
        let s: f64 = h.iter().map(|&v| v as f64).sum();
        assert!((s - 1.0).abs() < 1e-6);
        for n in 0..512 {
            assert!((h[n] - h[1023 - n]).abs() < 1e-9);
        }
        assert!(response_db(&h, 128, 0.0).abs() < 1e-6);
        assert!(response_db(&h, 128, 0.5) < -3.0 && response_db(&h, 128, 0.5) > -9.0);
        assert!(response_db(&h, 128, 2.0) < -50.0);
    }

    #[test]
    fn bessel_known_values() {
        assert!((bessel_i0(0.0) - 1.0).abs() < 1e-15);
        assert!((bessel_i0(1.0) - 1.266_065_877_752_008).abs() < 1e-12);
        assert!((bessel_i0(5.0) - 27.239_871_823_604_44).abs() < 1e-9);
    }
}
