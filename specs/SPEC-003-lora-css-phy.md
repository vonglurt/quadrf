# SPEC-003 — LoRa chirp-spread-spectrum physical layer, as used by Meshtastic

| Field | Value |
| --- | --- |
| Status | Reviewed (rev 1); rev 2 additions S-003-12…15 await a second read |
| Revision | 2 |
| Date | 2026-10-08 |
| Subject | The LoRa CSS waveform and demodulator behaviour relevant to link budgets and to SDR implementation |
| Primary sources | `resources/lora/lora-phy-paper-tapparel.pdf`, `resources/lora/gr-lora_sdr-README.md`, `resources/lora/meshtastic-radio-settings.html`, `resources/repos/quadrf-mesh/README.md`, `resources/datasheets/SX1261-2.pdf` (rev 2) |
| Depends on | — |

## Behaviour-goal statements

- S-003-1. A LoRa symbol is a linear frequency chirp of duration T_s = 2^SF / BW sweeping BW, cyclically shifted by one of 2^SF start frequencies; demodulation multiplies by the conjugate base chirp and takes a 2^SF-point DFT; the bin spacing is BW / 2^SF. `[S]` (Tapparel et al. 2020)
- S-003-2. Typical demodulator SNR thresholds: SF5 −2.5 dB, SF6 −5, SF7 −7.5, SF8 −10, SF9 −12.5, SF10 −15, SF11 −17.5, SF12 −20, referenced to noise in BW. `[S]` (resources/datasheets/SX1261-2.pdf, DS.SX1261-2.W.APP Rev 1.1, Table 6-1 p. 38; retagged from the Meshtastic table in rev 2)
- S-003-3. Sensitivity follows P_min = −174 dBm/Hz + 10·log10(BW) + NF + SNR_min. For a 6 dB NF receiver: SHORT_TURBO (500 k, SF7) −118.5 dBm; LONG_FAST (250 k, SF11) −131.5 dBm; LONG_SLOW (125 k, SF12) −137.0 dBm. With a 4.2 dB effective NF (1.2 dB front end + 3 dB implementation) each improves by 1.8 dB. `[D]` (analysis T1)
- S-003-4. The occupied bandwidth of a chirp equals BW to first order; a 250 kHz preset has a 6 dB bandwidth of ≈ 250 kHz and a 500 kHz preset ≈ 500 kHz. `[D]` (chirp sweep width; exact 6 dB width to be measured, U-003-1)
- S-003-5. Uniform sweep spreads power evenly over BW: at 30 dBm into a 500 kHz chirp the PSD is 30 − 10·log10(500/3) = 7.8 dBm per 3 kHz. `[D]`
- S-003-6. Meshtastic presets used in North America: SHORT_TURBO 500 kHz/SF7/4-5, SHORT_FAST 250/7, SHORT_SLOW 250/8, MEDIUM_FAST 250/9, MEDIUM_SLOW 250/10, LONG_TURBO 500/11/4-8, LONG_FAST 250/11/4-5 (default), LONG_MODERATE 125/11/4-8, LONG_SLOW 125/12/4-8 (deprecated). `[S]` (meshtastic-radio-settings.html; quadrf-mesh README preset table)
- S-003-7. Meshtastic frames use sync word 0x2B, 16-symbol preamble, explicit header, CRC on, and low-data-rate optimisation when T_s ≥ 16.4 ms. `[S]` (quadrf-mesh README)
- S-003-8. The US LONG_FAST channel plan has 104 slots of 250 kHz across 902–928 MHz; the default hashed slot is 20, centred at 906.875 MHz. `[S]` (meshtastic-radio-settings.html)
- S-003-9. Carrier wander of 800 Hz rms within a symbol equals 0.2 DFT bins at SHORT_TURBO, 6.6 bins at LONG_FAST and 26 bins at LONG_SLOW; tracking across symbol boundaries is required for SF ≥ 9 on an LO of that quality, and SF12 at ≤ 125 kHz fails without a better reference. `[D]`+`[S]` (analysis T12; quadrf-mesh README measurement)
- S-003-10. An open GNU Radio LoRa transceiver (`gr-lora_sdr`, GPLv3) implements SF 5–12, CR 0–4, selectable BW, sync word, explicit/implicit header, CRC, LDRO, and operates at low SNR; it is hardware-agnostic over SoapySDR/UHD. `[S]` (gr-lora_sdr README)
- S-003-11. Phase-steering (narrowband) beamforming is valid for LoRa because the aperture transit time (≤ 3.3 ns for a 1 m aperture) is < 10⁻³ of the inverse bandwidth (2 µs at 500 kHz). `[D]` (analysis T10)

## Revision 2 additions (SX1261/2 datasheet)

- S-003-12. The SX1261/2 LoRa modem offers spreading factors 5–12 (SF5 and SF6 are not backward-compatible with the SX127x) and signal bandwidths 7.81, 10.42, 15.63, 20.83, 31.25, 41.67, 62.5, 125, 250 and 500 kHz (double-sideband); up to 250 kHz the receiver uses a low-IF double conversion, at 500 kHz a single zero-IF conversion; coding rates 4/5…4/8; symbol rate BW / 2^SF; the signal is constant-envelope. `[S]` (SX1261-2.pdf §6.1.1, Tables 6-1…6-3, pp. 37–39)
- S-003-13. Datasheet sensitivities (Rx boosted gain, split paths, switch loss excluded): 125 kHz SF7 −124 dBm, SF12 −137; 250 kHz SF7 −121, SF12 −134; 500 kHz SF7 −117, SF12 −129; 10.4 kHz SF12 −148. The implied noise figure plus implementation loss is 6.0–8.0 dB, so the 6 dB model of S-003-3 is 0.5–2 dB optimistic for an SX1262 node; by interpolation LONG_FAST ≈ −131 dBm, SHORT_TURBO −117 dBm, LONG_TURBO ≈ −127 dBm. `[S]`+`[D]` (SX1261-2.pdf Table 3-8 p. 19; analysis T23)
- S-003-14. The receiver tolerates a transmitter–receiver frequency offset of ±25 % of the bandwidth without sensitivity loss for all spreading factors, further limited to ±50 ppm at SF12, ±100 ppm at SF11 and ±200 ppm at SF10; low-data-rate optimisation is recommended when the symbol time is ≥ 16.38 ms. A 4.6 kHz error (1 ppm free-running translator LO, G05 F.05.2) is within every preset's limit (≥ 31.25 kHz). `[S]`+`[D]` (SX1261-2.pdf Table 3-8 p. 20, §6.1.1.4 p. 39; analysis T23)
- S-003-15. SX1262 transmit power is +22 dBm at 107–118 mA (868/915 MHz; SX1261 +15 dBm at 32.5 mA), dropping 2–6 dB at supply voltages of 2.7–1.8 V; the synthesiser covers 150–960 MHz in 0.95 Hz steps with phase noise −75/−95/−100/−120/−135 dBc/Hz at 1 kHz/10 kHz/100 kHz/1 MHz/10 MHz offsets; blocking immunity at 125 kHz SF12 is 88/90/99 dB at 1/2/10 MHz, co-channel rejection 5 dB (SF7) to 19 dB (SF12), adjacent-channel rejection 60–72 dB, receiver IIP3 −5 dBm, image attenuation 54 dB with I/Q calibration. `[S]` (SX1261-2.pdf Tables 3-6, 3-7, 3-8 pp. 16–20)

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-003-1 | Measured 6 dB and 20 dB bandwidths of 250/500 kHz Meshtastic emissions from an SX1262 | G07 T-1 (spectrum analyser or calibrated dongle) |
| U-003-2 | Whether the newer SX1261/2 datasheet revisions (the copy in hand is Rev 1.1, 2017-12, a mirror copy) changed any value above | fetch the current revision from semtech.com when reachable |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 1 | 2026-10-08 | First reviewed version |
| 2 | 2026-10-08 | S-003-2 retagged to the SX1261/2 datasheet; S-003-12…15 added (SF/BW sets, sensitivities and implied NF, offset tolerance, Tx power, phase noise, blocking); U-003-2 added |
