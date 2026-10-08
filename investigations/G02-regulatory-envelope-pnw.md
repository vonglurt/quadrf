# G02 — Regulatory envelope: US 902–928 MHz and 5725–5850 MHz, Pacific Northwest

| Field | Value |
| --- | --- |
| Status | PASS |
| Opened | 2026-10-08 |
| Closed | 2026-10-08 |
| Author | project |
| Inherits from | G01 |

## 1. Question

What is the maximum lawful emission, as EIRP and as waveform constraints, for
a LoRa-type signal in 902–928 MHz and in 5725–5850 MHz from Washington or
Oregon, (a) unlicensed under Part 15 and (b) under an amateur licence, and
what does each imply for directional antennas?

## 2. Inherited facts

- F.01.9 (tile aggregate 4 W, Part 15 sums across elements), F.01.6 (tile beam), F.00.2 (frame).

## 3. Method

Read §15.247, §15.249, §15.23, §15.203, §15.205, §15.209 and §97.3, §97.113,
§97.119, §97.303, §97.311, §97.313 from the LII e-CFR mirror (retrieved
2026-10-08). Restate as SPEC-005 behaviour statements. Compute consequences
(PSD, EIRP ceilings, exposure distances) in `analysis/linkbudget.py` T7, T8,
T11.

## 4. Findings

Part 15, 902–928 MHz:

- F.02.1 A single-slot 250 kHz LoRa emission (LONG_FAST and all other 250/125 kHz presets) has a 6 dB bandwidth of ≈ 250 kHz and therefore does not meet the ≥ 500 kHz digital-modulation definition; it does not hop, so it is not a hopper either. Under §15.247 it has no home. `[D]` (SPEC-003 S-003-4, SPEC-004 S-004-8, SPEC-005 S-005-1, S-005-2)
- F.02.2 The Part 15 provision that remains for such an emission is §15.249: 50 mV/m at 3 m, equal to −1.2 dBm EIRP. `[S]`+`[D]` (SPEC-005 S-005-11)
- F.02.3 A 500 kHz preset (SHORT_TURBO, LONG_TURBO) satisfies the bandwidth definition and is permitted 1 W conducted with ≤ 6 dBi antenna, i.e. 36 dBm EIRP, and sits at 7.8 dBm/3 kHz against the 8 dBm/3 kHz PSD limit at exactly 1 W. `[D]` (SPEC-005 S-005-1, S-005-4, S-005-8; SPEC-003 S-003-5)
- F.02.4 For any antenna gain above 6 dBi in 902–928 MHz the conducted power is reduced dB-for-dB; the EIRP ceiling is 36 dBm independent of antenna. A 22 dBm module into a 12 dBi Yagi (34 dBm EIRP) is within the ceiling; a 30 dBm module must be set to 24 dBm with the same Yagi. `[D]` (analysis T8)
- F.02.5 Certified Meshtastic modules carry FCC grants whose basis (DTS vs FHSS, host hopping obligations) we have not read; the gap between F.02.1 and the existence of those grants is an open item, not a licence to transmit. `[C]` (SPEC-005 U-005-1)
- F.02.6 A home-built transmitter (≤ 5 units, not for sale) needs no equipment authorisation but must meet the same technical limits by good engineering practice. An SDR-generated LoRa waveform from the tile is such a device. `[S]` (SPEC-005 S-005-12)

Part 15, 5725–5850 MHz:

- F.02.7 A digitally modulated (≥ 500 kHz) emission is permitted 1 W conducted summed across elements, and for fixed point-to-point links the antenna gain is unlimited with no power reduction. A 4-element tile at ≈ 12 dBi gives 42 dBm EIRP; a 72-element array at ≈ 24.6 dBi gives 54.6 dBm. `[S]`+`[D]` (SPEC-005 S-005-4, S-005-6; analysis T7)
- F.02.8 Point-to-point excludes point-to-multipoint, omnidirectional applications and co-located radiators sending the same information; a tile steering between several far ends is one radiator, but serving several at once is point-to-multipoint. `[S]` (SPEC-005 S-005-7)
- F.02.9 The 500 kHz LoRa presets on the tile at 5.8 GHz are therefore lawful Part 15 waveforms at up to 1 W aggregate, subject to a measured 6 dB bandwidth ≥ 500 kHz and out-of-band attenuation per §15.247(d). `[D]` (F.02.3, F.02.7)

Part 97:

- F.02.10 With a Technician-class or higher licence, 902–928 MHz (33 cm) and 5650–5925 MHz (5 cm) are available; transmitter power up to 1.5 kW PEP with no antenna-gain restriction; minimum necessary power applies. Washington and Oregon are outside the 33 cm geographic exclusions. `[S]` (SPEC-005 S-005-14, S-005-15)
- F.02.11 Part 97 forbids encryption, broadcasting to the general public, and pecuniary communications; requires identification every 10 minutes; spread-spectrum users must keep a decodable record on request. Meshtastic's licensed mode disables encryption and inserts the call sign. `[S]` (SPEC-005 S-005-16 … S-005-18; SPEC-004 S-004-4)
- F.02.12 "Focused broadcast" in the user's sense (a directed emission toward a repeater) is a point-to-point communication, not a broadcast in the §97.113(b) sense, provided it addresses amateur stations and not the general public. `[D]` (definition in §97.3 of broadcasting)

Exposure:

- F.02.13 Far-field MPE compliance distance is 0.23 m at 36 dBm EIRP (915 MHz), 1.4 m at 52 dBm, 6.1 m at 64.6 dBm, 1.5 m at 54.6 dBm (5.8 GHz). Part 97 stations must evaluate exposure; the array's main lobe at summit-pointing elevation seldom intersects occupied space, but the evaluation is mandatory. `[D]`+`[S]` (analysis T11; 47 CFR 97.13(c), not retrieved)

## 5. Analysis

Two regimes result. Unlicensed: at 915 MHz the EIRP is pinned at 36 dBm by
rule and the only 1 W-eligible presets are the 500 kHz ones; directional
antennas cannot raise EIRP, so their lawful value is receive gain and spatial
selectivity. At 5.8 GHz the rule rewards directionality: unlimited gain for
fixed point-to-point. Licensed: at 915 MHz the power rule is 1.5 kW PEP with
no gain restriction, but encryption must be off and the link must be
identified; that is exactly the configuration Meshtastic's licensed mode
produces.

## 6. Answer

Unlicensed at 915 MHz: 36 dBm EIRP, 500 kHz presets only, any antenna, Tx
power reduced for gain > 6 dBi. Unlicensed at 5.8 GHz, fixed point-to-point:
1 W conducted into any gain. Licensed (Part 97), either band: 1.5 kW PEP,
any gain, no encryption, call-sign ID, PNW unrestricted. Single-slot 250 kHz
Meshtastic presets are not a §15.247 waveform on their own (F.02.1, F.02.5).

## 7. Recommendation (opinion)

For an unlicensed PNW LoRa link that must be legal on the rule text alone,
run SHORT_TURBO or LONG_TURBO. For the directional relay, obtain or use an
amateur licence and run licensed mode, or move the directional segment to
5.8 GHz where Part 15 already permits it. Before any transmission campaign,
read the FCC grant of each module in hand (F.02.5) and re-check ecfr.gov.

## 8. Gate record

| Field | Value |
| --- | --- |
| Pass criterion | EIRP ceiling, waveform eligibility, antenna-gain rule, and licensed-mode obligations stated for both bands and both regimes, each traceable to a CFR paragraph in `resources/regulatory/`. |
| Evidence | F.02.1–F.02.13 |
| Decision | PASS |
| Date / signatory | 2026-10-08 / project |
| Open conjectures carried as risk | F.02.5 (module grant basis) closes when grants are read; U-005-2 (ecfr.gov re-check); 47 CFR 1.1310 and 97.13 to be retrieved into resources/ |

## 9. Carry-forward

- F.02.1, F.02.3, F.02.4, F.02.6, F.02.7, F.02.8, F.02.9, F.02.10, F.02.11, F.02.12, F.02.13.

## 10. Errata and addenda (2026-10-08, second pass)

The gate decision is unchanged.

- F.02.14 §1.1310 and §97.13 were retrieved (LII mirror; public-domain text in `vendor/cfr47/`). The MPE figures used in F.02.13 are now sourced: general population f/1500 mW/cm² (300–1500 MHz) and 1.0 mW/cm² (1500–100 000 MHz), 30-minute average; an amateur licensee must evaluate exposure under §1.1307(b) before transmitting where the limits could be exceeded. `[S]` (SPEC-005 S-005-19…21)
- F.02.15 A reported FCC grant for a Seeed Wio-SX1262 module lists class DSS and frequency rows 902.3–914.9 MHz, which would place Meshtastic slots above 915 MHz outside that module's authorisation. The record is from a third-party mirror via a search summary and is unverified; it sharpens F.02.5 and U-005-1 but closes nothing. `[C]` (SPEC-005 S-005-22; U-005-4; backlog V-04)
- F.02.16 Erratum to the first-pass gate record: "47 CFR 1.1310 and 97.13 to be retrieved into resources/" is done; U-005-2 (ecfr.gov re-check) remains open until a transmission campaign is planned. `[S]`
