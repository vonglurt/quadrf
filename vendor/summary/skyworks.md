# Skyworks Solutions — SKY65404-31 LNA datasheet (201512K, 2015-11-06) and SE5004L-EK1 evaluation-kit datasheet (202643A, 2012-12-11): restatement

Attribution: `vendor/skyworks/ATTRIBUTION.md`. Sources imported from the UTM
share 2026-10-08: `resources/datasheets/SKY65404-31.pdf` (9 pages, sha256
`1f7fa39cd8e626225cb35d8f1559362319d1f08a43491f352faa42f00d41b649`) and
`resources/datasheets/SE5004L.pdf` (5 pages, the SE5004L-EK1 *evaluation
kit* data sheet DST-00317, not the device data sheet DST-00316; sha256
`9865f8cf179dea79ab4ef504773f6d7c2fa531eca3c1bab47ed8218649c37f5c`). All
facts `[S]`.

## SKY65404-31 — 5 GHz low-noise amplifier (pp. 1–3)

| Parameter (VCC 3.0 V, 25 °C) | Min | Typ | Max | Unit |
| --- | --- | --- | --- | --- |
| RF frequency range | 4900 | | 5900 | MHz |
| Gain S21 | 11 | 13 | 16 | dB |
| Noise figure | 0.8 | 1.0 | 1.5 | dB |
| Input IP3 | +5 | +7 | +9 | dBm |
| In-band input 1 dB compression | −5 | −4 | −2 | dBm |
| Out-of-band (2.45 GHz) input 1 dB compression | −7 | −3 | −2 | dBm |
| Input/output return loss | | −10 | −6 | dB |
| Reverse isolation S12 | −26 | −20 | | dB |
| Drain current (V_ENABLE 3 V) | 10 | 11 | 15 | mA |
| Disabled: gain / enable current | −25…−15 dB / 1.7–1.9 µA | | | |
| Supply range | 2.8 | 3.0 | 5.0 | V |
| Maximum input power | +1 dBm enabled, +10 dBm disabled | | | |

Package 6-pin QFN 1.5 × 1.5 mm; enable/disable pin; four external components.

## SE5004L — 5 GHz WLAN power amplifier, from the evaluation-kit sheet (pp. 1, 4)

- "High output power amplifier — 26 dBm at 5 V"; "3 % EVM @ 26 dBm, 64 QAM, 54 Mbps"; "32 dB gain"; integrated input, inter-stage and output matching, a power detector with 15 dB dynamic range and a 3.8 GHz notch filter; enabled by 2.85 V on VREF, shut down with VREF at ground; 20-pin 4 × 4 × 0.9 mm QFN; the kit's suggested starting input power is −20 dBm.
- The evaluation-kit sheet gives no saturated output power, no P1dB and no supply-current figure; those are in the device data sheet DST-00316, which is not in `resources/`.

## Consequences used in this project (`analysis/linkbudget.py` T18, T21)

- Receive chain: LNA typ (NF 1.0 dB, 13 dB) ahead of the MAX2851 (4.5 dB) gives 1.30 dB system NF with nothing ahead and 1.80 dB with a 0.5 dB switch ahead; the vendor's "≈ 1.2 dB" is reached only near the LNA's best-case corner (0.8 dB, 16 dB → 0.96 dB). `[D]`
- Antenna-referred compression of the receive chain is set by the MAX2851, not the LNA, at every gain setting above max − 32 dB: −47 dBm at maximum RF gain, −31 dBm at max − 16 dB, −14 dBm at max − 32 dB (then the LNA's −4 dBm applies). `[D]`
- Transmit: the vendor's "1 W per antenna" BOM wording describes the SE5004L's class; the document in hand supports 26 dBm (0.4 W) linear per element at 5 V, 1.6 W (32 dBm) aggregate linear for four; saturated power is unknown (SPEC-001 U-001-7). `[D]`+`[C]`
