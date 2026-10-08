# Analog Devices (Maxim) — MAX2871 datasheet 19-7106 Rev 4 (6/2020): restatement

Attribution: `vendor/analog-devices/ATTRIBUTION.md`. Source:
`resources/datasheets/MAX2871.pdf` (30 pages; text in `MAX2871.txt`),
imported from the UTM share 2026-10-08, sha256
`00234bdfef4efdf34aba6974bc56a1a14d5cb1bd83549c4a5e2f6695e782ec29`. Page
numbers are the datasheet's. All facts `[S]`; typical values at 3.3 V,
25 °C, 50 MHz reference, 25 MHz PFD, 6000 MHz output unless stated. This
is the candidate translator LO for SPEC-007 (S-007-9).

## Device (p. 1)

- Fractional/integer-N PLL with integrated VCOs (3000–6000 MHz fundamental) and output dividers 1–128 giving 23.5 MHz–6000 MHz; dual differential outputs programmable −4 to +5 dBm; 4-wire SPI (1.8 V logic); output phase reset and adjustment "allow synchronization of multiple synthesizers"; on-chip temperature sensor with 7-bit ADC for VCO selection; cycle-slip reduction and fast lock; 5 × 5 mm 32-pin TQFN; −40 to +85 °C; pin- and software-compatible with the MAX2870.

## DC (p. 2)

| Parameter | Value |
| --- | --- |
| Supply voltage | 3.0–3.6 V (3.3 V typ) |
| Supply current, both outputs at maximum power | 165 mA typ, 200 mA max; VCO + RF 122 mA; sleep 1 mA |
| RFOUT current per output | 9 mA (min power) to 25 mA (max power) |

## Reference and PFD (p. 2)

| Parameter | Value |
| --- | --- |
| REF_IN frequency range | 10–210 MHz |
| REF_IN sensitivity | 0.7 Vp-p to VCC |
| Phase-detector frequency | ≤ 140 MHz integer-N; ≤ 125 MHz fractional-N |
| Charge-pump current | 0.32–5.12 mA (RSET 5.1 kΩ) |

## RF outputs and noise (p. 3)

| Parameter | Value |
| --- | --- |
| Fundamental VCO range | 3000–6000 MHz; divided 23.4375–6000 MHz |
| VCO sensitivity / pushing / pulling | 100 MHz/V; 0.8 MHz/V; 70 kHz into 2:1 VSWR |
| Harmonics (fundamental output) | 2nd −40 dBc, 3rd −34 dBc |
| Output power | +5 dBm max, −4 dBm min at 3000 MHz; variation 1 dB over temperature, 0.2 dB over supply; muted −40 dBm |
| VCO phase noise, open loop, VCO at 4500 MHz | −77 (10 kHz), −106 (100 kHz), −132 (1 MHz), −147 (5 MHz) dBc/Hz |
| VCO phase noise, VCO at 6000 MHz | −71, −101, −128, −144 dBc/Hz at the same offsets |
| Normalised in-band noise floor | −230 dBc/Hz (Note 6: 200 kHz offset, 2 MHz loop, OCVCXO reference) |
| Normalised 1/f noise | −122 dBc/Hz (Note 7: in-band contribution = 1/f + 10log(10 kHz/f_offset) + 20log(f_RF/1 GHz)) |
| In-band phase noise (Note 8: 2113.5 MHz output, N = 169, 25 MHz PFD, 40 kHz loop, integer mode) | −102 dBc/Hz at 10 kHz |
| Integrated rms jitter (Note 9: 4400 MHz, 50 MHz PFD, 65 kHz loop) | 0.2 ps |
| Spurious signals due to PFD frequency | −88 dBc (50 kHz loop bandwidth) |
| VCO tune voltage | 0.5 V to VCC − 0.5 V |

## Consequences used in this project (`analysis/linkbudget.py` T22)

- At 4585 MHz on the fundamental VCO with the tile's 40 MHz reference as PFD in fractional mode (N = 114.625) the in-band floor is ≈ −113 dBc/Hz; integer-N alternatives (4580 MHz with a 20 MHz PFD, N = 229; 4585 MHz with a 5 MHz PFD, N = 917) avoid fractional spurs at a few dB of floor penalty. `[D]`
- The translator LO's integrated phase noise is of order −55 dBc (0.1–0.2° rms) against the tile LO's −35 dBc, and its PFD spurs (−88 dBc) are 46 dB below the tile's fractional spurs (−42 dBc); the FTFE's spurious budget (SPEC-007 S-007-3, ≤ −60 dBc) is therefore set by the mixer and filters, not by this synthesiser. `[D]`
- The multi-synthesiser phase-reset feature is not needed for the FTFE, whose four mixers share one LO through a divider (SPEC-007 S-007-7). `[D]`
