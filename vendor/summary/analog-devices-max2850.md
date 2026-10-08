# Analog Devices (Maxim) — MAX2850 datasheet 19-5009 Rev 2 (1/2019): restatement

Attribution: `vendor/analog-devices/ATTRIBUTION.md`. Source:
`resources/datasheets/MAX2850.pdf` (33 pages; text in `MAX2850.txt`),
imported from the UTM share 2026-10-08, sha256
`10c7cff652d24a3cf7583026c5189f8b83ec5bda6df2ff07f393217a19d57a32`. Page
numbers are the datasheet's. All facts `[S]`; typical values at VCC =
2.85 V, 25 °C, LO 5.35 GHz, 40 MHz reference, 40 MHz channel unless stated.

## Device (p. 1)

- Single-chip 4-channel RF transmitter for 5 GHz wireless HDMI with one 5 GHz OFDM reverse-link receiver; the up-conversion LO is coherent among all transmit channels; integrated VCO, fractional-N synthesiser (76 Hz step), crystal oscillator, programmable 20/40 MHz Tx anti-alias filters, Tx/Rx I/Q error and LO-leakage detection and adjustment, dynamic on/off control of four external PAs with programmable precision voltages, a 4-to-1 analog mux for PA power-detect voltages, 4-wire SPI, temperature sensor; 68-pin thin QFN 10 × 10 mm; 2.7–3.6 V; −25 to +85 °C; part MAX2850ITK+.

## Transmitter, baseband I/Q to RF (pp. 5–6; includes matching and balun loss)

| Parameter | Value |
| --- | --- |
| RF output frequency range | 4.9–5.9 GHz |
| Gain variation over the band, one temperature | 3 dB typ, 6.4 dB max peak-to-peak |
| Maximum output power | −4 dBm (20 MHz or 40 MHz OFDM meeting the spectral mask and −34 dB EVM) |
| Output 1 dB gain compression | +11 dBc relative to the typical maximum output power (9.5 MHz input) |
| Input 1 dB gain compression | 380 mVrms at 19 MHz input |
| Gain-control range / step | 31.5 dB typ (26–34.5) / 0.5 dB |
| RF output return loss | −3 dB |
| Unwanted sideband | −40 dBc |
| Carrier leakage | −29 dBc typ, −15 dBc max |
| Tx I/Q input impedance | ≥ 60 kΩ differential, ≤ 2 pF |
| Baseband filter stop-band rejection | 86 dB at 30 MHz (20 MHz channel); 67 dB at 60 MHz (40 MHz channel) |
| Tx calibration gain range | 35 dB |

## Frequency synthesiser (pp. 6–7)

Same architecture and figures as the MAX2851: 4.9–5.9 GHz, 76.294 Hz step,
integrated phase noise −35 dBc (1 kHz–10 MHz, 200 kHz loop), charge pump
0.8 mA, spurs −42 dBc (0–19 MHz offset) and −66 dBc (40 MHz), 40 MHz
reference, 800 mVp-p AC-coupled reference input, crystal tuning 30 pF in
140 fF steps, CLKOUT.

## Receiver (reverse link, pp. 3–5)

One receive channel with 4.5 dB noise figure, 70 dB gain range in 2 dB
steps, 60 dB RSSI dynamic range, 40 dB sideband suppression; not used by the
tile's receive path (the MAX2851 provides the four receive channels).

## Consequences used in this project

- The transmit chain per element is MAX2850 (−4 dBm linear OFDM; ≈ +7 dBm output P1dB) into the SE5004L (32 dB gain, 26 dBm linear at 5 V); the IC's output is enough to drive the PA to its linear rating with a few dB of pad (SPEC-001 rev 3 S-001-37/39). `[D]`
- Transmit and receive LOs are separate synthesisers of the same design in two ICs; their relative drift is what `quadrf-mesh` measured as ≈ 800 Hz rms between two tiles (SPEC-006 S-006-6), and the −35 dBc / −42 dBc figures apply to each. `[D]`
