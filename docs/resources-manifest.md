# Resources manifest

`resources/` is gitignored. This manifest is the record of what was retrieved,
when, and what each item was used for. `scripts/fetch-resources.sh` re-creates
it. Retrieval date for all items: 2026-10-08 unless stated.

## resources/scalerf/ — vendor primary documents

| File | URL | Used for |
| --- | --- | --- |
| `QuadRF_schematics.pdf` (10 pages, Altium export, dated 2026-05-22) | https://scalerf.com/docs/QuadRF_schematics.pdf | SPEC-001: RF chain parts, antenna designators ANT1–ANT4, clock, switches |
| `schematics.txt` | pdftotext of the above | grep-able designator list |
| `detailed.svg`, `chained.svg` | https://scalerf.com/docs/ | Block diagram, daisy-chain data path |
| `jtag_cable_pinout.drawio.svg` | same | 8-pin JTAG/power cable |
| `docs.html` | https://scalerf.com/docs/ | Specifications, BOM, SoapySDR API, calibration links |
| `updates.html` | https://scalerf.com/updates/ | Dated project history incl. 2026-07-15 Meshtastic note |
| `cal_antennas.html` | https://scalerf.com/cals/antennas.html | Array calibration method (EKF transponder) |
| `cal_txqec.html` | https://scalerf.com/cals/txqec.html | Tx quadrature-error calibration |

## resources/regulatory/ — 47 CFR via LII e-CFR mirror

| File | Section | Used for |
| --- | --- | --- |
| `cfr-47-15.23.html` | §15.23 Home-built devices | G02 |
| `cfr-47-15.203.html` | §15.203 Antenna requirement | G02 |
| `cfr-47-15.205.html` | §15.205 Restricted bands | G02 |
| `cfr-47-15.209.html` | §15.209 Radiated emission limits | G02 |
| `cfr-47-15.247.html` | §15.247 902–928 / 2400–2483.5 / 5725–5850 MHz | G02, SPEC-005 |
| `cfr-47-15.249.html` | §15.249 field-strength path | G02 |
| `cfr-47-97.3.html` | §97.3 definitions (SS emission) | G02 |
| `cfr-47-97.113.html` | §97.113 prohibited transmissions | G02 |
| `cfr-47-97.119.html` | §97.119 station identification | G02 |
| `cfr-47-97.303.html` | §97.303 frequency sharing | G02 |
| `cfr-47-97.311.html` | §97.311 SS emission types | G02 |
| `cfr-47-97.313.html` | §97.313 transmitter power | G02 |

Note: ecfr.gov itself redirected scripted requests to an unblock page; LII's
mirror was used. LII shows the amendment history through 85 FR 18149
(2020-04-01) for §15.247. Verify against ecfr.gov before any regulatory filing.

## resources/lora/

| File | URL | Used for |
| --- | --- | --- |
| `meshtastic-radio-settings.html` | https://meshtastic.org/docs/overview/radio-settings/ | Preset table, US slot plan (104 slots, slot 20 = 906.875 MHz) |
| `meshtastic-linux-rpi.html` | https://meshtastic.org/docs/linux/hardware/boards/raspberry-pi/ | meshtasticd on Pi 5, SPI overlay |
| `meshtastic-lora-config.html` | https://meshtastic.org/docs/configuration/radio/lora/ | override_frequency, tx_power semantics |
| `meshtastic-region-us.html` | https://meshtastic.org/docs/configuration/region-by-country/ | region codes |
| `gr-lora_sdr-README.md` | https://raw.githubusercontent.com/tapparelj/gr-lora_sdr/master/README.md | SDR LoRa PHY capabilities |
| `lora-phy-paper-tapparel.pdf` | https://arxiv.org/pdf/2002.08208 | LoRa PHY open implementation (SPAWC 2020) |

## resources/repos/ — shallow clones

| Directory | URL | Used for |
| --- | --- | --- |
| `quadrf-open-space-sdr` | https://github.com/open-space-sdr/main | Soapy driver limits, jtag tuning guard, GPIO map, demos (element pitch), docs/, install/ |
| `quadrf-mesh` | https://github.com/radioroy/quadrf-mesh | LoRa-compatible PHY + Meshtastic daemon for QuadRF; Air-IPC; measured LO wander |
| `gr-lora_sdr` | https://github.com/tapparelj/gr-lora_sdr | Reference SDR LoRa transceiver |
| `meshtastic-firmware` | https://github.com/meshtastic/firmware | RadioInterface abstraction, SimRadio, licensed-mode power logic, Linux config-dist.yaml |

## resources/datasheets/ — manual fetch required

analog.com and semtech.com rejected scripted retrieval (HTTP/2 reset, timeouts).
Place manually: `MAX2850.pdf`, `MAX2851.pdf` (4.9–5.9 GHz 4-ch Tx / 5-ch Rx
wireless-HDMI transceivers), `MAX2871.pdf` (23.5 MHz–6 GHz synthesiser,
candidate transverter LO), `SX1262.pdf`, `SE5004L.pdf`, `SKY65404-31.pdf`.
The frequency-range facts used in SPEC-001 were taken from the vendor product
pages and the QuadRF BOM, and are tagged accordingly.

## Secondary web sources consulted (not stored)

- Crowd Supply campaign page https://www.crowdsupply.com/scale-rf/quadrf (specs table, open-source statement, pricing; ships 2026-11-30 per page on 2026-10-08)
- J. Geerling review, 2026, https://www.jeffgeerling.com/blog/2026/quadrf-can-spot-drones-and-see-wifi-through-my-wall/ (MIPI over CSI/DSI, designer background)
- Hackaday 2026-06-20 https://hackaday.com/2026/06/20/seeing-the-world-in-radio-waves-with-the-quadrf/
- Analog Devices product pages for MAX2850 / MAX2851 (4900–5900 MHz)
