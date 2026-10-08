# G00 — Charter and constraints

| Field | Value |
| --- | --- |
| Status | PASS |
| Opened | 2026-10-08 |
| Closed | 2026-10-08 |
| Author | project |
| Inherits from | — |

## 1. Question

Is the following question well-posed and bounded: "Can a ScaleRF QuadRF
4-channel coherent 5 GHz tile be used, directly or with a defined
modification, for LoRa work in the US Pacific Northwest, specifically (a)
receiving and spatially mapping LoRa emitters, (b) forming a LoRa phased
array, (c) transmitting at the legal limit, and (d) relaying to visible
mountain-top LoRa repeaters with a highly directional link?"

## 2. Inherited facts

None. This gate establishes the axioms.

## 3. Method

Decompose the question into sub-questions each closable by a primary source,
a derivation, or a measurement; fix the physical and regulatory frame; state
what is out of scope; set the success criterion.

## 4. Findings (axioms and scope)

- F.00.1 Physical frame: linear, time-invariant propagation over the time of one LoRa frame; far-field array theory (plane-wave phase fronts) for sources beyond 2D²/λ of the aperture; thermal noise at 290 K. `[D]` (standard assumptions; each investigation states where they fail)
- F.00.2 Regulatory frame: 47 CFR Part 15 Subpart C for unlicensed operation; 47 CFR Part 97 for amateur operation; Washington and Oregon, United States, with Canada (ISED) as a boundary condition only. `[S]` (SPEC-005)
- F.00.3 Hardware frame: one QuadRF kit (tile + Pi 5), several ESP32 LoRa nodes, one Raspberry Pi Pico LoRa HAT (model to be recorded), optional one SX1262 HAT for the Pi 5. Vendor ship date for the kit is 2026-11-30, so G01–G04 are desk investigations and bench gates are scheduled after delivery. `[S]` (user statement; Crowd Supply page 2026-10-08)
- F.00.4 Software frame: the vendor's DietPi image and packages; Meshtastic firmware and `meshtasticd`; GNU Radio and SoapySDR; third-party `quadrf-mesh`. `[S]` (SPEC-002, SPEC-004, SPEC-006)
- F.00.5 Out of scope: any transmission that is not demonstrably within SPEC-005; any modification of the proprietary RF core itself; commercial product development; MoonRF-scale (≥ 72 element) builds except as a numerical comparison. `[D]` (charter decision)
- F.00.6 Success criterion for the whole investigation: a signed G06 that states, with numbers, which of (a)–(d) are achievable, with what hardware addition, under which rule part, and a G07 field-test plan that would falsify the G06 claims. `[D]` (charter decision)

## 5. Analysis

Sub-questions and the gate that answers each:

| Sub-question | Gate | Closable by |
| --- | --- | --- |
| What does the tile actually do (frequency, bandwidth, dynamic range, geometry, interfaces)? | G01 | primary sources |
| What is permitted at 902–928 and 5725–5850 MHz, with and without a licence? | G02 | CFR text |
| What runs on the Pi, and what would we add? | G03 | repositories |
| What are all the ways to put LoRa on or through the tile, and which die on the G01/G02 facts? | G04 | enumeration + kill criteria |
| Does the surviving modification (915 MHz translation) work? | G05 | desk analysis, then bench measurement after delivery |
| What should be done about mountain-top repeaters? | G06 | link-budget derivations on G01–G05 facts |
| How would we falsify G06 in the field? | G07 | test plan |

## 6. Answer

Yes: the question decomposes into seven closable sub-questions with
explicit evidence classes and a measurable success criterion.

## 7. Recommendation (opinion)

Run G01–G04 as desk work now. Treat G05 as the project's critical
experiment: its bench gate decides whether (a) and (b) are achievable at
915 MHz. Begin procurement of the translation front-end parts in parallel with
G04, because the desk analysis already shows they are required (see G01).

## 8. Gate record

| Field | Value |
| --- | --- |
| Pass criterion | Every sub-question in §5 maps to one gate and one evidence class; F.00.6 is measurable. |
| Evidence | F.00.1–F.00.6, table in §5 |
| Decision | PASS |
| Date / signatory | 2026-10-08 / project |
| Open conjectures | None |

## 9. Carry-forward

- F.00.1–F.00.6 as stated.
