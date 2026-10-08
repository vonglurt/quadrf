# SPEC-008 — System architecture: Rust processes on copal for capture, sensing, bearing and overlay

| Field | Value |
| --- | --- |
| Status | Draft |
| Revision | 0 |
| Date | 2026-10-08 |
| Subject | The software system this project builds ("qrf"): a set of Rust processes on a Raspberry Pi 5 running copal that capture from the QuadRF tile and from USB/HAT sensors, channelise, detect, bear and decode LoRa, publish typed feeds, and render a common overlay; plus the field-node firmware |
| Primary sources | SPEC-001 rev 2, SPEC-002 rev 2, SPEC-003, SPEC-004 rev 2, SPEC-006, SPEC-009, SPEC-010, SPEC-011; `lab/LR-003-rust-system-architecture.md`; `analysis/linkbudget.py` T14, T16, T19, T20 |
| Depends on | SPEC-002, SPEC-009, SPEC-010 |

## Scope

Process model, data path, DSP functions, bus, overlay, control-plane
security, toolchain and workspace. Every statement is a design decision
(`[D]`) traced to a requirement in the charter (G00), a gate finding, or
the user's 2026-10-08 extension (Rust, copal, USB and plug-in sensors, a
parallel data feed into one overlay). Not covered: the FTFE hardware
(SPEC-007), the plugin wire schema (SPEC-009), the platform (SPEC-010).

## Behaviour-goal statements

Process model:

- S-008-1. The system is a set of separately executing processes connected by documented IPC: `qrf-tiled` (tile capture and control), one `qrf-sensord` per USB or HAT sensor plugin, `qrf-dspd` (channeliser, detector, bearing, decoder), `qrf-overlay` (HTTP and WebSocket server with a static UI) and the `qrf` command-line tool. No process links GPL code; GPL components (vendor SoapySDR module, `quadrf-lora-phy`, `meshtasticd`, KrakenSDR software) run as their own processes behind their published interfaces. `[D]` (docs/00-process.md §6; SPEC-002 S-002-8 single-owner device nodes)
- S-008-2. `qrf-tiled` is the sole owner of `/dev/csi_stream0` and `/dev/dsi_stream0` while it runs: it maps the receive ring, consumes whole spans, stamps each span with CLOCK_MONOTONIC_RAW and CLOCK_TAI, de-interleaves to per-element CS8 buffers, publishes them to local consumers through a shared-memory ring (memfd, single producer, sequence-numbered) and to remote consumers through ZeroMQ, holds the JTAG lease, and exposes a control API (tune, bandwidth, gain, AGC set-point, element mask, per-element phases, interleave, polarisation, transmit enable) on a Unix socket. `[D]` (SPEC-002 S-002-14, S-002-16; SPEC-001 S-001-16/17; analysis T14)
- S-008-3. Lossless capture is a measured property: `qrf-tiled` reads `csi_stats` and `csi_event_stats` once per second and asserts that ring overflows, CSI overflows and buffer-starvation counts did not increase; any increase is logged as a loss event with the span sequence number, and the G05 bench criterion (5) is evaluated from this log. `[D]` (SPEC-002 S-002-15; G05 §8 criterion 5)
- S-008-4. The span-to-userspace latency budget is 10 ms (16 driver buffers × 630 µs at 4 × 26 MSPS); `qrf-tiled`'s consumer thread runs under SCHED_FIFO with memory locked, and on copal its DSP consumers may be pinned to isolated cores (SPEC-010 S-010-6). `[D]` (analysis T14)
- S-008-5. Samples stay CS8 until `qrf-dspd` converts them once to f32; all DSP is f32; NEON is used through portable-SIMD crates (`wide`, `pulp`) or `std::arch::aarch64` intrinsics confined to leaf kernels that each have a scalar reference implementation and a property test asserting equality within 1 LSB. `[D]` (docs/00-process.md §6 memory safety)
- S-008-6. `qrf-dspd` implements: a polyphase channeliser of M = 128 bins over the 26 MHz capture (203 kHz bins, ≥ 1 bin per 250 kHz slot) at ≤ 0.8 of one A76 core for four channels; a per-slot energy detector with a CFAR threshold and a duty-cycle estimator; a per-burst four-channel cross-spectral bearing estimator (two-element phase differences, optionally MUSIC on the 4 × 4 covariance) reporting azimuth, elevation and σ; and a LoRa CSS demodulator for up to four selected slots (SF 7–12; BW 125/250/500 kHz; sync word 0x2B; explicit header; CRC; LDRO) reporting SNR, RSSI, CFO and the decoded air frame. `[D]` (analysis T19; SPEC-003 S-003-1, S-003-6/7; G03 process graph B.iii)
- S-008-7. Feeds are published as ZeroMQ PUB/SUB messages carrying protobuf payloads whose schema is SPEC-009; every process publishes one `Health` message per second. `[D]` (SPEC-009 S-009-3, S-009-6)
- S-008-8. `qrf-overlay` subscribes to all feeds and renders, in one azimuth frame (true north) and one time base, these layers: the tile's RF-vision scatter at 4.9–6.0 GHz, 915 MHz bearing rays with uncertainty wedges, the 104-slot occupancy strip, the decoded-frame log, and the sensor health panel; it serves the page over HTTP and the data over WebSocket. `[D]` (SPEC-009 S-009-4/5/9; user requirement for a parallel feed into the overlay)
- S-008-9. Control sockets are Unix sockets owned by group `qrf`; the overlay binds only to localhost and the addresses of interfaces named in its configuration; any transmit command is refused unless an unlock file names the regulatory profile (SPEC-005 regime, callsign when Part 97, maximum conducted power and antenna gain), and every transmit command is logged with time, frequency, power, element mask and profile. `[D]` (docs/00-process.md §6 transmit safety; SPEC-005)
- S-008-10. `qrf-tiled` can read through the vendor's SoapySDR module (`soapysdr` crate, `driver=mipi`) instead of the ring for parity tests; parity is proven by capturing one CW tone both ways and comparing CS8 streams bit-exactly, or power and phase within 0.1 dB and 1°. `[D]` (SPEC-002 S-002-21)
- S-008-11. The field-node firmware for the Raspberry Pi Pico LoRa HAT is Rust on `embassy-rp` with `lora-phy` (SX1262) and acts as a calibration transponder (continuous preamble or CW on a chosen slot at a set conducted power, logged) and as a logging node; ESP32 nodes run upstream Meshtastic firmware unchanged. `[D]` (G05 F.05.12; G07 T-1…T-6)
- S-008-12. The Cargo workspace is: `qrf-core` (types, configuration, time), `qrf-mipi` (ring and ioctl bindings, de-interleave), `qrf-jtag` (transceiver register programming through the CSI node's JTAG ioctls, written from SPEC-001/002 behaviour, not from vendor code), `qrf-dsp` (kernels), `qrf-lora` (CSS modem), `qrf-bus` (schema and transport), the binaries `qrf-tiled`, `qrf-sensord`, `qrf-dspd`, `qrf-overlay`, `qrf`, and the separate-target firmware crate `qrf-node-pico`. `[D]`
- S-008-13. Toolchain: stable Rust pinned in `rust-toolchain.toml`, edition 2024, target `aarch64-unknown-linux-musl`; `Cargo.lock` committed; `cargo deny check` (licence allow-list, advisories, duplicate versions) and `cargo audit` in `make check`; release tags signed with `ssh-keygen -Y`. `[D]` (docs/00-process.md §6)
- S-008-14. Observability: structured logs through `tracing`; per-process metrics (ring fill, loss counters, CPU per thread, message rates) exported in `Health` and on a local text endpoint. `[D]`
- S-008-15. The transceiver register map needed by `qrf-jtag` is unknown to this project beyond the ioctl ABI; until it is derived from the vendor's published behaviour at the bench, `qrf-tiled` drives the vendor's GPL `quadrf-jtag` binary as a child process for tuning. `[C]` (SPEC-002 S-002-16; closes at backlog R-04)

## Interfaces we depend on

- SPEC-002 S-002-14…17 device-node ABI.
- Air-IPC v1 (SPEC-006 S-006-2) when the GPL LoRa PHY is used as a plugin.
- `meshtasticd` TCP 4403 (SPEC-004 S-004-7) for the Meshtastic bridge plugin.
- SPEC-009 message schema and transport.
- SPEC-011 DAQ output when a KrakenSDR plugin exists (U-011-1).

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-008-1 | Ring size and span (= U-002-3); sizing of the shared-memory ring | bench, backlog R-03 |
| U-008-2 | Whether the vendor's RF-vision `/ws` stream is documented enough to consume, or whether our own swept-LO scatter must be produced | backlog R-09 |
| U-008-3 | Interoperability of the native `zeromq` crate with libzmq consumers (GNU Radio ZMQ blocks) | backlog R-06 |
| U-008-4 | The transceiver register map (S-008-15) | backlog R-04 |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 0 | 2026-10-08 | Draft from the user's Rust/copal/plug-in extension and LR-003 |
