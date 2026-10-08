# Raspberry Pi Ltd — RP1 Peripherals datasheet and Pi 5 documentation

| Field | Value |
| --- | --- |
| Copyright holder | Raspberry Pi Ltd |
| Licence | "Raspberry Pi RP1 Peripherals" datasheet (RP-008370-DS-1, created 2023-11-07, 93 pages): proprietary document; not redistributed. The documentation site is CC-BY-SA-4.0 per its notice (not copied here). |
| Documents | `resources/datasheets/RP1-peripherals.pdf` retrieved 2026-10-08 from https://pip-assets.raspberrypi.com/categories/892-raspberry-pi-5/documents/RP-008370-DS-1-rp1-peripherals.pdf |

## What we use

Chapter 1 figures: PCIe 2.0 x4 link to BCM2712, two xHCI controllers each with
one USB 3.0 PHY, two 4-lane MIPI D-PHY transceivers with 8 Gbit/s aggregate,
eight-channel DMAC, QoS prioritisation of camera/display traffic. Restated in
SPEC-001 and `vendor/summary/raspberry-pi-rp1.md`.
