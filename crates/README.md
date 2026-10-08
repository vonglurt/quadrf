# crates/ — the qrf Cargo workspace

<!-- SPDX-License-Identifier: MIT -->

The Rust software of SPEC-008 lives here, one crate per responsibility
(S-008-12). The workspace root is the repository root (`Cargo.toml`,
`rust-toolchain.toml` pinned to 1.98.1, `deny.toml` licence allow-list,
`Cargo.lock` committed). Build and test with `make rust`; the full gate is
`make check`.

| Crate | Responsibility (SPEC-008) | State 2026-10-08 |
| --- | --- | --- |
| `qrf-analysis` | Rust port of `analysis/linkbudget.py`, tables T1–T23; the first Rust artefact and the parity oracle for the numbers in every document | byte-identical to the Python output (`make parity`) |
| `qrf-core` | types, configuration, time (S-008-12) | skeleton (`NAME` constant and a test); real types come with R-02/R-03 |
| `qrf-mipi` | `/dev/csi_stream0` ring and ioctl bindings, de-interleave (S-008-2) | skeleton: `lib.rs` names its statements; real work is R-03 |
| `qrf-jtag` | transceiver register programming through the CSI node's JTAG ioctls (S-008-15) | skeleton: `lib.rs` names its statements; real work is R-04 |
| `qrf-dsp` | channeliser, detector, bearing, SIMD kernels (S-008-6) | skeleton: `lib.rs` names its statements; real work is R-07 |
| `qrf-lora` | LoRa CSS modulator/demodulator (S-008-6) | skeleton: `lib.rs` names its statements; real work is R-08 |
| `qrf-bus` | protobuf schema and ZeroMQ transport (SPEC-009) | skeleton: `lib.rs` names its statements; real work is R-02 |
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
