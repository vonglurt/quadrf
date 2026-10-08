# SPEC-001 — QuadRF RF tile: observable behaviour

| Field | Value |
| --- | --- |
| Status | Reviewed (rev 1); rev 2 additions S-001-26…36 await a second read |
| Revision | 2 |
| Date | 2026-10-08 |
| Subject | One ScaleRF QuadRF RF tile (RF board + 4-element antenna module + ECP5 FPGA) as seen from its antenna ports, its Pi 5 interfaces, and its control registers |
| Primary sources | `resources/scalerf/QuadRF_schematics.pdf`, `resources/scalerf/docs.html`, `resources/repos/quadrf-open-space-sdr/` (README.md, sources/fpga/jtag_src/jtag.c, sources/soapy/MipiDevice.cpp, sources/fpga/interface/rpi5_ecp5_gpio.cfg, sources/demos/csi_sweep.c), `resources/scalerf/cal_antennas.html` |
| Depends on | — |

## Scope

The tile as a black box with four RF element ports, a MIPI CSI-2 receive
stream, a MIPI DSI transmit stream, a bit-banged JTAG control path, and a
daisy-chain FFC interface. The proprietary RF-core layout and the factory FPGA
bitstream are outside scope; only their externally observable behaviour is
stated.

## Behaviour-goal statements

### Frequency and bandwidth

- S-001-1. The tile tunes its transmit and receive local oscillators independently within 4900–6000 MHz; the control utility rejects any carrier outside that interval. `[S]` (jtag.c lines 254–255; SoapySDR `getFrequencyRange` returns one range bounded by `kLoMinHz`/`kLoMaxHz`, MipiDevice.cpp 529–532)
- S-001-2. Up- and down-conversion are performed by a 4-channel 4.9–5.9 GHz transmitter IC and a multi-channel 4.9–5.9 GHz receiver IC designed for wireless HDMI; the vendor states PLL lock is not guaranteed outside 4.9–6.0 GHz. `[S]` (BOM in docs.html: MAX2850, MAX2851; ADI product pages; docs.html note on out-of-band operation)
- S-001-3. Each element path has an instantaneous bandwidth of 40 MHz. `[S]` (docs.html, README.md)
- S-001-4. The host sample rate is settable from 1 MSPS upward; vendor documentation states 1–80 MSPS, the driver advertises 1–90 MSPS in one direction and 1–160 MSPS in the other. `[S]` (docs.html; MipiDevice.cpp 521–527)
- S-001-5. The entire US 902–928 MHz band (26 MHz) is narrower than one element's 40 MHz instantaneous bandwidth. `[D]` (26 < 40)

### Converters and dynamic range

- S-001-6. Samples are 8-bit I and 8-bit Q per element (CS8 on the host), with vendor-stated 7-bit ENOB in beamformed mode; daisy-chained receive sums may be widened to 12+12 or 16+16 bits. `[S]` (docs.html)
- S-001-7. With thermal noise set by receive gain to ≥ 2 LSB rms, the 8-bit quantiser adds ≤ 0.09 dB to the noise floor; at 1 LSB rms it adds 0.35 dB. `[D]` (analysis T13)
- S-001-8. Receive gain is settable over an index range 0–63 with an AGC set-point selectable between −40 and −6 dBFS; the vendor states a 70 dB gain range and ≈ 1.2 dB receive noise figure. `[S]` (MipiDevice.cpp 534–536, 314; docs.html)

### Transmit

- S-001-9. Each of the four elements is driven by its own SiGe power amplifier rated 1 W; the aggregate conducted power of a tile is up to 4 W (36 dBm) when all four are driven at rating. `[S]`+`[D]` (BOM: 4× SE5004L; sum)
- S-001-10. Transmit polarisation is right-hand circular only; receive polarisation is switch-selected RHCP or LHCP, one at a time. `[S]` (README.md; BOM lists 8 RF SPDT switches)
- S-001-11. A startup digital calibration corrects transmit quadrature error and LO leakage. `[S]` (docs.html; cal_txqec.html)

### Antenna module

- S-001-12. The antenna module carries four circular patch elements on FR4 at a centre-to-centre pitch of 45.5 mm and is a separate, swappable PCB mated to the RF board. `[S]` (README.md "swappable circular patch antenna modules"; `ANTENNA_SPACING_MM 45.5f` in csi_sweep.c; hardware/QuadRF_Antenna/ and hardware/Patch/Patch_B/ library files incl. a board-to-board RF connector footprint `BWCD-L5.0W2.0H2.5`)
- S-001-13. The schematic names four element ports ANT1–ANT4, each with two pads, consistent with dual-feed circular polarisation per element. `[S]` (schematics.txt designators ANT101/102 … ANT401/402)
- S-001-14. At 5800 MHz the pitch is 0.880 λ. A two-element broadside pair at that pitch has a 33.0° half-power beamwidth, admits a grating lobe into visible space when steered beyond 7.8°, and yields an unambiguous phase-difference direction estimate only within ±34.6° of broadside; at 4900 MHz the figures are 39.3°, 20.2°, ±42.2°. `[D]` (analysis T9)
- S-001-15. The patch element pattern confines the useful field of view to the forward hemisphere; the vendor sweeps the LO across 4.9–6.0 GHz in the "RF camera" mode, and the phase-to-angle scale in the demo code is a function of LO frequency. `[S]`+`[C]` (demo description "swept-LO phase scatter"; `SCALE_FACTOR_AT_MHZ(f)` in csi_sweep.c; whether the sweep is used to resolve the S-001-14 ambiguity is our reading, to be confirmed in G05)

### Beamforming and coherence

- S-001-16. The FPGA applies a per-element phase rotation and sums the four receive paths into one I/Q stream (default), or interleaves the four raw element streams into one stream for host de-multiplexing. `[S]` (docs.html; SoapySDR args `p1..p4`, `antennas` mask, `interleave`, `autosteer` in jtag.c and MipiDevice.cpp)
- S-001-17. Per-element phase is settable in degrees over 0–360 via SoapySDR device arguments; an automatic mode steers toward the strongest source at the current LO. `[S]` (MipiDevice.cpp 310–313, 327–330; `autosteer=1`)
- S-001-18. Steering phase for a far-field source at angle θ is φ_n = k·d_n·sin θ with k = 2π/λ; it depends only on the product k·d. A change of operating wavelength is therefore representable to the beamformer as a proportional change of element coordinates. `[D]` (array theory; basis for G05)
- S-001-19. Vendor-stated inter-element synchronisation jitter is ≈ 1.4 ps and RF-to-software latency < 10 ms. `[S]` (docs.html)
- S-001-20. A tile's elements can be located and phase-calibrated by tracking a moving transponder with an extended Kalman filter; the vendor method assumes a ≈ 5.6 GHz carrier and 1 kHz phase sampling but no lattice, so it is wavelength-agnostic in form. `[S]` (cal_antennas.html)

### Host interfaces

- S-001-21. Receive I/Q reaches the Pi 5 over the CSI-2 camera connector, transmit I/Q leaves over the DSI display connector, each a 4-lane differential link; the vendor states 5.6 Gbit/s aggregate and both run concurrently for full duplex. `[S]` (docs.html; Geerling 2026)
- S-001-22. Control and bitstream loading use a bit-banged JTAG on Pi 5 GPIO 14 (TCK), 15 (TMS), 18 (TDO), 23 (TDI), optional 22 (TRST), all on `gpiochip4`. `[S]` (rpi5_ecp5_gpio.cfg; fpga-csi.dts lines 28–31)
- S-001-23. Four interleaved 8+8-bit channels at 26 MSPS require 208 MB/s (1.66 Gbit/s), below the stated link capacity. `[D]` (4 × 26e6 × 2 B)
- S-001-24. Tiles daisy-chain over FFC: transmit samples pass down the chain with per-tile integer/fractional delay; receive sums cascade upward; each tile recovers the upstream clock with a digital PLL. `[S]` (docs.html; chained.svg)
- S-001-25. The tile is powered at 5 V from the Pi supply (5–17 V accepted in array builds); tile plus antenna ≈ 35 g; kit enclosure ≈ 15 × 11 × 4 cm. `[S]` (docs.html; README.md)

### Converter IC facts from the MAX2851 datasheet (revision 2)

- S-001-26. The receiver IC's RF input range is 4.9–5.9 GHz, its down-conversion LO is coherent among its receive channels, it takes a 40 MHz reference, and its fractional-N synthesiser steps by 76.294 Hz. `[S]` (resources/datasheets/MAX2851.pdf pp. 1, 3, 7; vendor/summary/analog-devices-max2851.md)
- S-001-27. The receiver IC's DSB noise figure is 4.5 dB at maximum RF gain and 15 dB at maximum − 16 dB; its total voltage gain spans ≈ −2 to 68 dB in RF steps of 8/16/32/40 dB plus 30 dB of baseband gain in 2 dB steps; gain settles within 400 ns (RF) and 200 ns (baseband). `[S]` (MAX2851.pdf pp. 3–4)
- S-001-28. Linearity at the IC input: 1 dB compression −34 dBm at maximum gain rising to −1 dBm at maximum − 32 dB; out-of-band IIP3 −13 to +11 dBm over the same settings; 1 dB desensitisation by an alternate-channel blocker at −24 dBm. `[S]` (MAX2851.pdf p. 4)
- S-001-29. Baseband filtering in the IC: low-pass −3 dB corner selectable 9.5 MHz or 19 MHz with stop-band rejection 74 dB at 30 MHz / 69 dB at 60 MHz; high-pass corner selectable 600 kHz, 10 kHz or 0.1 kHz; I/Q gain and phase imbalance 0.1 dB and 0.2°; sideband suppression 40 dB. `[S]` (MAX2851.pdf p. 5)
- S-001-30. Synthesiser quality: integrated phase noise −35 dBc (1 kHz–10 MHz, 200 kHz loop bandwidth); spur level −42 dBc for 0–19 MHz offsets and −66 dBc at 40 MHz; receiver LO leakage emission −75 dBm/MHz. `[S]` (MAX2851.pdf pp. 5, 7)
- S-001-31. The IC's gain varies ≤ 4.2 dB peak-to-peak (1.8 dB typical) over 4.9–5.9 GHz at one temperature, which bounds the flatness term of U-001-3: a 26 MHz window is 2.6 % of that span. `[S]`+`[D]` (MAX2851.pdf p. 3)
- S-001-32. Consequences: −35 dBc integrated phase noise is 1.44° rms of LO phase; the −42 dBc spurs exceed the 8-bit single-tone SFDR (≈ 49.9 dB), so a strong in-band emitter produces replicas 42 dB down at deterministic offsets; the vendor's 1.2 dB system NF is consistent only with an external LNA of NF ≤ 1 dB, gain ≥ 13.5 dB and ≤ 0.5 dB of loss ahead of it. `[D]` (analysis T17, T18)

### Receive bandwidth, data path and platform (revision 2)

- S-001-33. The analog receive bandwidth is programmed as 240/k MHz with integer k from 5 to 63, i.e. 3.81–48 MHz; the transmit bandwidth is 20 or 40 MHz. `[S]` (jtag.c lines 231–240, 671–675; MipiDevice.cpp lines 186–187)
- S-001-34. The CSI-2 link runs 4 data lanes at 700 Mbit/s (a 350 MHz DDR source) carrying RAW8 frames of 1024 bytes × 128 lines, 2.8 Gbit/s raw per direction; the RP1 provides 8 Gbit/s across its two 4-lane MIPI PHYs and reaches the BCM2712 over PCIe 2.0 x4. Four interleaved channels at 26 MSPS use 59 % of the CSI raw rate and arrive as one 131 072-byte frame every 630 µs; the driver's 16 DMA buffers hold 10 ms. `[S]`+`[D]` (fpga-csi.dts; resources/datasheets/RP1-peripherals.pdf ch. 1; fpga-csi.c line 204; analysis T14)
- S-001-35. The FPGA part number is LFE5U-25F-6BG256C in the schematic, while the vendor's OpenOCD board file and JTAG tap are named lfe5u45f; which device is fitted is unknown until the IDCODE is read. `[S]`+`[C]` (schematics.txt; sources/fpga/quadrf-load; U-001-6)
- S-001-36. Licences: the kernel modules are GPL-2.0; the SoapySDR module, CLI, GUI and demos are GPL-2.0/GPL-3.0; antenna design files are CC-BY-SA-4.0 with a patent covenant; the FPGA bitstream is proprietary but redistributable; the RF core is all rights reserved. `[S]` (debian/copyright; updates.html FAQ; licenses/LICENSE.md; vendor/scalerf/ATTRIBUTION.md)

## Interfaces we depend on

- Element RF port: the board-to-board connector between RF board and antenna module (footprint `BWCD-L5.0W2.0H2.5`). Impedance and mating part are not stated; see U-001-1.
- SoapySDR driver `mipi`: args `antennas`, `p1..p4`, `agc_setpoint`, `interleave`; `setFrequency`, `setSampleRate`, `setGain`.
- CLI `quadrf-jtag --rx/--tx key=value,...` with keys `freq` (MHz), `bw`, `gain`, `agc`, `antennas`, `autosteer`, `interleave`, `tone_en`, `tone_freq`.

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-001-1 | Element-port connector type, impedance, insertion loss, and whether a pigtail can replace the antenna module | G05 bench |
| U-001-2 | Whether the 40 MHz reference is accessible for export to an external LO; its identity (a MEMS SiT8008 per the vendor BOM page, not seen in the schematic text; G01 F.01.17) | G05 bench |
| U-001-3 | Actual receive noise figure and gain flatness across 5490–5530 MHz (the 915 MHz translation window) | G05 bench |
| U-001-4 | Element pattern and gain of the circular patch at 5.5 and 5.8 GHz (OpenEMS model exists; not simulated by us) | G05 or G06 desk |
| U-001-5 | Behaviour of the factory bitstream's auto-steer and RF-vision maths when fed a translated band | G05 bench |
| U-001-6 | Which ECP5 device is fitted (LFE5U-25F per schematic vs lfe5u45f per OpenOCD tap name); read IDCODE | bench, first power-up |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 1 | 2026-10-08 | First reviewed version |
| 2 | 2026-10-08 | S-001-26…36 from the MAX2851 datasheet (supplied by the user), the vendor driver sources, the RP1 datasheet and the licence files; U-001-3 bounded by S-001-31; U-001-6 added |
