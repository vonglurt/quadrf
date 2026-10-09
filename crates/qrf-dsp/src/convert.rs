//! CS8 to `Complex<f32>`, once per sample (SPEC-008 S-008-5: samples stay
//! CS8 until `qrf-dspd` converts them once to f32; all DSP is f32).

use num_complex::Complex;

/// Scale that maps the 8-bit full scale (±128) to ±1.0, so that power is in dBFS.
pub const CS8_SCALE: f32 = 1.0 / 128.0;

/// Convert `src.len() / 2` (I, Q) byte pairs into `out`; `out` must hold at least that many.
pub fn cs8_to_c32(src: &[u8], scale: f32, out: &mut [Complex<f32>]) -> usize {
    let n = src.len() / 2;
    assert!(out.len() >= n, "out holds {}, {n} needed", out.len());
    for (o, p) in out.iter_mut().zip(src.chunks_exact(2)) {
        *o = Complex::new(p[0] as i8 as f32 * scale, p[1] as i8 as f32 * scale);
    }
    n
}

/// Allocating form.
pub fn cs8_to_c32_vec(src: &[u8], scale: f32) -> Vec<Complex<f32>> {
    let mut v = vec![Complex::new(0.0, 0.0); src.len() / 2];
    cs8_to_c32(src, scale, &mut v);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_bytes_scale() {
        let v = cs8_to_c32_vec(&[0x7F, 0x80, 0xFF, 0x01], CS8_SCALE);
        assert_eq!(v[0], Complex::new(127.0 / 128.0, -1.0));
        assert_eq!(v[1], Complex::new(-1.0 / 128.0, 1.0 / 128.0));
    }
}
