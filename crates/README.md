# crates/ — the qrf Cargo workspace

<!-- SPDX-License-Identifier: MIT -->

The Rust software of SPEC-008 lives here, one crate per responsibility
(S-008-12). The workspace root is the repository root (`Cargo.toml`,
`rust-toolchain.toml` pinned to 1.98.1, `deny.toml` licence allow-list,
`Cargo.lock` committed). Build and test with `make rust`; the full gate is
`make check`.

| Crate | Responsibility (SPEC-008) | State 2026-10-08 |
| --- | --- | --- |
| `qrf-analysis` | Rust port of `analysis/linkbudget.py`, tables T1–T25; the first Rust artefact and the parity oracle for the numbers in every document | byte-identical to the Python output (`make parity`) |
| `qrf-core` | types, configuration, time (S-008-12) | `time`: CLOCK_TAI and CLOCK_MONOTONIC_RAW readers and `Stamp` (S-009-4), with tests; configuration types come with R-02b/R-03 |
| `qrf-mipi` | `/dev/csi_stream0` ring and ioctl bindings, de-interleave (S-008-2) | R-03 done: `abi` (ioctl numbers derived and verified against the compiled vendor header; `#[repr(C)]` argument structs), `ring` (`SpanSource` contract, zero-copy `Reader` with `Stamp`, `LossMonitor`), `csi`/`dsi` (the kernel nodes; untested until P-03), `deinterleave` (scalar reference, NEON kernel, property test), `mock` (seeded four-tone frames in the vendor layout at the T14 cadence, frame counter, drop-incoming policy); `make mipi-check` is the 60-s run (`examples/mipi_soak.rs`); results in `lab/LR-009-qrf-mipi-ring-and-mock.md` |
| `qrf-jtag` | transceiver register programming through the CSI node's JTAG ioctls (S-008-15) | skeleton: `lib.rs` names its statements; the raw JTAG ioctl wrappers it will use are `qrf_mipi::CsiDevice::jtag_*`; real work is R-04 |
| `qrf-dsp` | channeliser, detector, bearing, SIMD kernels (S-008-6) | R-07 function half done: `convert` (CS8 → f32 once), `filter` (Kaiser-sinc prototype, unit DC gain), `channeliser` (M = 128, P = 8 polyphase bank; scalar reference and NEON `vfmaq_f32` filter kernels, property-tested equal; rustfft), `cfar` (US-104 slot map, CA-CFAR with the prototype's effective looks and the ratio-law threshold of T25, duty estimator), `stat` (incomplete gamma and beta), `array` (geometry, steering, plane-wave simulator), `bearing` (two-element phase differences on the 2 × 2 square with the CRLB σ), `music` (Hermitian Jacobi, MUSIC peaks, up to 3 sources); `make dsp-check` is the function record and the 60-s budget run (`examples/dsp_soak.rs`), `tests/r07_soak.rs` the 3-s standing check, `make dsp-bench` the criterion benchmarks; results in `lab/LR-010-qrf-dsp-channeliser-detector-bearing.md`; the budget half needs a Pi 5 |
| `qrf-lora` | LoRa CSS modulator/demodulator (S-008-6) | R-08 done: `coding` (whitening, header, CRC, Hamming, interleaver, Gray mapping, frame layout), `chirp`, `Modulator`, streaming `Demodulator` (multi-phase detection, up/down-chirp time and frequency estimation, aligned refinement, interpolated symbol timing, explicit header, CRC), `sim` (noise, channel filter, CS8); conformance to the oracle's stage vectors in `tests/r08_vectors.rs`; the corpus check is `make lora-check` (`examples/corpus_check.rs`), the oracle check of the modulator `make lora-oracle-check` (`examples/make_cell.rs`); results in `docs/lora-corpus.md` §6 |
| `qrf-bus` | protobuf schema and ZeroMQ transport (SPEC-009) | R-02 done: `proto/qrf/v1/qrf.proto` (compiled at build time by `protox` + `prost-build`), `Publisher`/`Subscriber` over the native `zeromq` crate, topic strings, `Limits`/`RateLimiter` (S-009-10); check in `tests/r02_check.rs`; examples `sim_plugin` and `bus_dump` |
| `qrf-tiled`, `qrf-sensord`, `qrf-dspd`, `qrf-overlay`, `qrf` | the processes and CLI (S-008-1) | skeletons that identify themselves and exit; real work is R-10, R-02b, R-09 |
| `qrf-node-pico` | RP2040 field-node firmware, separate target (S-008-11) | not started (F-06); lives outside this workspace because of its target |

Rules: edition 2024, stable toolchain, no GPL dependencies in these crates
(`cargo deny check`), `unsafe` only in `qrf-mipi` and in SIMD leaf kernels
with scalar references and property tests (S-008-5).

## Why the analysis crate must match the Python script byte for byte

The documents cite table numbers, not values, so the two implementations must
print the same digits or one of them is wrong. The port keeps Python's
evaluation order and calls the same libm functions (`pow`, `log10`, `sqrt`,
`acos`, `asin`, `cos`) so that IEEE-754 results agree; Python's `e` format is
reproduced explicitly. When a table is added to the Python script it is added
to `crates/qrf-analysis/src/main.rs` in the same change, and `make parity`
is the check.
