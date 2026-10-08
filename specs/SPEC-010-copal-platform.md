# SPEC-010 — Platform: the QuadRF tile and the qrf software on copal (Alpine Linux, Raspberry Pi 5)

| Field | Value |
| --- | --- |
| Status | Draft |
| Revision | 0 |
| Date | 2026-10-08 |
| Subject | The operating-system layer this project targets instead of the vendor's DietPi image: kernel, modules, boot configuration, services, tuning options, packaging and security on a copal-installed Alpine Linux Raspberry Pi 5 |
| Primary sources | SPEC-002 rev 2; `vendor/alpine/ATTRIBUTION.md` (package versions from the Alpine index, 2026-10-08); copal `README.md` and `docs/copal-handbook.md` (github.com/copallinux/copal); `resources/datasheets/RP1-peripherals.pdf`; `lab/LR-004` |
| Depends on | SPEC-002, SPEC-008 |

## Scope

Everything between the hardware and the qrf processes. Statements are `[S]`
platform facts or `[D]` decisions. Not covered: the vendor's Debian packaging
(SPEC-002), the application code (SPEC-008).

## Behaviour-goal statements

Operating system:

- S-010-1. The target is Alpine Linux 3.24 aarch64 installed by copal on a Raspberry Pi 5: `linux-rpi` 6.18.52-r0 (6.18.55 on edge), musl libc, OpenRC, apk; the vendor's Debian packages are not installed. `[S]`+`[D]` (vendor/alpine/ATTRIBUTION.md; copal README)
- S-010-2. The two vendor kernel modules are built unmodified from their GPL-2.0 sources against `linux-rpi-dev` with `akms` (Alpine's kernel-module build service), their device-tree overlays compiled with `dtc`, and the overlays installed to the boot partition; a local patch, if any is needed for 6.18, is published with the build. `[D]`+`[S]` (SPEC-002 S-002-18/20; akms 0.3.0 in the index)
- S-010-3. `config.txt` carries `dtoverlay=fpga-csi`, `dtoverlay=fpga-dsi`, `dtparam=i2c_arm=on` and `temp_limit=75`; the vendor's framebuffer, KMS and USB-gadget lines are applied only when copal's own configuration does not already set them; `enable_uart` stays as copal sets it because the tile uses GPIO 14/15 for JTAG, not UART. `[D]` (SPEC-002 S-002-18; SPEC-001 S-001-22)
- S-010-4. The FPGA bitstream `quadrf.svf` is copied by the operator from the kit's SD card (`/usr/share/quadrf/fpga/quadrf.svf`) to the same path on copal; it is never committed; OpenOCD 0.12 from Alpine programs it through the bit-banged GPIO JTAG using a board configuration written by this project. `[S]`+`[D]` (SPEC-002 S-002-19; SPEC-001 S-001-36)
- S-010-5. OpenRC services: `qrf-load` (root; unload `fpga_csi`, OpenOCD SVF, `modprobe fpga-dsi`, `modprobe fpga-csi`, transceiver initialisation), then `qrf-tiled`, `qrf-dspd`, `qrf-sensord`, `qrf-overlay` as user `qrf` with `need qrf-load`; each has a `status` that reports device presence and the loss counters. `[D]` (SPEC-002 S-002-19; SPEC-008 S-008-1)

Tuning (each an option with a measured check in the backlog, none assumed):

- S-010-6. (a) the `performance` cpufreq governor during capture; (b) IRQ affinity placing RP1/PCIe interrupts and the CSI copy workqueue on core 1, DSP threads on cores 2–3, housekeeping on core 0; (c) `isolcpus=2,3 nohz_full=2,3 rcu_nocbs=2,3`; (d) a PREEMPT_RT build of `linux-rpi` 6.18 (`CONFIG_PREEMPT_RT=y` through the aports `common-changes.config` mechanism; RT is mainline since 6.12); (e) zram swap disabled during capture; (f) transparent hugepages `never`. Each option is adopted only if it lowers the measured loss-event count or the cyclictest 99.99-percentile latency without raising the channeliser's CPU time. `[D]`+`[S]` (RP1 datasheet QoS note; kernel 6.12 RT merge reported in secondary sources, `[C]` until the config is built)
- S-010-7. Sustained four-core DSP at 2.4 GHz requires active cooling; the thermal-throttle counter must read zero over a 10-minute 4 × 26 MSPS capture, else the tuning option or the cooling is changed. `[D]` (vendor `temp_limit=75`)

Network and security:

- S-010-8. The overlay is reachable over Ethernet or the USB gadget interface only; the vendor's fallback open Wi-Fi access point is not reproduced; SSH is key-only as copal configures it; the firewall denies inbound except SSH and the overlay port on the chosen interface. `[D]`
- S-010-9. A dedicated user and group `qrf` own the device nodes (`/dev/csi_stream0`, `/dev/dsi_stream0`, USB SDRs, serial GNSS) through device-manager rules; no DSP process runs as root; the transmit unlock file is root-owned and readable by `qrf`. `[D]` (SPEC-008 S-008-9)
- S-010-10. Installation is a copal playbook (`playbooks/Engineering/quadrf.sh` in the copal repository) that installs the apk dependencies (`openocd`, `soapy-sdr`, `linux-rpi-dev`, `akms`, `zeromq`, `rust`/`cargo`, `gpsd`, `chrony`), builds the modules with akms, builds the Cargo workspace, installs the OpenRC services and records versions in `/etc/copal`; this repository documents it and keeps the service files. `[D]` (copal playbook convention)
- S-010-11. Build hosts: the workspace cross-compiles from the development VM (`aarch64-unknown-linux-musl` host) and builds natively on the Pi; a release is reproducible from a tagged commit and the locked dependency set, and is signed. `[D]` (SPEC-008 S-008-13)

## Interfaces we depend on

- Alpine `linux-rpi` headers and `akms`; `config.txt`/`cmdline.txt` on the boot partition; OpenRC; the device manager's rule syntax.

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-010-1 | Whether the vendor modules (written against the Pi OS 6.12 tree) build and run unmodified against Alpine's `linux-rpi` 6.18.52 | backlog P-03 |
| U-010-2 | Whether Alpine's `linux-rpi` configuration enables the RP1 CSI/DSI front-end the vendor overlay targets (`csi1`, compatible `brcm,rp1-csi`) and whether the in-tree RP1 camera driver must be blacklisted | backlog P-03 |
| U-010-3 | Behaviour of the RP1 PCIe/MIPI path under PREEMPT_RT | backlog P-07 |
| U-010-4 | Whether copal's Pi 5 image runs mdev, udev or eudev (affects S-010-9 and SPEC-009 U-009-2) | backlog P-02 |
| U-010-5 | musl-specific build issues when the vendor SoapySDR module is rebuilt for parity tests | backlog R-05 |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 0 | 2026-10-08 | Draft |
