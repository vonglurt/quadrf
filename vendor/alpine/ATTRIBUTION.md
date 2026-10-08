# Alpine Linux packages used on copal

copal (the user's own installer, MIT, github.com/copallinux/copal) is an
aggregation of stock Alpine Linux. The packages below are Alpine's builds of
upstream projects, each under its upstream licence; nothing from them is
copied into this repository. Versions are from the Alpine package index as
seen on the development VM on 2026-10-08 (Alpine 3.24).

| Package | Version | Upstream licence | Role |
| --- | --- | --- | --- |
| `linux-rpi`, `linux-rpi-dev` | 6.18.52-r0 (v3.24), 6.18.55-r0 (edge) | GPL-2.0-only | Pi 5 kernel and headers for building `fpga-csi`/`fpga-dsi` |
| `akms` | 0.3.0-r0 | MIT | Alpine kernel module build service (the DKMS analogue) |
| `openocd` | 0.12.0-r7 | GPL-2.0-or-later | loads the ECP5 bitstream over GPIO JTAG |
| `soapy-sdr` | 0.8.1-r4 | BSL-1.0 | SoapySDR 0.8 ABI for the vendor `mipi` module |
| `rtl-sdr` | 2.0.2-r1 | GPL-2.0-or-later | RTL-SDR dongles (optional; Rust path uses `nusb`) |
| `hackrf` | 2026.01.3-r1 | GPL-2.0 | HackRF One (optional wideband sweep) |
| `gnuradio`, `gr-osmosdr` | 3.10.12.0-r12, 0.2.6-r5 | GPL-3.0 | bench reference tooling only |
| `fftw` | 3.3.11-r0 | GPL-2.0-or-later | used by vendor demos only |
| `zeromq` | 4.3.5-r2 | MPL-2.0 (verify) | bus transport for vendor tools; our Rust uses the native `zeromq` crate |
| `liquid-dsp` | 1.5.0-r0 | MIT | optional C DSP reference |
| `rust`, `cargo` | 1.96.1-r0 | MIT OR Apache-2.0 | toolchain (rustup 1.98.1 is also available) |
| `python3` | 3.14.8-r0 | PSF-2.0 | `analysis/` and `scripts/` |
| `poppler-utils` | 25.12.0-r1 | GPL-2.0-or-later | `pdftotext` for datasheets |
| `gpsd`, `chrony`, `linuxptp` | 3.27.3-r1, 4.8-r7, 4.4-r0 | BSD-2-Clause; GPL-2.0; GPL-2.0 | time base for multi-sensor feeds |
