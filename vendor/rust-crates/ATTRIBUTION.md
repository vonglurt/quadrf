# Rust crates we intend to depend on

Dependencies are fetched by Cargo from crates.io and pinned in `Cargo.lock`;
nothing is copied here. `cargo deny` will enforce the licence allow-list
(MIT, Apache-2.0, BSD-2/3-Clause, ISC, Zlib, Unicode-3.0, MPL-2.0 for
unmodified use) and reject GPL crates in MIT binaries.

| Crate | Version seen 2026-10-08 (crates.io API) | Licence | Role |
| --- | --- | --- | --- |
| `nusb` | 0.2.7 | MIT OR Apache-2.0 | pure-Rust USB access for SDR dongles and LoRa sticks |
| `rtlsdr-nusb` | 0.3.0 | per crate (verify) | Rust-native RTL-SDR driver |
| `seify` | 0.26.0 | Apache-2.0 | SDR hardware abstraction (SoapySDR backend optional) |
| `soapysdr` / `soapysdr-sys` | 0.5.1 / 0.8.1 | MIT OR Apache-2.0 (bindings) to libSoapySDR (Boost-1.0) | bridge to the vendor `mipi` module for parity tests |
| `futuresdr` | 0.9.0 | Apache-2.0 | async flowgraph runtime (evaluation only; nightly toolchain) |
| `rustradio` | 0.18.8 | MIT | DSP block library (evaluation) |
| `rustfft` / `realfft` | 6.4.1 (in use since R-08) / 3.5.0 | MIT OR Apache-2.0 | FFTs for the channeliser and LoRa dechirp (`qrf-lora` dechirps with it; transitive `primal-check` 0.3.4, `strength_reduce` 0.2.4, `transpose` 0.2.3, `num-integer`, `num-traits`) |
| `num-complex`, `ndarray` | 0.4.6 (in use since R-08) / 0.17.2 | MIT OR Apache-2.0 | numerics |
| `wide`, `pulp` | 1.7.1 / 0.22.3 | Zlib OR Apache-2.0 OR MIT; MIT | portable SIMD (NEON) |
| `tokio`, `axum`, `tokio-tungstenite` | 1.53.2 / 0.8.9 / 0.30.0 | MIT | async runtime, HTTP, WebSocket for the overlay |
| `zeromq` | 0.6.0 (in use since R-02) | MIT | native-Rust ZeroMQ for the feed bus (vendor tools speak ZeroMQ) |
| `prost` | 0.14.4 (in use since R-02) | Apache-2.0 | protobuf schema for bus messages |
| `prost-build`, `protox` | 0.14.4 / 0.10.0 (build-dependencies since R-02) | Apache-2.0; MIT OR Apache-2.0 | compile `qrf.proto` at build time without a system `protoc` |
| `bytes` | 1.12.1 (in use since R-02) | MIT | message frames |
| `serde`, `serde_json` | 1.0.229 / 1.0.151 (in use since R-08 for the corpus sidecars and test vectors; `serde_json` brings `zmij` 1.0.23, `itoa`, `memchr`) | MIT OR Apache-2.0 | configuration files, JSON sidecars |
| `memmap2`, `nix`, `libc` | 0.9.11 / 0.31.3 (`nix` in use since R-02 for CLOCK_TAI) / 0.2.190 | MIT OR Apache-2.0 | ring mmap and ioctls on `/dev/csi_stream0`; the two clocks of S-009-4 |
| `gpiocdev`, `spidev`, `linux-embedded-hal` | 0.8.0 / 0.7.1 / 0.5.0 | MIT OR Apache-2.0 | GPIO and SPI for a HAT radio |
| `sx1262` | 0.3.0 | per crate (verify) | SX1262 driver for a host-attached HAT or stick |
| `lora-phy`, `embassy-rp` | 3.0.1 / 0.10.0 | MIT OR Apache-2.0 | Pico (RP2040) field-node firmware |
| `tokio-serial`, `nmea` | 5.5.0 / 0.8.0 | MIT; Apache-2.0 | GNSS time and position |
| `clap`, `tracing`, `anyhow`, `thiserror` | 4.6.7 / 0.1.44 / 1.0.104 / 2.0.21 | MIT OR Apache-2.0 | CLI, logs, errors |
| `criterion` | 0.8.2 | MIT OR Apache-2.0 | benchmarks |
| `cargo-deny`, `cargo-audit`, `cargo-vet` | 0.20.2 / 0.22.2 / 0.10.2 | MIT OR Apache-2.0 | supply-chain checks |
| `meshtastic` | 0.1.9 | **GPL-3.0** | Meshtastic client; usable only in a separate GPL-licensed plugin process |
| `librtlsdr-rs` | — | **GPL-2.0** | not used |

Licence columns marked "verify" are confirmed by `cargo deny` at the first
build that uses the crate; until then they are `[C]`. The R-02 build
(2026-10-08) brought in `zeromq`, `prost`, `prost-build`, `protox`, `bytes`,
`thiserror`, `tokio` and `nix` with their transitive closure, and
`cargo deny check` passed the allow-list on it. The R-08 build (2026-10-08)
added `rustfft`, `num-complex`, `serde` and `serde_json` with theirs, and
`cargo deny check` passed again.
