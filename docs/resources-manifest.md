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
| `MAX2850.pdf` (+ `.txt`) | Maxim 19-5009 Rev 2, 1/2019, 33 pp.; supplied by the user through the UTM share 2026-10-08; sha256 `10c7cff652d24a3cf7583026c5189f8b83ec5bda6df2ff07f393217a19d57a32` | SPEC-001 rev 3 S-001-37; vendor/summary/analog-devices-max2850.md |
| `MAX2871.pdf` (+ `.txt`) | Maxim 19-7106 Rev 4, 6/2020, 30 pp.; UTM share 2026-10-08; sha256 `00234bdfef4efdf34aba6974bc56a1a14d5cb1bd83549c4a5e2f6695e782ec29` | SPEC-007 rev 1 S-007-15/16; T22; vendor/summary/analog-devices-max2871.md |
| `SKY65404-31.pdf` (+ `.txt`) | Skyworks 201512K, 2015-11-06, 9 pp. (mirror copy with an "Alldatasheet" title tag); UTM share 2026-10-08; sha256 `1f7fa39cd8e626225cb35d8f1559362319d1f08a43491f352faa42f00d41b649` | SPEC-001 rev 3 S-001-38; T18, T21; vendor/summary/skyworks.md |
| `SE5004L.pdf` (+ `.txt`) | Skyworks SE5004L-EK1 evaluation-kit data sheet 202643A, 2012-12-11, 5 pp. (arrived as `se5004l-ek1_202643a.pdf`; the device data sheet DST-00316 is still missing); UTM share 2026-10-08; sha256 `9865f8cf179dea79ab4ef504773f6d7c2fa531eca3c1bab47ed8218649c37f5c` | SPEC-001 rev 3 S-001-39; U-001-7; vendor/summary/skyworks.md |
| `SX1261-2.pdf` (+ `.txt`) | Semtech SX1261/2 datasheet DS.SX1261-2.W.APP Rev 1.1, 2017-12, 107 pp. (mirror copy; arrived as `SX1261.PDF`); UTM share 2026-10-08; sha256 `93ee7130c727f611786539a53a83f0073d03e8377d8d30b3fe45278032ceb996` | SPEC-003 rev 2 S-003-2, S-003-12…15; T23; vendor/summary/semtech.md |
| *missing* SE5004L device data sheet (DST-00316) | not on the share; needed for saturated power / P1dB (U-001-7) | SPEC-001 S-001-9 |

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
| `semtech-lr1121-product.html` | https://www.semtech.com/products/wireless-rf/lora-connect/lr1121 | LR1121 bands (150–960 MHz, 2.4 GHz, S/L-band); C10 closed (V-05) |
| `semtech-sx1280-product.html` | https://www.semtech.com/products/wireless-rf/lora-connect/sx1280 | SX1280 2.4 GHz; C10 closed (V-05) |

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

## resources/oracle/, resources/build/, resources/corpus/, resources/tmp/ — built and generated here (backlog V-08)

Not downloads: products of the scripts named, reproducible from the clone
above and the VM's packages. Nothing in them is committed.

| Path | Produced by | Identity | Purpose |
| --- | --- | --- | --- |
| `oracle/` (+ `build/gr-lora_sdr/`) | `scripts/build-oracle.sh` (`make oracle`): `gr-lora_sdr` at `862746dd1cf635c9c8a4bfbaa2c3a0ec3a5306c9` built against Alpine `gnuradio` 3.10.12.0-r12 into this private prefix; versions in `oracle/VERSION.txt` | GPL-3.0, run as a separate process only (`vendor/tapparel-gr-lora_sdr/ATTRIBUTION.md`) | LoRa test oracle: transmitter for the corpus, receiver for the reference PER (`docs/lora-corpus.md`) |
| `corpus/qrf-lora-v1/` | `scripts/make-corpus.py generate` then `verify` (`make corpus`), seed 20261008, 4 × BW, ±2 ppm, 2026-10-08 | 37 cells, 111 files, 3.4 GB; `MANIFEST.json` (sha256 of every file) itself `4cce02e671b2399a7c2b7b89ab5736076adca38790a13e960ae414a56502803d`; `make corpus-check` compares | The R-08 test corpus with recorded truth and the oracle's per-cell result |
| `vectors/lora-tx-vectors.json` | `python3 -I scripts/oracle-vectors.py` (2026-10-08): the oracle transmitter run on 29 payload/preset cases with every stage's output recorded (whitened, header and CRC nibbles, Hamming codewords, interleaved words, chirp indices, first 64 samples) | 178 KB; a copy is committed as `crates/qrf-lora/tests/data/lora-tx-vectors.json` because the Rust conformance test `tests/r08_vectors.rs` must run without the oracle; it is program output, not oracle code | Fixes the coding conventions `qrf-lora` implements (backlog R-08) |
| `corpus/qrf-lora-rs/` | `make lora-oracle-check` (2026-10-08): seven cells of 50 frames synthesised by `crates/qrf-lora/examples/make_cell.rs` with the corpus channel model at 0 dB in BW, seed 20261008, then verified by the oracle receiver (`scripts/make-corpus.py verify --out resources/corpus/qrf-lora-rs`) | regenerated by the target; sizes and results in `docs/lora-corpus.md` §6 | The modulator half of the R-08 check |
| `tmp/` | the generator's and verifier's intermediates (removed after each cell) | — | Kept off `/tmp`, a 1.2 GB tmpfs on the VM |

## resources/measurements/ — measurement records written by `make` targets (backlog R-03)

| Path | How produced | Identity | Purpose |
| --- | --- | --- | --- |
| `measurements/R-03/mipi-soak-60s.json` | `make mipi-check` (2026-10-08, this VM): `crates/qrf-mipi/examples/mipi_soak.rs`, 60 s of the mock at one 131 072 B frame per 630 µs, seed 20261008 | sha256 `5ba0f8f29e0d977677bba8913a10a081157058e85ef71897cc6f5e152b0c468f`; regenerated by the target (timing fields differ per run; the pass/fail fields must not) | The R-03 check record quoted in `lab/LR-009` §IV.C and plotted by `analysis/plot-lr009.py` |
| `measurements/R-03/frame0.cs8` | the first frame of the same run (`--dump-frame`) | 131 072 B; sha256 `4ebede0c28b92b370aa46c377264883658ebbe8b6623e44fcac9f1a3aef207c3` (deterministic: the seeded template with counter 0) | Figure 2 of `lab/LR-009` |

## Secondary web sources consulted (not stored; facts from them are `[C]`)

- Crowd Supply campaign page https://www.crowdsupply.com/scale-rf/quadrf (specs table, open-source statement, pricing; ships 2026-11-30 per page on 2026-10-08)
- J. Geerling review, 2026, https://www.jeffgeerling.com/blog/2026/quadrf-can-spot-drones-and-see-wifi-through-my-wall/ (MIPI over CSI/DSI, designer background)
- Hackaday 2026-06-20 https://hackaday.com/2026/06/20/seeing-the-world-in-radio-waves-with-the-quadrf/
- Analog Devices product pages for MAX2850 / MAX2851 (4900–5900 MHz); superseded by the MAX2851 datasheet
- Search-engine summaries (2026-10-08) of: KrakenSDR retail and forum pages (2.4 MSPS practical rate); fccid.io records Z4T-WIO-SX1262 and 2AJUZ0M62H (fccid.io serves a bot challenge to scripts, and apps.fcc.gov's equipment-authorisation search returns an Akamai "Access Denied" to scripted requests, 2026-10-08); Raspberry Pi 5 USB power budget (1.6 A with a 5 A supply); crates.io crate pages; Alpine 3.19 release notes (Pi 5 support); Phoronix/Wikipedia on PREEMPT_RT in 6.12
- crates.io API (`https://crates.io/api/v1/crates/<name>`) queried 2026-10-08 for the versions in `vendor/rust-crates/ATTRIBUTION.md`
- Alpine package index pages for `linux-rpi` (v3.24: 6.18.52-r0; edge: 6.18.55-r0) and the local `apk search` output in `vendor/alpine/ATTRIBUTION.md`
