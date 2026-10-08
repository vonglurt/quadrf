# Scale RF Inc. — QuadRF: restatement of the facts we rely on

Attribution: `vendor/scalerf/ATTRIBUTION.md`. Sources: repository
`open-space-sdr/main` at commit `8b61ae5` (2026-10-07), schematic export
2026-05-22, web pages retrieved 2026-10-08. Facts are `[S]` unless marked.

## Product

- A 4-channel coherent, full-duplex 4.9–6.0 GHz SDR tile (RF board + 4-element circular-patch antenna module + Lattice ECP5 FPGA) that attaches to a Raspberry Pi 5 through the CSI-2 (receive) and DSI (transmit) flat cables, with control over bit-banged JTAG on GPIO 14/15/18/23 (22 optional). (`README.md`; `docs/overview.md`; `fpga-csi.dts` lines 28–31)
- Converters: MAX2850 (4-channel transmitter) and MAX2851 (5-channel receiver), 4.9–5.9 GHz design range; PA SE5004L ×4 (1 W class); LNA SKY65404-31; eight RF SPDT switches for polarisation. (BOM in `docs.html`; schematic designator histogram: SE5004L ×5 mentions, MAX2850 ×3, MAX2851 ×2)
- FPGA: schematic lists `LFE5U-25F-6BG256C`; the OpenOCD board file and tap name are `rpi5_lfe5u45f` / `lfe5u45f.tap`. The discrepancy is unresolved (SPEC-001 U-001-6). `[C]`
- Vendor-stated performance: 40 MHz instantaneous bandwidth per element, 8-bit I/Q (7-bit ENOB beamformed), ~1.2 dB receive NF, 70 dB gain range, ~1.4 ps inter-element jitter, < 10 ms RF-to-software latency, 5.6 Gbit/s aggregate FFC data. (`docs.html`)

## Kernel drivers (GPL-2.0)

- `fpga-csi`: RP1 CSI-2 RAW8/RAW10 DMA character device `/dev/csi_stream0`. A single CSI2-DMA channel with 16 DMA buffers (`DMA_BUF_COUNT 16`) cycling FREE → QUEUED → ACTIVE → COPY_PENDING; completed buffers are copied by an ordered high-priority workqueue into a vmalloc-backed ring that userspace maps with `mmap()`; `CSI_IOC_GET_RING_INFO` returns `ring_size`, `span_bytes`, `head`, `tail`; `CSI_IOC_CONSUME_BYTES` advances `tail`; `poll()` signals readiness; module parameter `drop_oldest`. (`fpga-csi.c` lines 14–25, 64, 204–211, 539–541; `fpga_csi.h`)
- Device-tree overlay `fpga-csi.dts`: targets `csi1` (CAM/DISP 1), RAW8 data type 0x2a, 4 data lanes, D-PHY tuning for a 350 MHz DDR source (700 Mbit/s per lane), frame geometry 1024 bytes × 128 lines.
- JTAG register access through the same device: `CSI_IOC_JTAG_SETUP/RELEASE`, `REG_WRITE/REG_READ` (8-bit address, 16-bit value), `BATCH_WRITE` with inter-write delay, and an `ACQUIRE_LEASE/RELEASE_LEASE` pair with a 100 ms lock timeout. (`fpga_csi.h`; `jtag.c` 547–568)
- `fpga-dsi`: DSI byte-stream "panel" `/dev/dsi_stream0`; `DSI_IOC_GET_FB_INFO` (bytes per frame, frame count), `mmap()` of a staging area, `DSI_IOC_QUEUE_NEXT`; a flip thread copies staging to the scanout framebuffer. (`fpga-dsi.c` lines 6, 71–88)

## SoapySDR module `mipi` (GPL-2.0 per vendor table; GPL-3.0 per `debian/copyright`)

- Opens both devices, maps the ring and the staging area, offers CS8 (native) and CF32, a polynomial Farrow resampler and NEON pack/unpack/de-interleave. Interleaved 4-channel data is stored as consecutive 16-bit (I,Q) pairs per element, element 0..3 repeating (`vld4q_s16`). (`MipiDevice.cpp` 113–180, 493–503, 1948–2062; `NEON.cpp` 56–82)
- Limits: LO 4.9–6.0 GHz; analog receive bandwidth 240/k MHz, k = 5…63 (3.81–48 MHz); host rate 1–200 MSPS accepted by the driver; device args `antennas`, `p1..p4`, `agc_setpoint`, `interleave`, `autosteer`, `pol`. (`MipiDevice.cpp` 183–189, 305–330; `jtag.c` 671–675)

## `quadrf-jtag` CLI (GPL)

Keys per direction: `freq` (MHz, rejected outside 4900–6000), `bw` (tx 20 or 40; rx any positive, quantised to 240/k), `gain` (tx 0–63; rx signed), `antennas`/`channels` (mask 0–15), `agc` (dBFS, rx only), `pol` (rhcp/lhcp, rx only), `interleave` (rx only), `autosteer`, `tx_follow_rx`, `tone_en`, `tone_freq`, `p1`…`p4` (degrees). `--init` runs MAX2850 and MAX2851 initialisation after a DSI link calibration. (`jtag.c` 231–321, 576–587)

## Boot and services (Debian/DietPi image)

- `config.txt` block written by `10-boot`: 1920×1080 framebuffer, `enable_uart=0`, `temp_limit=75`, `dtparam=i2c_arm=on`, `dtoverlay=vc4-kms-v3d,noaudio`, `dtoverlay=fpga-csi`, `dtoverlay=fpga-dsi`, `dtoverlay=dwc2,dr_mode=peripheral`; `cmdline.txt` gains `modules-load=dwc2,g_ether` and loses any serial console.
- `quadrf-load`: unload `fpga_csi`, program the ECP5 with OpenOCD (`svf -tap lfe5u45f.tap … quadrf.svf`), `modprobe fpga-dsi`, `modprobe fpga-csi`, then `quadrf-jtag --init`.
- Build dependencies: DKMS with `linux-headers-rpi-2712`, cmake, libfftw3, SDL2, libzmq, device-tree-compiler; SoapySDR 0.8 module directory. (`docs/develop.md`)

## Licensing (from the updates page FAQ and `debian/copyright`)

Drivers, SoapySDR module, ZeroMQ modules, RF visualisation, NTSC decoder, spectrum analyser, web GUI and CLI: GPL-2.0; GNU Radio demos: GPL-3.0; packaging default GPL-3.0; "Agentic RF generated code: not copyrighted"; ECP5 bitstream: all rights reserved, redistribution allowed, not published; antenna/mechanical files: CC-BY-SA-4.0 plus patent covenant; RF board/core: all rights reserved, patent pending; components marked CC BY-NC or proprietary are for personal, educational or internal non-profit research use; commercial licences offered.
