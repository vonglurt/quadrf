# The LoRa I/Q test corpus and its oracle (backlog V-08)

<!-- SPDX-License-Identifier: MIT -->

What `qrf-lora` (backlog R-08) is tested against before any radio exists:
synthetic Meshtastic-style LoRa frames with a recorded truth, and the decode
result of an independent receiver on the same bytes. The corpus lives in the
gitignored `resources/corpus/qrf-lora-v1/`; everything here is reproduced by
`make oracle` and `make corpus` (`scripts/build-oracle.sh`,
`scripts/make-corpus.py`).

## 1. The oracle

- The oracle is `gr-lora_sdr` (Tapparel et al., EPFL TCL; GPL-3.0) at commit `862746d`, built against the VM's GNU Radio 3.10.12 into the private prefix `resources/oracle/` and run as a separate process; nothing from it is linked into a qrf binary. `[S]` (`vendor/tapparel-gr-lora_sdr/ATTRIBUTION.md`; `docs/resources-manifest.md`; `docs/00-process.md` §6)
- Its transmitter chain (whitening, header, CRC, Hamming, interleaver, Gray mapping, modulate) produces the clean frames; its receiver chain (frame sync, FFT demodulation with hard decisions, Gray mapping, deinterleaver, Hamming decoder, header decoder, dewhitening, CRC check) re-decodes the finished corpus. The frame boundaries are taken from the modulator's own `frame_len` tags, not from a formula. `[D]` (`scripts/make-corpus.py`)
- The receiver is timing-sensitive by construction: `frame_sync` learns a frame's symbol count from `header_decoder`, five blocks downstream, through an asynchronous `frame_info` message, and its work function takes another path while that message has not arrived (`m_received_head`). `[S]` (`resources/repos/gr-lora_sdr/lib/frame_sync_impl.cc` lines 394–426 and 849–867) Fed from a file it outruns the message and loses most payloads. Upstream's own simulation harness paces the stream with a 10 × real-time throttle. `[S]` (`resources/repos/gr-lora_sdr/apps/simulation/flowgraph/tx_rx_simulation.py`, `blocks.throttle(…, samp_rate*10, True)`)
- A rate alone is not the cure: GNU Radio's throttle releases items in chunks of up to the buffer size (8 191 items), which at SF7 and 4 × BW is sixteen symbols, a whole header and more delivered at once, so the synchroniser can pass the header before the message exists whatever the average rate; at SF11–12 a chunk is about one symbol and the same rate is repeatable (§4). The verifier therefore paces by symbols: chunks bounded to one symbol, 2 ms of wall-clock time between chunks, at every spreading factor (about 0.13 × real time at SF7, 16 × at SF12). `[D]` (`scripts/make-corpus.py`, `SYMBOL_INTERVAL_MS`)
- Verification belongs on an otherwise idle machine (§4), and the verifier runs each cell twice and accepts identical results; if they differ it runs a third time, keeps the best and records `stable: false` with the three counts, so an unrepeatable cell is visible in `oracle.json` and in the table of §5. `[D]`
- Intermediates (the transmitter's float stream, up to a gigabyte per cell) go under `resources/tmp/`, never `/tmp`, which is a 1.2 GB tmpfs on the VM. `[M]` (`df /tmp`, 2026-10-08)

## 2. Conventions

- Air interface (SPEC-003 S-003-7): sync word 0x2B, 16-symbol preamble, explicit header, CRC on, low-data-rate optimisation when the symbol lasts more than 16 ms (the module's AUTO rule, which is Meshtastic's threshold). `[D]`
- SNR is defined in the signal bandwidth, as the SX1261/2 datasheet defines it (S-003-2); the white-noise level before filtering is therefore 10 log10(4) = 6.0 dB below the stated SNR per sample, and the channel filter restores it: the noise power spectral density inside the signal band is exactly what the stated SNR requires. `[D]`
- Channel filter: a Kaiser-window low-pass on signal plus noise, unity DC gain, −6 dB at 0.6 BW, 60 dB stopband from 0.7 BW; 73 taps at 4 × BW. It stands in for the channel filter of any receiver front end (or the qrf channeliser's bin): a demodulator that does no filtering of its own sees the stated SNR. The group delay is recorded and the frame start indices in the sidecar already include it. `[D]`
- Per frame: payload length uniform in 8–32 bytes with the frame index in the first two bytes (so a receiver's output is matched to the truth by content, and a CRC-valid but wrong payload is counted as such); carrier offset uniform within ±2 ppm of 915 MHz (±1.83 kHz); initial phase uniform; a noise-only gap of 0.5–2 symbols before it. `[D]`
- Quantisation: CS8 (interleaved int8 I, Q) at 4 × BW with the filtered noise at 20 LSB rms per complex sample, so that the 8-bit quantisation noise (0.29 LSB rms) is 37 dB below the thermal noise and peaks stay inside ±127 (clipped samples are counted in the sidecar; the generated cells report zero). The signal scale in LSB per unit amplitude is in the sidecar. `[D]`
- Determinism: every random draw comes from `numpy.random.default_rng([seed, cell_index])`; the oracle's transmitter is deterministic; the same machine reproduces every byte (`make corpus-check`). Across machines the truth is identical and the samples may differ in the last bit where GNU Radio's VOLK picks another kernel. `[D]`

## 3. Layout

```
resources/corpus/qrf-lora-v1/
  MANIFEST.json                 seed, oversampling, sha256 and size of every file
  <cell>/iq.cs8                 the samples
  <cell>/truth.json             parameters, scale, filter, one record per frame (index, payload, start, length, CFO, phase, gap)
  <cell>/oracle.json            the oracle receiver's result: headers found, CRC ok, correct, missed, PER
```

Cells: `<PRESET>_snr<±x.x>` for the seven Meshtastic presets at the S-003-2
threshold −5, −2.5, 0, +2.5 and +5 dB (100 frames), and
`r08_SHORT_TURBO_snr-5.0`, `r08_LONG_FAST_snr-15.0` (1 000 frames): the two
cells of the R-08 check. `python3 -I scripts/make-corpus.py list` prints them.

## 4. Measured limits of the oracle (2026-10-08, this VM)

Measured with 40–60-frame diagnostic cells while the generator was being
written; they decide the corpus defaults and bound what the oracle's PER can
be compared with. `[M]` (`scripts/make-corpus.py generate --frames N --os … --cfo-ppm …`, 2026-10-08)

| Condition | Oracle result |
| --- | --- |
| White noise over the whole 4 × BW band, no filter, SF7/500 kHz at −5 dB in BW | 2 of 40 frames found: the receiver decimates without filtering and loses the full oversampling ratio in SNR; hence the channel filter |
| Filtered, no carrier offset, 2.5 dB above the threshold: SF7/500 kHz −5 dB, SF11/250 kHz −15 dB, SF12/125 kHz −17.5 dB | 40 of 40 decoded in each |
| Filtered, no carrier offset, at the threshold: SF7/500 kHz −7.5 dB | 31 of 40 (PER 0.225) |
| Carrier offset ±10 ppm, 4 × BW: SF7 −5 dB / SF11 −15 dB | PER 0.10 / 0.45 |
| Carrier offset ±5 ppm, 4 × BW: SF11 −15 dB | PER 0.20 |
| Carrier offset ±2 ppm, 4 × BW: SF11 −15 dB | PER 0.083 (55 of 60) |
| Carrier offset ±2 ppm at 2 × BW: SF11 −15 dB | 53 of 60 frames found, 0 decoded: the offset compensation fails systematically at 2 × BW; hence 4 × BW |
| SF11 at 4 × BW, GNU Radio default buffers | scheduler error (frame sync asks for 8 200 items of an 8 191-item buffer); the generator enlarges the source buffer |
| `SHORT_TURBO_snr-5.0` and `MEDIUM_FAST_snr-10.0` generated in two separate processes (final defaults) | `iq.cs8` and `truth.json` sha256 identical in both cells: the generator is deterministic |
| One file verified three times, unpaced, other work running | 5, 35 and 97 of 97 found frames decoded: the header-message race of §1 |
| The same file, paced at 10 × real time, machine otherwise idle, three sequential runs | 97, 97, 97 (and a second cell 53, 53, 53): repeatable |
| Paced at 10 × real time but three verifications running at once | 97, 88 and 0 decoded: pacing is not enough under CPU contention; hence the double run and the idle-machine rule |
| Full sweep paced at 10 × real time, idle machine | 33 of 37 cells repeatable; the four that were not are all SF7 or SF9 (`SHORT_TURBO` −5.0 and −2.5 dB, `MEDIUM_FAST` −10 dB, the 1 000-frame `SHORT_TURBO` cell: runs 277, 359, 930) |
| SF7 cells re-verified at 2.5 × real time with default chunks | still unrepeatable (77/92/92, 87/97/97, 95/54/95; the 1 000-frame cell 562/56/35) with 7 s of CPU in 65 s: not load but chunk size |
| The same SF7 cells paced one symbol per 2 ms, two invocations | 92/92, 97/97, 95/95 and again 92/92: repeatable; adopted for every cell |

Consequences for R-08: the oracle's PER is a reference at ±2 ppm and
4 × BW, from a symbol-paced, repeated, idle-machine run; `qrf-lora` is expected to
match it there and to beat it at larger offsets, where the oracle, not the
corpus, is the weak element. `qrf-lora` itself must be deterministic from a
file; a result that depends on pacing is a defect there, not a property to
tolerate. A corpus
variant with ±10–20 ppm (crystal-class nodes) is generated with
`--cfo-ppm 10` when R-08 needs it; its oracle column is then not a reference.

## 5. Results

The table below is the output of `python3 -I scripts/make-corpus.py report`
after `make corpus` (seed 20261008, 4 × BW, ±2 ppm) and a verification of
every cell with the symbol pacing of §1 on the idle VM; PER is strict (a
frame counts only when the CRC passes and the payload equals the truth). `[M]`

| Cell | Preset | BW kHz | SF | CR | LDRO | SNR dB (in BW) | Frames | MB | Oracle: headers | CRC ok | correct | missed | PER | Repeatable |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `SHORT_TURBO_snr-12.5` | SHORT_TURBO | 500 | 7 | 4/5 | off | -12.5 | 100 | 6.6 | 0 | 0 | 0 | 100 | 1.000 | yes |
| `SHORT_TURBO_snr-10.0` | SHORT_TURBO | 500 | 7 | 4/5 | off | -10.0 | 100 | 6.4 | 5 | 1 | 1 | 95 | 0.990 | yes |
| `SHORT_TURBO_snr-7.5` | SHORT_TURBO | 500 | 7 | 4/5 | off | -7.5 | 100 | 6.4 | 65 | 64 | 64 | 35 | 0.360 | yes |
| `SHORT_TURBO_snr-5.0` | SHORT_TURBO | 500 | 7 | 4/5 | off | -5.0 | 100 | 6.6 | 92 | 92 | 92 | 8 | 0.080 | yes |
| `SHORT_TURBO_snr-2.5` | SHORT_TURBO | 500 | 7 | 4/5 | off | -2.5 | 100 | 6.5 | 97 | 97 | 97 | 3 | 0.030 | yes |
| `SHORT_FAST_snr-12.5` | SHORT_FAST | 250 | 7 | 4/5 | off | -12.5 | 100 | 6.6 | 0 | 0 | 0 | 100 | 1.000 | yes |
| `SHORT_FAST_snr-10.0` | SHORT_FAST | 250 | 7 | 4/5 | off | -10.0 | 100 | 6.5 | 2 | 0 | 0 | 98 | 1.000 | yes |
| `SHORT_FAST_snr-7.5` | SHORT_FAST | 250 | 7 | 4/5 | off | -7.5 | 100 | 6.4 | 58 | 53 | 53 | 42 | 0.470 | yes |
| `SHORT_FAST_snr-5.0` | SHORT_FAST | 250 | 7 | 4/5 | off | -5.0 | 100 | 6.4 | 91 | 91 | 91 | 9 | 0.090 | yes |
| `SHORT_FAST_snr-2.5` | SHORT_FAST | 250 | 7 | 4/5 | off | -2.5 | 100 | 6.4 | 95 | 95 | 95 | 5 | 0.050 | yes |
| `MEDIUM_FAST_snr-17.5` | MEDIUM_FAST | 250 | 9 | 4/5 | off | -17.5 | 100 | 22.3 | 0 | 0 | 0 | 100 | 1.000 | yes |
| `MEDIUM_FAST_snr-15.0` | MEDIUM_FAST | 250 | 9 | 4/5 | off | -15.0 | 100 | 22.7 | 10 | 3 | 3 | 90 | 0.970 | yes |
| `MEDIUM_FAST_snr-12.5` | MEDIUM_FAST | 250 | 9 | 4/5 | off | -12.5 | 100 | 22.8 | 75 | 74 | 74 | 25 | 0.260 | yes |
| `MEDIUM_FAST_snr-10.0` | MEDIUM_FAST | 250 | 9 | 4/5 | off | -10.0 | 100 | 22.5 | 91 | 91 | 91 | 9 | 0.090 | yes |
| `MEDIUM_FAST_snr-7.5` | MEDIUM_FAST | 250 | 9 | 4/5 | off | -7.5 | 100 | 22.4 | 93 | 93 | 93 | 7 | 0.070 | yes |
| `LONG_TURBO_snr-22.5` | LONG_TURBO | 500 | 11 | 4/8 | off | -22.5 | 100 | 98.9 | 0 | 0 | 0 | 100 | 1.000 | yes |
| `LONG_TURBO_snr-20.0` | LONG_TURBO | 500 | 11 | 4/8 | off | -20.0 | 100 | 103.8 | 18 | 14 | 14 | 82 | 0.860 | yes |
| `LONG_TURBO_snr-17.5` | LONG_TURBO | 500 | 11 | 4/8 | off | -17.5 | 100 | 102.2 | 72 | 72 | 72 | 28 | 0.280 | yes |
| `LONG_TURBO_snr-15.0` | LONG_TURBO | 500 | 11 | 4/8 | off | -15.0 | 100 | 103.9 | 95 | 95 | 95 | 5 | 0.050 | yes |
| `LONG_TURBO_snr-12.5` | LONG_TURBO | 500 | 11 | 4/8 | off | -12.5 | 100 | 98.8 | 97 | 97 | 97 | 3 | 0.030 | yes |
| `LONG_FAST_snr-22.5` | LONG_FAST | 250 | 11 | 4/5 | off | -22.5 | 100 | 83.8 | 0 | 0 | 0 | 100 | 1.000 | yes |
| `LONG_FAST_snr-20.0` | LONG_FAST | 250 | 11 | 4/5 | off | -20.0 | 100 | 80.9 | 18 | 12 | 12 | 82 | 0.880 | yes |
| `LONG_FAST_snr-17.5` | LONG_FAST | 250 | 11 | 4/5 | off | -17.5 | 100 | 83.3 | 75 | 75 | 75 | 25 | 0.250 | yes |
| `LONG_FAST_snr-15.0` | LONG_FAST | 250 | 11 | 4/5 | off | -15.0 | 100 | 80.3 | 90 | 90 | 90 | 10 | 0.100 | yes |
| `LONG_FAST_snr-12.5` | LONG_FAST | 250 | 11 | 4/5 | off | -12.5 | 100 | 81.1 | 95 | 95 | 95 | 5 | 0.050 | yes |
| `LONG_MODERATE_snr-22.5` | LONG_MODERATE | 125 | 11 | 4/8 | on | -22.5 | 100 | 113.3 | 0 | 0 | 0 | 100 | 1.000 | yes |
| `LONG_MODERATE_snr-20.0` | LONG_MODERATE | 125 | 11 | 4/8 | on | -20.0 | 100 | 110.0 | 25 | 21 | 21 | 75 | 0.790 | yes |
| `LONG_MODERATE_snr-17.5` | LONG_MODERATE | 125 | 11 | 4/8 | on | -17.5 | 100 | 109.2 | 76 | 75 | 75 | 24 | 0.250 | yes |
| `LONG_MODERATE_snr-15.0` | LONG_MODERATE | 125 | 11 | 4/8 | on | -15.0 | 100 | 111.9 | 93 | 93 | 93 | 7 | 0.070 | yes |
| `LONG_MODERATE_snr-12.5` | LONG_MODERATE | 125 | 11 | 4/8 | on | -12.5 | 100 | 117.5 | 97 | 97 | 97 | 3 | 0.030 | yes |
| `LONG_SLOW_snr-25.0` | LONG_SLOW | 125 | 12 | 4/8 | on | -25.0 | 100 | 208.2 | 0 | 0 | 0 | 100 | 1.000 | yes |
| `LONG_SLOW_snr-22.5` | LONG_SLOW | 125 | 12 | 4/8 | on | -22.5 | 100 | 211.1 | 20 | 16 | 16 | 80 | 0.840 | yes |
| `LONG_SLOW_snr-20.0` | LONG_SLOW | 125 | 12 | 4/8 | on | -20.0 | 100 | 218.1 | 90 | 87 | 87 | 10 | 0.130 | yes |
| `LONG_SLOW_snr-17.5` | LONG_SLOW | 125 | 12 | 4/8 | on | -17.5 | 100 | 210.0 | 97 | 93 | 93 | 3 | 0.070 | yes |
| `LONG_SLOW_snr-15.0` | LONG_SLOW | 125 | 12 | 4/8 | on | -15.0 | 100 | 208.0 | 100 | 98 | 98 | 0 | 0.020 | yes |
| `r08_SHORT_TURBO_snr-5.0` | SHORT_TURBO | 500 | 7 | 4/5 | off | -5.0 | 1000 | 65.1 | 930 | 930 | 930 | 70 | 0.070 | yes |
| `r08_LONG_FAST_snr-15.0` | LONG_FAST | 250 | 11 | 4/5 | off | -15.0 | 1000 | 818.0 | 947 | 947 | 947 | 53 | 0.053 | yes |

All 37 cells were repeatable (two identical runs each; `stable: true` in
every `oracle.json`). Read across a preset, the oracle's hard-decision
receiver decodes with PER ≤ 0.10 at 2.5 dB above the S-003-2 threshold,
0.13–0.47 at the threshold, ≥ 0.79 at 2.5 dB below it and nothing at 5 dB
below: its waterfall sits within about 2.5 dB of the datasheet figure for
every preset. `[D]` (the table) The R-08 check cells give the reference
`qrf-lora` is compared with: 930 of 1 000 frames at SHORT_TURBO −5 dB and
947 of 1 000 at LONG_FAST −15 dB. Corpus: 3.4 GB, 111 files;
`MANIFEST.json` sha256 `4cce02e671b2399a7c2b7b89ab5736076adca38790a13e960ae414a56502803d`
(also in `docs/resources-manifest.md`). `[M]` (`make corpus-check`, 2026-10-08)

## 6. `qrf-lora` against the corpus (backlog R-08)

The receiver and modulator of `crates/qrf-lora` were checked against this
corpus and this oracle on 2026-10-08; the method, the coding conventions the
oracle's stage vectors fixed, the receiver design and the full per-cell table
are in `lab/LR-008-qrf-lora-modem.md`. `[D]`

| Check | Result |
| --- | --- |
| `r08_SHORT_TURBO_snr-5.0`, 1 000 frames | qrf-lora 935 correct (PER 0.065); oracle 930 (0.070) |
| `r08_LONG_FAST_snr-15.0`, 1 000 frames | qrf-lora 988 correct (PER 0.012); oracle 947 (0.053) |
| All 37 cells, 5500 frames | qrf-lora 4091 correct, oracle 3761; qrf-lora decodes at least as many frames as the oracle in every cell |
| CRC verdicts | 0 of 5500 frames with a `crc_ok` verdict that disagrees with the CRC recomputed over the delivered payload; 1 CRC collision (`LONG_FAST_snr-20.0` index 48: the same bit flipped in the last payload byte and in the low CRC byte, which the LoRa CRC construction cannot see; LR-008 §4) |
| Modulator in the oracle | 7 cells × 50 frames at 0 dB in BW (`resources/corpus/qrf-lora-rs/`): the oracle decodes 336 of 350, no wrong payload, every cell repeatable; qrf-lora decodes all 350 |

`[M]` (`make lora-check`, `make lora-oracle-check`, 2026-10-08; `resources/corpus/qrf-lora-v1/qrf-lora-results.json`, `resources/corpus/qrf-lora-rs/*/oracle.json`)

The oracle's stage vectors that fixed `qrf-lora`'s coding conventions come
from `scripts/oracle-vectors.py` (`make lora-vectors`); the committed copy
`crates/qrf-lora/tests/data/lora-tx-vectors.json` is program output of the
oracle, not its code, and `tests/r08_vectors.rs` holds the encoder to it. `[D]`
