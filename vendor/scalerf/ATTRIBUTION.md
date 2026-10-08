# Scale RF Inc. — QuadRF

| Field | Value |
| --- | --- |
| Copyright holder | Scale RF Inc. (software: "Martin McCormick, Scale RF Inc." in the CSI module; packaging: "2025-2026 Scale RF") |
| Licences | Linux kernel modules `fpga-csi`, `fpga-dsi`: GPL-2.0 (`MODULE_LICENSE("GPL")`, SPDX header in `fpga-dsi.c`, `debian/copyright`). Everything else in the software repository: GPL-3.0 per `debian/copyright` `Files: *`; the vendor's updates page lists GPL-2.0 for drivers, SoapySDR module, ZeroMQ modules, spatial RF visualisation, NTSC decoder, spectrum analyser, web GUI and CLI, and GPL-3.0 for GNU Radio demos. Antenna design files, OpenEMS models, mechanical files: CC-BY-SA-4.0 with a defensive patent covenant (`licenses/LICENSE.md`, `licenses/PATENT_COVENANT.md`). ECP5 bitstream `quadrf.svf`: all rights reserved, redistribution allowed, not published in the repository. RF board, RF core, layouts, BOM: all rights reserved, patent pending. Schematic PDF: "source-available for debugging, education, and academic research". `mongoose.c/.h` in demos: GPL-2.0, Cesanta. |
| URLs | https://github.com/open-space-sdr/main ; https://scalerf.com/docs/ ; https://scalerf.com/updates/ ; https://scalerf.com/cals/antennas.html ; https://scalerf.com/docs/QuadRF_schematics.pdf |
| Revision read | Repository commit `8b61ae5bd2d0635da8eabfa6daa623968ea14af1` ("debian: 1.0.37", 2026-10-07); schematic export dated 2026-05-22; web pages retrieved 2026-10-08 |
| Files read | `README.md`, `docs/{overview,applications,develop,tile-connection}.md`, `install/README.md`, `sources/fpga/drivers/csi/{fpga_csi.h,fpga-csi.c,fpga-csi.dts}`, `sources/fpga/drivers/dsi/fpga-dsi.c`, `sources/fpga/jtag_src/jtag.c`, `sources/fpga/quadrf-load`, `sources/fpga/interface/rpi5_ecp5_gpio.cfg`, `sources/soapy/{MipiDevice.cpp,MipiDevice.hpp,NEON.cpp}`, `sources/demos/csi_sweep.c`, `sources/boot/10-boot`, `debian/copyright`, `licenses/*` |
| Trademarks | ScaleRF, QuadRF, MoonRF are marks of Scale RF Inc.; used here only to name the product (`licenses/TRADEMARKS.md`) |

## What we use

Behaviour facts for SPEC-001 (tile) and SPEC-002 (host software): tuning
range, interfaces, ioctl and ring semantics, boot configuration, licences.
Our restatement is `vendor/summary/scalerf-quadrf.md`; the specs cite the
`resources/` paths.

## What we do not do

- Copy any source file, schematic page, or design file into this repository.
- Link GPL code into our Rust binaries. Our capture daemon talks to the GPL
  kernel modules through their public ioctl/mmap interface (a userspace
  program using a kernel ABI is not a derivative work of the module) and to
  the vendor's SoapySDR module only as an optional, separate shared object
  loaded by SoapySDR itself.
- Redistribute the bitstream through git. It is copied from the kit's SD card
  onto the copal build host outside the repository.
- Reproduce the RF board or RF core. The schematic is read for debugging and
  specification only, as the vendor's notice permits.
