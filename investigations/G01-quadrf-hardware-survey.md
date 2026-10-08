# G01 — QuadRF hardware survey

| Field | Value |
| --- | --- |
| Status | PASS |
| Opened | 2026-10-08 |
| Closed | 2026-10-08 |
| Author | project |
| Inherits from | G00 |

## 1. Question

What does the QuadRF tile do, from primary sources, in the quantities that
decide LoRa applicability: tuning range, bandwidth, dynamic range, element
geometry, coherence, interfaces?

## 2. Inherited facts

- F.00.1 (physical frame), F.00.3 (one kit, desk-only until 2026-11-30).

## 3. Method

Read the vendor schematic export (10 pages), the vendor documentation page,
the vendor calibration notes, and the open software repository, in particular
the tuning guard in the JTAG utility, the SoapySDR driver limits, the device
tree and OpenOCD GPIO map, and the demo constant for element pitch. Record
each fact into SPEC-001 and SPEC-002 as behaviour statements. Compute the
geometry and quantisation consequences in `analysis/linkbudget.py`.

## 4. Findings

- F.01.1 Tuning range is 4900–6000 MHz, enforced by two independent software guards and bounded by the converter ICs' 4.9–5.9 GHz design range. `[S]` (SPEC-001 S-001-1, S-001-2)
- F.01.2 No component in the receive chain (patch element, SPDT polarisation switch, 5 GHz LNA, 4.9–5.9 GHz receiver IC) is specified to pass 902–928 MHz; the patch element is resonant at ≈ 5.5 GHz and is electrically ≈ 0.05 λ at 915 MHz. `[S]`+`[D]` (BOM; a circular patch on FR4 at 5.5 GHz is ≈ 15 mm across; 15 mm / 328 mm = 0.046 λ)
- F.01.3 Therefore: 902–928 MHz energy cannot be received or transmitted by the unmodified tile at any useful sensitivity. Native LoRa-band operation is excluded by the physics of the front end, not by firmware. `[D]` (F.01.1, F.01.2)
- F.01.4 The 26 MHz US LoRa band fits inside one element's 40 MHz instantaneous bandwidth, and four interleaved 8-bit channels at 26 MSPS need 1.66 Gbit/s of the 5.6 Gbit/s link. A single fixed-LO translation would expose all 104 Meshtastic slots to the coherent 4-channel receiver at once. `[D]` (SPEC-001 S-001-5, S-001-23)
- F.01.5 The 8-bit converters cost ≤ 0.1 dB of noise figure when gain is set so thermal noise is ≥ 2 LSB rms; the AGC set-point range −40…−6 dBFS makes that setting reachable. The penalty of 8 bits is dynamic range (≈ 42 dB between quantisation floor and full scale per sample, more after processing gain), not sensitivity. `[D]` (SPEC-001 S-001-7, S-001-8)
- F.01.6 Element pitch 45.5 mm = 0.880 λ at 5800 MHz: 2-element HPBW 33°, grating lobe on steering beyond 7.8°, unambiguous DoA ±34.6°. The tile is a direction-finding and modest-gain instrument in its native band, not a narrow-beam one; narrow beams come from chaining tiles (72-element: +18.6 dB array gain; 240-element: +23.8 dB). `[D]` (analysis T9)
- F.01.7 Per-element phase is host-settable in degrees, raw per-element streams are available, and steering depends only on k·d. Any 4-element geometry at any wavelength can be presented to the FPGA beamformer by scaling coordinates, or bypassed entirely by beamforming on the host from interleaved streams. `[S]`+`[D]` (SPEC-001 S-001-16 … S-001-18)
- F.01.8 Transmit is RHCP only; receive is RHCP or LHCP. Linear-polarised LoRa antennas (whips, Yagis) incur a 3 dB polarisation mismatch to a circular aperture; a single specular reflection reverses handedness, so a circularly polarised receiver rejects first-order multipath by its axial-ratio-limited cross-polar discrimination. `[S]`+`[D]` (SPEC-001 S-001-10; polarisation algebra)
- F.01.9 Aggregate conducted transmit capability is 4 W (36 dBm); Part 15 digital-modulation limits are stated as a sum across elements (1 W), so a tile transmitting under §15.247 must be driven at ≤ 1/4 of its PA rating in aggregate. `[S]`+`[D]` (SPEC-001 S-001-9; SPEC-005 S-005-4)
- F.01.10 JTAG occupies GPIO 14, 15, 18, 23 (22 optional) on the Pi 5 header; SPI0 (GPIO 7–11) is free, UART0 (14/15) is not, and GPIO 18 is not. `[S]` (SPEC-001 S-001-22)
- F.01.11 The antenna module is a separate PCB on a board-to-board RF connector (footprint `BWCD-L5.0W2.0H2.5`); the element ports ANT1–ANT4 are therefore physically reachable without modifying the RF board. Connector type, impedance and loss are unknown. `[S]`+`[C]` (SPEC-001 S-001-12, U-001-1)
- F.01.12 The vendor's array-calibration method solves per-element position and static phase from a moving transponder without assuming a lattice; it is wavelength-agnostic in form. `[S]` (SPEC-001 S-001-20)

## 5. Analysis

The decisive chain is F.01.1 → F.01.2 → F.01.3. Every remaining LoRa-at-915
option must insert a frequency translation between 915 MHz antennas and the
element ports (F.01.11), and must supply a 915 MHz aperture of appropriate
pitch (F.01.6 scaled: 164 mm for λ/2, 288 mm to preserve the tile's d/λ).
The tile's contributions that survive translation are: four coherent
receivers with 1.2 dB NF behind the translator (F.01.5), host-settable
per-element phase (F.01.7), 40 MHz capture covering the whole band (F.01.4),
and a calibration method that transfers (F.01.12).

In its native band the tile is a complete LoRa-capable SDR: F.01.9 bounds the
Part 15 power, F.01.8 fixes polarisation, F.01.6 bounds the beam.

## 6. Answer

The tile is a 4.9–6.0 GHz, 40 MHz, 8-bit, four-element coherent transceiver
with a 0.88 λ pitch and open host-side phase control. It cannot touch
902–928 MHz unmodified. It can be used at 915 MHz only as a coherent 4-channel
back end behind a translation front end and an external aperture, and it can
be used at 5.8 GHz directly.

## 7. Recommendation (opinion)

Accept F.01.3 as settled; do not spend bench time attempting to tune the tile
below 4.9 GHz. Carry the translation path to G04/G05 and the native 5.8 GHz
path to G04/G06.

## 8. Gate record

| Field | Value |
| --- | --- |
| Pass criterion | Tuning range, bandwidth, bit depth, element pitch, polarisation, PA rating, GPIO map and antenna-port accessibility are each established from a primary source or a derivation, not from marketing text. |
| Evidence | F.01.1–F.01.12 with SPEC-001 pointers |
| Decision | PASS |
| Date / signatory | 2026-10-08 / project |
| Open conjectures carried as risk | U-001-1 (connector), U-001-2 (reference export), U-001-3 (NF in translation window), U-001-5 (factory bitstream behaviour with translated band) — all closed at G05 bench |

## 9. Carry-forward

- F.01.1, F.01.3, F.01.4, F.01.5, F.01.6, F.01.7, F.01.8, F.01.9, F.01.10, F.01.11, F.01.12.

## 10. Errata and addenda (2026-10-08, second pass)

The gate decision is unchanged. New primary sources (the MAX2851 datasheet
supplied by the user, the RP1 datasheet, the vendor driver sources and
licence files) add the following; see LR-006 and SPEC-001 revision 2.

- F.01.13 The receiver IC's RF input range is 4.9–5.9 GHz, its LO is coherent among its channels, its DSB noise figure is 4.5 dB at maximum gain, its synthesiser has −35 dBc integrated phase noise and −42 dBc spurs at 0–19 MHz offsets, and its baseband high-pass corner is selectable among 600 kHz, 10 kHz and 0.1 kHz. F.01.1–F.01.3 therefore rest on the datasheet, not on product pages. `[S]` (SPEC-001 S-001-26…30)
- F.01.14 The vendor's 1.2 dB system noise figure requires the external LNA to have NF ≤ 1 dB and gain ≥ 13.5 dB with ≤ 0.5 dB of loss ahead of it; the −42 dBc spurs exceed the 8-bit SFDR and will produce deterministic replicas of strong in-band emitters. `[D]` (analysis T17, T18; carried to G05 F.05.14)
- F.01.15 The CSI-2 link carries 4 lanes at 700 Mbit/s (2.8 Gbit/s raw); four interleaved channels at 26 MSPS use 59 % of it and arrive as 131 072-byte frames every 630 µs; the driver buffers 16 frames (10 ms). The RP1 provides 8 Gbit/s across its two MIPI PHYs over a PCIe 2.0 x4 link. `[S]`+`[D]` (SPEC-001 S-001-34; analysis T14)
- F.01.16 The FPGA part is LFE5U-25F-6BG256C in the schematic but the OpenOCD tap is named lfe5u45f; which is fitted is unknown until the IDCODE is read. `[C]` (SPEC-001 S-001-35; U-001-6; backlog H-01)
- F.01.17 Erratum to U-001-2: the "SiT8008 MEMS" identification of the 40 MHz reference came from the vendor BOM page; the schematic text contains no such designator. The reference's identity is `[C]` until the schematic page is read visually; the 40 MHz value itself is datasheet-backed. `[C]`+`[S]` (schematics.txt grep; SPEC-001 S-001-26)
- F.01.18 Licences: kernel modules GPL-2.0; SoapySDR module, CLI, GUI and demos GPL-2.0/3.0; antenna files CC-BY-SA-4.0 with a patent covenant; bitstream proprietary but redistributable. `[S]` (vendor/scalerf/ATTRIBUTION.md)

Carry-forward additions: F.01.13, F.01.14, F.01.15 to G05; F.01.18 to the
design track (SPEC-008, SPEC-010).

Later the same day (second batch of datasheets, LR-006 addendum):

- F.01.19 The transmitter IC delivers at most −4 dBm per channel of linear OFDM (≈ +7 dBm at 1 dB compression) with coherent LO among its four channels, −40 dBc sideband and −29 dBc carrier leakage; its synthesiser matches the receiver's (−35 dBc, −42 dBc spurs, 76.294 Hz). `[S]` (SPEC-001 S-001-37)
- F.01.20 The receive LNA is NF 1.0 dB / 13 dB gain typical with −4 dBm input P1dB; the datasheet-typical system NF is 1.30 dB (1.80 dB with 0.5 dB of switch loss ahead), and the vendor's ≈ 1.2 dB is reached only near the LNA's best-case corner; antenna-referred compression is −47 dBm at maximum RF gain and −14 dBm at max − 32 dB. `[S]`+`[D]` (SPEC-001 S-001-38, S-001-41; analysis T18, T21)
- F.01.21 Erratum to F.01.9: the PA document in hand supports 26 dBm (0.4 W) linear per element, 1.6 W (32 dBm) aggregate linear; "1 W per antenna" is the vendor's BOM wording and the saturated rating is unknown (U-001-7). A §15.247 transmitter must still be driven below the hardware's capability; the factor becomes 1/1.6 instead of 1/4. `[S]`+`[C]`+`[D]` (SPEC-001 S-001-39/40)
- F.01.22 Erratum to F.01.16: the vendor BOM page names the FPGA LFE5U-45F-7BG256C, matching the OpenOCD tap; the schematic's LFE5U-25F is the outlier; U-001-6 stays open until the IDCODE is read. `[S]`+`[C]` (SPEC-001 S-001-42)
