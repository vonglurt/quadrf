# Backlog — the design and build track

<!-- SPDX-License-Identifier: MIT -->

Work that is known and not yet done, in the order it should be done. Phases
P1–P7 are entered in sequence; a phase's **exit check** is the last entry in
it. P0 is a standing phase (see *Sequencing*). The reasoning behind the
entries is in the lab reports named in each phase; the behaviour each entry
must deliver is in the spec statement it names.

## How to use this

1. Take the first entry under **Open** in the current phase.
2. Make the change in the place the entry names.
3. Run the entry's **Check**. An entry is done when its check passes, not
   when the change is made.
4. Move the row to **Done**, with the date and the commit.
5. An entry that depends on an open conjecture (`U-…`, `C…`) is not marked
   done before the gate that closes the conjecture is signed
   (`investigations/README.md`).

## Sequencing (re-sequenced 2026-10-08 after the review in `lab/LR-007-backlog-review-and-resequencing.md`)

- **P0 is standing.** Its entries are lookups that need a browser or a
  second reader, one licensing decision, the procurement list, and field
  work with the nodes already in hand. It runs concurrently with P1, has no
  exit check and never blocks P1; its entries should be done before the kit
  arrives.
- **P1 is the critical path** and runs now on the development VM; only the
  R-07 budget needs a Pi 5, and no entry needs the tile. The order inside P1
  is the order of dependency: bus, oracle, demodulator, ring mock, DSP,
  overlay, supervisor, interoperability, control plane, release procedure,
  exit check.
- **P2 is entered when R-P1 passes and the dongle and the stick (D-09) are
  on the desk.** F-01, F-02 and F-05 moved to P0 because they need no qrf
  software.
- **P3 onward wait for the kit** (vendor date 2026-11-30). P-03b keeps data
  flowing if the kernel-module port stalls.
- Opinions on duration are in LR-007 §V, not here.

| Prefix | Means | Where |
| --- | --- | --- |
| D | documents and ledger | this repository |
| A | analysis scripts | `analysis/` |
| V | vendored sources, licences, regulatory reading, third-party test tools | `vendor/`, `resources/`, `docs/resources-manifest.md` |
| R | Rust software | the `qrf` Cargo workspace at the repository root (`Cargo.toml`, `crates/`) |
| P | platform: copal, kernel, services | copal repository playbook + `platform/` here |
| H | hardware: tile bench, FTFE, apertures, HATs | `investigations/records/` |
| F | field tools, nodes, firmware, field tests | `investigations/G07`, `investigations/records/` |
| S | system-level tests | `investigations/records/` |

**Standing:** 57 open · 0 in progress · 20 done · 0 dropped. Written 2026-10-08; re-sequenced 2026-10-08 (LR-007).

---

## Open

### P0 — Standing: ledger, sources, decisions, procurement and field work with hardware in hand (now; concurrent with P1; no tile, no qrf software)

| ID | Entry | Rule / gate | Check |
| --- | --- | --- | --- |
| D-05a | Second read by the user of SPEC-001 rev 2–3 (S-001-26…42) and SPEC-005 rev 2 (S-005-20…22), the hardware and regulatory specs: each `[S]` opened at its cited line | process §3.1 | Reader's initials and date in both revision tables; status returns to Reviewed |
| D-05b | Second read, by a session that did not write them, of SPEC-002 rev 2 (S-002-14…22), SPEC-003 rev 2 (S-003-12…15), SPEC-004 rev 2 (S-004-9…12) and SPEC-008…011: each `[S]` opened at its cited line; the reviewer named | process §3.1 | Reviewer and date in each revision table; status returns to Reviewed |
| V-02b | Import the SE5004L *device* data sheet (DST-00316; the file in hand is the evaluation-kit sheet) for saturated power and P1dB | SPEC-001 U-001-7 | `import-shared.sh` reports it; S-001-9/40 retagged; U-001-7 closed |
| V-02c | Fetch, in a browser (semtech.com returns an interstitial to scripts), the current SX1261/2 datasheet revision (the copy in hand is Rev 1.1, 2017) and diff the values used in S-003-12…15 | SPEC-003 U-003-2 | Revision and date recorded in SPEC-003's revision table; any changed value updated |
| V-04 | Read the FCC grants of every module in hand at fcc.gov/oet/ea/fccid (class, frequency rows, grant notes). Needs a browser: apps.fcc.gov returns "Access Denied" to scripted requests and fccid.io a JavaScript challenge (2026-10-08) | U-005-1, U-005-4 | SPEC-005 S-005-22 retagged `[S]` or deleted; a table of FCC IDs in G07 §7 |
| V-06 | Re-check §15.247 and §97.311 at ecfr.gov before any transmission campaign | U-005-2 | Date and "no change" or the diff recorded in SPEC-005's revision table |
| V-07a | Decide now whether licensed (Part 97) operation is part of the plan: it is the only regime in which the 915 MHz transmit array (P7) and G06's licensed branch have lawful purpose (F.06.11), and a licence has lead time. If yes, obtain the licence and record the callsign | G06 F.06.11; SPEC-005 S-005-14…18 | Decision, date and, if yes, the callsign recorded in a dated G06 addendum and in this file; if no, P7 and V-07b move to Dropped |
| D-09 | Procurement list `docs/procurement.md`, one row per part an open entry needs: RTL-SDR dongle and USB SX1262 stick (F-03, F-04, F-05); FTFE evaluation boards per F.05.13 and SPEC-007 S-007-15 (915 MHz BPF, LNA, mixer (U-007-3), MAX2871-class synthesiser board, 4-way divider, 5.5 GHz BPF, pads) (H-04, H-05); the 164 mm 2 × 2 aperture: monopoles, ground plane, SMA feeds (H-06); a PPS-capable GNSS receiver (H-08); active cooling for the Pi 5 (P-08); a Pi 5 for the R-07 budget if none is on the desk before the kit; the KrakenSDR as an option (H-07). Columns: candidate part, vendor, unit cost, lead time, needed-by date derived from the phase it unblocks, order date, arrival date | F.05.13; U-007-3; LR-005 §VI.1; LR-007 §VI | Every H- and F- entry and R-07 cites a row; every row has a needed-by date; the dongle and stick rows carry order dates |
| F-01 | Inventory: each ESP32 node's board, radio IC, firmware tag, max `tx_power`, antenna, FCC ID; the Pico HAT's model, band and IC | G07 §7 | Table in G07 §7 with every row `[S]` |
| F-02 | Flash all nodes with one firmware tag, one preset, one channel, one `tx_power`; store `meshtastic --info` dumps | G07 §3 | Dumps under `resources/measurements/` listed in the manifest with sha256 |
| F-05 | Run G07 T-1…T-4 (bandwidth, free-space law, Yagi gain, repeater budgets) with a spectrum analyser, or with the dongle (D-09), the `rtl-sdr` tools and a calibrated attenuator (G07 §8) | G07 | Four `M-nnn` records filed; residuals explained |

### P1 — Rust workspace with simulated feeds (now; the critical path; the VM, plus any Pi 5 for the R-07 budget)

| ID | Entry | Rule / gate | Check |
| --- | --- | --- | --- |
| R-07 | `qrf-dsp`: polyphase channeliser (M = 128, P = 8), CFAR detector, two-element phase-difference bearing, 4 × 4 covariance + MUSIC; criterion benchmarks | SPEC-008 S-008-6; T16, T19 | Function, on the VM: bearing error on simulated plane waves ≤ 0.1° at 20 dB SNR (T16 bound 0.057°); a tone's power through the channeliser within 0.1 dB of its input. Budget, on a Pi 5 running copal (any Pi 5, no tile; the VM's cores are not a proxy for the A76, LR-007 §IV): 4 ch × 26 MSPS channeliser ≤ 0.8 core from `/proc/<pid>/stat` over 60 s, filed as an `M-nnn` record; the VM figure recorded beside it for reference only |
| R-09 | `qrf-overlay`: static page + WebSocket; layers for scatter, bearing rays with σ wedges, 104-slot occupancy strip, frame log, health | SPEC-008 S-008-8 | With simulated feeds, all layers render at the declared rates in a browser on the VM; CPU of the server ≤ 0.1 core |
| R-02b | `qrf-sensord` supervisor with `nusb` hot-plug and a TOML sensor declaration; conformance test harness | SPEC-009 S-009-1/2/12 | A dummy USB device (or a simulated plug-in) passes the 10-cycle unplug/replug test |
| R-06 | ZeroMQ interoperability: the native `zeromq` crate against `pyzmq` (installed on the VM) and against a GNU Radio ZMQ SUB source block from the V-08 install | U-008-3 | 10 000 messages each way with each peer, none lost or reordered |
| R-14 | Control plane before any binary can command a transmitter: in `qrf-core` the unlock-file schema (SPEC-005 regime, callsign when Part 97, maximum conducted power in dBm, antenna gain in dBi, validity dates), the Unix control-socket protocol that `qrf-tiled` and the `qrf` CLI will share, the refusal path and the transmit log, exercised against a mock transmitter | SPEC-008 S-008-9; SPEC-010 S-010-9; process §6 | Tests: a transmit command with no unlock file, with a malformed file, with an expired file, and with a request above the file's power or gain limit are each refused and logged; a valid Part 15 request reaches the mock and its log line carries time, frequency, power, element mask and profile; the socket is created mode 0660 in the configured group; the mock asserts it is never reached without a passed check. R-10 may not add a control socket to `qrf-tiled` before this passes |
| D-10 | Release procedure: `docs/release.md` with `make release` and `make verify`: clean checkout of the tag, `cargo deny check`, `cargo audit`, `cargo build --locked --release`, a reproducible tarball (sorted names, fixed mtime), `ssh-keygen -Y sign` in the namespace copal uses, the public key in a committed `allowed_signers`, `ssh-keygen -Y verify` | SPEC-008 S-008-13; SPEC-010 S-010-11 | On a tagged commit `make release` yields an artefact that `make verify` accepts; a one-byte change to the artefact or to the signature makes `make verify` fail; the procedure exists before P-01 installs anything on the Pi |
| R-P1 | **Exit check:** end-to-end simulated run (T-10 in simulation) with injected errors: one simulated emitter at a known true azimuth is seen by the simulated tile feed and the simulated 915 MHz feed; the 915 MHz feed declares a non-zero mounting yaw and a declination in its TOML (S-009-7) and its messages are delivered 300 ms later than the tile's | SPEC-009 S-009-7, S-009-13 | 30-minute run: skew ≤ 100 ms by `t_tai_ns`, azimuth error ≤ 2° after transforms; negative control: the same run with the transform zeroed, or with association by arrival time, fails |

### P2 — Field tools (after R-P1 and the D-09 purchases: one RTL-SDR dongle, one USB SX1262 stick)

| ID | Entry | Rule / gate | Check |
| --- | --- | --- | --- |
| F-03 | RTL-SDR plug-in (`rtlsdr-nusb`): occupancy of the 104 slots in 2.4 MHz slices; single-slot decode through `qrf-lora` | G04 C13; SPEC-009 S-009-3(b) | Test node on slot 20 appears in `Occupancy` within 5 s; its frames decode with CRC OK |
| F-04 | USB SX1262 stick (or a JTAG-clear HAT) through `meshtasticd` and the GPL bridge plug-in | G04 C14; SPEC-004 S-004-10/11 | Frames from the test node appear as `LoraFrame` in the overlay |
| F-06 | Pico transponder firmware in Rust (`embassy-rp` + `lora-phy`): CW or continuous preamble on a chosen slot at a set power; may start once F-01 has confirmed the HAT's band and IC; its check needs the dongle | SPEC-008 S-008-11; G05 F.05.12 | Dongle measures the emission at the set power ±1.5 dB and the set frequency ±2 kHz |
| F-P2 | **Exit check:** overlay shows live occupancy and decoded frames from the dongle and the stick for 1 hour without a loss event or a plug-in restart | SPEC-009 S-009-12 | Health log clean |

### P3 — Platform bring-up on copal (kit arrives; vendor date 2026-11-30)

| ID | Entry | Rule / gate | Check |
| --- | --- | --- | --- |
| H-01 | Read the ECP5 IDCODE with OpenOCD | U-001-6 | SPEC-001 S-001-35 retagged `[M]` with the device |
| P-01 | copal playbook `playbooks/Engineering/quadrf.sh` installing `linux-rpi-dev`, `akms`, `openocd`, `soapy-sdr`, `zeromq`, `rust`, `cargo`, `gpsd`, `chrony`, `dtc` | SPEC-010 S-010-10 | `apk info` lists them; playbook commit recorded here |
| P-02 | Identify copal's device manager on the Pi 5; write rules giving group `qrf` the device nodes | U-010-4, U-009-2 | `/dev/csi_stream0` is `root:qrf 0660` after module load |
| P-03 | Build `fpga-csi`/`fpga-dsi` with akms against 6.18.52; blacklist the in-tree RP1 camera driver if it binds `csi1`; record copal's page size and the ring size `CSI_IOC_GET_RING_INFO` returns (the ring is 512 pages: 8 MiB and 40 ms on 16 KiB pages, 2 MiB and 9.5 ms on 4 KiB, LR-009 §IV.A); if 2 MiB, decide between a `RING_ORDER` patch and a 16 KiB-page kernel before P-05 | U-010-1, U-010-2; U-002-3 | Modules load; `dmesg` shows the probe; both device nodes exist; page size and ring size recorded in an `M-nnn` record; any patch published |
| P-03b | Fallback if P-03 is not green within 7 days of the kit's arrival: boot the kit's vendor image and run its SoapyRemote server (SPEC-002 S-002-4); on copal or the VM, `qrf-tiled` behind the `soapysdr` crate (S-008-10) or a `soapy-remote` plug-in consumes CS8 over Ethernet at a host rate the link carries (T24: two channels at 26 MSPS or four at 13 MSPS; four at 26 MSPS never fit); bring-up, R-04's observation and a CW bearing check proceed while the module port continues | U-010-1, U-010-2; SPEC-008 S-008-10; T24 | `Health` and `Spectrum` from the tile in the overlay on copal; the sustained rate and the loss count over 60 s in an `M-nnn` record |
| P-04 | OpenRC `qrf-load` (unload, OpenOCD SVF from the copied bitstream, modprobe, vendor `quadrf-jtag --init` as a child) | SPEC-010 S-010-5; SPEC-002 S-002-19 | `csi_stats.frame_count` increases at rest with `interleave=1`; `rc-service qrf-load status` reports it |
| P-05 | Lossless capture: 60 s at 4 × 26 MSPS; copy-workqueue CPU measured | G05 criterion 5 precondition; U-002-4 | Zero loss events; CPU shares recorded in an `M-nnn` record |
| P-06 | Tuning options (governor, IRQ affinity, isolcpus/nohz_full) each measured | SPEC-010 S-010-6 | Each option's loss events and 99.99-percentile latency recorded; adopted only if better |
| P-07 | PREEMPT_RT build of `linux-rpi` 6.18 via aports `common-changes.config`; measured as P-06 | U-010-3 | cyclictest and loss events recorded; decision in SPEC-010 revision 1 |
| P-08 | Thermal: 10-minute 4 × 26 MSPS capture with the chosen cooling | SPEC-010 S-010-7 | Throttle counter zero |
| P-P3 | **Exit check:** P-05 repeated on the tuned platform with the overlay running | SPEC-010 | Zero loss events over 10 minutes |

### P4 — Tile software on copal

| ID | Entry | Rule / gate | Check |
| --- | --- | --- | --- |
| R-04 | Observe the vendor CLI's register writes (`strace` of the JTAG ioctls) for init, tune, gain, mask, phases; write the sequences into SPEC-001 as `[M]`; implement native tuning in `qrf-jtag` | U-008-4; SPEC-008 S-008-15 | Native and vendor paths produce identical register sequences for 10 tune/gain/mask cases |
| R-05 | Parity: ring path vs vendor SoapySDR module on one CW tone | SPEC-008 S-008-10; U-002-5, U-010-5 | CS8 bit-exact, or power within 0.1 dB and phase within 1° |
| R-03b | Record 10 s of the real ring during P-05's capture to `resources/measurements/` (manifest row with sha256); the R-03 mock replays it | U-008-1, U-002-3 | Replay at the T14 cadence without loss on the Pi and on the VM; ring and span sizes recorded; U-008-1 and U-002-3 closed |
| R-10 | `qrf-tiled`: ring consumer on `qrf-mipi::Reader` (`SCHED_FIFO` when permitted), timestamps, de-interleave, shared-memory ring, ZeroMQ, the R-14 control socket, loss accounting; refuses to start when `/sys/module/fpga_csi/parameters/drop_oldest` is `1`, because that policy moves `tail` under a mapped span (LR-009 §IV.D) | SPEC-008 S-008-2/3/4 | 60 s lossless at 4 × 26 MSPS with `qrf-dspd` consuming; consumer wake-to-consume 99.99 % < 2 ms (the histogram `examples/mipi_soak.rs` records); the `drop_oldest=1` refusal tested |
| R-11 | 4.9–6.0 GHz layer: own swept-LO scatter, or the vendor `/ws` stream consumed by a plug-in | U-008-2; SPEC-008 S-008-8 | A 5.8 GHz CW source appears in the overlay at the right azimuth ±5° |
| H-02 | Measure and identify the antenna-module connector; pigtail loss at 5500 MHz | U-001-1; G05 criterion 1 | ≤ 1 dB loss, `[M]` record; mating part number in SPEC-007 S-007-12 |
| H-03 | Check whether the 40 MHz reference can be exported | U-001-2 | `[M]` record; SPEC-007 S-007-9 updated |
| R-P4 | **Exit check:** tile at 5.8 GHz as a plug-in, overlay layer live, 10 minutes lossless | SPEC-008 | Health log clean |

### P5 — The 915 MHz layer (G05 bench)

| ID | Entry | Rule / gate | Check |
| --- | --- | --- | --- |
| H-04 | FTFE single channel per SPEC-007 (LNA, BPF, mixer, 4585 MHz LO, 5.5 GHz BPF, pad) on evaluation boards | G05 recommendation; F.05.13 | G05 criteria (1) and (2): NF ≤ 4 dB, spurs ≤ −60 dBc |
| H-04b | G05 criterion 6: LO-spur replica table with a −30 dBm CW | F.05.14 | Offsets and levels recorded; replica rule parameters set |
| H-05 | Four channels with one LO through a matched 4-way divider | SPEC-007 S-007-7/8 | Criterion (3): ≤ 5° rms over 10 min |
| H-06 | 164 mm 2 × 2 monopole aperture, positions surveyed ≤ 2 mm | SPEC-007 S-007-10 | Survey record filed |
| H-08 | GNSS/PPS time source plug-in | SPEC-009 S-009-6; U-009-3 | `clock_quality = PPS` in Health; offset to NTP < 1 ms |
| R-13 | Channeliser + bearing + decoder on the FTFE stream; replica flagging; `f_translate` configuration | SPEC-008 S-008-6; SPEC-009 S-009-8; F.05.14 | G05 criteria (4) and (5): bearing ≤ 5° rms over ±60° at 20 m; all 104 slots ≥ 60 s at ≤ 80 % of four cores |
| F-07 | Field tests T-5…T-8 | G07 | Records filed; G05 bench gate signed PASS/FAIL |
| R-12 | (Optional, only if a KrakenSDR is acquired, H-07) read the DAQ output format; plug-in | U-011-1, U-009-1 | KrakenSDR bearings appear as `Bearing` messages |
| H-07 | (Optional) KrakenSDR comparator: phase stability between recalibrations at 915 MHz | U-011-2; LR-005 | `[M]` record compared with H-05 |
| S-01 | **Exit check:** T-10 parallel-feed alignment with real sensors | SPEC-009 S-009-13 | 30-minute run, skew ≤ 100 ms, azimuth error ≤ 2° |

### P6 — Directional relay (G06)

| ID | Entry | Rule / gate | Check |
| --- | --- | --- | --- |
| F-08 | Phase 1: SX1262 node with a 10–12 dBi vertical Yagi, conducted power 30 − (G − 6) dBm, 500 kHz preset; T-3 and T-4 | G06 §7 Phase 1; SPEC-005 S-005-5 | Records filed; measured RSSI within tolerance for ≥ 2 summits |
| V-07b | Part 97 profile, only if V-07a decided yes: unlock file naming the callsign, exposure evaluation on file | SPEC-005 S-005-14…18, S-005-21; SPEC-008 S-008-9 | Unlock file present and logged; evaluation document under `investigations/records/` |
| F-09 | Phase 3: tile-to-tile 5.8 GHz LoRa link at 1 km (needs a second tile) with 500 kHz preset at 1 W aggregate, fixed point-to-point | G06 §7 Phase 3; G07 T-9 | PER ≤ 1 %; SNR within ±3 dB of budget |
| S-P6 | **Exit check:** G06 gate signed | G06 | Gate record PASS |

### P7 — Transmit beamforming at 915 MHz (Part 97 only; after G05 PASS, V-07a yes and V-07b)

| ID | Entry | Rule / gate | Check |
| --- | --- | --- | --- |
| H-09 | SPEC-007 revision 1: transmit down-converter and 915 MHz PA per element; coherence and PSD under §97.311 | G04 C4; G06 F.06.11 | Spec reviewed; parts quoted |
| S-P7 | **Exit check:** four-element transmit beam measured on a surveyed arc at ≤ 10 W PEP | G06 | Pattern within 3 dB of the model; ID and logging verified |

---

## In progress

| ID | Entry | Rule / gate | Where it stands |
| --- | --- | --- | --- |
| | *nothing* | | |

## Done

| ID | Entry | Done | Commit |
| --- | --- | --- | --- |
| V-08 | LoRa test oracle and corpus: `scripts/build-oracle.sh` builds `gr-lora_sdr` `862746d` (GPL-3, a separate process) into gitignored `resources/oracle/`; `scripts/make-corpus.py` generates `resources/corpus/qrf-lora-v1/` (37 cells: 7 Meshtastic presets × 5 SNRs around the S-003-2 threshold × 100 frames, plus the two R-08 cells × 1 000 frames; CS8 at 4 × BW, ±2 ppm carrier offset, 60 dB channel filter, seed 20261008; 3.4 GB) and verifies every cell with the oracle's receiver, symbol-paced and run twice; `make oracle`, `make corpus`, `make corpus-check`; conventions, the oracle's measured limits and the per-cell table in `docs/lora-corpus.md`. **Check:** two independent generations byte-identical (`iq.cs8`, `truth.json` sha256); `make corpus-check` 111 files, 0 differ; `MANIFEST.json` sha256 in `docs/resources-manifest.md`; `oracle.json` per cell, all 37 repeatable; R-08 reference cells SHORT_TURBO −5 dB 930/1 000 (PER 0.070), LONG_FAST −15 dB 947/1 000 (PER 0.053) | 2026-10-08 | c9653e8 |
| R-02 | `qrf-bus`: wire schema `crates/qrf-bus/proto/qrf/v1/qrf.proto` (package `qrf.v1`: `Header` with `power_ref`, `Spectrum`, `Occupancy`, `Bearing`, `LoraFrame`, `Scatter`, `Calibration`, `Health` with `throttled`; SPEC-009 rev 1) compiled at build time by `protox` + `prost-build`; `Publisher`/`Subscriber` over the native `zeromq` crate with `qrf.v1.<Type>/<sensor_id>` topics and `ipc:///run/qrf/<id>.pub` endpoints; `Limits`/`RateLimiter` for S-009-10; `qrf-core::time` (CLOCK_TAI, CLOCK_MONOTONIC_RAW). **Check:** `tests/r02_check.rs`: a simulated plug-in publishes all seven messages, each with a completed `Header` (seq +1 per message, both clocks, clock quality), a subscriber decodes all of them field for field over ipc; prefix subscription selects one type; 4 097 bins refused, 4 096 pass; 100 `Occupancy` in a burst pass at ≤ 10 Hz with the rest counted; `Health` limited to 1 Hz and carrying the throttle count. 12 tests pass; `make check` and `cargo audit` pass with the new dependencies | 2026-10-08 | c9653e8 |
| R-03 | `qrf-mipi`: ioctl numbers derived from direction, magic, number and argument size and verified against the vendor header compiled with gcc (18 requests, 10 struct sizes); `#[repr(C)]` argument structs; `SpanSource` ring contract with `Reader` (zero-copy spans, `Stamp` on arrival) and `LossMonitor` (S-008-3); `CsiDevice`/`DsiDevice` (mmap, poll, ioctls, JTAG wrappers; untested until P-03); de-interleave with scalar reference and NEON `vld4q_u16` kernel; `MockDevice` producing seeded four-tone frames in the vendor layout with a frame counter at the T14 cadence and the driver's drop-incoming policy; `lab/LR-009-qrf-mipi-ring-and-mock.md` with figures and architecture drawings. **Check:** `abi::tests` 18 numbers and 10 sizes equal the compiled header; `tests/r03_property.rs` NEON == scalar on 10⁶ random spans (1.02 GB); `make mipi-check` 60 s at one 131 072 B frame per 630 µs: 95 238 frames, 0 dropped, 0 counter gaps, 0 loss events, 208.0 MB/s, worst phase error 0.0028°, max backlog 43 of 63 spans, consumer 0.36 core on the VM (`resources/measurements/R-03/mipi-soak-60s.json`); NEON 40.6 GB/s vs scalar 3.2 GB/s; 17 tests; the replay of a recorded ring stays R-03b | 2026-10-08 | next commit |
| R-08 | `qrf-lora`: CSS modulator and streaming demodulator (SF 7–12, BW 125/250/500 kHz, sync 0x2B, explicit header, CRC, LDRO), written from the published PHY descriptions with every coding convention fixed by the oracle transmitter's stage vectors (`scripts/oracle-vectors.py`, `tests/r08_vectors.rs`: whitening, header checksum, CRC, Hamming, interleaver, Gray mapping, chirp indices and waveform identical to the oracle on 29 cases); receiver with multi-phase detection, up/down-chirp time and frequency estimation, refinement on aligned windows, interpolated symbol timing; `lab/LR-008-qrf-lora-modem.md`. **Check:** `make lora-check`: SHORT_TURBO −5 dB 935/1 000 (PER 0.065; oracle 930, 0.070), LONG_FAST −15 dB 988/1 000 (PER 0.012; oracle 947, 0.053); all 37 cells 4091/5500 against the oracle's 3761, at least as many as the oracle in every cell; 0 CRC verdicts inconsistent with the delivered payload (1 CRC collision, a construction limit of the LoRa CRC, LR-008 §4); `make lora-oracle-check`: the oracle decodes 336/350 frames of the Rust modulator with no wrong payload, qrf-lora all 350; 25 tests | 2026-10-08 | 01a6eda |
| R-01 | Cargo workspace at the repository root: `Cargo.toml` (resolver 3, edition 2024), `rust-toolchain.toml` pinned 1.98.1, `deny.toml` allow-list, `Cargo.lock` committed; crates `qrf-analysis` (real) and skeletons `qrf-core`, `qrf-mipi`, `qrf-jtag`, `qrf-dsp`, `qrf-lora`, `qrf-bus`, `qrf-tiled`, `qrf-sensord`, `qrf-dspd`, `qrf-overlay`, `qrf`, each `lib.rs`/`main.rs` naming the SPEC-008/009 statements it will implement; `#![forbid(unsafe_code)]` everywhere except `qrf-mipi` and `qrf-dsp`. **Check:** `cargo build --locked --release --workspace` (host triple is `aarch64-unknown-linux-musl`), `cargo test` (6 unit tests), `cargo deny check` (advisories, bans, licenses, sources ok) and `cargo audit` (12 crates, no advisories) pass; `target/` ignored | 2026-10-08 | `d35bdf1` |
| A-02 | `crates/qrf-analysis`: Rust port of `analysis/linkbudget.py`, tables T1–T23. **Check:** `make parity` — 189 lines byte-identical to the Python output on the first build | 2026-10-08 | `d35bdf1` |
| D-06 | `Makefile` with `check` (lint, links, cites, analysis, shell syntax, rust build+test, parity, deny-if-installed) | 2026-10-08 | `d35bdf1` |
| D-08 | `scripts/check-cites.py`: every S-/F.-/U-/SPEC-/Tnn reference and every cited `resources/` path resolves; 194 S, 98 F, 41 U, 23 tables, 11 specs defined; 0 unresolved | 2026-10-08 | `d35bdf1` |
| V-05 | Semtech LR1121 and SX1280 product pages filed; G04 C10 retagged `[S]`; risk register row closed | 2026-10-08 | `d35bdf1` |
| V-02 | MAX2850, MAX2871, SKY65404-31, SE5004L-EK1 and SX1261/2 datasheets imported through the UTM share (six PDFs in `resources/datasheets/`), manifest rows with sha256, T18 placeholders replaced by datasheet values, T21–T23 added | 2026-10-08 | `d35bdf1` |
| V-03 | SPEC-003 S-003-2 retagged to the SX1261/2 datasheet Table 6-1; S-003-12…15 added | 2026-10-08 | `d35bdf1` |
| D-01 | `index.md`: one page that links every document, states the two tracks and today's state | 2026-10-08 | `49d5293` |
| D-02 | `backlog.md` (this file) with phases P0–P7 and a check per entry | 2026-10-08 | `49d5293` |
| D-03 | `vendor/` ledger: policy README, 11 attribution files, 9 own-words summaries, public-domain CFR text regenerated by `scripts/vendor-cfr.py` | 2026-10-08 | `49d5293` |
| D-04 | `scripts/lint-tags.py`; 0 untagged statements across 8 gates and 11 specs | 2026-10-08 | `49d5293` |
| D-07 | Lab reports LR-001…LR-006; errata/addenda in G01–G05 and G07; specs revision 2; SPEC-008…011 | 2026-10-08 | `49d5293` |
| A-01 | `analysis/linkbudget.py` T14–T20 (data path, apertures, CRLB, LO quality, cascade NF, CPU, USB power) | 2026-10-08 | `49d5293` |
| V-01 | Manifest with commit hashes, sha256 of imported datasheets, and the list of blocked sites | 2026-10-08 | `49d5293` |
| V-01b | MAX2851 datasheet imported through the UTM share; RP1 datasheet fetched; §1.1310 and §97.13 fetched; KrakenSDR pages fetched | 2026-10-08 | `49d5293` |
| V-01c | `scripts/import-shared.sh` and `scripts/fetch-resources.sh` extended | 2026-10-08 | `49d5293` |

## Dropped

| ID | Entry | Reason |
| --- | --- | --- |
| | *nothing yet* | |
