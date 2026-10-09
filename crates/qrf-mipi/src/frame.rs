//! The CSI frame as the tile sends it (SPEC-002 S-002-21; `fpga-csi.dts`).
//!
//! One frame is 1024 bytes × 128 lines = 131 072 bytes (analysis T14). The
//! bytes are consecutive 16-bit (I, Q) words, one per element, elements 0–3
//! repeating: byte 2e of every 8-byte group is element e's I sample, byte
//! 2e + 1 its Q sample, both signed 8-bit (CS8). A frame therefore carries
//! 16 384 time samples of each of the four elements, 630 µs at 26 MSPS.

/// Bytes per CSI line (device tree: RAW8, width 1024).
pub const LINE_BYTES: usize = 1024;
/// Lines per frame (device tree).
pub const LINES: usize = 128;
/// Bytes per frame, the ring's span (S-002-14; T14).
pub const FRAME_BYTES: usize = LINE_BYTES * LINES;
/// Receive elements on the tile (SPEC-001).
pub const ELEMENTS: usize = 4;
/// Bytes of one time sample across the four elements: 4 × (I, Q).
pub const BYTES_PER_SAMPLE: usize = 2 * ELEMENTS;
/// Time samples per element per frame.
pub const SAMPLES_PER_FRAME: usize = FRAME_BYTES / BYTES_PER_SAMPLE;

/// Byte offset of element `e`'s (I, Q) word within sample `n` of an interleaved span.
pub const fn offset(n: usize, e: usize) -> usize {
    n * BYTES_PER_SAMPLE + 2 * e
}

/// The (I, Q) pair of element `e` at time sample `n` of an interleaved span.
pub fn sample(span: &[u8], n: usize, e: usize) -> (i8, i8) {
    let o = offset(n, e);
    (span[o] as i8, span[o + 1] as i8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn geometry_is_t14() {
        assert_eq!(FRAME_BYTES, 131_072);
        assert_eq!(SAMPLES_PER_FRAME, 16_384);
        // 16 384 samples at 26 MSPS is 630 µs (T14).
        assert_eq!((SAMPLES_PER_FRAME as f64 / 26.0e6 * 1e6).round() as u32, 630);
    }

    #[test]
    fn offsets_follow_the_interleave() {
        assert_eq!(offset(0, 0), 0);
        assert_eq!(offset(0, 3), 6);
        assert_eq!(offset(1, 0), 8);
        let mut s = vec![0u8; 16];
        s[offset(1, 2)] = 0xFF; // -1
        s[offset(1, 2) + 1] = 0x7F; // 127
        assert_eq!(sample(&s, 1, 2), (-1, 127));
    }
}
