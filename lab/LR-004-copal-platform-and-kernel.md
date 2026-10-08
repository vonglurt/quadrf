# LR-004 — Running the tile on copal: platform port and kernel tuning plan

*Lab report. 2026-10-08.*

<!-- SPDX-License-Identifier: MIT -->

| Field | Value |
| --- | --- |
| Status | Final (desk); every tuning claim is an option to be measured |
| Author | project |
| Feeds | SPEC-010; backlog P-01…P-08 |

## Abstract

The vendor stack runs on DietPi (Debian Trixie, systemd, DKMS, apt, glibc).
copal is the user's installer for stock Alpine Linux (OpenRC, apk, musl) on
Raspberry Pi hardware, and the user wants the tile there with custom kernel
work where it buys determinism. This report maps every vendor dependency to
its Alpine counterpart from the package index (linux-rpi 6.18.52-r0,
linux-rpi-dev, akms 0.3.0, openocd 0.12.0, soapy-sdr 0.8.1, rust/cargo
1.96.1), states the two unknowns that decide whether the vendor's GPL-2.0
modules build unmodified against 6.18, and lists the kernel tuning options
with the measurement that admits each. The port is feasible with no
proprietary piece beyond the bitstream, which the vendor allows to be
redistributed and which is copied from the kit's card.

## I. Objective

1. Map the vendor's Debian stack to copal/Alpine, package by package.
2. Define the bring-up sequence on copal with a check per step.
3. List kernel options (affinity, isolation, PREEMPT_RT, governor, memory) and the measurement that decides each.

## II. Materials

| Item | Detail |
| --- | --- |
| copal | github.com/copallinux/copal: eighteen re-runnable stages, `playbooks/` catalogue (`# build: git rust cargo` headers), `copal-build` compiles cargo checkouts in `~/code`, signed installer (`ssh-keygen -Y`), Pi 1–5 and PC targets; MIT |
| Alpine index (2026-10-08) | `vendor/alpine/ATTRIBUTION.md` |
| Vendor boot and load | SPEC-002 rev 2 S-002-18/19/20; `sources/boot/10-boot`; `sources/fpga/quadrf-load`; `fpga-csi.dts` |
| RP1 | `vendor/summary/raspberry-pi-rp1.md`: PCIe 2.0 x4, QoS prioritisation of camera traffic, 1 µs link latency |
| Development VM | Alpine 3.24.1, kernel 6.18.52-0-lts (same version line as the Pi kernel) |

## III. Method

Each vendor component was classified as kernel, boot, service, build
dependency, or application; for each the Alpine package or the qrf
replacement was identified from the index; unknowns were written as
backlog checks rather than assumptions.

## IV. Results

### A. Mapping

| Vendor component | On DietPi | On copal | Status |
| --- | --- | --- | --- |
| Kernel | Pi OS 6.12 tree, `linux-headers-rpi-2712` | `linux-rpi` 6.18.52-r0 + `linux-rpi-dev` | `[S]` index; API drift `[C]` U-010-1 |
| `fpga-csi`, `fpga-dsi` modules | DKMS | `akms` (same sources, same Makefiles) | `[D]`; compat strings `brcm,rp1-csi` vs in-tree RP1 camera driver `[C]` U-010-2 |
| Overlays | `/boot/firmware/overlays/*.dtbo` via `10-boot` | same files on the boot partition; `dtc` from Alpine | `[D]` |
| `config.txt` | vendor block | `dtoverlay=fpga-csi`, `dtoverlay=fpga-dsi`, I²C, `temp_limit=75`; copal keeps its own video lines | `[D]` SPEC-010 S-010-3 |
| FPGA load | OpenOCD + `quadrf.svf` from the vendor package | `openocd` 0.12.0-r7; SVF copied from the kit card | `[S]` index; `[S]` redistribution allowed |
| Transceiver init | `quadrf-jtag --init` (GPL C) | same binary rebuilt from source as a child of `qrf-load` until R-04 | `[D]` |
| SoapySDR module | `libmipi.so` into SoapySDR 0.8 | `soapy-sdr` 0.8.1-r4; module rebuilt only for parity tests | `[S]`; musl build `[C]` U-010-5 |
| Services | systemd units | OpenRC scripts `qrf-load`, `qrf-tiled`, `qrf-dspd`, `qrf-sensord`, `qrf-overlay` | `[D]` |
| GUI / desktop | Flask + KasmVNC | `qrf-overlay` web page; Hyprland is copal's own desktop | `[D]` |
| Network | nginx, dnsmasq, hostapd AP, USB gadget | copal networking; gadget optional; no AP | `[D]` |
| GNU Radio | Debian packages | `gnuradio` 3.10.12 available for bench use only | `[S]` |
| Mesh | `quadrf-mesh` Debian package | built from source as a GPL plug-in process if used | `[D]` |

### B. Bring-up sequence (each step is a backlog entry with its check)

1. P-01 copal image for Pi 5 with the apk set installed by a playbook; check: `apk info` lists them and `uname -r` is 6.18.52 or later.
2. P-02 device manager identified (mdev/udev/eudev) and `qrf` group rules written; check: `/dev/csi_stream0` is `root:qrf 0660` after module load (U-010-4).
3. P-03 modules built with akms; check: `modprobe fpga-dsi fpga-csi` succeeds, `dmesg` shows the CSI probe on `csi1`, `/dev/csi_stream0` and `/dev/dsi_stream0` exist (U-010-1/2).
4. P-04 `qrf-load` OpenRC service programs the FPGA and initialises the transceivers; check: `csi_stats.frame_count` increases at rest with `interleave=1`.
5. P-05 lossless capture: 60 s at 4 × 26 MSPS with zero loss events; the copy workqueue's CPU share measured with `top -H`/`perf` (U-002-4).
6. P-06 tuning options (§C) each measured against P-05's numbers.
7. P-07 PREEMPT_RT build measured the same way.
8. P-08 thermal: throttle counter zero over a 10-minute capture with the chosen cooling.

### C. Kernel tuning options and how each is admitted

| Option | Expected effect | Admission measurement |
| --- | --- | --- |
| `cpufreq` performance governor | removes frequency ramp latency at burst start | loss events and 99.99-percentile consumer latency (`tracing` timestamps) over 10 min |
| IRQ affinity: PCIe/RP1 MSIs + CSI copy workqueue on core 1; DSP on 2–3; housekeeping on 0 | isolates the 208 MB/s copy from the DSP threads | same, plus per-core `mpstat` |
| `isolcpus=2,3 nohz_full=2,3 rcu_nocbs=2,3` | no scheduler ticks on DSP cores | cyclictest on cores 2–3 vs baseline; cost: those cores are unavailable to everything else on a 4-core board |
| PREEMPT_RT (`CONFIG_PREEMPT_RT=y` via aports `common-changes.config`) | bounded wake latency for the SCHED_FIFO consumer | cyclictest 99.99 % and loss events; throughput of the channeliser must not fall by > 10 % |
| zram off during capture | avoids compression bursts on the housekeeping core | memory pressure test with the 1 s history enabled |
| THP `never` | avoids compaction stalls | stall counters in `/proc/vmstat` |
| CMA | not needed: 16 × 131 072 B = 2 MiB of DMA buffers | `[D]` from `fpga-csi.c` DMA_BUF_COUNT and the frame geometry |

Mainline Linux has carried PREEMPT_RT since 6.12 and Alpine's `linux-rpi`
is 6.18, so an RT variant is a configuration change and a rebuild, not a
patch set; whether the RP1 PCIe/MIPI path behaves under RT is U-010-3.
`[S]` (secondary reports of the 6.12 merge) + `[C]`

### D. What copal already gives

- A signed, re-runnable installer and a playbook format with build metadata; `copal-build` already compiles cargo checkouts in `~/code` and installs their binaries, which is exactly how the qrf binaries would reach `~/.local/bin` on a developer machine. `[S]` (copal README, `playbooks/Code/*.sh` headers)
- Hyprland/Wayland desktop, so a native GUI is unnecessary; the overlay page works in copal's browser or from any laptop on the LAN. `[S]` (copal README)

## V. Discussion

The two unknowns that can stop the port are both in P-03: whether 6.18
changed the RP1 CSI/DSI kernel interfaces the vendor modules use (they were
written against the Pi OS 6.12 tree), and whether Alpine's kernel binds the
in-tree RP1 camera front-end driver to `csi1` before the vendor overlay can.
Both are answered in an afternoon with the kit; a build failure is fixed by
a small published patch, a binding conflict by a module blacklist. Nothing
else in the vendor stack is needed on copal.

The tuning options are deliberately not promised: on a four-core board,
isolating two cores for DSP leaves the overlay, the bus and the OS on two,
and the measured loss-event count decides whether that trade is worth it.

## VI. Recommendations and best practice

1. Keep the vendor modules unmodified and build them with akms; publish any patch beside the build (GPL-2.0 obligation and reproducibility).
2. Write the OpenRC services to report device presence and loss counters in `status`, so a field operator can see at a glance that capture is lossless.
3. Treat every kernel tuning as an experiment with a number, recorded as an `M-nnn` measurement record.
4. Put the copal playbook in the copal repository and reference its commit from `backlog.md`.

## VII. References

SPEC-002 rev 2, SPEC-010; `vendor/alpine/ATTRIBUTION.md`; `vendor/summary/scalerf-quadrf.md`; `vendor/summary/raspberry-pi-rp1.md`; copal `README.md`, `docs/copal-handbook.md`; `analysis/linkbudget.py` T14.
