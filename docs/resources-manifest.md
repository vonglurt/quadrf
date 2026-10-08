# Resources manifest

`resources/` is gitignored. This manifest is the record of what was retrieved,
when, from where, and what each item was used for. `scripts/fetch-resources.sh`
re-creates everything that can be fetched by script; `scripts/import-shared.sh`
imports what must be fetched in a browser. Retrieval date for all items is
2026-10-08 unless stated. Attribution and licence of each upstream:
`vendor/README.md`.

## resources/scalerf/ — vendor primary documents

| File | URL | Used for |
| --- | --- | --- |
| `QuadRF_schematics.pdf` (10 pages, Altium export, dated 2026-05-22) | https://scalerf.com/docs/QuadRF_schematics.pdf | SPEC-001: RF chain parts, antenna designators ANT1–ANT4, clock, switches, FPGA part (S-001-35) |
| `schematics.txt` | pdftotext of the above | grep-able designator list |
| `detailed.svg`, `chained.svg` | https://scalerf.com/docs/ | Block diagram, daisy-chain data path |
| `jtag_cable_pinout.drawio.svg` | same | 8-pin JTAG/power cable |
| `docs.html` | https://scalerf.com/docs/ | Specifications, BOM, SoapySDR API, calibration links |
| `updates.html` | https://scalerf.com/updates/ | Dated project history incl. 2026-07-15 Meshtastic note; licensing FAQ (S-001-36) |
| `cal_antennas.html` | https://scalerf.com/cals/antennas.html | Array calibration method (EKF transponder) |
| `cal_txqec.html` | https://scalerf.com/cals/txqec.html | Tx quadrature-error calibration |

## resources/regulatory/ — 47 CFR via LII e-CFR mirror

| File | Section | Used for |
| --- | --- | --- |
| `cfr-47-1.1310.html` | §1.1310 RF exposure limits (Table 1) | SPEC-005 S-005-19/20 (added second pass) |
| `cfr-47-15.23.html` | §15.23 Home-built devices | G02 |
| `cfr-47-15.203.html` | §15.203 Antenna requirement | G02 |
| `cfr-47-15.205.html` | §15.205 Restricted bands | G02 |
| `cfr-47-15.209.html` | §15.209 Radiated emission limits | G02 |
| `cfr-47-15.247.html` | §15.247 902–928 / 2400–2483.5 / 5725–5850 MHz | G02, SPEC-005 |
| `cfr-47-15.249.html` | §15.249 field-strength path | G02 |
| `cfr-47-97.3.html` | §97.3 definitions (SS emission) | G02 |
| `cfr-47-97.13.html` | §97.13 station location; (c) exposure | SPEC-005 S-005-21 (added second pass) |
| `cfr-47-97.113.html` | §97.113 prohibited transmissions | G02 |
| `cfr-47-97.119.html` | §97.119 station identification | G02 |
| `cfr-47-97.303.html` | §97.303 frequency sharing | G02 |
| `cfr-47-97.311.html` | §97.311 SS emission types | G02 |
| `cfr-47-97.313.html` | §97.313 transmitter power | G02 |

The public-domain section text is extracted into `vendor/cfr47/` by
`scripts/vendor-cfr.py`. ecfr.gov redirects scripted requests to an unblock
page; LII's mirror was used. LII shows the amendment history through
85 FR 18149 (2020-04-01) for §15.247. Verify against ecfr.gov before any
regulatory filing (U-005-2).

## resources/datasheets/

| File | Provenance | Used for |
| --- | --- | --- |
| `MAX2851.pdf` (+ `.txt`) | Maxim 19-5121 Rev 1, 3/2010, 37 pp.; supplied by the user through the UTM share 2026-10-08 (arrived as "EV Bible.pdf"); sha256 `4b6bb5e303586f611ff78384298787639ec126949bdbb84266a489e66f6c97db` | SPEC-001 S-001-26…32; G01/G05 addenda; T17, T18; LR-006 |
| `RP1-peripherals.pdf` (+ `.txt`) | Raspberry Pi RP-008370-DS-1, created 2023-11-07, 93 pp.; https://pip-assets.raspberrypi.com/categories/892-raspberry-pi-5/documents/RP-008370-DS-1-rp1-peripherals.pdf (two redirects from datasheets.raspberrypi.com/rp1/rp1-peripherals.pdf) | SPEC-001 S-001-34; SPEC-010; T14 |
| *missing* `MAX2850.pdf`, `MAX2871.pdf` | analog.com resets scripted HTTP/2 streams; fetch in a browser, drop in `~/Downloads/SharedVM/quadrf/`, run `scripts/import-shared.sh` | SPEC-001 transmit statements; SPEC-007 LO |
| *missing* `SX1262.pdf` | semtech.com returns an HTML interstitial for the datasheet link | SPEC-003 S-003-2 thresholds (V-03) |
| *missing* `SE5004L.pdf`, `SKY65404-31.pdf` | skyworksinc.com returns HTML | S-001-9; T18 placeholders |

## resources/lora/

| File | URL | Used for |
| --- | --- | --- |
| `meshtastic-radio-settings.html` | https://meshtastic.org/docs/overview/radio-settings/ | Preset table, US slot plan (104 slots, slot 20 = 906.875 MHz) |
| `meshtastic-linux-rpi.html` | https://meshtastic.org/docs/linux/hardware/boards/raspberry-pi/ | meshtasticd on Pi 5, SPI overlay |
| `meshtastic-lora-config.html` | https://meshtastic.org/docs/configuration/radio/lora/ | override_frequency, tx_power semantics |
| `meshtastic-region-us.html` | https://meshtastic.org/docs/configuration/region-by-country/ | region codes |
| `gr-lora_sdr-README.md` | https://raw.githubusercontent.com/tapparelj/gr-lora_sdr/master/README.md | SDR LoRa PHY capabilities |
| `lora-phy-paper-tapparel.pdf` | https://arxiv.org/pdf/2002.08208 | LoRa PHY open implementation (SPAWC 2020) |
| `semtech-sx1262-product.html` | https://www.semtech.com/products/wireless-rf/lora-connect/sx1262 | SX1262 range, power, sensitivity (second pass; C10) |

## resources/krakensdr/ — C12 comparator (second pass)

| File | URL | Used for |
| --- | --- | --- |
| `wiki-Home.md` | https://raw.githubusercontent.com/wiki/krakenrf/krakensdr_docs/Home.md | SPEC-011 hardware facts |
| `wiki-04-Antenna-Array-Setup.md` | https://raw.githubusercontent.com/wiki/krakenrf/krakensdr_docs/04.-Antenna-Array-Setup.md | SPEC-011 array rules |
| `heimdall_daq_fw-README.md` | https://raw.githubusercontent.com/krakenrf/heimdall_daq_fw/main/README.md | SPEC-011 software; licence |
| `krakensdr_doa-README.md` | https://raw.githubusercontent.com/krakenrf/krakensdr_doa/main/README.md | SPEC-011 software; ports |

## resources/repos/ — shallow clones (commit read)

| Directory | URL | Commit | Used for |
| --- | --- | --- | --- |
| `quadrf-open-space-sdr` | https://github.com/open-space-sdr/main | `8b61ae5bd2d0635da8eabfa6daa623968ea14af1` (2026-10-07, "debian: 1.0.37") | Soapy driver limits, jtag tuning guard, GPIO map, demos (element pitch), docs/, install/, driver sources, boot hook, licences |
| `quadrf-mesh` | https://github.com/radioroy/quadrf-mesh | `ad3ed3109c7d6cfd7db673ffde4ba6e02639fa28` (2026-10-08, "debian: 0.1.9") | LoRa-compatible PHY + Meshtastic daemon for QuadRF; Air-IPC; measured LO wander; defaults |
| `gr-lora_sdr` | https://github.com/tapparelj/gr-lora_sdr | `862746dd1cf635c9c8a4bfbaa2c3a0ec3a5306c9` (2026-01-05) | Reference SDR LoRa transceiver (test oracle only) |
| `meshtastic-firmware` | https://github.com/meshtastic/firmware | `364a111f4a5601708f3b9d60527aca02b6ac73ef` (2026-10-08) | RadioInterface abstraction, SimRadio, licensed-mode power logic, Linux config-dist.yaml and config.d templates |

## Secondary web sources consulted (not stored; facts from them are `[C]`)

- Crowd Supply campaign page https://www.crowdsupply.com/scale-rf/quadrf (specs table, open-source statement, pricing; ships 2026-11-30 per page on 2026-10-08)
- J. Geerling review, 2026, https://www.jeffgeerling.com/blog/2026/quadrf-can-spot-drones-and-see-wifi-through-my-wall/ (MIPI over CSI/DSI, designer background)
- Hackaday 2026-06-20 https://hackaday.com/2026/06/20/seeing-the-world-in-radio-waves-with-the-quadrf/
- Analog Devices product pages for MAX2850 / MAX2851 (4900–5900 MHz); superseded by the MAX2851 datasheet
- Search-engine summaries (2026-10-08) of: KrakenSDR retail and forum pages (2.4 MSPS practical rate); Semtech LR1121 and distributor pages (band list); fccid.io records Z4T-WIO-SX1262 and 2AJUZ0M62H (fccid.io serves a bot challenge to scripts); Raspberry Pi 5 USB power budget (1.6 A with a 5 A supply); crates.io crate pages; Alpine 3.19 release notes (Pi 5 support); Phoronix/Wikipedia on PREEMPT_RT in 6.12
- crates.io API (`https://crates.io/api/v1/crates/<name>`) queried 2026-10-08 for the versions in `vendor/rust-crates/ATTRIBUTION.md`
- Alpine package index pages for `linux-rpi` (v3.24: 6.18.52-r0; edge: 6.18.55-r0) and the local `apk search` output in `vendor/alpine/ATTRIBUTION.md`
