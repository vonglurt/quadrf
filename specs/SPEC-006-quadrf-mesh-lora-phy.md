# SPEC-006 — `quadrf-mesh`: LoRa-compatible PHY and Meshtastic daemon on the QuadRF

| Field | Value |
| --- | --- |
| Status | Reviewed |
| Revision | 1 |
| Date | 2026-10-08 |
| Subject | The third-party (radioroy) software that modulates and demodulates LoRa-compatible chirps through the QuadRF SDR and bridges them to a Meshtastic daemon |
| Primary sources | `resources/repos/quadrf-mesh/README.md`, `docs/air-ipc-v1.md`, `docs/mesh-monitor.md`, `debian/quadrf-lora-phy.default`, `include/phy/lora/*.hpp` |
| Depends on | SPEC-001, SPEC-002, SPEC-003, SPEC-004 |

## Behaviour-goal statements

- S-006-1. Two processes: `quadrf-lora-phy` owns the radio (SoapySDR `mipi` + `quadrf-jtag`), performs chirp modulation/demodulation and framing, and serves a Unix socket `/run/quadrf/phy.sock`; `quadrf-meshtasticd` is a Meshtastic portduino build with an out-of-tree `RadioInterface` (`QuadRFRadio`) speaking that socket, with web UI on 9443 and phone API on 4403. `[S]` (README; air-ipc-v1.md)
- S-006-2. Air-IPC v1 frames: 6-byte header (magic `QF`, version 1, type, big-endian length); types TxEnqueue (centre frequency ignored, payload = Meshtastic air frame), RxIndicate (SNR, RSSI, CFO Hz, sample-rate ppm, centre frequency, SIR, level dBFS, payload), SetModem (preset id 0–9). `[S]` (air-ipc-v1.md)
- S-006-3. All ten Meshtastic presets are implemented with Meshtastic framing conventions; unsupported presets revert to SHORT_TURBO. `[S]` (README table)
- S-006-4. Startup RF is set in `/etc/default/quadrf-lora-phy`: centre frequency (default 5800 MHz), Tx gain 25, Rx gain 45, amplitude 0.7, Tx/Rx antenna masks default 1 (single element, chosen to avoid multipath self-cancellation), low-IF receive with LO 500 kHz below the channel, host rate 8 MSPS with a NEON DDC to 2·BW, Rx baseband filter 4 MHz, Tx 20 MHz. `[S]` (quadrf-lora-phy.default)
- S-006-5. The PHY follows live LO changes made in the vendor GUI or by `quadrf-jtag` (TX LO = channel, RX LO = channel − IF); Meshtastic's `override_frequency` is inert. `[S]` (README; air-ipc-v1.md)
- S-006-6. Vendor-measured performance, two tiles at 5.8 GHz, Tx gain 0: SHORT_TURBO … LONG_FAST and LONG_TURBO deliver 85–100 % of beacons; LONG_MODERATE 60–100 %; LONG_SLOW and VERY_LONG_SLOW decode headers but fail payload CRC. Cause: relative LO wander ≈ 800 Hz rms (±1 kHz over 10–30 ms). Stated fix: a shared reference or a quieter LO. `[S]` (README "Measured between two QuadRFs")
- S-006-7. Reported SNR is post-dechirp SNR in the LoRa bandwidth, accurate to ±0.5 dB from the demodulation cliff to ≈ +15 dB; LO phase noise counts as noise in that figure. `[S]` (mesh-monitor.md)
- S-006-8. The receiver's DDC path improves the SHORT_TURBO 10 % PER threshold from −2.6 to −6.4 dB S/(N₀·BW) relative to the zero-IF path; the ideal SF7 threshold is −7.5 dB, so implementation loss is ≈ 1.1 dB. `[S]`+`[D]` (quadrf-lora-phy.default comment; SPEC-003 S-003-2)
- S-006-9. A "parrot" repeater mode re-broadcasts received texts with hop limit 0 and appends call sign and RF metrics; loop protection and rate limiting are included. `[S]` (mesh-monitor.md)
- S-006-10. The PHY is GPLv3, adapts bit-level chains from `gr-lora_sdr` and `gr-lora`, and declares itself not an official Scale RF or Meshtastic product. `[S]` (README licence section)

## Interfaces we depend on

- Air-IPC v1 on `/run/quadrf/phy.sock` (either side is replaceable by our own code).
- `/etc/default/quadrf-lora-phy` keys `QUADRF_LORA_PHY_{MODE,FREQ,PRESET,TX_GAIN,RX_GAIN,AMPLITUDE,TX_ANT,RX_ANT,RX_RATE,RX_IF_KHZ,LO_FOLLOW,RX_BW,TX_BW,PA_DRAIN_MS}`.

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-006-1 | Whether the PHY accepts a centre frequency it cannot tune (e.g. 906.875) and what it does: needed to drive it through a translated front end where the tile LO ≠ channel | G05 |
| U-006-2 | Whether beamformed Rx (mask 15 with phases) improves PER versus a single element in an outdoor link | G06 bench |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 1 | 2026-10-08 | First reviewed version |
