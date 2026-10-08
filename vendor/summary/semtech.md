# Semtech — SX1261/2 datasheet and product data: restatement

Attribution: `vendor/semtech/ATTRIBUTION.md`.

## SX1261/2 datasheet (DS.SX1261-2.W.APP Rev 1.1, December 2017, 107 pages) `[S]`

Source: `resources/datasheets/SX1261-2.pdf` (text in `SX1261-2.txt`), supplied
by the user through the UTM share 2026-10-08 as `SX1261.PDF`; a mirror copy
(the PDF carries an "Alldatasheet" title tag); sha256
`93ee7130c727f611786539a53a83f0073d03e8377d8d30b3fe45278032ceb996`. Newer
revisions exist at semtech.com (U-003-2). Page numbers are the datasheet's.

### LoRa modem (§6.1.1, pp. 37–39)

| Item | Value |
| --- | --- |
| Spreading factors | 5–12 (2^SF chips/symbol 32–4096); SF5/SF6 new in this family and not backward-compatible with SX127x SF6; 12-symbol preamble recommended for SF5/6 |
| Typical demodulator SNR (Table 6-1) | SF5 −2.5, SF6 −5, SF7 −7.5, SF8 −10, SF9 −12.5, SF10 −15, SF11 −17.5, SF12 −20 dB |
| Bandwidths (Table 6-2, DSB) | 7.81, 10.42, 15.63, 20.83, 31.25, 41.67, 62.5, 125, 250, 500 kHz (some unavailable below 400 MHz) |
| Receiver architecture | low-IF double conversion for BW ≤ 250 kHz; zero-IF single conversion at 500 kHz |
| Coding rates (Table 6-3) | 4/5, 4/6, 4/7, 4/8 (overhead 1.25–2) |
| LDRO | recommended when the symbol time ≥ 16.38 ms; reduces bits per symbol to SF − 2 |
| Symbol rate | Rs = BW / 2^SF; constant-envelope signal; one chip per second per hertz |

### Receive mode (Table 3-8, pp. 19–20; Rx boosted gain, split Rx/Tx paths, RF-switch loss excluded)

| Item | Value |
| --- | --- |
| LoRa sensitivity | 10.4 kHz: SF7 −134, SF12 −148 dBm; 125 kHz: SF7 −124, SF12 −137; 250 kHz: SF7 −121, SF12 −134; 500 kHz: SF7 −117, SF12 −129 dBm |
| LoRa sensitivity, power-saving gain, direct-tie | 125 kHz SF12 −133 dBm |
| Co-channel rejection | FSK −9 dB; LoRa SF7 5 dB, SF12 19 dB |
| Adjacent-channel rejection (±1.5 × BW) | 125 kHz SF7 60 dB, SF12 72 dB |
| Blocking immunity (125 kHz SF12) | 88 / 90 / 99 dB at ±1 / ±2 / ±10 MHz |
| IIP3 | −5 dBm (tones 1 and 1.96 MHz above LO) |
| Image attenuation | 35 dB without, 54 dB with I/Q calibration |
| Receiver wake-up | 41 µs (FS to RX) |
| Tolerated Tx–Rx frequency offset, no sensitivity loss | ±25 % of BW (SF5–12); and SF12 ±50 ppm, SF11 ±100 ppm, SF10 ±200 ppm, the tighter applying |

### General and transmit (Tables 3-6, 3-7, pp. 16–18)

| Item | Value |
| --- | --- |
| Synthesiser range / step | 150–960 MHz / 0.95 Hz |
| Synthesiser phase noise (868/915 MHz) | −75, −95, −100, −120, −135 dBc/Hz at 1 kHz, 10 kHz, 100 kHz, 1 MHz, 10 MHz |
| Synthesiser wake-up / hop | 40 µs / 30 µs (10 MHz step) |
| Tx power and current, 868/915 MHz | SX1262: +22 dBm at 118 mA (107 mA with optimal settings), +20 dBm 102/90 mA, +17 dBm 95/75 mA, +14 dBm 90/63 mA; SX1261: +15 dBm at 32.5 mA (3.3 V), +14 dBm 21 mA (434/490 MHz) |
| Tx power drop at low supply (SX1262, +22 dBm) | 2 dB at 2.7 V, 3 dB at 2.4 V, 6 dB at 1.8 V |
| LoRa bit rate | 0.018 (SF12, 7.8 kHz) to 62.5 kbit/s (SF5, 500 kHz); FSK 0.6–300 kbit/s |
| TCXO | DIO3 supplies 1.6–3.3 V, ≤ 4 mA; clipped-sine ≤ 1.2 Vpp through 220 Ω and 10 pF to XTA |

### Consequences (`analysis/linkbudget.py` T23)

- Implied NF plus implementation loss from the sensitivity rows is 6.0–8.0 dB; the 6 dB model in SPEC-003 S-003-3 (T1) is 0.5–2 dB optimistic for an SX1262 node; by interpolation LONG_FAST ≈ −131 dBm, SHORT_TURBO −117 dBm, LONG_TURBO ≈ −127 dBm. `[D]`
- Every Meshtastic preset tolerates ≥ 31.25 kHz of offset, so a 1 ppm free-running translator LO (4.6 kHz) is within limits on the transmit side; our own demodulator must acquire ≥ ±5 kHz. `[D]`

## SX1262 product page (retrieved 2026-10-08, `resources/lora/semtech-sx1262-product.html`) `[S]`

- Continuous coverage 150–960 MHz; up to +22 dBm; sensitivity "down to −148 dBm"; 170 dB maximum link budget; 88 dB blocking immunity at 1 MHz offset; up to 62.5 kbit/s LoRa and 300 kbit/s FSK.

## LR1121 and SX1280 product pages (retrieved 2026-10-08, `resources/lora/semtech-lr1121-product.html`, `semtech-sx1280-product.html`) `[S]`

- LR1121: multi-band LoRa and LR-FHSS transceiver covering 150–960 MHz (sub-GHz, PA up to +22 dBm), the 2.4 GHz ISM band (PA up to +11.5 dBm), 2 GHz S-band and 1.55 GHz L-band; one BOM adapts to regions through the matching network; development kits exist for 490, 868, 915 MHz with 2.4 GHz.
- SX1280: 2.4 GHz LoRa/FLRC/(G)FSK transceiver with ranging; sensitivity down to −132 dBm; +12.5 dBm PA.
- No Semtech LoRa transceiver page lists any band above 2.5 GHz; a 5.8 GHz LoRa link exists only as an SDR-generated waveform (G04 C10, closed).
