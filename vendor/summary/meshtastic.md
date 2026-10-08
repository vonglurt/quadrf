# Meshtastic — restatement of the facts we rely on

Attribution: `vendor/meshtastic/ATTRIBUTION.md`. Sources: firmware commit
`364a111` (2026-10-08); documentation pages retrieved 2026-10-08. Facts `[S]`.

## Radio layer

- `RadioInterface` is the base of every radio backend; hardware backends wrap RadioLib drivers (SX126x, SX127x/RF95, SX128x, LR11xx, LR20x0, STM32WL); `SimRadio` loops frames without RF. (`src/mesh/*.h`, `src/platform/portduino/SimRadio.h`)
- Transmit power is clamped to the region limit unless the owner is marked licensed; `lora.override_frequency` replaces the slot frequency when non-zero; licensed mode is tied to a call sign; the firmware does not frequency-hop. (`RadioInterface.cpp` ≈ lines 1070–1084, 1225, 1332; radio-settings documentation)
- North American presets and the 104-slot US LONG_FAST plan (slot 20 = 906.875 MHz) are as listed in SPEC-003 S-003-6/8. (radio-settings documentation)

## Linux daemon

- `meshtasticd` runs on all Raspberry Pi models including the Pi 5 (`gpiochip4` for the header); hardware is declared in `/etc/meshtasticd/config.yaml`; per-board templates live in `bin/config.d/` and are activated by copying or linking into `/etc/meshtasticd/config.d/`; SPI0 via the `spi0-0cs` overlay; phone API TCP 4403; optional BLE; MQTT bridging. (`bin/config-dist.yaml` lines 1–6, 102–108; documentation)
- Pin assignments of SX126x boards in `bin/config.d/` (BCM numbering):

| Template | CS | IRQ | Busy | Reset | TXen/RXen | Clear of tile JTAG 14/15/18/23 (22)? |
| --- | --- | --- | --- | --- | --- | --- |
| `lora-waveshare-sxxx` (also the Pico-to-RPi adapter) | 21 | 16 | 20 | 18 | — | No (Reset 18) |
| `lora-MeshAdv-900M30S` (30 dBm) | 21 | 16 | 20 | 18 | 13/12 | No (Reset 18) |
| `lora-MeshAdv-Mini-900M22S` (22 dBm) | 8 | 16 | 20 | 24 | –/12 | Yes |
| `lora-PiTastic-1W`, `lora-ZebraHat_1W/2W` | 24 | 22 | 27 | 17 | –/25 (2W) | Yes if GPIO 22 (optional TRST) is unused |
| `lora-RAK6421-13300/13302-slot1`, `lora-hat-rak-6421-pi-hat`, `lora-station-g3` | default | 22 | 24 | 16 | — | Yes if GPIO 22 is unused |
| `lora-NebraHat_1W/2W` | default | 22 | 4 | 18 | –/25 | No (Reset 18) |
| `lora-pimesh-1w-v1` / `-v2` | 21 / 8 | 16 / 6 | 20 / 5 | 18 / 18 | 13/12 / — | No (Reset 18) |
| `lora-starter-edition-sx1262-i2c` | 8 | 22 | 4 | 18 | — | No (Reset 18) |
| USB sticks `lora-usb-meshstick-1262`, `-meshtoad-e22`, `-umesh-1262-30dbm`, `-umesh-1268-30dbm`, `-rak19714`, `lora-usb-frametastic-1262` | 0 | 6 | 4 | 2 (1 for uMesh) | RXen 1 (2 for uMesh) | Yes: pins are on a USB bridge chip, no header GPIO used |

(The numbers in USB-stick rows are the bridge chip's GPIO numbers, not Pi header pins.)
