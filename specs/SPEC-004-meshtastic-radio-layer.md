# SPEC-004 — Meshtastic radio layer and Linux daemon

| Field | Value |
| --- | --- |
| Status | Reviewed |
| Revision | 1 |
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

## Interfaces we depend on

- `RadioInterface` C++ ABI for an out-of-tree backend (as done by `quadrf-mesh`).
- `meshtastic --host 127.0.0.1:4403 --set lora.* …` CLI.
- `/etc/meshtasticd/config.yaml`.

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-004-1 | Whether two `meshtasticd` instances on one host can be bridged without MQTT | G03 |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 1 | 2026-10-08 | First reviewed version |
