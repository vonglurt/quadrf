# Gate ledger

Synchronous, ordered. A gate is opened only when the previous one is PASS or
WAIVED. Status is mirrored in the top-level README.

| Gate | File | Question | Status | Closed | Carries forward |
| --- | --- | --- | --- | --- | --- |
| G00 | `G00-charter.md` | Is the question well-posed? | PASS | 2026-10-08 | F.00.1–6 |
| G01 | `G01-quadrf-hardware-survey.md` | What does the tile do? | PASS | 2026-10-08 | F.01.1–12 |
| G02 | `G02-regulatory-envelope-pnw.md` | What is lawful at 915 MHz and 5.8 GHz in WA/OR? | PASS | 2026-10-08 | F.02.1–13 |
| G03 | `G03-pi-software-stack.md` | What runs on the Pi; what do we write? | PASS (desk) | 2026-10-08 | F.03.2–8, graphs A/B/C |
| G04 | `G04-lora-options-inventory.md` | Which LoRa architectures survive? | PASS | 2026-10-08 | C3, C5–C9, C12; F.04.1–3 |
| G05 | `G05-915mhz-translation-frontend.md` | Does the 915 MHz translation front end work? | OPEN (desk done; bench after 2026-11-30 delivery) | — | F.05.1–12 provisional |
| G06 | `G06-mountain-repeater-directional-relay.md` | What should the directional relay be? | OPEN (desk done; signs with G05) | — | Phase 1–3 plan |
| G07 | `G07-field-test-plan.md` | How do we falsify G05/G06 in the field? | DRAFT | — | T-1…T-9 |

## Open conjectures (project-wide risk register)

| Id | Statement | Closes at |
| --- | --- | --- |
| U-001-1 | Element-port connector type/impedance/loss; pigtail feasibility | G05 bench |
| U-001-2 | 40 MHz reference exportable to the translator LO | G05 bench |
| U-001-3 | Tile NF and flatness at 5487–5513 MHz | G05 bench |
| U-001-5 | Factory bitstream auto-steer / RF-vision behaviour on a translated band | G05 bench |
| U-002-1 | Two mesh daemons coexisting on one Pi | G03 bench |
| U-003-1 | Measured 6/20 dB bandwidths of SX1262 presets | G07 T-1 |
| U-005-1 | FCC grant basis of the modules in hand | G02 follow-up |
| U-005-2 | ecfr.gov current text vs LII mirror | before any Tx campaign |
| U-006-1 | `quadrf-lora-phy` behaviour when channel ≠ tile LO | G05 |
| F.03.8 / F.05.11 | CPU budget for whole-band occupancy + DoA | G05 bench / G07 T-7 |
| F.05.13 | FTFE parts cost | procurement |
| C10 | No LoRa silicon above 2.5 GHz | verify |
| C12 | Off-the-shelf coherent receiver comparator specs | verify |
