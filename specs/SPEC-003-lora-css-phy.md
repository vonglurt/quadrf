# SPEC-003 — LoRa chirp-spread-spectrum physical layer, as used by Meshtastic

| Field | Value |
| --- | --- |
| Status | Reviewed |
| Revision | 1 |
| Date | 2026-10-08 |
| Subject | The LoRa CSS waveform and demodulator behaviour relevant to link budgets and to SDR implementation |
| Primary sources | `resources/lora/lora-phy-paper-tapparel.pdf`, `resources/lora/gr-lora_sdr-README.md`, `resources/lora/meshtastic-radio-settings.html`, `resources/repos/quadrf-mesh/README.md` |
| Depends on | — |

## Behaviour-goal statements

- S-003-1. A LoRa symbol is a linear frequency chirp of duration T_s = 2^SF / BW sweeping BW, cyclically shifted by one of 2^SF start frequencies; demodulation multiplies by the conjugate base chirp and takes a 2^SF-point DFT; the bin spacing is BW / 2^SF. `[S]` (Tapparel et al. 2020)
- S-003-2. Demodulator SNR thresholds (CR 4/5): SF7 −7.5 dB, SF8 −10, SF9 −12.5, SF10 −15, SF11 −17.5, SF12 −20, referenced to noise in BW. `[S]` (Semtech SX126x datasheet values as reproduced in the Meshtastic link-budget table; vendor PDF not retrieved, see manifest)
- S-003-3. Sensitivity follows P_min = −174 dBm/Hz + 10·log10(BW) + NF + SNR_min. For a 6 dB NF receiver: SHORT_TURBO (500 k, SF7) −118.5 dBm; LONG_FAST (250 k, SF11) −131.5 dBm; LONG_SLOW (125 k, SF12) −137.0 dBm. With a 4.2 dB effective NF (1.2 dB front end + 3 dB implementation) each improves by 1.8 dB. `[D]` (analysis T1)
- S-003-4. The occupied bandwidth of a chirp equals BW to first order; a 250 kHz preset has a 6 dB bandwidth of ≈ 250 kHz and a 500 kHz preset ≈ 500 kHz. `[D]` (chirp sweep width; exact 6 dB width to be measured, U-003-1)
- S-003-5. Uniform sweep spreads power evenly over BW: at 30 dBm into a 500 kHz chirp the PSD is 30 − 10·log10(500/3) = 7.8 dBm per 3 kHz. `[D]`
- S-003-6. Meshtastic presets used in North America: SHORT_TURBO 500 kHz/SF7/4-5, SHORT_FAST 250/7, SHORT_SLOW 250/8, MEDIUM_FAST 250/9, MEDIUM_SLOW 250/10, LONG_TURBO 500/11/4-8, LONG_FAST 250/11/4-5 (default), LONG_MODERATE 125/11/4-8, LONG_SLOW 125/12/4-8 (deprecated). `[S]` (meshtastic-radio-settings.html; quadrf-mesh README preset table)
- S-003-7. Meshtastic frames use sync word 0x2B, 16-symbol preamble, explicit header, CRC on, and low-data-rate optimisation when T_s ≥ 16.4 ms. `[S]` (quadrf-mesh README)
- S-003-8. The US LONG_FAST channel plan has 104 slots of 250 kHz across 902–928 MHz; the default hashed slot is 20, centred at 906.875 MHz. `[S]` (meshtastic-radio-settings.html)
- S-003-9. Carrier wander of 800 Hz rms within a symbol equals 0.2 DFT bins at SHORT_TURBO, 6.6 bins at LONG_FAST and 26 bins at LONG_SLOW; tracking across symbol boundaries is required for SF ≥ 9 on an LO of that quality, and SF12 at ≤ 125 kHz fails without a better reference. `[D]`+`[S]` (analysis T12; quadrf-mesh README measurement)
- S-003-10. An open GNU Radio LoRa transceiver (`gr-lora_sdr`, GPLv3) implements SF 5–12, CR 0–4, selectable BW, sync word, explicit/implicit header, CRC, LDRO, and operates at low SNR; it is hardware-agnostic over SoapySDR/UHD. `[S]` (gr-lora_sdr README)
- S-003-11. Phase-steering (narrowband) beamforming is valid for LoRa because the aperture transit time (≤ 3.3 ns for a 1 m aperture) is < 10⁻³ of the inverse bandwidth (2 µs at 500 kHz). `[D]` (analysis T10)

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-003-1 | Measured 6 dB and 20 dB bandwidths of 250/500 kHz Meshtastic emissions from an SX1262 | G02 measurement (spectrum analyser) |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 1 | 2026-10-08 | First reviewed version |
