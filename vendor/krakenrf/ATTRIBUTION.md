# KrakenRF Inc. — KrakenSDR

| Field | Value |
| --- | --- |
| Copyright holder | KrakenRF Inc. |
| Licences | `heimdall_daq_fw` and `krakensdr_doa`: GPL-3.0 (repository licence fields). Hardware documentation (GitHub wiki `krakensdr_docs`): no licence file found; treated as all rights reserved for copying purposes. |
| URLs | https://github.com/krakenrf/krakensdr_docs/wiki ; https://github.com/krakenrf/heimdall_daq_fw ; https://github.com/krakenrf/krakensdr_doa |
| Revision read | Wiki `Home` and `04. Antenna Array Setup` pages and both README files, retrieved 2026-10-08 into `resources/krakensdr/` |

## What we use

Hardware facts (five coherent 8-bit RTL2832U/R820T2 channels, 24–1766 MHz,
2.56 MHz maximum channel bandwidth, 1 ppm oscillator, noise-source
calibration, 2.2 A at 5 V) and the array-geometry rules (spacing multiplier
≤ 0.5, typical 0.33, UCA radius formula) as the C12 comparator in G04/G05 and
as a candidate USB sensor plugin. Restated in SPEC-011 and
`vendor/summary/krakenrf.md`.

## What we do not do

Copy or link the GPL-3.0 DAQ or DoA code. If the KrakenSDR becomes a sensor
plugin, the GPL software runs as its own process and our plugin consumes its
published output, or we write an independent Rust coherent-capture path.
