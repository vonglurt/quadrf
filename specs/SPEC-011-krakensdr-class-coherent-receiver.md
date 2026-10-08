# SPEC-011 — KrakenSDR-class five-channel coherent receiver (C12 comparator and candidate USB sensor)

| Field | Value |
| --- | --- |
| Status | Draft |
| Revision | 0 |
| Date | 2026-10-08 |
| Subject | A five-channel coherent RTL2832U/R820T2 receiver with a shared clock and a switched noise source, as a comparator for the FTFE-fed tile (G04 C12, G05) and as a candidate 915 MHz sensor plugin on the feed bus (SPEC-009) |
| Primary sources | `resources/krakensdr/wiki-Home.md`, `resources/krakensdr/wiki-04-Antenna-Array-Setup.md`, `resources/krakensdr/heimdall_daq_fw-README.md`, `resources/krakensdr/krakensdr_doa-README.md` (retrieved 2026-10-08); attribution `vendor/krakenrf/ATTRIBUTION.md`; restatement `vendor/summary/krakenrf.md` |
| Depends on | SPEC-003, SPEC-009 |

## Scope

The receiver as seen from its five SMA ports, its two USB-C ports and the
host software's published behaviour. Not covered: its internal firmware, the
Android application, passive-radar modes.

## Behaviour-goal statements

Hardware:

- S-011-1. The receiver has five receive channels, each an R820T2 tuner feeding an RTL2832U 8-bit ADC; it has no transmitter. `[S]` (wiki-Home.md lines 49–54, 133)
- S-011-2. Each channel tunes 24–1766 MHz with a maximum channel bandwidth of 2.56 MHz; the vendor's stated oscillator stability is 1 ppm. `[S]` (wiki-Home.md lines 52–55)
- S-011-3. All five tuners share one clock source; a switched wideband noise source and an internal USB hub are built in. `[S]` (wiki-Home.md lines 101–105)
- S-011-4. Coherence is established in software: the noise source is switched into all inputs, each channel is cross-correlated against the master channel (CH0), and the resulting sample-timing and phase offsets are corrected in software; the vendor states the hardware alone is not coherent. `[S]` (wiki-Home.md lines 107–109)
- S-011-5. Power is supplied through a dedicated USB-C port from a 5 V supply rated ≥ 2.4 A (2.2 A nominal draw); the separate USB-C data port carries no power and is to be used with a cable ≤ 1 m. `[S]` (wiki-Home.md lines 63–71)
- S-011-6. The practical sustained rate per channel is slightly below 2.56 MSPS (≈ 2.4 MSPS) because of the coherence resampler. `[C]` (vendor forum via search; measure if the device is acquired)

Array geometry (vendor rules):

- S-011-7. Inter-element spacing shall be s·λ with s ≤ 0.5 (ambiguity above), s ≥ 0.2 (accuracy below), typically 0.33; a uniform circular array of n elements has radius r = s·λ / √(2(1 − cos(360°/n))); a uniform linear array resolves 180° only with aperture (n − 1)·s·λ. `[S]` (wiki-04-Antenna-Array-Setup.md lines 12, 52–67, 71–79)
- S-011-8. Cables shall be length-matched to about 1 cm; a 1 cm mismatch is ≈ 14° of phase at 800 MHz and 7° at 400 MHz. `[S]` (wiki-04 line 90)
- S-011-9. The vendor's resolution estimate is Rayleigh 1.22 λ/D improved by about 10× with MUSIC: ≈ 8.3° for a 5-element UCA at s = 0.5 and ≈ 3.4° for a 5-element ULA. `[S]` (wiki-04 lines 150–158)

Software:

- S-011-10. The DAQ chain (`heimdall_daq_fw`) is a C core built with `make` plus a conda Python environment (numba, scipy) using ZeroMQ and Python shared memory; it is GPL-3.0, "tested on the Raspberry Pi 4", and ready-made images exist for the Raspberry Pi 4/5 and Orange Pi 5B. `[S]` (heimdall_daq_fw-README.md lines 4, 11, 42, 161–167, 257)
- S-011-11. The DoA application (`krakensdr_doa`) is GPL-3.0, serves a web UI on port 8080, its settings file on 8081 and a JSON middleware API on 8042, and recommends cooling and ≥ 2000 MHz on a Pi 4; numba JIT warm-up takes 1–2 minutes. `[S]` (krakensdr_doa-README.md)

Derived, for this project:

- S-011-12. At 915 MHz a 5-element UCA at s = 0.33 has 108.1 mm spacing, 92.0 mm radius and 184 mm aperture; at s = 0.5, 163.8 mm, 139.4 mm and 279 mm (Rayleigh 82°, vendor 83°). `[D]` (analysis T15)
- S-011-13. One 2.4 MHz capture covers 9–10 of the 104 US LONG_FAST slots; a whole-band occupancy map needs 11 retunes, each a recalibration risk, whereas the FTFE-fed tile sees all 104 slots in one 26 MHz capture. `[D]` (26 / 2.4 = 10.8; SPEC-001 S-001-5)
- S-011-14. Its five channels at 2.4 MSPS produce 192 Mbit/s (40 % of one USB 2.0 link); its 2.2 A draw exceeds the Pi 5's 1.6 A USB peripheral budget, so it must be powered separately. `[D]` (analysis T14, T20)
- S-011-15. Its receive sensitivity is set by the R820T2 noise figure (≈ 3.5 dB class, unverified) against the FTFE path's ≈ 1.2–3.2 dB (analysis T18); its 8-bit dynamic range equals the tile's. `[C]`+`[D]`

## Interfaces we depend on

- Published DAQ output (shared memory / ZeroMQ) format — to be read from the source tree before any plugin is written (U-011-1).
- Settings API on port 8042 (JSON GET/POST).

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-011-1 | DAQ output frame format and rate as published by heimdall | backlog R-12 (read the source) |
| U-011-2 | Phase stability between noise-source recalibrations at 915 MHz | measurement if acquired (backlog H-07) |
| U-011-3 | Licence of the wiki documentation | ask vendor; until then treated as all rights reserved |
| U-011-4 | R820T2 noise figure at 915 MHz | datasheet or measurement |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 0 | 2026-10-08 | Draft from vendor wiki and README files |
