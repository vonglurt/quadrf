# SPEC-002 — QuadRF host software stack on Raspberry Pi 5

| Field | Value |
| --- | --- |
| Status | Reviewed |
| Revision | 1 |
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

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 1 | 2026-10-08 | First reviewed version |
