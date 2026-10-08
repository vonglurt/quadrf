# G03 — Raspberry Pi software stack

| Field | Value |
| --- | --- |
| Status | PASS (desk); bench items listed |
| Opened | 2026-10-08 |
| Closed | 2026-10-08 |
| Author | project |
| Inherits from | G02 |

## 1. Question

What software do we expect to run on the Pi 5 for each LoRa use of the tile,
what already exists, and what must we write?

## 2. Inherited facts

- F.01.7 (host phase control, raw streams), F.01.10 (GPIO map), F.02.9 (500 kHz presets lawful at 5.8 GHz), F.02.11 (licensed mode).

## 3. Method

Read the vendor repository (`docs/`, `install/`, `sources/`), the third-party
`quadrf-mesh` repository, the Meshtastic firmware radio abstraction and Linux
daemon configuration, and the `gr-lora_sdr` README. Map each use case to a
process graph.

## 4. Findings

- F.03.1 Vendor baseline (SPEC-002): DietPi, OpenOCD + `quadrf-jtag` for bring-up, DKMS kernel drivers for CSI/DSI DMA, SoapySDR `mipi` module, SoapyRemote, GNU Radio with QuadRF blocks, ZeroMQ, Flask control panel, KasmVNC desktop, demo apps, and `quadrf-mesh`. `[S]`
- F.03.2 A LoRa-compatible PHY and a Meshtastic daemon with a custom radio backend already run on the tile at 5.8 GHz and are packaged into the vendor image; they communicate over a documented Unix-socket protocol (Air-IPC v1). All ten presets are implemented; 500 kHz presets and LONG_FAST deliver 85–100 % of frames tile-to-tile; SF12 at ≤ 125 kHz fails on LO wander. `[S]` (SPEC-006)
- F.03.3 The Meshtastic `RadioInterface` ABI is small enough that `quadrf-mesh` implemented an out-of-tree backend; the same seam allows us to put any PHY (e.g. a translated-915 MHz PHY, or a host-side 4-channel beamforming PHY) under an unmodified mesh stack. `[S]` (SPEC-004 S-004-1; SPEC-006 S-006-1)
- F.03.4 A conventional 915 MHz Meshtastic node on the same Pi is `meshtasticd` with an SX1262 HAT over SPI0; config via `/etc/meshtasticd/config.yaml`; pins must avoid GPIO 14, 15, 18, 23 (22). The commonly documented Waveshare pin set uses GPIO 18 for reset and must be re-strapped or a different HAT chosen. `[S]`+`[D]` (SPEC-004 S-004-5, S-004-6; F.01.10)
- F.03.5 Two mesh daemons on one host (one on the tile PHY, one on the SX1262) can be bridged over IP with MQTT (a local broker) or kept as separate meshes; port 4403 and BLE must be assigned to one of them. `[S]`+`[C]` (SPEC-004 S-004-7; U-002-1)
- F.03.6 For spectrum and spatial mapping at 915 MHz through a translation front end, the existing PSD app, the interleaved 4-channel SoapySDR stream, and GNU Radio suffice for capture; the RF-vision app's phase-to-angle scale is a compile-time function of LO frequency (`SCALE_FACTOR_AT_MHZ`) and element pitch (`ANTENNA_SPACING_MM`), so a translated band needs a two-constant patch: true wavelength = c / (f_LO,tile − f_LO,translator) and the 915 MHz pitch. `[S]`+`[D]` (csi_sweep.c lines 93–100)
- F.03.7 For LoRa demodulation on raw I/Q (research, multi-slot monitoring, DoA per packet) `gr-lora_sdr` runs on GNU Radio 3.10 and is hardware-agnostic; `quadrf-mesh` reuses its bit-level chain in C++ with NEON. Either can take the tile's beamformed stream or one element stream. `[S]` (SPEC-003 S-003-10; SPEC-006 S-006-10)
- F.03.8 CPU: the `quadrf-mesh` PHY costs ≈ 0.35 of one A76 core at 8 MSPS single channel; PhaseGaze runs at 38 MSPS; full-band (26 MSPS × 4 ch) continuous demodulation of all 104 slots is not feasible on the Pi 5, but a channeliser-plus-energy-detector for occupancy and DoA of active slots is, with demodulation of a few selected slots. `[S]`+`[C]` (quadrf-lora-phy.default comment; applications.md; feasibility to be measured, G05)

## 5. Analysis

Three process graphs cover the use cases:

A. Native 5.8 GHz LoRa link (exists today):
`quadrf-jtag` → `quadrf-lora-phy` (Soapy `mipi`) ⇄ Air-IPC ⇄ `quadrf-meshtasticd` ⇄ phone API 4403 / web 9443. Configuration: `/etc/default/quadrf-lora-phy` FREQ=5800, PRESET=shortturbo or longturbo, antenna masks; set `CALLSIGN` and licensed mode only if operating under Part 97.

B. 915 MHz mapping and direction finding (needs G05 hardware):
Translation front end → tile (interleave=1, 26 MSPS, fixed LO) → Soapy `mipi` → (i) PSD app for occupancy, (ii) patched RF-vision for spatial scatter, (iii) GNU Radio flowgraph: polyphase channeliser → per-slot energy detector → 4-channel phase-difference DoA per burst → `gr-lora_sdr` decoder on selected slots → ZeroMQ to a map UI. All of (iii) is to be written; blocks exist.

C. 915 MHz Meshtastic participation (no tile needed):
SX1262 HAT → `meshtasticd` (SPI0, pins avoiding JTAG) → phone API; optional MQTT bridge to A. Directional antenna per G06; licensed mode per G02 if Part 97.

## 6. Answer

Expected stack: DietPi + vendor packages + `quadrf-mesh` (exists) for 5.8 GHz
LoRa; `meshtasticd` + SX1262 HAT (exists) for 915 MHz participation; a GNU
Radio / C++ channeliser-DoA-decoder chain (to write, ≈ 3 blocks of new code)
plus a two-constant patch to the RF-vision demo for 915 MHz mapping through
the G05 front end.

## 7. Recommendation (opinion)

Do not write a new LoRa PHY; reuse `quadrf-mesh` and `gr-lora_sdr`. Spend
development effort on (B.iii) and on the Air-IPC-compatible glue so that the
mesh daemon is unaware of which PHY is underneath. Choose a Pi 5 LoRa HAT
whose reset line is not GPIO 18 or is jumper-selectable, and keep UART0 free
(GPS over USB or I²C instead).

## 8. Gate record

| Field | Value |
| --- | --- |
| Pass criterion | Each use case maps to a process graph with named existing components and an explicit list of components to be written. |
| Evidence | F.03.1–F.03.8, §5 |
| Decision | PASS |
| Date / signatory | 2026-10-08 / project |
| Open conjectures carried as risk | U-002-1 (daemon coexistence) and F.03.8 (CPU budget) close at G05 bench; F.03.4 pin re-strap verified when HAT model is chosen |

## 9. Carry-forward

- F.03.2, F.03.3, F.03.4, F.03.6, F.03.7, F.03.8, process graphs A/B/C.

## 10. Errata and addenda (2026-10-08, second pass)

The gate decision is unchanged. The user's extension (Rust, copal, USB and
plug-in sensors, a parallel feed) moved the "what we write" answer into
implementation specs; this gate's process graphs A/B/C stand as the
functional description.

- F.03.9 Of the fourteen SX126x board templates shipped with the Linux daemon, the MeshAdv-Mini 900M22S (CS 8 / IRQ 16 / Busy 20 / Reset 24 / RXen 12) uses none of the tile's JTAG pins; PiTastic/ZebraHat and RAK6421/Station G3 sets are clear if GPIO 22 is unused; every template with Reset on GPIO 18 (Waveshare SX126x HAT, MeshAdv 900M30S, NebraHat, PiMesh, Starter edition) collides. This answers the first-pass recommendation. `[S]`+`[D]` (SPEC-004 S-004-9/10)
- F.03.10 USB-attached SX1262 radios (meshstick, meshtoad-E22, uMesh 30 dBm, RAK19714, frametastic) use a USB SPI/GPIO bridge and no header pin; they are the simplest 915 MHz participation path beside the tile and fit the plug-in model. `[S]` (SPEC-004 S-004-11)
- F.03.11 Everything the vendor stack needs below the application layer exists in Alpine 3.24 for the Pi 5 (`linux-rpi` 6.18.52 with headers, `akms`, `openocd` 0.12, `soapy-sdr` 0.8.1, `rtl-sdr`, `hackrf`, `gnuradio` 3.10.12, `zeromq`, `rust`/`cargo`, `gpsd`, `chrony`); the vendor's Debian/systemd/DKMS/Flask layer is replaced, not ported. `[S]` (vendor/alpine/ATTRIBUTION.md; SPEC-010)
- F.03.12 The Rust ecosystem on 2026-10-08 provides pure-Rust USB (`nusb` 0.2.7), an RTL-SDR driver on it, a native ZeroMQ, protobuf, FFT and portable-SIMD crates, an SX1262 driver and an Embassy LoRa PHY for the Pico; it provides no production-grade SDR framework (FutureSDR requires nightly; seify's native drivers are experimental). The architecture therefore uses small crates and our own kernels. `[S]` (vendor/rust-crates/ATTRIBUTION.md; LR-003)
- F.03.13 What we write is now specified: SPEC-008 (processes `qrf-tiled`, `qrf-dspd`, `qrf-sensord`, `qrf-overlay`; workspace crates), SPEC-009 (plug-ins and bus), SPEC-010 (platform). Process graph B.iii's "≈ 3 blocks of new code" becomes the channeliser, detector/bearing and CSS demodulator of `qrf-dsp`. `[D]`
