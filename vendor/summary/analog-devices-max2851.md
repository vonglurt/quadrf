# Analog Devices (Maxim) — MAX2851 datasheet 19-5121 Rev 1 (3/2010): restatement

Attribution: `vendor/analog-devices/ATTRIBUTION.md`. Source:
`resources/datasheets/MAX2851.pdf` (37 pages; text in `MAX2851.txt`). Page
numbers are the datasheet's. All facts `[S]`; typical values at VCC = 2.85 V,
25 °C, LO 5.35 GHz, 40 MHz reference, 40 MHz channel unless stated.

## Device

- Single-chip 5-channel RF receiver for 5 GHz wireless HDMI with one 5 GHz OFDM reverse-link transmitter; integrated VCO, fractional-N synthesiser (76.294 Hz step), crystal oscillator, I/Q baseband filters (20/40 MHz channels), RF and in-channel RSSI, temperature sensor, 4-wire SPI; the down-conversion LO is coherent among all receive channels; 68-pin TQFN 10 × 10 mm; 2.7–3.6 V; −25 to +85 °C; part MAX2851ITK+. (pp. 1, 7)

## Receiver, RF input to I/Q baseband (p. 3–5)

| Parameter | Value |
| --- | --- |
| RF input frequency range | 4.9–5.9 GHz |
| Gain variation over 4.9–5.9 GHz, one temperature | 1.8 dB typ, 4.2 dB max peak-to-peak |
| RF input return loss | −6 dB (all LNA settings) |
| Total voltage gain, max / min setting | 61.8 (min) – 68 (typ) dB / −2 (typ) – +6.9 (max) dB |
| RF gain steps relative to max | −8, −16, −32, −40 dB |
| Baseband gain range / step | 28–32 dB (30 typ) / 2 dB |
| Gain settling (RF / baseband) | 400 ns / 200 ns to ±0.5 dB |
| DSB noise figure, balun-input referred, max RF gain | 4.5 dB (20 MHz and 40 MHz bandwidths) |
| DSB noise figure at max RF gain − 16 dB | 15 dB |
| Out-of-band input IP3 | −13 dBm at max gain; −5 dBm at max − 16 dB; +11 dBm at max − 32 dB |
| 1 dB desensitisation by alternate-channel blocker | −24 dBm (±40 MHz for 20 MHz channel; ±80 MHz for 40 MHz channel) |
| Input 1 dB compression | −34 dBm (max gain), −25 (max − 8), −18 (max − 16), −1 (max − 32) |
| Output 1 dB compression | 0.63 Vp-p |
| Baseband low-pass −3 dB corner | 9.5 MHz or 19 MHz (register selectable) |
| Baseband stop-band rejection | 74 dB at 30 MHz (20 MHz channel); 69 dB at 60 MHz (40 MHz channel) |
| Baseband high-pass −3 dB corner | 600 kHz, 10 kHz or 0.1 kHz (register selectable) |
| Steady-state I/Q DC error (AC-coupled, after RXHP toggle) | 2 mV, 1-sigma |
| I/Q gain / phase imbalance | 0.1 dB / 0.2° (1-sigma) |
| Sideband suppression | 40 dB |
| Receiver spurious emissions | LO −75, 2×LO −62, 3×LO −75, 4×LO −54 dBm/MHz |
| RF RSSI output | 1.6 V at −25 dBm input; baseband RSSI slope 26.5 mV/dB (18–37) |
| RF loopback conversion gain | −10 dB typ |

## Frequency synthesiser (p. 7)

| Parameter | Value |
| --- | --- |
| RF channel centre frequency | 4.9–5.9 GHz |
| Programming step | 76.294 Hz |
| Closed-loop integrated phase noise, 1 kHz–10 MHz, loop BW 200 kHz | −35 dBc |
| Charge-pump current | 0.8 mA |
| Spur level | −42 dBc at 0–19 MHz offset; −66 dBc at 40 MHz offset |
| Reference frequency | 40 MHz; reference input 800 mVp-p AC-coupled to XTAL |
| Crystal | motional resistance ≤ 50 Ω; capacitance tuning range 30 pF in 140 fF steps |
| Clock outputs | CLKOUT (VCC − 0.8 … VCC − 0.1 Vp-p into 10 pF), CLKOUT2 (0.3 Vp-p into 4 pF) |

## Transmitter (p. 1, 6)

- 4.9–5.9 GHz output, −5 dBm typical for 54 Mbit/s OFDM, 31 dB gain control in 0.5 dB steps, programmable 20/40 MHz anti-alias filters, Tx/Rx I/Q error and LO leakage detection and adjustment, PA on/off control, analog mux for PA power detect.

## Consequences used in this project (`analysis/linkbudget.py`)

- Integrated phase noise −35 dBc → 1.44° rms (T17). The −42 dBc fractional spurs lie above the 8-bit converter's single-tone SFDR (≈ 49.9 dB); a strong in-band emitter therefore produces LO-spur replicas 42 dB down that can mask weak LoRa slots at deterministic offsets (T17). `[D]`
- The 4.5 dB device NF and the vendor's 1.2 dB system NF are consistent only with an external LNA of NF ≤ 1 dB and gain ≥ 13.5 dB with ≤ 0.5 dB of loss ahead of it (T18). `[D]`
- The 0.1 kHz high-pass setting (or a low-IF offset as `quadrf-mesh` uses) is needed to keep narrowband chirps away from the baseband DC servo; `quadrf-mesh`'s 500 kHz low-IF choice is explained by the 600 kHz/10 kHz/0.1 kHz corner options and the RXHP DC-servo behaviour. `[D]`
