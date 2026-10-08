# SPEC-007 — 915 MHz frequency-translating front end (FTFE): implementation specification

| Field | Value |
| --- | --- |
| Status | Draft (becomes Reviewed when G05 bench criteria (1)–(3) pass) |
| Revision | 1 |
| Date | 2026-10-08 |
| Subject | A four-channel coherent block up-converter and external aperture that presents 902–928 MHz to the QuadRF element ports as 5487–5513 MHz |
| Primary sources | G05 desk findings; SPEC-001; SPEC-003 |
| Depends on | SPEC-001, SPEC-003, SPEC-006 |

## Scope

Receive-only in revision 0. Transmit translation (C4) is a later revision
gated on Part 97 operation.

## Behaviour-goal statements

Signal plan:

- S-007-1. The FTFE shall translate each of four 915 MHz inputs to the tile band by non-inverting mixing with one common LO at 4585.000 MHz, so that 902.000–928.000 MHz maps to 5487.000–5513.000 MHz. `[D]` (G05 F.05.1)
- S-007-2. The image band 3657–3683 MHz shall be attenuated ≥ 60 dB relative to the wanted band ahead of the mixer. `[D]` (G05 F.05.1; 8-bit floor)
- S-007-3. LO leakage and all spurious products within 5470–5530 MHz at the output shall be ≤ −60 dBc relative to a −30 dBm in-band tone. `[D]` (G05 criterion 2)

Noise and gain:

- S-007-4. Each channel shall have NF ≤ 3.0 dB at 915 MHz, 290 K, measured at the antenna port, with the tile set to its minimum-NF gain. `[D]` (G05 F.05.4)
- S-007-5. Net conversion gain shall be +10 ± 2 dB so that the tile's AGC set-point range (−40…−6 dBFS) places the thermal floor at ≥ 2 LSB rms. `[D]` (SPEC-001 S-001-7, S-001-8)
- S-007-6. Input 1 dB compression shall be ≥ −25 dBm so that a co-sited node at 22 dBm and 10 m does not compress the chain; a switchable 30 dB attenuator shall be provided for closer sources. `[D]` (G05 F.05.6)

Coherence:

- S-007-7. The four LO drives shall come from one synthesiser through one 4-way divider with electrical lengths matched to ≤ 2° at 4585 MHz (≤ 0.36 mm in PTFE cable). `[D]` (G05 F.05.3)
- S-007-8. Inter-channel phase shall be stable to ≤ 5° rms over 10 min at constant temperature and ≤ 15° over 0–40 °C; static offsets are calibrated out. `[D]` (G05 criterion 3)
- S-007-9. The LO reference shall be a ≤ 1 ppm TCXO in revision 0, with a provision (SMA input) to accept the tile's 40 MHz reference if U-001-2 closes positively. `[D]` (G05 F.05.2)

Aperture:

- S-007-10. The revision 0 aperture is four vertical quarter-wave monopoles (≈ 78 mm) on a common ground plane at 164 mm pitch in a 2 × 2 square, each with an SMA feed; element positions surveyed to ≤ 2 mm. `[D]` (G05 F.05.7)
- S-007-11. Beamforming through the FPGA shall use scaled coordinates d' = d × 6.01 or direct per-element phases; host-side beamforming shall use true coordinates. `[D]` (G05 F.05.8)

Interfaces:

- S-007-12. Output to the tile: four 50 Ω cables terminating in the mating part of the element-port connector (part to be identified under U-001-1), each ≤ 1 dB loss at 5500 MHz. `[C]`
- S-007-13. Power: 5 V, ≤ 1.5 A total, from the Pi 5 supply rail or a separate regulator; no connection to Pi GPIO. `[D]` (budget: 4 × LNA 60 mA + synthesiser 300 mA + margin)
- S-007-14. Software: a one-line configuration (`f_translate = 4585e6`) consumed by the DoA/channeliser flowgraph and the RF-vision constant patch. `[D]` (G03 F.03.6)

Local oscillator and level plan (revision 1):

- S-007-15. The translator LO shall be a MAX2871-class synthesiser on its fundamental VCO (3000–6000 MHz), referenced from a 40 MHz source (its reference input accepts 10–210 MHz, so the tile's 40 MHz can drive it if U-001-2 closes), with a 40 MHz PFD in fractional mode at 4585 MHz (N = 114.625, in-band floor ≈ −113 dBc/Hz) or, to avoid fractional spurs, a 20 MHz PFD in integer mode at 4580 MHz (N = 229, floor ≈ −110 dBc/Hz, band mapped to 5482–5508 MHz); its −4 to +5 dBm output drives the 4-way divider through a buffer. `[S]`+`[D]` (resources/datasheets/MAX2871.pdf pp. 1–3; analysis T22)
- S-007-16. The translator LO's integrated phase noise (≈ −56 dBc, 0.13° rms with a 150 kHz loop and the 4500 MHz VCO curve) and PFD spurs (−88 dBc) are small against the tile LO's −35 dBc and −42 dBc; S-007-3's spurious budget is therefore set by the mixer and its filters, not by the synthesiser. `[D]` (analysis T22, T17)
- S-007-17. Level plan: with +10 dB net FTFE gain a 22 dBm node at 10 m presents −15 dBm at the element port, above the tile chain's compression at any RF gain above max − 32 dB (−14 dBm); the switchable 30 dB attenuator of S-007-6 shall be engaged for co-sited nodes within ≈ 100 m, and the tile RF gain shall be set from the measured element-port level rather than left at maximum. `[D]` (analysis T21; SPEC-001 S-001-41)

## Test points

TP1 antenna port (915 MHz), TP2 post-LNA, TP3 mixer output (before 5.5 GHz
filter), TP4 FTFE output, TP5 LO divider port. Each a 50 Ω SMA or U.FL.

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-007-1 | Element-port mating connector (= U-001-1) | G05 bench |
| U-007-2 | Whether the tile's AGC fights a fixed-gain front end at full-band capture | G05 bench |
| U-007-3 | Mixer part and its conversion loss and LO-RF isolation at 5.5 GHz (7 dB and 30 dB assumed in F.05.4 and S-007-3) | procurement (H-04) |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 0 | 2026-10-08 | Draft from G05 desk analysis |
| 1 | 2026-10-08 | S-007-15…17 (LO candidate from the MAX2871 datasheet; level plan from T21); U-007-3 added |
