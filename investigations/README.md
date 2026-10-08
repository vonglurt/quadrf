# Gate ledger

Synchronous, ordered. A gate is opened only when the previous one is PASS or
WAIVED. Status is mirrored in the top-level README and in `index.md`. Signed
gates are not edited; new evidence is appended as dated errata/addenda.

| Gate | File | Question | Status | Closed | Carries forward |
| --- | --- | --- | --- | --- | --- |
| G00 | `G00-charter.md` | Is the question well-posed? | PASS | 2026-10-08 | F.00.1–6 |
| G01 | `G01-quadrf-hardware-survey.md` | What does the tile do? | PASS (+ addenda F.01.13–18) | 2026-10-08 | F.01.1–12, 13–15, 18 |
| G02 | `G02-regulatory-envelope-pnw.md` | What is lawful at 915 MHz and 5.8 GHz in WA/OR? | PASS (+ addenda F.02.14–16) | 2026-10-08 | F.02.1–13 |
| G03 | `G03-pi-software-stack.md` | What runs on the Pi; what do we write? | PASS (desk) (+ addenda F.03.9–13) | 2026-10-08 | F.03.2–8, graphs A/B/C; SPEC-008/009/010 |
| G04 | `G04-lora-options-inventory.md` | Which LoRa architectures survive? | PASS (+ addenda C12–C14, F.04.4) | 2026-10-08 | C3, C5–C9, C12–C14; F.04.1–4 |
| G05 | `G05-915mhz-translation-frontend.md` | Does the 915 MHz translation front end work? | OPEN (desk done + addenda F.05.14–20; bench after 2026-11-30 delivery) | — | F.05.1–20 provisional |
| G06 | `G06-mountain-repeater-directional-relay.md` | What should the directional relay be? | OPEN (desk done; signs with G05) | — | Phase 1–3 plan |
| G07 | `G07-field-test-plan.md` | How do we falsify G05/G06 in the field? | DRAFT (+ T-10) | — | T-1…T-10 |

## Design track

The design and build work runs in parallel as desk work and is ledgered in
`backlog.md` (phases P0–P7). Its documents are the implementation specs
SPEC-007 (FTFE), SPEC-008 (system architecture), SPEC-009 (plug-ins and
bus), SPEC-010 (platform), with SPEC-011 (KrakenSDR-class receiver) as a
clean-room comparator spec. A backlog entry that depends on an open
conjecture is not marked done before the gate that closes it is signed.

## Open conjectures (project-wide risk register)

| Id | Statement | Closes at |
| --- | --- | --- |
| U-001-1 | Element-port connector type/impedance/loss; pigtail feasibility | G05 bench (backlog H-02) |
| U-001-2 | 40 MHz reference exportable to the translator LO; oscillator identity (SiT8008 claim is from the BOM page only) | G05 bench (H-03) |
| U-001-3 | Tile NF at 5487–5513 MHz (flatness now bounded ≤ 4.2 dB p-p by the datasheet) | G05 bench |
| U-001-5 | Factory bitstream auto-steer / RF-vision behaviour on a translated band | G05 bench |
| U-001-6 | Fitted ECP5 device (LFE5U-25F vs lfe5u45f) | first power-up (H-01) |
| U-002-1 | Two mesh daemons coexisting on one Pi | G03 bench |
| U-002-3 | Ring and span sizes | bench (R-03) |
| U-002-4 | CPU cost of the kernel copy workqueue at 208 MB/s | bench (P-05) |
| U-002-5 | Whether the CSI hardware rate is fixed or follows `bw` | bench (R-05) |
| U-003-1 | Measured 6/20 dB bandwidths of SX1262 presets | G07 T-1 |
| U-004-2 | Whether GPIO 22 (TRST) is driven on the kit | bench |
| U-005-1 / U-005-4 | FCC grant basis and frequency coverage of the modules in hand | V-04 |
| U-005-2 | ecfr.gov current text vs LII mirror | before any Tx campaign (V-06) |
| U-006-1 | `quadrf-lora-phy` behaviour when channel ≠ tile LO | G05 |
| U-008-1…4 | Ring sizing; RF-vision stream consumability; ZeroMQ interop; transceiver register map | R-03, R-09, R-06, R-04 |
| U-009-1…3 | Kraken DAQ format; copal device manager; PPS source | R-12, P-02, H-08 |
| U-010-1…5 | Vendor modules on 6.18; RP1 CSI binding; RT behaviour; device manager; musl build | P-03, P-07, R-05 |
| U-011-1…4 | Kraken DAQ format; phase stability; wiki licence; R820T2 NF | R-12, H-07 |
| F.03.8 / F.05.11 | CPU budget for whole-band occupancy + DoA (model: 0.73 core, T19) | G05 bench / G07 T-7 (R-07) |
| F.05.13 | FTFE parts cost | procurement (H-04) |
| C10 | No LoRa silicon above 2.5 GHz — closed 2026-10-08: SX1262, LR1121 and SX1280 product pages in `resources/lora/` `[S]` | closed (V-05) |
