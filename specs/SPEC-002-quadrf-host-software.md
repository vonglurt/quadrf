# SPEC-002 — QuadRF host software stack on Raspberry Pi 5

| Field | Value |
| --- | --- |
| Status | Reviewed (rev 1); rev 2 additions S-002-14…22 await a second read |
| Revision | 2 |
| Date | 2026-10-08 |
| Subject | The Linux software the vendor image runs on the Pi 5: services, packages, interfaces |
| Primary sources | `resources/repos/quadrf-open-space-sdr/docs/overview.md`, `docs/applications.md`, `docs/develop.md`, `install/README.md` |
| Depends on | SPEC-001 |

## Scope

What runs, how it is layered, what ports and files it exposes, what is
replaceable. Not covered: the vendor web UI internals.

## Behaviour-goal statements

- S-002-1. The base OS is DietPi (Debian "Trixie") for Raspberry Pi 5, arm64; the vendor stack installs as Debian packages from a vendor APT repository. `[S]` (install/README.md)
- S-002-2. Two metapackages exist: `quadrf` (full: desktop, GNU Radio, demos, mesh) and `quadrf-headless` (drivers, SoapySDR, web panel, network). `[S]` (overview.md §2)
- S-002-3. Boot sequence: `load-quadrf.service` programs the ECP5 via OpenOCD over GPIO JTAG, loads the `fpga-csi` and `fpga-dsi` DKMS kernel modules, and initialises the transceivers with `quadrf-jtag --init`. If the RF board is absent, networking and web services still start. `[S]` (overview.md §4)
- S-002-4. The SDR API is SoapySDR with the vendor module `libmipi.so` (`driver=mipi`); a SoapyRemote server on TCP 55132 exposes the same device to the LAN. `[S]` (overview.md; docs.html API section)
- S-002-5. GNU Radio Companion is installed with QuadRF source/sink blocks; ZeroMQ streaming is supported. `[S]` (overview.md; sources/grc_blocks/)
- S-002-6. Web surfaces: control panel at `https://quadrf.local/` (Flask on 8080 behind nginx), remote desktop (KasmVNC, display `:1`, port 8444 / 6080), AR view `/AR/`, vision stream `/ws` (port 8000), PhaseGaze `/phasegaze/` (port 8001). `[S]` (overview.md §3)
- S-002-7. Network: mDNS `quadrf.local`, direct-Ethernet 10.55.1.1 with DHCP server, USB-gadget 10.55.0.1, fallback open Wi-Fi AP `QuadRF` at 192.168.44.1. `[S]` (overview.md §3)
- S-002-8. Applications register through a `.desktop` file, a systemd unit and a JSON catalogue entry; `exclusive: true` apps stop other radio apps because the CSI/DSI device nodes are single-owner. `[S]` (applications.md)
- S-002-9. Included applications: Spatial RF Vision (swept-LO, 30 fps), NTSC decoder/encoder, PSD plot (1–4 ch), near-field 4×4 phasors with TDM Tx cycling, 802.11 OFDM TUN link between two tiles, PhaseGaze (38 MSPS, 4-lane CSI), QuadRF Mesh (Meshtastic daemon + LoRa-compatible PHY), GNU Radio Companion, QRadioLink. `[S]` (applications.md §1)
- S-002-10. The `quadrf` metapackage pulls `quadrf-mesh`, which installs `quadrf-lora-phy` and `quadrf-meshtasticd` from the third-party `radioroy/quadrf-mesh` release, pinned by SHA-256. `[S]` (overview.md §2 table; packaging/thirdparty/pin-quadrf-mesh.sh)
- S-002-11. Appliance configuration lives in `/etc/quadrf/quadrf.conf` (includes `CALLSIGN`, default `NOCALL`); `sudo quadrf apply` regenerates dependent configs. `[S]` (overview.md §5)
- S-002-12. Drivers, the Soapy module, the JTAG utility and demos ship as source under `/usr/src/` and rebuild with `make`; the ECP5 bitstream `quadrf.svf` is redistributable but closed. `[S]` (develop.md §3; updates.html FAQ)
- S-002-13. A SPI0-attached LoRa HAT needs GPIO 7–11 (SPI0) plus CS/IRQ/BUSY/RESET lines; the tile's JTAG occupies GPIO 14, 15, 18, 23 (22 optional). GPIO 18 and the UART0 pair 14/15 are therefore unavailable to a HAT while the tile is attached. `[S]`+`[D]` (SPEC-001 S-001-22; Pi 5 pinout)

### Device nodes, kernel interfaces and bring-up (revision 2)

- S-002-14. `/dev/csi_stream0` exposes a read-only `mmap()` ring that the driver's ordered high-priority copy workqueue fills from 16 DMA buffers; `CSI_IOC_GET_RING_INFO` returns ring size, span size, head and tail, `CSI_IOC_CONSUME_BYTES` advances the tail, `poll()` reports readiness, a `read()` path also exists, and the `drop_oldest` module parameter selects the overflow policy. `[S]` (fpga_csi.h; fpga-csi.c lines 14–25, 64, 204, 539–541; MipiDevice.cpp lines 113–139, 1575–1636)
- S-002-15. `CSI_IOC_GET_STATS` and `CSI_IOC_GET_EVENTS` report DMA bytes, bytes read, CSI overflow IRQs, per-virtual-channel discards, ring overflows, frame-start/frame-end IRQ counts, inferred frame ends, channel recoveries and buffer-starvation counts, which is the observability a capture daemon needs to prove lossless capture. `[S]` (fpga_csi.h `csi_stats`, `csi_event_stats`)
- S-002-16. FPGA register access goes through the same node: JTAG setup/release, 8-bit address / 16-bit value register read and write, batched writes with a per-write microsecond delay, and a lease with a 100 ms acquisition timeout; one process holds the lease at a time. `[S]` (fpga_csi.h ioctls 0x10–0x16; jtag.c lines 547–568)
- S-002-17. `/dev/dsi_stream0` exposes `DSI_IOC_GET_FB_INFO` (bytes per frame, frame count, 4 by default in the client), a writable `mmap()` staging area and `DSI_IOC_QUEUE_NEXT`; a kernel flip thread copies staging frames to the scan-out buffer. `[S]` (MipiDevice.hpp lines 99–117; fpga-dsi.c lines 6, 71–88; MipiDevice.cpp lines 141–180)
- S-002-18. The vendor boot hook writes a `config.txt` block (1920×1080 framebuffer, `enable_uart=0`, `temp_limit=75`, I²C on, `vc4-kms-v3d,noaudio`, `fpga-csi`, `fpga-dsi`, `dwc2` in peripheral mode), adds `modules-load=dwc2,g_ether` to `cmdline.txt`, removes serial consoles, and installs the overlays to the firmware partition's `overlays/` directory. `[S]` (sources/boot/10-boot)
- S-002-19. Bring-up order is: unload `fpga_csi`, program the ECP5 with OpenOCD from the SVF file (`svf -tap lfe5u45f.tap … quadrf.svf`), `modprobe fpga-dsi`, `modprobe fpga-csi`, then `quadrf-jtag --init`, which calibrates the DSI link taps and initialises the MAX2850 and MAX2851. `[S]` (sources/fpga/quadrf-load; jtag.c lines 576–587)
- S-002-20. Out-of-tree builds need the running kernel's headers (`linux-headers-rpi-2712` on the vendor image), DKMS, cmake, libfftw3, SDL2, libzmq and the device-tree compiler; the SoapySDR module installs under SoapySDR's `modules0.8` directory. `[S]` (docs/develop.md)
- S-002-21. The SoapySDR module's native sample format is CS8; CF32 is produced by NEON conversion; interleaved data is laid out as consecutive 16-bit (I,Q) pairs for elements 0–3 repeating; a polynomial Farrow resampler converts between the hardware rate and the requested host rate. `[S]` (MipiDevice.cpp lines 493–503, 1174–1300, 1948–2100; NEON.cpp lines 56–82)
- S-002-22. The stack's licences are as SPEC-001 S-001-36; the GUI is Python/Flask with Socket.IO, the desktop is KasmVNC with openbox, and all services are systemd units specific to Debian/DietPi. `[S]` (docs/develop.md; docs/overview.md; debian/copyright; sources/systemd/)

## Interfaces we depend on

- SoapySDR device `driver=mipi` (local) or `driver=remote,remote=quadrf.local` (LAN).
- `/run/quadrf/phy.sock` Air-IPC (SPEC-006).
- `/etc/default/quadrf-lora-phy` (SPEC-006).
- `quadrf-app register|start|stop` for our own applications.

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-002-1 | Whether upstream `meshtasticd` can coexist with `quadrf-meshtasticd` (two daemons, two radios) on one Pi, incl. BLE and port 4403 collisions | G03 bench |
| U-002-2 | CPU budget: PhaseGaze at 38 MSPS plus a LoRa demodulator; the PHY alone costs ≈ 0.35 of one A76 core at 8 MSPS | G05 bench |
| U-002-3 | Ring size and span size (bytes) as configured by the driver at probe | bench, `CSI_IOC_GET_RING_INFO` |
| U-002-4 | CPU cost of the kernel copy workqueue at 208 MB/s (4 ch × 26 MSPS) | bench, backlog P-05 |
| U-002-5 | Whether the hardware sample rate presented on CSI is fixed (40 MSPS) or follows the `bw` setting | bench, backlog R-05 |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 1 | 2026-10-08 | First reviewed version |
| 2 | 2026-10-08 | S-002-14…22 from the driver sources, SoapySDR module, boot hook and loader script; U-002-3…5 added |
