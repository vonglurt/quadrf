//! Bit-level coding of a LoRa frame: whitening, header, CRC, Hamming codes, diagonal interleaver, Gray mapping
//! and the frame layout (SPEC-003 S-003-7; SPEC-008 S-008-6).
//!
//! The LoRa physical layer is proprietary and its coding conventions are published only through
//! reverse-engineering (Knight & Seeber 2016, Robyns et al. 2018, Tapparel et al. 2020, the latter in
//! `resources/lora/lora-phy-paper-tapparel.pdf`). Every convention implemented here was fixed by
//! comparing with the observable output of the oracle transmitter stage by stage
//! (`scripts/oracle-vectors.py` → `tests/data/lora-tx-vectors.json`, checked by `tests/r08_vectors.rs`):
//! the item documentation states each convention; the test is the evidence. No oracle code is used.

/// What the decoder found out about one codeword.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodewordStatus {
    /// Parity consistent.
    Clean,
    /// A single bit error corrected (coding rates 4/7 and 4/8 only).
    Corrected,
    /// Parity inconsistent and not correctable; the data bits are passed through as received.
    Error,
}

/// Whitening sequence: payload byte `i` is XORed with `seq[i]`. `seq` is the state of an 8-bit Fibonacci
/// linear-feedback shift register with feedback polynomial x⁸ + x⁶ + x⁵ + x⁴ + 1 seeded with 0xFF and shifted
/// left by one bit per byte (consecutive bytes overlap by seven bits; period 255). The 16-bit CRC that follows
/// the payload is not whitened.
pub fn whitening_sequence(len: usize) -> Vec<u8> {
    let mut state: u8 = 0xFF;
    let mut out = Vec::with_capacity(len);
    for _ in 0..len {
        out.push(state);
        let fresh = ((state >> 7) ^ (state >> 5) ^ (state >> 4) ^ (state >> 3)) & 1;
        state = (state << 1) | fresh;
    }
    out
}

/// XORs the whitening sequence into `bytes` (the operation is its own inverse).
pub fn whiten(bytes: &mut [u8]) {
    let seq = whitening_sequence(bytes.len());
    for (b, w) in bytes.iter_mut().zip(seq) {
        *b ^= w;
    }
}

/// CRC-16/CCITT: polynomial 0x1021, initial value 0, no reflection, no final XOR (check value 0x31C3 for "123456789").
pub fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &b in data {
        crc ^= u16::from(b) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 != 0 { (crc << 1) ^ 0x1021 } else { crc << 1 };
        }
    }
    crc
}

/// The CRC transmitted after a LoRa payload: CRC-16/CCITT over every byte but the last two, XORed with the
/// last two bytes (`payload[len − 2]` into the high byte, `payload[len − 1]` into the low byte). It is sent
/// low byte first, each byte low nibble first, and is not whitened. Payloads shorter than two bytes are not
/// produced by Meshtastic and not covered by the oracle vectors; the rule used for them here is a guess.
pub fn payload_crc(payload: &[u8]) -> u16 {
    match payload.len() {
        0 => 0,
        1 => crc16_ccitt(&[]) ^ u16::from(payload[0]),
        n => crc16_ccitt(&payload[..n - 2]) ^ (u16::from(payload[n - 2]) << 8) ^ u16::from(payload[n - 1]),
    }
}

fn parities(nibble: u8) -> (u8, u8, u8, u8) {
    let d = |i: u8| (nibble >> i) & 1;
    (d(0) ^ d(1) ^ d(2), d(1) ^ d(2) ^ d(3), d(0) ^ d(1) ^ d(3), d(0) ^ d(2) ^ d(3))
}

fn reverse4(x: u8) -> u8 {
    ((x & 1) << 3) | ((x & 2) << 1) | ((x & 4) >> 1) | ((x & 8) >> 3)
}

/// Hamming codeword of a nibble at coding rate `cr` (1..=4 for 4/5..4/8), right-aligned in `4 + cr` bits.
/// The four data bits come first in reversed order (bit 0 of the nibble highest), then the parity bits:
/// at 4/5 one even-parity bit over the nibble; at 4/6, 4/7 and 4/8 the first `cr` of
/// p0 = d0⊕d1⊕d2, p1 = d1⊕d2⊕d3, p2 = d0⊕d1⊕d3, p3 = d0⊕d2⊕d3. With p0…p2 this is a (7,4) Hamming
/// code (every data bit has a distinct syndrome) and p3 is its overall parity, so 4/8 is the extended
/// (8,4) code that corrects one error and detects two.
pub fn hamming_encode(nibble: u8, cr: u8) -> u8 {
    let nibble = nibble & 0xF;
    let data_rev = reverse4(nibble);
    let (p0, p1, p2, p3) = parities(nibble);
    match cr {
        1 => (data_rev << 1) | (p0 ^ ((nibble >> 3) & 1)),
        2 => (data_rev << 2) | (p0 << 1) | p1,
        3 => (data_rev << 3) | (p0 << 2) | (p1 << 1) | p2,
        _ => (data_rev << 4) | (p0 << 3) | (p1 << 2) | (p2 << 1) | p3,
    }
}

/// Decodes a right-aligned `4 + cr`-bit codeword (see [`hamming_encode`]) into its nibble and the decoder's verdict.
/// 4/7 and 4/8 correct a single error; 4/5 and 4/6 only detect.
pub fn hamming_decode(cw: u8, cr: u8) -> (u8, CodewordStatus) {
    let cr = cr.clamp(1, 4);
    let mask: u8 = if cr >= 4 { 0xFF } else { (1u8 << (4 + cr)) - 1 };
    let cw = cw & mask;
    let mut nibble = reverse4((cw >> cr) & 0xF);
    let (p0, p1, p2, p3) = parities(nibble);
    let status = match cr {
        1 => {
            let even = p0 ^ ((nibble >> 3) & 1);
            if cw & 1 == even { CodewordStatus::Clean } else { CodewordStatus::Error }
        }
        2 => {
            if (cw >> 1) & 1 == p0 && cw & 1 == p1 { CodewordStatus::Clean } else { CodewordStatus::Error }
        }
        _ => {
            // Syndrome of the (7,4) part: which of p0, p1, p2 disagree.
            let shift = cr - 3; // 0 at 4/7, 1 at 4/8
            let s0 = ((cw >> (2 + shift)) & 1) ^ p0;
            let s1 = ((cw >> (1 + shift)) & 1) ^ p1;
            let s2 = ((cw >> shift) & 1) ^ p2;
            let syndrome = (s0 << 2) | (s1 << 1) | s2;
            // Data bit covered by exactly this set of parities, if any (0b101 → d0, 0b111 → d1, 0b110 → d2, 0b011 → d3).
            let data_bit = match syndrome {
                0b101 => Some(0),
                0b111 => Some(1),
                0b110 => Some(2),
                0b011 => Some(3),
                _ => None,
            };
            if cr == 3 {
                if syndrome == 0 {
                    CodewordStatus::Clean
                } else {
                    if let Some(i) = data_bit {
                        nibble ^= 1 << i;
                    }
                    CodewordStatus::Corrected
                }
            } else {
                // Extended code: the overall parity of all eight bits is even for a valid codeword.
                let overall = (cw.count_ones() & 1) as u8;
                let _ = p3;
                if syndrome == 0 && overall == 0 {
                    CodewordStatus::Clean
                } else if overall == 1 {
                    if let Some(i) = data_bit {
                        nibble ^= 1 << i;
                    }
                    CodewordStatus::Corrected
                } else {
                    CodewordStatus::Error
                }
            }
        }
    };
    (nibble, status)
}

/// The explicit header: payload length, coding rate and CRC presence, protected by a 5-bit checksum.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Header {
    pub payload_len: u8,
    /// 1..=4 for 4/5..4/8.
    pub cr: u8,
    pub has_crc: bool,
}

impl Header {
    /// Checksum over the three header nibbles n0 = length high nibble, n1 = length low nibble,
    /// n2 = (cr << 1) | crc, as the parity-check matrix published by Robyns et al. 2018 gives it:
    /// c4 = n0[3]⊕n0[2]⊕n0[1]⊕n0[0], c3 = n0[3]⊕n1[3]⊕n1[2]⊕n1[1]⊕n2[0], c2 = n0[2]⊕n1[3]⊕n1[0]⊕n2[3]⊕n2[1],
    /// c1 = n0[1]⊕n1[2]⊕n1[0]⊕n2[2]⊕n2[1]⊕n2[0], c0 = n0[0]⊕n1[1]⊕n2[3]⊕n2[2]⊕n2[1]⊕n2[0] (bit 3 is a nibble's MSB).
    pub fn checksum(n0: u8, n1: u8, n2: u8) -> u8 {
        let b = |n: u8, i: u8| (n >> i) & 1;
        let c4 = b(n0, 3) ^ b(n0, 2) ^ b(n0, 1) ^ b(n0, 0);
        let c3 = b(n0, 3) ^ b(n1, 3) ^ b(n1, 2) ^ b(n1, 1) ^ b(n2, 0);
        let c2 = b(n0, 2) ^ b(n1, 3) ^ b(n1, 0) ^ b(n2, 3) ^ b(n2, 1);
        let c1 = b(n0, 1) ^ b(n1, 2) ^ b(n1, 0) ^ b(n2, 2) ^ b(n2, 1) ^ b(n2, 0);
        let c0 = b(n0, 0) ^ b(n1, 1) ^ b(n2, 3) ^ b(n2, 2) ^ b(n2, 1) ^ b(n2, 0);
        (c4 << 4) | (c3 << 3) | (c2 << 2) | (c1 << 1) | c0
    }

    /// The five header nibbles: length high, length low, (cr << 1) | crc, 000c4, c3c2c1c0.
    pub fn to_nibbles(self) -> [u8; 5] {
        let n0 = self.payload_len >> 4;
        let n1 = self.payload_len & 0xF;
        let n2 = ((self.cr & 7) << 1) | u8::from(self.has_crc);
        let c = Self::checksum(n0, n1, n2);
        [n0, n1, n2, c >> 4, c & 0xF]
    }

    /// Parses five received nibbles; `None` when the checksum fails or the coding rate is not 1..=4.
    pub fn from_nibbles(n: &[u8]) -> Option<Header> {
        if n.len() < 5 {
            return None;
        }
        let (n0, n1, n2) = (n[0] & 0xF, n[1] & 0xF, n[2] & 0xF);
        let received = ((n[3] & 1) << 4) | (n[4] & 0xF);
        if Self::checksum(n0, n1, n2) != received {
            return None;
        }
        let cr = n2 >> 1;
        if !(1..=4).contains(&cr) {
            return None;
        }
        Some(Header { payload_len: (n0 << 4) | n1, cr, has_crc: n2 & 1 == 1 })
    }
}

/// Diagonal interleaver: `rows` codewords of `cols` bits become `cols` symbols of `rows` bits;
/// bit `j` of symbol `i` is bit `cols − 1 − i` of codeword `(i + j) mod rows`. `cws` must hold `rows` entries.
pub fn interleave(cws: &[u8], rows: usize, cols: usize) -> Vec<u16> {
    debug_assert_eq!(cws.len(), rows);
    (0..cols)
        .map(|i| {
            let mut s: u16 = 0;
            for j in 0..rows {
                let bit = (cws[(i + j) % rows] >> (cols - 1 - i)) & 1;
                s |= u16::from(bit) << j;
            }
            s
        })
        .collect()
}

/// Inverse of [`interleave`]: `cols` symbols of `rows` bits give `rows` codewords of `cols` bits.
pub fn deinterleave(syms: &[u16], rows: usize, cols: usize) -> Vec<u8> {
    debug_assert_eq!(syms.len(), cols);
    let mut cws = vec![0u8; rows];
    for (i, &s) in syms.iter().enumerate() {
        for (j, cw) in (0..rows).map(|j| (j, (i + j) % rows)) {
            let bit = ((s >> j) & 1) as u8;
            cws[cw] |= bit << (cols - 1 - i);
        }
    }
    cws
}

/// Binary-reflected Gray code of `x`.
pub fn gray(x: u16) -> u16 {
    x ^ (x >> 1)
}

/// Inverse of [`gray`].
pub fn gray_inverse(x: u16) -> u16 {
    let mut y = x;
    let mut s = x >> 1;
    while s != 0 {
        y ^= s;
        s >>= 1;
    }
    y
}

/// Chirp index transmitted for the interleaver output `v`: at full rate `(gray⁻¹(v) + 1) mod 2^SF`; at the
/// reduced rate of the header block and of LDRO payload blocks (SF − 2 bits per symbol) `4·gray⁻¹(v) + 1`,
/// so that a ±1 chirp-index error leaves the reduced-rate value intact.
pub fn symbol_from_bits(v: u16, sf: u8, reduced: bool) -> u16 {
    let n = 1u16 << sf;
    let g = gray_inverse(v);
    if reduced { (4u16.wrapping_mul(g).wrapping_add(1)) & (n - 1) } else { (g + 1) & (n - 1) }
}

/// Inverse of [`symbol_from_bits`] for a demodulated chirp index `s`: full rate `gray((s − 1) mod 2^SF)`;
/// reduced rate `gray(((s + 1) >> 2) mod 2^(SF−2))`, which absorbs chirp-index errors of −2…+1.
pub fn bits_from_symbol(s: u16, sf: u8, reduced: bool) -> u16 {
    let n = 1u16 << sf;
    if reduced {
        let m = (1u16 << (sf - 2)) - 1;
        gray(((s & (n - 1)).wrapping_add(1) >> 2) & m)
    } else {
        gray(s.wrapping_sub(1) & (n - 1))
    }
}

/// Nibbles of a frame: five of header, two per payload byte, four of CRC when present.
pub fn nibble_count(payload_len: usize, has_crc: bool) -> usize {
    5 + 2 * payload_len + if has_crc { 4 } else { 0 }
}

/// Symbols after the 8-symbol header block: the nibbles that do not fit in the header block (which carries
/// SF − 2 of them) fill blocks of SF (or SF − 2 with LDRO) codewords, each block giving 4 + cr symbols.
/// Equals the SX1261/2 formula 8 + max(⌈(8·PL − 4·SF + 28 + 16·CRC) / (4·(SF − 2·DE))⌉·(CR + 4), 0) minus 8.
pub fn payload_symbol_count(payload_len: usize, sf: u8, cr: u8, has_crc: bool, ldro: bool) -> usize {
    let total = nibble_count(payload_len, has_crc);
    let in_header = usize::from(sf) - 2;
    let rest = total.saturating_sub(in_header);
    let rows = if ldro { usize::from(sf) - 2 } else { usize::from(sf) };
    rest.div_ceil(rows) * (4 + usize::from(cr))
}

/// Low nibble, then high nibble, of every byte.
pub fn bytes_to_nibbles(bytes: &[u8]) -> Vec<u8> {
    bytes.iter().flat_map(|&b| [b & 0xF, b >> 4]).collect()
}

/// Inverse of [`bytes_to_nibbles`]; a trailing odd nibble is dropped.
pub fn nibbles_to_bytes(nibbles: &[u8]) -> Vec<u8> {
    nibbles.chunks_exact(2).map(|p| (p[0] & 0xF) | ((p[1] & 0xF) << 4)).collect()
}

/// The chirp indices of a frame after the 2¼ downchirps: the 8-symbol header block followed by the payload
/// blocks. The header block carries the five header nibbles and the first SF − 7 payload nibbles, all at
/// coding rate 4/8 and at the reduced symbol rate; the payload blocks use `cr` and the reduced rate only with
/// `ldro`. Missing codewords in the last block are zero.
pub fn encode_frame(sf: u8, cr: u8, ldro: bool, has_crc: bool, payload: &[u8]) -> Vec<u16> {
    let header = Header { payload_len: payload.len() as u8, cr, has_crc };
    let mut white = payload.to_vec();
    whiten(&mut white);
    let mut nibbles: Vec<u8> = header.to_nibbles().to_vec();
    nibbles.extend(bytes_to_nibbles(&white));
    if has_crc {
        let c = payload_crc(payload);
        nibbles.extend(bytes_to_nibbles(&[(c & 0xFF) as u8, (c >> 8) as u8]));
    }
    let rows_h = usize::from(sf) - 2;
    let cws: Vec<u8> = nibbles
        .iter()
        .enumerate()
        .map(|(i, &nb)| hamming_encode(nb, if i < rows_h { 4 } else { cr }))
        .collect();
    let mut out = Vec::new();
    let mut block: Vec<u8> = cws.iter().take(rows_h).copied().collect();
    block.resize(rows_h, 0);
    out.extend(interleave(&block, rows_h, 8).into_iter().map(|v| symbol_from_bits(v, sf, true)));
    let rows = if ldro { usize::from(sf) - 2 } else { usize::from(sf) };
    let cols = 4 + usize::from(cr);
    if cws.len() > rows_h {
        for chunk in cws[rows_h..].chunks(rows) {
            let mut block = chunk.to_vec();
            block.resize(rows, 0);
            out.extend(interleave(&block, rows, cols).into_iter().map(|v| symbol_from_bits(v, sf, ldro)));
        }
    }
    out
}

/// What the header block yields: the header and the payload nibbles that rode along in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderBlock {
    pub header: Header,
    /// The SF − 7 payload nibbles carried in the header block (fewer when the frame is that short).
    pub nibbles: Vec<u8>,
    /// Codewords the 4/8 decoder could not repair.
    pub errors: u32,
    pub corrected: u32,
}

/// Decodes the eight header-block chirp indices; `None` when the header checksum fails.
pub fn decode_header_block(symbols: &[u16], sf: u8) -> Option<HeaderBlock> {
    if symbols.len() < 8 {
        return None;
    }
    let rows = usize::from(sf) - 2;
    let v: Vec<u16> = symbols[..8].iter().map(|&s| bits_from_symbol(s, sf, true)).collect();
    let cws = deinterleave(&v, rows, 8);
    let mut errors = 0;
    let mut corrected = 0;
    let nibbles: Vec<u8> = cws
        .iter()
        .map(|&cw| {
            let (nb, st) = hamming_decode(cw, 4);
            match st {
                CodewordStatus::Error => errors += 1,
                CodewordStatus::Corrected => corrected += 1,
                CodewordStatus::Clean => {}
            }
            nb
        })
        .collect();
    let header = Header::from_nibbles(&nibbles[..5])?;
    Some(HeaderBlock { header, nibbles: nibbles[5..].to_vec(), errors, corrected })
}

/// The decoded payload of a frame.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decoded {
    pub payload: Vec<u8>,
    /// `None` when the header announced no CRC.
    pub crc_ok: Option<bool>,
    /// The 16-bit CRC as received (low byte first on the air), when the header announced one.
    pub crc_received: Option<u16>,
    pub codeword_errors: u32,
    pub codeword_corrections: u32,
}

/// Decodes the payload-block chirp indices that follow the header block (as many as
/// [`payload_symbol_count`] says) using the header's coding rate and CRC flag.
pub fn decode_payload(symbols: &[u16], sf: u8, ldro: bool, hb: &HeaderBlock) -> Decoded {
    let header = hb.header;
    let rows = if ldro { usize::from(sf) - 2 } else { usize::from(sf) };
    let cols = 4 + usize::from(header.cr);
    let mut nibbles = hb.nibbles.clone();
    let mut errors = 0;
    let mut corrections = 0;
    for chunk in symbols.chunks(cols) {
        if chunk.len() < cols {
            break;
        }
        let v: Vec<u16> = chunk.iter().map(|&s| bits_from_symbol(s, sf, ldro)).collect();
        for cw in deinterleave(&v, rows, cols) {
            let (nb, st) = hamming_decode(cw, header.cr);
            match st {
                CodewordStatus::Error => errors += 1,
                CodewordStatus::Corrected => corrections += 1,
                CodewordStatus::Clean => {}
            }
            nibbles.push(nb);
        }
    }
    let len = usize::from(header.payload_len);
    let needed = 2 * len + if header.has_crc { 4 } else { 0 };
    if nibbles.len() < needed {
        errors += 1;
        nibbles.resize(needed, 0);
    }
    let bytes = nibbles_to_bytes(&nibbles[..needed]);
    let mut payload = bytes[..len].to_vec();
    whiten(&mut payload);
    let crc_received = header.has_crc.then(|| u16::from(bytes[len]) | (u16::from(bytes[len + 1]) << 8));
    let crc_ok = crc_received.map(|received| received == payload_crc(&payload));
    Decoded { payload, crc_ok, crc_received, codeword_errors: errors, codeword_corrections: corrections }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitening_starts_as_published() {
        // The first bytes of the sequence every LoRa reverse-engineering effort reports.
        assert_eq!(
            whitening_sequence(16),
            [0xFF, 0xFE, 0xFC, 0xF8, 0xF0, 0xE1, 0xC2, 0x85, 0x0B, 0x17, 0x2F, 0x5E, 0xBC, 0x78, 0xF1, 0xE3]
        );
        assert_eq!(whitening_sequence(256)[255], 0xFF, "the LFSR period is 255");
        let mut b = vec![0x12, 0x34, 0x56];
        whiten(&mut b);
        whiten(&mut b);
        assert_eq!(b, [0x12, 0x34, 0x56]);
    }

    #[test]
    fn crc_check_value() {
        assert_eq!(crc16_ccitt(b"123456789"), 0x31C3);
        assert_eq!(payload_crc(&[0; 8]), 0);
    }

    #[test]
    fn hamming_round_trips_and_corrects() {
        for cr in 1..=4u8 {
            for nb in 0..16u8 {
                let cw = hamming_encode(nb, cr);
                assert!(u16::from(cw) < 1u16 << (4 + cr));
                assert_eq!(hamming_decode(cw, cr), (nb, CodewordStatus::Clean), "cr {cr} nibble {nb}");
                for bit in 0..(4 + cr) {
                    let (got, st) = hamming_decode(cw ^ (1 << bit), cr);
                    match cr {
                        3 | 4 => assert_eq!((got, st), (nb, CodewordStatus::Corrected), "cr {cr} nibble {nb} bit {bit}"),
                        _ => assert_eq!(st, CodewordStatus::Error, "cr {cr} nibble {nb} bit {bit}"),
                    }
                }
            }
        }
        // 4/8 detects every double error without miscorrecting into a clean verdict.
        for nb in 0..16u8 {
            let cw = hamming_encode(nb, 4);
            for a in 0..8 {
                for b in (a + 1)..8 {
                    let (_, st) = hamming_decode(cw ^ (1 << a) ^ (1 << b), 4);
                    assert_eq!(st, CodewordStatus::Error);
                }
            }
        }
    }

    #[test]
    fn header_round_trip_and_checksum() {
        for len in [0u8, 1, 8, 32, 200, 255] {
            for cr in 1..=4u8 {
                for crc in [false, true] {
                    let h = Header { payload_len: len, cr, has_crc: crc };
                    let n = h.to_nibbles();
                    assert_eq!(Header::from_nibbles(&n), Some(h));
                    for i in 0..5 {
                        for bit in 0..4 {
                            if i == 3 && bit > 0 {
                                continue; // the three unused bits of nibble 3 are not protected
                            }
                            let mut m = n;
                            m[i] ^= 1 << bit;
                            assert_ne!(Header::from_nibbles(&m), Some(h), "single-bit error in nibble {i} bit {bit} went unnoticed");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn interleaver_inverse() {
        for (rows, cols) in [(5, 8), (7, 5), (7, 8), (9, 8), (10, 6), (12, 8)] {
            let cws: Vec<u8> = (0..rows).map(|r| ((r * 37 + 11) & ((1 << cols) - 1)) as u8).collect();
            let syms = interleave(&cws, rows, cols);
            assert!(syms.iter().all(|&s| s < 1 << rows));
            assert_eq!(deinterleave(&syms, rows, cols), cws);
        }
    }

    #[test]
    fn gray_and_symbol_mapping_inverse() {
        for x in 0..4096u16 {
            assert_eq!(gray_inverse(gray(x)), x);
        }
        for sf in 7..=12u8 {
            let n = 1u16 << sf;
            for v in 0..n {
                let s = symbol_from_bits(v, sf, false);
                assert_eq!(bits_from_symbol(s, sf, false), v);
            }
            for v in 0..(n >> 2) {
                let s = symbol_from_bits(v, sf, true);
                for e in [-2i32, -1, 0, 1] {
                    let s2 = ((i32::from(s) + e).rem_euclid(i32::from(n))) as u16;
                    assert_eq!(bits_from_symbol(s2, sf, true), v, "sf {sf} v {v} error {e}");
                }
            }
        }
    }

    #[test]
    fn symbol_count_matches_the_datasheet_formula() {
        for sf in 7..=12u8 {
            for cr in 1..=4u8 {
                for crc in [false, true] {
                    for ldro in [false, true] {
                        for pl in [1usize, 2, 8, 13, 32, 100, 255] {
                            let de = i64::from(ldro);
                            let num = 8 * pl as i64 - 4 * i64::from(sf) + 28 + 16 * i64::from(crc);
                            let den = 4 * (i64::from(sf) - 2 * de);
                            let blocks = (num + den - 1).div_euclid(den).max(0);
                            let datasheet = (blocks * (i64::from(cr) + 4)) as usize;
                            assert_eq!(payload_symbol_count(pl, sf, cr, crc, ldro), datasheet, "sf {sf} cr {cr} crc {crc} ldro {ldro} pl {pl}");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn frame_round_trip() {
        for sf in 7..=12u8 {
            for cr in 1..=4u8 {
                for ldro in [false, true] {
                    for len in [2usize, 8, 13, 32, 100] {
                        let payload: Vec<u8> = (0..len).map(|i| (i * 73 + usize::from(sf) * 5 + usize::from(cr)) as u8).collect();
                        let syms = encode_frame(sf, cr, ldro, true, &payload);
                        assert_eq!(syms.len(), 8 + payload_symbol_count(len, sf, cr, true, ldro));
                        let hb = decode_header_block(&syms[..8], sf).expect("header");
                        assert_eq!(hb.header, Header { payload_len: len as u8, cr, has_crc: true });
                        let d = decode_payload(&syms[8..], sf, ldro, &hb);
                        assert_eq!(d.payload, payload);
                        assert_eq!(d.crc_ok, Some(true));
                        assert_eq!(d.codeword_errors, 0);
                        // A corrupted payload must fail the CRC.
                        let mut bad = syms.clone();
                        bad[8] ^= 0x15;
                        let d2 = decode_payload(&bad[8..], sf, ldro, &hb);
                        assert!(d2.payload != payload || d2.crc_ok == Some(false) || cr >= 3, "sf {sf} cr {cr}");
                    }
                }
            }
        }
    }
}
