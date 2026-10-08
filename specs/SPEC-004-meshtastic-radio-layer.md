# SPEC-004 — Meshtastic radio layer and Linux daemon

| Field | Value |
| --- | --- |
| Status | Reviewed (rev 1); rev 2 additions S-004-9…12 await a second read |
| Revision | 2 |
| Date | 2026-10-08 |
| Subject | The parts of Meshtastic firmware that decide frequency, power, modem settings and radio backends, and the Linux-native daemon |
| Primary sources | `resources/repos/meshtastic-firmware/src/mesh/RadioInterface.{h,cpp}`, `src/platform/portduino/SimRadio.{h,cpp}`, `bin/config-dist.yaml`, `resources/lora/meshtastic-linux-rpi.html`, `resources/lora/meshtastic-lora-config.html` |
| Depends on | SPEC-003 |

## Behaviour-goal statements

- S-004-1. Every radio backend is a subclass of `RadioInterface`; hardware backends wrap RadioLib drivers (SX126x, SX127x/RF95, SX128x, LR11xx, LR20x0, STM32WL); `SimRadio` is a backend with no RF that loops frames through the phone API. A new backend needs to implement `send`, `startReceive`, channel-activity and queue methods only. `[S]` (src/mesh/*.h listing; SimRadio.h)
- S-004-2. Transmit power is clamped to the region's limit unless the owner record is marked licensed; with a licensed owner the configured `tx_power` is applied as given. `[S]` (RadioInterface.cpp ≈ line 1332)
- S-004-3. `lora.override_frequency` replaces the slot-derived frequency when non-zero. `[S]` (RadioInterface.cpp ≈ line 1225; lora-config docs)
- S-004-4. Licensed ("ham") mode sets the owner licensed flag and is used with a call sign; regions marked licensed-only are refused for unlicensed owners. `[S]` (RadioInterface.cpp ≈ lines 1070–1084; ITU amateur regions defined with `licensedOnly`)
- S-004-5. The Linux-native daemon `meshtasticd` runs on all Pi models incl. Pi 5 (BCM2712); radio hardware is declared in `/etc/meshtasticd/config.yaml` (`Lora: Module: sx1262`, `CS`, `IRQ`, `Busy`, `Reset`, optional `TXen`/`RXen`, `DIO2_AS_RF_SWITCH`, `DIO3_TCXO_VOLTAGE`, `SX126X_MAX_POWER`, `spidev`, `gpiochip`); Pi 5 header GPIOs are on `gpiochip4`; SPI0 is enabled with the `spi0-0cs` overlay. `[S]` (config-dist.yaml; meshtastic-linux-rpi.html)
- S-004-6. Example pin sets in the distributed config use CS 21 / IRQ 16 / Busy 20 / Reset 18 for Waveshare-class HATs; GPIO 18 collides with the QuadRF JTAG TDO line. `[S]`+`[D]` (config-dist.yaml; SPEC-001 S-001-22)
- S-004-7. The daemon exposes the phone API on TCP 4403 and may expose BLE; MQTT bridging connects meshes over IP. `[S]` (quadrf-mesh README; Meshtastic docs)
- S-004-8. Meshtastic does not frequency-hop: a node stays on one slot. `[S]` (slot model in radio-settings docs; no hop scheduler in RadioInterface)

### Raspberry Pi HAT and USB radio pin assignments (revision 2)

- S-004-9. Board templates in `bin/config.d/` give these SX126x pin sets (BCM numbering): Waveshare SX126x HAT and the Pico-to-RPi adapter CS 21 / IRQ 16 / Busy 20 / Reset 18; MeshAdv 900M30S CS 21 / IRQ 16 / Busy 20 / Reset 18 / TXen 13 / RXen 12; MeshAdv-Mini 900M22S CS 8 / IRQ 16 / Busy 20 / Reset 24 / RXen 12; PiTastic 1W and ZebraHat 1W/2W CS 24 / IRQ 22 / Busy 27 / Reset 17 (RXen 25 on the 2W); RAK6421 slot 1, the RAK 6421 Pi HAT and Station G3 IRQ 22 / Reset 16 / Busy 24; NebraHat 1W/2W IRQ 22 / Busy 4 / Reset 18 / RXen 25; PiMesh 1W v2 CS 8 / IRQ 6 / Busy 5 / Reset 18; Starter-edition SX1262 CS 8 / IRQ 22 / Busy 4 / Reset 18. `[S]` (resources/repos/meshtastic-firmware/bin/config.d/*.yaml at commit 364a111; vendor/summary/meshtastic.md)
- S-004-10. Of those, the MeshAdv-Mini 900M22S uses no pin of the tile's JTAG set {14, 15, 18, 23, 22}; the PiTastic/ZebraHat and RAK6421/Station G3 sets are clear if GPIO 22 (optional TRST) is unused; every template with Reset on GPIO 18 collides with the tile's TDO line. `[D]` (S-004-9; SPEC-001 S-001-22)
- S-004-11. USB-attached radios (meshstick-1262, meshtoad-E22, uMesh 1262/1268 30 dBm, RAK19714, frametastic-1262) are driven through a USB SPI/GPIO bridge; their templates name bridge-chip GPIO numbers 0–6 and use no Pi header pin, so they coexist with the tile without constraint. `[S]`+`[D]` (bin/config.d/lora-usb-*.yaml)
- S-004-12. The daemon activates a board template by copying or linking it from the distributed `available.d` into `/etc/meshtasticd/config.d/`; the Pi 5 header is `gpiochip4`, selectable globally or per pin. `[S]` (bin/config-dist.yaml lines 1–6, 32–71, 102–104)

## Interfaces we depend on

- `RadioInterface` C++ ABI for an out-of-tree backend (as done by `quadrf-mesh`).
- `meshtastic --host 127.0.0.1:4403 --set lora.* …` CLI.
- `/etc/meshtasticd/config.yaml`.

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-004-1 | Whether two `meshtasticd` instances on one host can be bridged without MQTT | G03 |
| U-004-2 | Whether GPIO 22 (optional TRST) is driven by the vendor's OpenOCD configuration on the kit as shipped | bench, read `rpi5_ecp5_gpio.cfg` of the installed package and probe the pin |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 1 | 2026-10-08 | First reviewed version |
| 2 | 2026-10-08 | S-004-9…12 from the per-board Linux templates; U-004-2 added |
