# LR-003 — Software architecture in Rust: capture, plug-in sensors, feed bus, overlay; security and performance practice

*Lab report. 2026-10-08.*

<!-- SPDX-License-Identifier: MIT -->

| Field | Value |
| --- | --- |
| Status | Final (desk); measurements pending hardware |
| Author | project |
| Feeds | SPEC-008, SPEC-009; backlog R-01…R-13 |

## Abstract

The user's 2026-10-08 extension of the charter asks for a system that is
Rust-first, runs on a Raspberry Pi 5 under copal, accepts USB and plug-in
sensors, and merges a second data feed (a 915 MHz LoRa array) into the
tile's visualisation overlay, securely. This report derives the architecture
from the measured constraints of the vendor stack (an 8-bit CS8 stream of
208 MB/s at 4 × 26 MSPS arriving as 131 072-byte spans every 630 µs through a
mmap ring with 10 ms of kernel buffering; GPL-2.0 drivers and GPL-3.0
applications; single-owner device nodes) and from the Rust ecosystem as it
stands (pure-Rust USB through `nusb`, a native ZeroMQ, mature FFT and SIMD
crates, no production-grade Rust SDR framework). The decision is a set of
small processes behind typed IPC: a capture daemon that reads the kernel
ring directly, a DSP daemon with a 128-bin polyphase channeliser costing
0.73 of one core for four channels, a plug-in supervisor for USB sensors, a
ZeroMQ/protobuf bus, and an HTTP/WebSocket overlay. GPL components run as
separate processes. The architecture is written as SPEC-008 and SPEC-009;
this report records the reasoning and the alternatives rejected.

## I. Objective

1. Derive a process and data-path architecture that satisfies the charter's measurable goals (whole-band 915 MHz occupancy and bearing, decoding of selected slots, a 5.8 GHz layer) and the 2026-10-08 extension (Rust, copal, USB/plug-in sensors, parallel feed, security).
2. Choose the capture path, DSP strategy, bus, overlay technology and security model, each with a stated reason and a rejected alternative.
3. Set the performance budgets that the build phase must meet, from `analysis/` tables.

## II. Materials

| Item | Detail |
| --- | --- |
| Vendor data path | SPEC-002 rev 2 S-002-14…21 (ring, ioctls, stats, DSI staging, CS8 interleave) |
| Budgets | `analysis/linkbudget.py` T14 (data path), T16 (bearing CRLB), T19 (CPU), T20 (USB power) |
| Crate survey | `vendor/rust-crates/ATTRIBUTION.md` (crates.io API, 2026-10-08) |
| Platform | SPEC-010; copal conventions (`copal-build` compiles cargo checkouts in `~/code`; playbooks) |
| Reference implementations read for behaviour only | `quadrf-mesh` (Air-IPC, DDC, wander tracking), vendor SoapySDR module (ring consumption, NEON de-interleave) |

## III. Method

Requirements were listed from G00 and the user's extension; constraints from
SPEC-001/002 rev 2 and the platform; for each architectural choice two or
three candidates were compared on licence, copies of the data, CPU, risk, and
time to first data; the winner was written into SPEC-008/009 as behaviour
statements with the budget it must meet.

## IV. Results

### A. Requirements and constraints

| Requirement | Source | Consequence |
| --- | --- | --- |
| Lossless 4 × 26 MSPS capture for ≥ 60 s | G05 criterion 5 | consumer latency < 10 ms (T14); loss counters as the proof (S-002-15) |
| Occupancy and bearing for all 104 slots; decode ≤ 4 | G03 F.03.8, G05 F.05.11 | channeliser ≤ 0.8 core (T19); decoder per slot small |
| Two sensors drawn on one overlay | user extension | common time base and azimuth frame (SPEC-009 S-009-6/7) |
| USB and plug-in sensors | user extension | per-device process, hot-plug supervision (SPEC-009 S-009-1/2) |
| Secure, Rust, cargo | user extension; process §6 | memory-safe code, locked dependencies, licence allow-list, least privilege |
| copal (Alpine, musl, OpenRC) | user extension | no systemd units, no glibc assumptions, no Python services |
| GPL boundary | process §6 | vendor module, mesh daemons, Kraken software as separate processes |

### B. Decisions

| Choice | Decision | Rejected alternatives and why |
| --- | --- | --- |
| Capture path | Read the kernel ring directly: `mmap` + `CSI_IOC_GET_RING_INFO`/`CONSUME_BYTES` + `poll` in `qrf-mipi` | (a) vendor SoapySDR module through the `soapysdr` crate: loads a GPL-2.0 shared object into our process and adds a Farrow resampler copy; kept only for parity tests (S-008-10). (b) `seify`/FutureSDR: FutureSDR needs nightly and its README advises against depending on it; seify's native drivers are marked experimental. |
| Ownership | One process owns both device nodes and the JTAG lease | shared ownership breaks the vendor's single-owner rule (S-002-8) and the lease (S-002-16) |
| Sample type | CS8 until one f32 conversion in the DSP daemon | converting in the capture daemon doubles memory traffic for no gain |
| DSP | Own kernels (`qrf-dsp`): polyphase channeliser M = 128, P = 8; CFAR energy detector; cross-spectral bearing; CSS demodulator; NEON through `wide`/`pulp` with scalar references | GNU Radio flowgraph: C++/Python, GPL-3.0, heavy on a Pi; `rustradio`: evaluated later for blocks we do not write |
| Bus | ZeroMQ PUB/SUB (native `zeromq` crate) + protobuf (`prost`) | D-Bus: no high-rate path; custom TCP: no ecosystem (vendor tools and GNU Radio already speak ZeroMQ) |
| Raw I/Q to DSP | shared-memory ring (memfd, sequence-numbered) | ZeroMQ copies 208 MB/s twice |
| Overlay | `axum` HTTP + WebSocket serving one static page drawing on a canvas | `egui` native: needs a display on the Pi; vendor Flask GUI: Python, GPL-2.0, Debian-specific |
| USB | `nusb` (pure Rust, hotplug API) with `rtlsdr-nusb`; `seify` for HackRF | `rusb`/libusb: C dependency, no advantage; `librtlsdr-rs`: GPL-2.0 |
| Meshtastic bridge | separate GPL plug-in process using the `meshtastic` crate over TCP 4403 | linking the GPL-3.0 crate into MIT binaries |
| Field node | Pico with `embassy-rp` + `lora-phy` | writing a Meshtastic-compatible node in Rust (the only candidate, `meshrustic`, is AGPL and experimental) |
| Transceiver control | drive the vendor GPL CLI as a child until the register sequence is observed and re-implemented (S-008-15) | copying `jtag.c` (GPL) |

### C. Budgets the build must meet

| Quantity | Budget | Source |
| --- | --- | --- |
| Capture thread wake-to-consume latency | < 10 ms, target < 2 ms (3 spans) | T14 |
| Channeliser CPU, 4 ch × 26 MSPS | ≤ 0.8 core (model 0.73) | T19 |
| Decoder CPU per 250 kHz slot | ≤ 0.1 core `[C]` | T19 note; quadrf-mesh 0.35 core at 8 MSPS as an upper reference |
| Overlay message rate | ≤ 10 Hz occupancy, ≤ 30 Hz scatter | SPEC-009 S-009-10 |
| Bearing precision, thermal | ≪ 1° (0.06° at 20 dB, 1024 samples); the 5° criterion is calibration-limited | T16 |
| USB power | ≤ 1.6 A total; Kraken self-powered | T20 |
| Memory | ring ≤ 64 MB (0.3 s at 208 MB/s); 1 s history optional to disk | T14 |

### D. Security model

- Memory safety by language; `unsafe` only in `qrf-mipi` (ioctl/mmap) and leaf SIMD kernels, each with a safety comment and a test.
- Process isolation: one process per device; user `qrf`; device access by group; no root after `qrf-load`.
- Supply chain: `Cargo.lock` committed; `cargo deny` licence allow-list and advisory check; `cargo audit`; releases tagged and signed with `ssh-keygen -Y` as copal already does for its installer.
- Network: overlay bound to configured interfaces only; no open AP; SSH key-only (platform).
- RF: transmit refused without an unlock file naming the regulatory profile; all transmit commands logged (SPEC-008 S-008-9).

### E. Comparison with the vendor stack

| Aspect | Vendor (DietPi image) | qrf |
| --- | --- | --- |
| Languages | C (drivers, CLI), C++ (SoapySDR, demos), Python (Flask GUI) | Rust; vendor C drivers rebuilt unmodified |
| Licences | GPL-2.0/3.0 | MIT; GPL parts isolated in processes |
| Init | systemd | OpenRC |
| GUI | KasmVNC desktop + Flask control panel | one web page over WebSocket |
| IPC | Unix socket (Air-IPC), ZeroMQ, Socket.IO | ZeroMQ + protobuf; shared memory for I/Q |
| Sensors | tile only | tile + USB SDRs + LoRa sticks/HATs + GNSS |
| Observability | journald | `tracing` JSON + `Health` messages with loss counters |

## V. Discussion

The main technical risk is the capture daemon's latency on a general-purpose
kernel: 10 ms of buffering is comfortable for a SCHED_FIFO consumer but not
for a process that shares a core with the overlay's WebSocket encoder; hence
the pinning options in SPEC-010 and the measured checks in the backlog. The
main licensing risk is the transceiver register map: until it is observed at
the bench, tuning goes through the vendor binary as a child process, which is
lawful and slow (fork per retune) but adequate for a fixed-LO 915 MHz map.
The main schedule risk is the DSP work: a correct CSS demodulator is a few
thousand lines; `gr-lora_sdr` serves as a test oracle only.

What would refute the choices: a measured channeliser cost above 1.5 cores
(then M or P must drop, or the band must be split), or a loss event rate that
pinning does not remove (then the kernel copy workqueue, not userspace, is
the bottleneck and a driver patch is needed).

## VI. Recommendations and best practice

1. Build the workspace skeleton and the bus first, with simulated feeds, so the overlay and conformance tests exist before hardware (backlog R-01, R-02, R-09).
2. Validate the CSS demodulator against recorded I/Q from an SX1262 node and from `gr-lora_sdr` before touching the tile (R-08).
3. Measure, do not assume: every budget in §C becomes a benchmark in `make check` on the Pi (R-07).
4. Observe the vendor CLI's register writes with `strace` at the bench and write the table into SPEC-001 as `[M]` facts (R-04).

## VII. References

SPEC-001 rev 2, SPEC-002 rev 2, SPEC-006, SPEC-008, SPEC-009, SPEC-010; `analysis/linkbudget.py` T14, T16, T19, T20; `vendor/rust-crates/ATTRIBUTION.md`; `vendor/summary/scalerf-quadrf.md`; `vendor/summary/radioroy-quadrf-mesh.md`.
