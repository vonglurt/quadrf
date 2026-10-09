# LR-008 — `qrf-lora`: a LoRa CSS modem written against the oracle's observable behaviour, and what the corpus measured (backlog R-08)

*Lab report. 2026-10-08.*

<!-- SPDX-License-Identifier: MIT -->

| Field | Value |
| --- | --- |
| Status | Final |
| Author | project (crate `crates/qrf-lora`, this VM) |
| Feeds | `backlog.md` R-08 (done); `docs/lora-corpus.md` §6; `crates/README.md`; the demodulator `qrf-dspd` will embed (R-07, R-P1) |

## Abstract

The LoRa physical layer is proprietary; its coding conventions are known only
through reverse engineering, and the one open implementation at hand
(`gr-lora_sdr`, GPL-3.0) may not be copied into this repository. `qrf-lora`
was therefore written from the published descriptions (Tapparel et al. 2020;
Robyns et al. 2018; Knight & Seeber 2016) and every convention was fixed by
black-box measurement: the oracle's transmitter was run on 29 payloads with
each stage's output recorded, and the Rust encoder must reproduce the chirp
indices and the waveform sample for sample (it does). The receiver is a
streaming demodulator with hard decisions. On the two R-08 cells of the corpus
it decodes 935 of 1 000 frames at SHORT_TURBO −5 dB (oracle 930) and 988 of
1 000 at LONG_FAST −15 dB (oracle 947), with no CRC-passing wrong payload;
across the 37 cells it decodes more frames than the oracle in every cell where
either decodes anything. The oracle receiver decodes the frames the Rust
modulator synthesises. One receiver effect cost a day's debugging and is worth
recording: at one sample per chip, a window misaligned by a fractional chip
sees the two sides of the chirp fold alias with a phase difference of 2π times
the fraction, which biases the peak by up to about 0.6 bins; detecting on
every sample phase and refining on aligned windows removes it.

## 1. Method

- The clean-room rule (docs/00-process.md §6; AGENTS.md): no GPL code in the repository. What may be used is the oracle's *behaviour*: the output of its transmitter for a given input, and its decode result on a given file. `scripts/oracle-vectors.py` runs the oracle transmitter chain on 29 cases (the seven Meshtastic presets with an all-zero, a short and a 32-byte payload; every coding rate at SF7; SF8, SF10, SF12) with a vector sink after every block, and writes `resources/vectors/lora-tx-vectors.json` (178 KB; a copy is committed under `crates/qrf-lora/tests/data/` so that the conformance test runs without the oracle). `[D]`
- `crates/qrf-lora/tests/r08_vectors.rs` asserts, for every case: the whitened nibbles, the header nibbles, the CRC nibbles, the Hamming codewords, the chirp indices and the first 64 modulated samples (within 2 × 10⁻³) equal the oracle's; and that the decoder inverts the chirp indices to the payload with a passing CRC. It passes on all 29 cases. `[M]` (`cargo test -p qrf-lora --test r08_vectors`, 2026-10-08)
- The receiver was developed against the modulator (loopback tests in `src/demodulator.rs`: every preset clean with ±2 ppm carrier offset; 1, 2 and 8 samples per chip; chunk sizes 1 and 2²⁰; ±50 ppm at SF9 and ±30 ppm at SF12; the two R-08 presets 2.5 dB above the S-003-2 threshold through the corpus channel filter; chunk independence; pure noise) and then measured on the corpus with `examples/corpus_check.rs` (`make lora-check`). `[D]`

## 2. Conventions the vectors fixed

All `[M]` (`resources/vectors/lora-tx-vectors.json`, 2026-10-08; `tests/r08_vectors.rs`). The module documentation in `crates/qrf-lora/src/coding.rs` carries the same statements next to the code.

| Stage | Convention |
| --- | --- |
| Whitening | payload byte *i* XOR the state of an 8-bit Fibonacci LFSR, polynomial x⁸ + x⁶ + x⁵ + x⁴ + 1, seed 0xFF, shifted one bit per byte (0xFF, 0xFE, 0xFC, 0xF8, 0xF0, 0xE1, 0xC2, 0x85, …; period 255); the CRC is not whitened |
| Nibble order | low nibble first within every byte, for payload and CRC; the header's five nibbles are length high, length low, (CR ≪ 1) \| CRC, 000c₄, c₃c₂c₁c₀ |
| Header checksum | the five parity equations over the three header nibbles published by Robyns et al. 2018 (`Header::checksum`); verified on six (length, CR) combinations |
| Payload CRC | CRC-16/CCITT (0x1021, init 0) over all bytes but the last two, XORed with those two (second-last into the high byte); sent low byte first |
| Hamming | codeword right-aligned in 4 + CR bits: the nibble's bits in reversed order (bit 0 highest), then parities p₀ = d₀⊕d₁⊕d₂, p₁ = d₁⊕d₂⊕d₃, p₂ = d₀⊕d₁⊕d₃, p₃ = d₀⊕d₂⊕d₃ (the first CR of them; p₃ is the overall parity, so 4/8 is the extended (8,4) code); 4/5 sends the nibble's even parity instead |
| Header block | always 8 symbols at the reduced rate (SF − 2 bits) and coding rate 4/8, carrying the five header nibbles and the first SF − 7 payload nibbles, those too at 4/8 |
| Interleaver | diagonal: bit *j* of symbol *i* is bit (cols − 1 − *i*) of codeword (*i* + *j*) mod rows; rows = SF (SF − 2 at the reduced rate), cols = 4 + CR; missing codewords in the last block are zero |
| Chirp index | full rate (gray⁻¹(v) + 1) mod 2^SF; reduced rate 4·gray⁻¹(v) + 1, so that ±1 index errors cancel in the header and under LDRO |
| Frame | preamble upchirps, two sync chirps at index (nibble × 8) each (16 and 88 for 0x2B), two downchirps and a quarter, then the symbols; every symbol starts at phase 0 and folds its frequency at +BW/2 |
| Symbol count | the SX1261/2 formula and the nibble count agree for every SF, CR, CRC, LDRO and payload length tried (`coding::tests::symbol_count_matches_the_datasheet_formula`) |

## 3. Receiver design and the fractional-chip fold

- Detection: the dechirped 2^SF-point spectrum of symbol-length windows at the symbol stride; a preamble is declared when four consecutive windows agree on the peak bin within ±1 and each peak exceeds the mean of the other bins by 6 (7.8 dB). Windows are computed on every sample phase of the oversampled stream (four at 4 × BW) and the phase whose run of windows has the highest mean peak-to-floor ratio wins. `[D]`
- Coarse estimate (Tapparel et al. 2020 §III; Bernier et al. 2020): the preamble peak sits at ε − d and a downchirp window's at ε + d (ε the carrier offset in bins, d the delay in chips), so 2ε ≡ b_up + b_down (mod 2^SF) and |ε| ≤ 2^SF/4 picks the solution, the same ±25 % of BW the SX1262 tolerates (S-003-14); the fractional part of ε is the phase advance per symbol at the preamble bin; the tone position between bins comes from the ratio of the larger neighbour to the peak, |Y[b ± 1]|/|Y[b]| = |δ|/(1 − |δ|) for a rectangular window. `[D]`
- At one sample per chip a window that starts τ chips (fractional) before the symbol boundary dechirps the two sides of the received chirp's fold onto the same tone with a relative phase of 2πτ (the folded side's frequency, s/N − 1 cycles per chip, aliases onto s/N with that constant when sampled at the chip rate). For τ = ½ and a fold mid-window the tone cancels at its own bin and reappears ±1 bin away at −3.9 dB; in the loopback test with a frame starting 308.5 chips into the stream, both the up- and the down-chirp peaks read 0.6 bins low and the integer part of ε came out wrong. `[M]`+`[D]` (debug trace of `reports_offsets_and_timing`, 2026-10-08; derivation in `src/demodulator.rs`)
- Remedy: after the coarse estimate the preamble and the two full downchirps are dechirped again on windows aligned to it (misalignment now ≤ 1 chip, so the offending side of the fold is at most one sample long), the residuals of the two directions give the final ε and d, and symbols are picked with cubic Lagrange interpolation at the fractional timing. The residual the sample grid leaves is at most 1/(2·os) chip; for 1/8 chip the worst symbol (fold mid-symbol) loses |½ + ½e^{jπ/4}| = −0.7 dB and the average symbol about half that. `[D]`
- Sync word: the two sync symbols demodulated with the final estimate must read within ±2 bins of (16, 88); when both are off by the same whole bin the integer part of ε is moved by it. This check also decides which of three candidate symbol positions the downchirp window fell in. `[D]`
- Decoding: hard decisions; header block → `Header` (checksum) → symbol count → payload blocks → deinterleave → Hamming (4/7 and 4/8 correct one error; 4/5 and 4/6 detect) → de-whiten → CRC. Timing is tracked through the payload from the tone's fractional position averaged over eight symbols. The result is deterministic for a given sample stream whatever the chunking (`deterministic_and_chunk_independent`). `[D]`
- Reported per frame: payload, CRC verdict, header, start and end sample, carrier offset in Hz, SNR in BW from the dechirped peak-to-floor ratio divided by 2^SF, RSSI in dBFS, codeword errors and corrections, the raw chirp indices (SPEC-009 S-009-5 `LoraFrame` fields). `[D]`

## 4. Results on the corpus

Every cell of the corpus (`docs/lora-corpus.md` §5: seed 20261008, 4 × BW, ±2 ppm, 100 frames per sweep cell and 1 000 per R-08 cell) through `examples/corpus_check.rs` on this VM, single-threaded; PER is strict (CRC passed and payload equal to the truth); the oracle columns are the symbol-paced, repeated runs recorded in each cell's `oracle.json`; "CFO error" and "start error" are the mean absolute errors of the reported carrier offset and frame start over the correctly decoded frames. `[M]` (`make lora-check`, 2026-10-08; `resources/corpus/qrf-lora-v1/qrf-lora-results.json`)

| Cell | SF | BW kHz | CR | LDRO | SNR dB (in BW) | Frames | qrf-lora correct | PER | Oracle correct | Oracle PER | CFO error Hz | Start error samples | Run s |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `SHORT_TURBO_snr-12.5` | 7 | 500 | 4/5 | off | -12.5 | 100 | 0 | 1.000 | 0 | 1.000 | 0.0 | 0.0 | 0.0 |
| `SHORT_TURBO_snr-10.0` | 7 | 500 | 4/5 | off | -10.0 | 100 | 2 | 0.980 | 1 | 0.990 | 1885.4 | 2.0 | 0.1 |
| `SHORT_TURBO_snr-7.5` | 7 | 500 | 4/5 | off | -7.5 | 100 | 66 | 0.340 | 64 | 0.360 | 345.2 | 0.1 | 0.0 |
| `SHORT_TURBO_snr-5.0` | 7 | 500 | 4/5 | off | -5.0 | 100 | 95 | 0.050 | 92 | 0.080 | 299.2 | 0.1 | 0.0 |
| `SHORT_TURBO_snr-2.5` | 7 | 500 | 4/5 | off | -2.5 | 100 | 100 | 0.000 | 97 | 0.030 | 253.9 | 0.1 | 0.0 |
| `SHORT_FAST_snr-12.5` | 7 | 250 | 4/5 | off | -12.5 | 100 | 0 | 1.000 | 0 | 1.000 | 0.0 | 0.0 | 0.0 |
| `SHORT_FAST_snr-10.0` | 7 | 250 | 4/5 | off | -10.0 | 100 | 4 | 0.960 | 0 | 1.000 | 575.0 | 1.0 | 0.1 |
| `SHORT_FAST_snr-7.5` | 7 | 250 | 4/5 | off | -7.5 | 100 | 74 | 0.260 | 53 | 0.470 | 166.9 | 0.2 | 0.0 |
| `SHORT_FAST_snr-5.0` | 7 | 250 | 4/5 | off | -5.0 | 100 | 95 | 0.050 | 91 | 0.090 | 152.8 | 0.1 | 0.0 |
| `SHORT_FAST_snr-2.5` | 7 | 250 | 4/5 | off | -2.5 | 100 | 99 | 0.010 | 95 | 0.050 | 128.1 | 0.1 | 0.0 |
| `MEDIUM_FAST_snr-17.5` | 9 | 250 | 4/5 | off | -17.5 | 100 | 0 | 1.000 | 0 | 1.000 | 0.0 | 0.0 | 0.1 |
| `MEDIUM_FAST_snr-15.0` | 9 | 250 | 4/5 | off | -15.0 | 100 | 6 | 0.940 | 3 | 0.970 | 55.8 | 0.3 | 0.3 |
| `MEDIUM_FAST_snr-12.5` | 9 | 250 | 4/5 | off | -12.5 | 100 | 90 | 0.100 | 74 | 0.260 | 43.0 | 0.2 | 0.1 |
| `MEDIUM_FAST_snr-10.0` | 9 | 250 | 4/5 | off | -10.0 | 100 | 94 | 0.060 | 91 | 0.090 | 33.8 | 0.1 | 0.1 |
| `MEDIUM_FAST_snr-7.5` | 9 | 250 | 4/5 | off | -7.5 | 100 | 100 | 0.000 | 93 | 0.070 | 28.6 | 0.1 | 0.1 |
| `LONG_TURBO_snr-22.5` | 11 | 500 | 4/8 | off | -22.5 | 100 | 0 | 1.000 | 0 | 1.000 | 0.0 | 0.0 | 0.6 |
| `LONG_TURBO_snr-20.0` | 11 | 500 | 4/8 | off | -20.0 | 100 | 42 | 0.580 | 14 | 0.860 | 24.8 | 0.2 | 1.1 |
| `LONG_TURBO_snr-17.5` | 11 | 500 | 4/8 | off | -17.5 | 100 | 98 | 0.020 | 72 | 0.280 | 21.0 | 0.2 | 0.6 |
| `LONG_TURBO_snr-15.0` | 11 | 500 | 4/8 | off | -15.0 | 100 | 100 | 0.000 | 95 | 0.050 | 15.8 | 0.1 | 0.5 |
| `LONG_TURBO_snr-12.5` | 11 | 500 | 4/8 | off | -12.5 | 100 | 100 | 0.000 | 97 | 0.030 | 12.2 | 0.0 | 0.6 |
| `LONG_FAST_snr-22.5` | 11 | 250 | 4/5 | off | -22.5 | 100 | 0 | 1.000 | 0 | 1.000 | 0.0 | 0.0 | 0.5 |
| `LONG_FAST_snr-20.0` | 11 | 250 | 4/5 | off | -20.0 | 100 | 23 | 0.770 | 12 | 0.880 | 11.6 | 0.4 | 1.1 |
| `LONG_FAST_snr-17.5` | 11 | 250 | 4/5 | off | -17.5 | 100 | 90 | 0.100 | 75 | 0.250 | 9.7 | 0.1 | 0.5 |
| `LONG_FAST_snr-15.0` | 11 | 250 | 4/5 | off | -15.0 | 100 | 98 | 0.020 | 90 | 0.100 | 8.2 | 0.1 | 0.4 |
| `LONG_FAST_snr-12.5` | 11 | 250 | 4/5 | off | -12.5 | 100 | 99 | 0.010 | 95 | 0.050 | 5.5 | 0.0 | 0.4 |
| `LONG_MODERATE_snr-22.5` | 11 | 125 | 4/8 | on | -22.5 | 100 | 0 | 1.000 | 0 | 1.000 | 0.0 | 0.0 | 0.6 |
| `LONG_MODERATE_snr-20.0` | 11 | 125 | 4/8 | on | -20.0 | 100 | 42 | 0.580 | 21 | 0.790 | 6.0 | 0.2 | 1.3 |
| `LONG_MODERATE_snr-17.5` | 11 | 125 | 4/8 | on | -17.5 | 100 | 100 | 0.000 | 75 | 0.250 | 5.3 | 0.2 | 0.7 |
| `LONG_MODERATE_snr-15.0` | 11 | 125 | 4/8 | on | -15.0 | 100 | 100 | 0.000 | 93 | 0.070 | 4.3 | 0.1 | 0.6 |
| `LONG_MODERATE_snr-12.5` | 11 | 125 | 4/8 | on | -12.5 | 100 | 100 | 0.000 | 97 | 0.030 | 3.1 | 0.0 | 1.0 |
| `LONG_SLOW_snr-25.0` | 12 | 125 | 4/8 | on | -25.0 | 100 | 0 | 1.000 | 0 | 1.000 | 0.0 | 0.0 | 1.5 |
| `LONG_SLOW_snr-22.5` | 12 | 125 | 4/8 | on | -22.5 | 100 | 53 | 0.470 | 16 | 0.840 | 3.0 | 0.3 | 2.5 |
| `LONG_SLOW_snr-20.0` | 12 | 125 | 4/8 | on | -20.0 | 100 | 98 | 0.020 | 87 | 0.130 | 2.5 | 0.1 | 1.3 |
| `LONG_SLOW_snr-17.5` | 12 | 125 | 4/8 | on | -17.5 | 100 | 100 | 0.000 | 93 | 0.070 | 2.3 | 0.1 | 1.2 |
| `LONG_SLOW_snr-15.0` | 12 | 125 | 4/8 | on | -15.0 | 100 | 100 | 0.000 | 98 | 0.020 | 1.4 | 0.0 | 1.2 |
| `r08_SHORT_TURBO_snr-5.0` | 7 | 500 | 4/5 | off | -5.0 | 1000 | 935 | 0.065 | 930 | 0.070 | 302.2 | 0.1 | 0.4 |
| `r08_LONG_FAST_snr-15.0` | 11 | 250 | 4/5 | off | -15.0 | 1000 | 988 | 0.012 | 947 | 0.053 | 8.3 | 0.1 | 4.5 |

- Totals: 5500 frames, qrf-lora 4091 correct, oracle 3761 correct. In every cell where either receiver decodes anything, `qrf-lora` decodes at least as many frames as the oracle; the gain is largest 2.5 dB below the threshold (LONG_SLOW −22.5 dB: 53 against 16; LONG_TURBO −20 dB: 42 against 14), where the oracle's own synchroniser, not the channel, limits it. `[M]`
- The R-08 criterion: SHORT_TURBO −5 dB PER 0.065 (oracle 0.070), LONG_FAST −15 dB PER 0.012 (oracle 0.053), both ≤ 0.10. `[M]`
- Carrier-offset estimates are within a tenth of a bin at every spreading factor (302 Hz at SF7/500 kHz where a bin is 3 906 Hz; 8 Hz at SF11/250 kHz where a bin is 122 Hz; 1–3 Hz at SF12/125 kHz); the frame start is placed within 0.1–0.4 samples at 4 × BW. `[M]`
- CRC verdicts: every frame's `crc_ok` equals the CRC recomputed over the payload it delivered (0 inconsistencies in 5500 frames; 319 frames were reported with a failed CRC), so no CRC-failed frame is reported `crc_ok`. 1 frame in 5500 (`LONG_FAST_snr-20.0`, index 48, SNR 2.5 dB below the threshold, six uncorrectable codewords) passed the CRC with a wrong payload: its last payload byte and the low CRC byte each differ from the truth in bit 5. The LoRa CRC covers the last two payload bytes only by XORing them into the check, so an error that flips the same bit in the last byte and in the low CRC byte is invisible to it; the diagonal interleaver makes that pattern likely, because the bits of one symbol land at the same bit position of codewords two apart, and the last payload byte's high nibble and the low CRC byte's high nibble are exactly two codewords apart. The oracle's check has the same construction (it matched the vectors) and would pass the same frame. A consumer that needs better than this re-checks at the application layer (Meshtastic frames carry their own integrity). `[M]`+`[D]`
- Run time: all 37 cells (3.4 GB of CS8) in 24 s of single-core time on the VM, dominated by the SF11–12 cells at 0.5–4.4 s each; the receiver is far from a real-time concern at these rates (R-07 budgets it on the Pi 5). `[M]`

## 5. The modulator in the oracle

Seven cells of 50 frames each, synthesised by `examples/make_cell.rs` with `qrf_lora::Modulator` and the corpus channel model (docs/lora-corpus.md §2: ±2 ppm, 0.5–2 symbol gaps, the Kaiser channel filter, CS8 at 4 × BW) at 0 dB SNR in BW (7.5 dB above the SF7 threshold, 20 dB above SF12), seed 20261008; then the oracle receiver with the symbol pacing and double run of `scripts/make-corpus.py verify`, and `qrf-lora`'s own receiver on the same files. `[M]` (`make lora-oracle-check`, 2026-10-08; `resources/corpus/qrf-lora-rs/*/oracle.json`)

| Cell | Preset | Frames | Oracle: headers | CRC ok | correct | missed | PER | Repeatable | qrf-lora correct |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `rs_SHORT_TURBO_snr+0.0` | SHORT_TURBO | 50 | 46 | 46 | 46 | 4 | 0.080 | yes | 50 |
| `rs_SHORT_FAST_snr+0.0` | SHORT_FAST | 50 | 48 | 48 | 48 | 2 | 0.040 | yes | 50 |
| `rs_MEDIUM_FAST_snr+0.0` | MEDIUM_FAST | 50 | 47 | 47 | 47 | 3 | 0.060 | yes | 50 |
| `rs_LONG_TURBO_snr+0.0` | LONG_TURBO | 50 | 50 | 50 | 50 | 0 | 0.000 | yes | 50 |
| `rs_LONG_FAST_snr+0.0` | LONG_FAST | 50 | 50 | 50 | 50 | 0 | 0.000 | yes | 50 |
| `rs_LONG_MODERATE_snr+0.0` | LONG_MODERATE | 50 | 49 | 49 | 49 | 1 | 0.020 | yes | 50 |
| `rs_LONG_SLOW_snr+0.0` | LONG_SLOW | 50 | 49 | 46 | 46 | 1 | 0.080 | yes | 50 |

- The oracle decodes 336 of 350 frames made by the Rust modulator, every cell repeatable, with no CRC-passing wrong payload and no header error; the misses (none at SF11, up to 4 of 50 at SF7) are in the range the oracle shows on its own transmitter's frames well above the threshold (`SHORT_TURBO_snr-2.5`: 3 of 100 missed; `LONG_SLOW_snr-15.0`: 2 of 100, docs/lora-corpus.md §5), so they are the oracle's synchroniser, not the waveform. `qrf-lora` decodes all 350. The modulator half of the R-08 check is met. `[M]`

## 6. Limits and follow-ups

- Explicit header only (S-003-7); coding rates 4/5…4/8; SF 5–12 accepted, 7–12 tested; payloads ≤ 255 bytes; the CRC rule for payloads shorter than two bytes is a guess (`[C]`, no Meshtastic frame is that short). `[D]`
- The oracle's receiver is also hard-decision; neither side uses soft decoding, so the comparison is like for like. Soft-decision Hamming decoding would gain about 1 dB at 4/8 and nothing at 4/5. `[C]` (closes with a soft-decision variant measured on the same corpus)
- Sampling-frequency offset is tracked only through the fractional tone position; the corpus has ideal clocks, so this path is untested beyond the loopback. A field test with a crystal-class node (R-P1 injection, G07 T-1) measures it. `[C]`
- At one sample per chip the interpolation cannot recover the fractional timing (critically sampled chirp); the loopback passes at 10 dB, and sensitivity there is not characterised. The system runs the demodulator at 2 or 4 samples per chip (channeliser output, R-07). `[D]`
- Carrier offsets beyond ±25 % of BW are rejected by construction, as on the SX1262 (S-003-14). `[D]`
