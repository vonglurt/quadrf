# SPEC-001 — QuadRF RF tile: observable behaviour

| Field | Value |
| --- | --- |
| Status | Reviewed |
| Revision | 1 |
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

## Interfaces we depend on

- Element RF port: the board-to-board connector between RF board and antenna module (footprint `BWCD-L5.0W2.0H2.5`). Impedance and mating part are not stated; see U-001-1.
- SoapySDR driver `mipi`: args `antennas`, `p1..p4`, `agc_setpoint`, `interleave`; `setFrequency`, `setSampleRate`, `setGain`.
- CLI `quadrf-jtag --rx/--tx key=value,...` with keys `freq` (MHz), `bw`, `gain`, `agc`, `antennas`, `autosteer`, `interleave`, `tone_en`, `tone_freq`.

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-001-1 | Element-port connector type, impedance, insertion loss, and whether a pigtail can replace the antenna module | G05 bench |
| U-001-2 | Whether the 40 MHz reference (SiT8008 MEMS) is accessible for export to an external LO | G05 bench |
| U-001-3 | Actual receive noise figure and gain flatness across 5490–5530 MHz (the 915 MHz translation window) | G05 bench |
| U-001-4 | Element pattern and gain of the circular patch at 5.5 and 5.8 GHz (OpenEMS model exists; not simulated by us) | G05 or G06 desk |
| U-001-5 | Behaviour of the factory bitstream's auto-steer and RF-vision maths when fed a translated band | G05 bench |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 1 | 2026-10-08 | First reviewed version |
