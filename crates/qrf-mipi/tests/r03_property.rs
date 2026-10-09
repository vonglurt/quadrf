//! Backlog R-03 check, part 1: the NEON de-interleave equals the scalar
//! reference on 10⁶ random spans (SPEC-008 S-008-5). Spans are 0–256 time
//! samples of random bytes at a random alignment of the source (0–7 bytes),
//! so every block count, remainder and alignment is covered.

use qrf_mipi::deinterleave::{buffers, views, deinterleave, deinterleave_scalar};
use qrf_mipi::frame::{BYTES_PER_SAMPLE, ELEMENTS};
use qrf_mipi::rng::Rng64;

#[test]
fn neon_equals_scalar_on_a_million_random_spans() {
    const SPANS: usize = 1_000_000;
    const MAX_SAMPLES: usize = 256;
    let mut rng = Rng64::new(0x5EED_2026_1008);
    let mut raw = vec![0u8; MAX_SAMPLES * BYTES_PER_SAMPLE + 8];
    let mut a = buffers(MAX_SAMPLES);
    let mut b = buffers(MAX_SAMPLES);
    let mut bytes = 0u64;
    for _ in 0..SPANS {
        let n = rng.below(MAX_SAMPLES as u64 + 1) as usize;
        let align = rng.below(8) as usize;
        let src = &mut raw[align..align + n * BYTES_PER_SAMPLE];
        rng.fill(src);
        let src = &raw[align..align + n * BYTES_PER_SAMPLE];
        for e in 0..ELEMENTS {
            a[e][..2 * n].fill(0xAA);
            b[e][..2 * n].fill(0x55);
        }
        deinterleave_scalar(src, &mut views(&mut a));
        deinterleave(src, &mut views(&mut b));
        for e in 0..ELEMENTS {
            assert_eq!(a[e][..2 * n], b[e][..2 * n], "element {e}, {n} samples, alignment {align}");
        }
        bytes += src.len() as u64;
    }
    eprintln!("neon_equals_scalar: {SPANS} spans, {bytes} bytes");
}
