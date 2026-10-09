//! De-interleave an element-interleaved span into four per-element CS8 buffers.
//!
//! Implements: SPEC-008 S-008-2 (de-interleave to per-element CS8), S-008-5
//! (NEON kernels confined to leaf functions with a scalar reference and a
//! property test asserting equality). Samples stay CS8; nothing is converted.

use crate::frame::{BYTES_PER_SAMPLE, ELEMENTS};

/// Time samples `src` holds (whole 8-byte groups; a trailing partial group is ignored).
pub fn samples_in(src: &[u8]) -> usize {
    src.len() / BYTES_PER_SAMPLE
}

fn check(src: &[u8], out: &[&mut [u8]; ELEMENTS]) -> usize {
    let n = samples_in(src);
    for (e, o) in out.iter().enumerate() {
        assert!(o.len() >= 2 * n, "output {e} holds {} bytes, {} needed", o.len(), 2 * n);
    }
    n
}

/// Reference implementation: one byte at a time, in the order the layout states.
pub fn deinterleave_scalar(src: &[u8], out: &mut [&mut [u8]; ELEMENTS]) {
    let n = check(src, out);
    for (i, group) in src.chunks_exact(BYTES_PER_SAMPLE).take(n).enumerate() {
        for e in 0..ELEMENTS {
            out[e][2 * i] = group[2 * e];
            out[e][2 * i + 1] = group[2 * e + 1];
        }
    }
}

/// NEON kernel: eight time samples (64 bytes) per iteration through a
/// four-way interleaved 16-bit load, the remainder through the scalar path.
#[cfg(target_arch = "aarch64")]
pub fn deinterleave_neon(src: &[u8], out: &mut [&mut [u8]; ELEMENTS]) {
    use std::arch::aarch64::{vld4q_u16, vst1q_u16};
    let n = check(src, out);
    let blocks = n / 8;
    // SAFETY (S-008-5): `src` holds at least `blocks * 64` bytes and every
    // output at least `blocks * 16` bytes (checked above); the loads and
    // stores stay inside those bounds; AArch64 NEON loads and stores accept
    // unaligned addresses, so the u16 casts need no alignment; NEON is a
    // mandatory feature of AArch64 Linux targets, so the intrinsics are
    // always available. The property test `neon_equals_scalar` compares this
    // kernel with the scalar reference on random spans and alignments.
    unsafe {
        let s = src.as_ptr();
        for b in 0..blocks {
            let v = vld4q_u16(s.add(b * 64).cast::<u16>());
            vst1q_u16(out[0].as_mut_ptr().add(b * 16).cast::<u16>(), v.0);
            vst1q_u16(out[1].as_mut_ptr().add(b * 16).cast::<u16>(), v.1);
            vst1q_u16(out[2].as_mut_ptr().add(b * 16).cast::<u16>(), v.2);
            vst1q_u16(out[3].as_mut_ptr().add(b * 16).cast::<u16>(), v.3);
        }
    }
    let done = blocks * 8;
    if done < n {
        let tail = &src[done * BYTES_PER_SAMPLE..n * BYTES_PER_SAMPLE];
        let mut rest: [&mut [u8]; ELEMENTS] = out.each_mut().map(|o| &mut o[2 * done..2 * n]);
        deinterleave_scalar(tail, &mut rest);
    }
}

/// The fastest kernel for this target.
pub fn deinterleave(src: &[u8], out: &mut [&mut [u8]; ELEMENTS]) {
    #[cfg(target_arch = "aarch64")]
    {
        deinterleave_neon(src, out)
    }
    #[cfg(not(target_arch = "aarch64"))]
    {
        deinterleave_scalar(src, out)
    }
}

/// Name of the kernel `deinterleave` dispatches to, for logs and reports.
pub const fn kernel_name() -> &'static str {
    if cfg!(target_arch = "aarch64") { "neon" } else { "scalar" }
}

/// Four output buffers sized for `samples` time samples each.
pub fn buffers(samples: usize) -> [Vec<u8>; ELEMENTS] {
    std::array::from_fn(|_| vec![0u8; 2 * samples])
}

/// Mutable slice views of four buffers, the shape the kernels take.
pub fn views(bufs: &mut [Vec<u8>; ELEMENTS]) -> [&mut [u8]; ELEMENTS] {
    bufs.each_mut().map(|v| v.as_mut_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::offset;

    #[test]
    fn scalar_places_every_byte() {
        let src: Vec<u8> = (0..32u8).collect();
        let mut bufs = buffers(4);
        deinterleave_scalar(&src, &mut views(&mut bufs));
        for n in 0..4 {
            for e in 0..ELEMENTS {
                assert_eq!(bufs[e][2 * n], src[offset(n, e)]);
                assert_eq!(bufs[e][2 * n + 1], src[offset(n, e) + 1]);
            }
        }
    }

    #[test]
    fn dispatch_equals_scalar_on_a_frame() {
        let mut x = 0x9E37_79B9_7F4A_7C15u64;
        let src: Vec<u8> = (0..crate::frame::FRAME_BYTES)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                x as u8
            })
            .collect();
        let mut a = buffers(crate::frame::SAMPLES_PER_FRAME);
        let mut b = buffers(crate::frame::SAMPLES_PER_FRAME);
        deinterleave_scalar(&src, &mut views(&mut a));
        deinterleave(&src, &mut views(&mut b));
        assert_eq!(a, b);
    }
}
