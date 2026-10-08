# LR-005 — Coherent 915 MHz receiver options and the parallel data feed into the overlay

*Lab report. 2026-10-08.*

<!-- SPDX-License-Identifier: MIT -->

| Field | Value |
| --- | --- |
| Status | Final (desk) |
| Author | project |
| Feeds | G04 addendum (C12 specified, C13/C14 added); G05 addendum; SPEC-009 S-009-3/13; SPEC-011; backlog H-04…H-08, R-12, F-03 |

## Abstract

The user asked whether the tile can be used "in addition to a phased array
for LoRa itself", with a parallel data feed into the overlay, and whether a
wider spectrum can be scanned with a custom addition. Four ways to obtain
915 MHz data are compared on bandwidth, noise figure, coherence, bits, data
rate, power, licence and time to first data: the tile behind the
frequency-translating front end (G05 C3), a KrakenSDR-class five-channel
coherent receiver (C12), single USB SDR dongles (new C13), and SX1262 nodes
or sticks (C9). The decisive numbers: the FTFE path sees all 104 slots in
one 26 MHz capture at NF ≈ 1.2–3.2 dB; a KrakenSDR sees 9–10 slots per
2.4 MHz capture, needs 11 retunes with recalibration for a whole-band map,
draws 2.2 A and runs a GPL Python/C stack; a dongle gives occupancy and
decoding of a few slots with no bearing. The recommended sequence is dongle
and SX1262 now (occupancy, decoding, field tests T-1…T-4), the tile at 5.8 GHz
when the kit arrives (RF-vision layer), the FTFE for whole-band bearings
(G05), and a KrakenSDR only if bearings are wanted before the FTFE is proven.
All four are sensor plug-ins on the same bus, which is what makes the
parallel feed a configuration rather than a design.

## I. Objective

1. Compare the ways of producing a 915 MHz feed (occupancy, decoded frames, bearings) with numbers from `analysis/`.
2. Decide how a second array "for LoRa itself" relates to the tile: receive, transmit, both, under which rule part.
3. Define the wider-spectrum supplement.
4. State the overlay alignment requirement that proves the parallel feed works.

## II. Materials

| Item | Detail |
| --- | --- |
| Tile + FTFE | SPEC-001 rev 2, SPEC-007, G05 F.05.1…13 and addendum |
| KrakenSDR-class | SPEC-011 (vendor wiki and READMEs in `resources/krakensdr/`) |
| USB dongles | RTL-SDR (R820T/RTL2832U, 24–1766 MHz, 2.4 MSPS, 8 bit, ≈ 0.3 A) `[C]` until a datasheet is filed; HackRF One (1 MHz–6 GHz, 20 MSPS, 8 bit, half-duplex, ≈ 0.5 A) `[C]` |
| SX1262 nodes | SPEC-003, SPEC-004 rev 2 (HAT pin table, USB sticks), Semtech product page |
| Budgets | T14, T15, T16, T18, T20 |

## III. Method

Criteria were fixed first (bandwidth per capture, slots per capture, NF,
channels/coherence, bits, bearing capability, data rate, power, software
licence and language, integration effort, cost class, time to first data);
each candidate was scored from sourced or derived numbers; a sequence was
chosen that gets each capability at the earliest date its hardware exists.

## IV. Results

### A. Comparison

| Criterion | Tile + FTFE (C3) | KrakenSDR-class (C12) | RTL-SDR dongle (C13) | SX1262 node/stick (C9) |
| --- | --- | --- | --- | --- |
| Capture bandwidth | 26 MHz (all 104 slots) `[S]` S-001-5 | 2.4–2.56 MHz (9–10 slots) `[S]`+`[D]` S-011-2/13 | 2.4 MHz (9–10 slots) `[C]` | one slot `[S]` S-004-8 |
| Whole-band map | 1 capture | 11 retunes, recalibration each `[D]` | 11 retunes | 104 retunes |
| Channels / coherence | 4, hardware-coherent LO, calibrated by transponder `[S]` S-001-16/20 | 5, software-coherent via noise source `[S]` S-011-4 | 1 | 1 |
| Bearing | yes, 4-element aperture of our choice (164 mm square: HPBW 60°, unambiguous ±87°) `[D]` T15 | yes, 5-element UCA (r = 92 mm at s = 0.33; vendor ≈ 8° with MUSIC at s = 0.5) `[D]` T15 | no | no (RSSI only) |
| NF | ≈ 1.2 dB without pre-filter, ≈ 3.2 dB with a 2 dB SAW `[D]` T18 | R820T2 class ≈ 3.5 dB `[C]` | ≈ 3.5 dB `[C]` | ≈ 6 dB class (sensitivity −148 dBm best) `[S]` S-003-3; Semtech page |
| Bits / dynamic range | 8 bit `[S]` | 8 bit `[S]` | 8 bit | demodulator only |
| Data rate to host | 208 MB/s (59 % of CSI) `[D]` T14 | 24 MB/s USB 2.0 `[D]` T14 | 4.8 MB/s | bytes per frame |
| Power | from the Pi 5 V rail (tile) + FTFE ≤ 1.5 A `[D]` S-007-13 | 2.2 A own supply `[S]` | 0.3 A `[C]` | 0.2 A `[C]` |
| Software | ours (Rust); vendor GPL drivers | GPL-3.0 C + Python/numba/conda | ours via `rtlsdr-nusb` | `meshtasticd` (GPL) as a plug-in process |
| Integration | FTFE build + aperture + calibration (G05) | USB; DAQ format to read (U-011-1) | USB; hot-plug | USB/SPI; HAT pin check |
| Cost class | US$ 300–500 parts `[C]` F.05.13 | ≈ US$ 500 `[C]` | US$ 30–50 `[C]` | US$ 20–60 `[C]` |
| First data | after kit (≥ 2026-11-30) + FTFE bench | days after purchase | days | days |

### B. "A phased array for LoRa itself"

- Receive: both C3 and C12 are receive arrays at 915 MHz; neither transmits. Under Part 15 a transmit array at 915 MHz buys nothing (EIRP capped at 36 dBm, G02 F.02.4); under Part 97 it is lawful and is G04 C4, deferred until C3 is proven. `[D]` (G02, G04)
- Therefore "the tile in addition to a phased array for LoRa" means, physically: the tile (5.8 GHz native, RF-vision layer; or the FTFE back end) plus a 915 MHz receive aperture, and a transmit path that is an SX1262 node with a directional antenna (C9). The 915 MHz aperture can be the FTFE's own 4-element array (164 mm) or a KrakenSDR's UCA; both are "a phased array for LoRa" in the receive sense. `[D]`

### C. Wider spectrum, with a custom addition

- Native coverage of the sensor set: tile 4.9–6.0 GHz; RTL-SDR 24–1766 MHz; HackRF 1 MHz–6 GHz in 20 MHz sweeps; FTFE 902–928 MHz into the tile. A composite `Spectrum` layer (SPEC-009) stitches them with per-sensor calibration offsets; the gap 1766–4900 MHz is covered only by HackRF sweeps. `[D]`
- The custom addition the user mentioned is, in this frame, either a second FTFE channel plan (another LO moves another 26 MHz window into the tile, e.g. 2400–2426 MHz with LO 3100 MHz) or a HackRF plug-in; the former keeps the tile's four coherent channels, the latter is one channel but cheap. `[D]`

### D. The parallel feed, as a test

SPEC-009 S-009-13: the tile (5 GHz layer) and a 915 MHz coherent receiver
publish concurrently; the overlay draws both aligned to ≤ 100 ms in time and
≤ 2° in azimuth after mounting transforms against one surveyed emitter that
radiates at 915 MHz (LoRa node) and at 5.8 GHz (second tile or CW source).
This is G07 draft test T-10. `[D]`

## V. Discussion

The FTFE path is the only one that satisfies the charter's "see every slot
with a bearing" on one capture, and its unknowns are mechanical (connector,
reference export), not physical. The KrakenSDR is the fastest way to a
bearing but its 2.4 MHz slices, software coherence and GPL Python stack make
it a comparator and a fallback, not the design. A dongle costs little and
exercises the entire software chain (plug-in, bus, occupancy, decoder,
overlay) months before the kit; that is its value. The LO-spur issue found
in LR-006 applies to the FTFE path only and is handled at the bench.

What would change the recommendation: a measured FTFE phase stability worse
than 15° over temperature (then the KrakenSDR's per-boot recalibration is
the better engineering), or a decision to operate under Part 97 (then C4
makes a transmit array worth building and the FTFE grows a transmit side).

## VI. Recommendations and best practice

1. Buy one RTL-SDR dongle and one USB SX1262 stick now; build the plug-in chain against them (backlog F-03, F-04).
2. Build the FTFE single channel as G05 recommends; defer any KrakenSDR purchase until G05 criterion (3) is measured (H-04, H-05, H-07).
3. Treat every sensor as a plug-in from day one; the overlay never knows which hardware produced a `Bearing`.
4. Write T-10 into G07 and run it the day two sensors exist.

## VII. References

G02, G04, G05, G07; SPEC-001 rev 2, SPEC-003, SPEC-004 rev 2, SPEC-007, SPEC-009, SPEC-011; `analysis/linkbudget.py` T14, T15, T16, T18, T20; `vendor/summary/krakenrf.md`, `vendor/summary/semtech.md`.
