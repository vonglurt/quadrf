# G04 — Inventory: ways to put LoRa on or through the QuadRF

| Field | Value |
| --- | --- |
| Status | PASS |
| Opened | 2026-10-08 |
| Closed | 2026-10-08 |
| Author | project |
| Inherits from | G03 |

## 1. Question

What are all the architectures by which the tile can take part in LoRa work,
and which survive the G01 physics and G02 rules?

## 2. Inherited facts

- F.01.3 (no native 915 MHz), F.01.4 (band fits in 40 MHz), F.01.6/F.01.7 (geometry, phase control), F.01.8 (polarisation), F.01.9 (4 W aggregate), F.01.11 (element ports reachable), F.02.1–F.02.12 (rules), F.03.2 (5.8 GHz LoRa PHY exists), F.03.6 (RF-vision two-constant patch).

## 3. Method

Enumerate every placement of the tile relative to a LoRa signal (frequency,
direction, role), state each candidate's critical assumption, and apply a
kill criterion from the inherited facts. Survivors receive a capability
statement and a cost estimate.

## 4. Findings: the inventory

| Id | Candidate | Critical assumption | Kill criterion | Result |
| --- | --- | --- | --- | --- |
| C1 | Tune the tile to 915 MHz by software | Tuning range reaches 915 MHz | F.01.1, F.01.3 | Dead |
| C2 | Harmonic or sub-harmonic reception of 915 MHz at a 5 GHz LO | Front end passes 915 MHz | F.01.2: patch, switch, LNA and IC reject it | Dead |
| C3 | Rx-only frequency-translating front end: 915 MHz antenna → LNA → mixer (common LO ≈ 4585 MHz) → 5500 MHz → element port, ×4, external 915 MHz aperture | Coherent translation preserves inter-element phase; element ports accessible | F.01.11 (ports) `[C]` until bench; phase preservation is `[D]` for a common LO | **Alive — G05** |
| C4 | Full-duplex translation (C3 plus Tx down-converter and 915 MHz PA per element) | As C3 plus Tx coherence and PA linearity under §15.247(e) PSD | F.02.4: EIRP capped at 36 dBm unlicensed; Part 97 only makes sense | Alive, deferred (Part 97 only; after C3 proven) |
| C5 | Tile as native 5.8 GHz LoRa transceiver (quadrf-mesh) | 500 kHz presets lawful; link closes | F.02.9 lawful; F.03.2 exists | **Alive — G06 backhaul** |
| C6 | Tile pair as fixed point-to-point 5.8 GHz LoRa link with unlimited antenna gain | Fixed P2P definition met | F.02.7, F.02.8 | **Alive — G06** |
| C7 | 802.11 OFDM TUN link between tiles at 5.8 GHz carrying Meshtastic over IP (MQTT or TCP proxy) | Same as C6; existing vendor demo | F.02.7 (hop must be P2P; OFDM 20 MHz is ≥ 500 kHz) | Alive, alternative to C6 with higher throughput |
| C8 | Host-side 4-channel beamforming/DoA at 915 MHz through C3 (interleaved streams, our own weights) | C3 | as C3 | Alive — part of G05 (B.iii) |
| C9 | 915 MHz Tx via SX1262 HAT + Yagi, Rx via C3 beamformer (asymmetric node) | HAT pins avoid JTAG; Rx aperture ≠ Tx aperture is acceptable | F.03.4 | Alive — G06 recommendation |
| C10 | LoRa waveform at 5.8 GHz from the tile to consumer LoRa devices | Consumer radios at 5.8 GHz exist | No LoRa silicon above 2.5 GHz (SX1280 is 2.4 GHz) `[S]` common knowledge, verify | Dead for consumer nodes; alive tile-to-tile only (C5) |
| C11 | Use the tile's 240-element MoonRF form for EME-class EIRP at 915 MHz | Hardware at 915 MHz | F.01.3 | Dead (and out of scope F.00.5) |
| C12 | Off-the-shelf coherent 915 MHz receiver (e.g. a 5-channel RTL-based coherent SDR) instead of C3 for Rx-only DoA | Cheaper, native band | Not a tile use; benchmark only | Comparator for G05 (`[C]` specs to verify) |

Capability statements for survivors:

- F.04.1 C3 yields a four-channel coherent 902–928 MHz receiver with front-end NF set by the 915 MHz LNA (≈ 1 dB) plus a small cascade term, full-band 26 MHz capture, per-element phase control, DoA in the external aperture's geometry, and the vendor calibration method at 915 MHz. It makes "see every Meshtastic slot in the band with a bearing" physically possible on one tile. `[D]` (F.01.4, F.01.5, F.01.7, F.01.12; cascade NF derived in G05)
- F.04.2 C5/C6 yield a LoRa-modulated 5.8 GHz point-to-point link lawful at 1 W aggregate into any gain; tile-to-tile margin at 40 km LOS is 47.6 dB at LONG_FAST with 12 dBi apertures, rising by 12.6 dB per 72-element far end. `[D]` (analysis T7)
- F.04.3 C9 is the only lawful unlicensed way to combine the tile's spatial selectivity with 915 MHz Meshtastic participation, because Tx EIRP is rule-capped and need not come from the array. `[D]` (F.02.4)

## 5. Analysis

The inventory collapses to two live lines: (I) the tile at 915 MHz as a
coherent receive back end behind a translation front end (C3, C8, C9), and
(II) the tile at 5.8 GHz as a directional LoRa or OFDM backhaul between two
tiles (C5, C6, C7). Line I answers the user's "tune into LoRa / map on
screen / LoRa phased array (receive)" questions. Line II answers "focused,
highly directional relay at legal power". Transmit beamforming at 915 MHz
(C4) is lawful only under Part 97 and is deferred until C3 is measured.

## 6. Answer

Seven candidates survive; they reduce to two architectures. The modification
that creates a (receive) LoRa phased array is C3: a four-way coherent
frequency translator plus an external 915 MHz aperture. The lawful
high-directivity relay is C6 at 5.8 GHz, or C9 at 915 MHz under Part 97.

## 7. Recommendation (opinion)

Proceed to G05 on C3 with C12 as a cost/performance comparator. Carry C5/C6/C9
to G06. Do not pursue C4 before a C3 bench result exists.

## 8. Gate record

| Field | Value |
| --- | --- |
| Pass criterion | Every candidate has a stated critical assumption and a kill criterion traceable to inherited facts; survivors have a quantitative capability statement. |
| Evidence | Table in §4, F.04.1–F.04.3 |
| Decision | PASS |
| Date / signatory | 2026-10-08 / project |
| Open conjectures carried as risk | C3 port accessibility (U-001-1), C10 "no LoRa silicon above 2.5 GHz" to verify, C12 comparator specs to verify |

## 9. Carry-forward

- C3, C5, C6, C7, C8, C9, C12; F.04.1–F.04.3.
