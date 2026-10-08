# G07 — Field-test plan with ESP32 and Pico LoRa nodes

| Field | Value |
| --- | --- |
| Status | DRAFT |
| Opened | 2026-10-08 |
| Closed | — |
| Author | project |
| Inherits from | G06 |

## 1. Question

Which measurements, with the LoRa hardware in hand (several ESP32 LoRa nodes,
one Pico LoRa HAT, optionally one Pi 5 SX1262 HAT), would falsify the G05
and G06 claims, and in what order?

## 2. Inherited facts

- F.05.5, F.05.7, G05 criteria (1)–(5), F.06.6, F.06.9, Phase 1–3 plan.

## 3. Method

Each test states the claim under test, the prediction with its tolerance,
the setup, and the record format (`docs/templates/measurement-record.md`).
Nodes are flashed with a known firmware version, a known preset, a known
conducted power (`lora.tx_power`), and a known antenna, and their positions
are surveyed (GNSS, ±3 m). Tests are run in order; a failed test stops the
sequence until explained.

## 4. Tests

| Id | Claim | Prediction | Setup | Pass |
| --- | --- | --- | --- | --- |
| T-1 | Node transmit power and bandwidth are as configured (SPEC-003 S-003-4) | 6 dB BW 250 ± 25 kHz at LONG_FAST, 500 ± 50 kHz at SHORT_TURBO; conducted power within ±1.5 dB of setting | Node into 30 dB pad into spectrum analyser (or calibrated SDR) | Both within tolerance; records U-003-1 |
| T-2 | Free-space law holds over 50–500 m on open ground (F.06.2) | RSSI slope −20 dB/decade ± 2 dB after ground-reflection correction; absolute within ±4 dB of FSPL + antenna gains | Two ESP32 nodes, 2 m masts, open field, 5 distances | Slope and offset within tolerance |
| T-3 | Yagi gain is realised on receive (F.06.9) | RSSI increase = G_yagi − G_whip ± 1.5 dB when pointing at the far node; sidelobe suppression ≥ 15 dB at 60° off-axis | Pi/Pico node with Yagi vs whip, far ESP32 at 300 m | Within tolerance |
| T-4 | Repeater link budget (F.06.6) | RSSI from the summit repeater within ±6 dB of EIRP − FSPL − knife-edge − foliage + G_rx, computed per path profile | Yagi node aimed at each visible summit; log 30 min of frames per summit | ≥ 2 summits within tolerance; residuals explained by terrain |
| T-5 | FTFE sensitivity (F.05.5) | Decode threshold at SHORT_TURBO ≤ −120 dBm ± 2 at the 915 MHz antenna port | Node → step attenuator → FTFE → tile; `quadrf-mesh` or gr-lora_sdr decoding | Threshold within tolerance (G05 criterion 2 corollary) |
| T-6 | FTFE bearing (G05 criterion 4) | Bearing rms error ≤ 5° over ±60°, 20 m, open ground | ESP32 node walked on a surveyed arc; 4-channel capture; host DoA | Within tolerance |
| T-7 | Whole-band occupancy (G05 criterion 5) | All 104 slots scanned continuously ≥ 60 s; slot 20 shows known test node; CPU ≤ 80 % of 4 cores | FTFE + tile at 26 MSPS interleaved; channeliser | Sustained; CPU within budget |
| T-8 | Summit bearing agrees with map (F.06.9 pointing) | Bearing of each summit repeater within ±5° of surveyed azimuth | T-7 setup, outdoor, 30 min | Within tolerance for ≥ 2 summits |
| T-9 | 5.8 GHz tile-to-tile LoRa link (F.06.7) | At 1 km LOS, SHORT_TURBO PER ≤ 1 % at 1 W aggregate; measured SNR within ±3 dB of budget | Two tiles (requires second kit or a loan), tripods, surveyed LOS | Within tolerance |

## 5. Records

One `M-nnn` record per test run under `investigations/records/` (to be
created), raw I/Q and node logs under `resources/measurements/` (gitignored;
listed in the manifest with checksums).

## 6. Gate record

| Field | Value |
| --- | --- |
| Pass criterion | Plan reviewed; T-1…T-4 executable with hardware in hand before kit delivery; T-5…T-8 after G05; T-9 requires a second tile. |
| Decision | DRAFT |

## 7. Notes on hardware in hand

- ESP32 LoRa nodes: record board, radio IC (SX1262 vs SX1276), firmware version, max `tx_power`, antenna. `[C]` until recorded.
- Pico LoRa HAT: record model and radio IC; a 22 dBm SX1262 HAT is "slightly more powerful" than 20 dBm ESP32 boards; a 30 dBm (E22-900M30S class) module requires the §15.247(b)(4) back-off with any antenna above 6 dBi. `[C]` until recorded.
- Flashing many nodes: use one firmware tag, one preset, one channel, one `tx_power`; record all in the measurement header.

## 8. Addenda (2026-10-08, second pass; plan remains DRAFT)

- T-10 (draft). Claim: the parallel feed is aligned (SPEC-009 S-009-13). Prediction: tile (5 GHz layer) and a 915 MHz coherent receiver draw one surveyed emitter within ≤ 100 ms and ≤ 2° after mounting transforms. Setup: one node radiating LoRa at 915 MHz and a 5.8 GHz CW source (or second tile) co-located at a surveyed point 20 m from the sensors; `qrf-overlay` log. Pass: 30-minute run within tolerance.
- Hardware note: the Pico LoRa HAT is probably a Waveshare "Pico-LoRa-SX1262" (SPI, SX1262; an 868M variant is confirmed at retail; a 915M variant and its maximum power are unverified). The field-node firmware for it is Rust (`embassy-rp` + `lora-phy`, SPEC-008 S-008-11); record the model, band and FCC ID when the board is in hand (backlog F-01). `[C]`
- T-1…T-4 can be run with an RTL-SDR dongle as the receiver (C13) if no spectrum analyser is available; absolute power then needs a calibrated attenuator and a reference level, and the tolerance in T-1 widens to ±3 dB. `[D]`
