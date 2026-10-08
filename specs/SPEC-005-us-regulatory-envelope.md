# SPEC-005 — US regulatory envelope for 902–928 MHz and 5725–5850 MHz emissions

| Field | Value |
| --- | --- |
| Status | Reviewed (rev 1); rev 2 additions S-005-20…22 await a second read |
| Revision | 2 |
| Date | 2026-10-08 |
| Subject | The constraints a transmitter in this project must satisfy, as behaviour goals, under 47 CFR Part 15 (unlicensed) and Part 97 (amateur) |
| Primary sources | `resources/regulatory/cfr-47-15.247.html`, `-15.249`, `-15.23`, `-15.203`, `-15.205`, `-15.209`, `-97.3`, `-97.113`, `-97.119`, `-97.303`, `-97.311`, `-97.313` (LII mirror, retrieved 2026-10-08) |
| Depends on | SPEC-003 |

This is our reading of the rule text, written so that each statement can be
checked against the cited paragraph. It is not legal advice.

## Part 15, §15.247 (frequency hopping and digitally modulated systems)

- S-005-1. An emission qualifies as digitally modulated only if its 6 dB bandwidth is ≥ 500 kHz. `[S]` (§15.247(a)(2))
- S-005-2. A frequency-hopping emission in 902–928 MHz with hopping-channel 20 dB bandwidth ≥ 250 kHz shall use ≥ 25 hopping frequencies with ≤ 0.4 s average occupancy per frequency in any 10 s; with 20 dB bandwidth < 250 kHz, ≥ 50 frequencies and 0.4 s in 20 s; the 20 dB channel bandwidth shall not exceed 500 kHz; channels are selected pseudo-randomly and used equally on average. `[S]` (§15.247(a)(1), (a)(1)(i))
- S-005-3. In 5725–5850 MHz a hopper shall use ≥ 75 frequencies, 20 dB bandwidth ≤ 1 MHz, 0.4 s in 30 s. `[S]` (§15.247(a)(1)(ii))
- S-005-4. Peak conducted output power shall not exceed 1 W for digitally modulated systems in all three bands; for 902–928 MHz hoppers, 1 W with ≥ 50 channels or 0.25 W with 25–49 channels. Conducted power is the total delivered to all antennas and antenna elements, summed. `[S]` (§15.247(b)(2), (b)(3))
- S-005-5. Those limits assume antenna directional gain ≤ 6 dBi. For higher gain the conducted power shall be reduced by the excess in dB, except as (c) permits. Consequence: in 902–928 MHz the EIRP ceiling is 36 dBm for any antenna. `[S]`+`[D]` (§15.247(b)(4); analysis T8)
- S-005-6. Fixed point-to-point systems in 5725–5850 MHz may use any antenna gain with no reduction in conducted power. In 2400–2483.5 MHz the reduction is 1 dB per 3 dB of excess. No such relief exists for 902–928 MHz. `[S]` (§15.247(c)(1)(i), (ii))
- S-005-7. "Fixed point-to-point" excludes point-to-multipoint, omnidirectional applications, and multiple co-located radiators transmitting the same information; the operator or professional installer is responsible for exclusive point-to-point use. `[S]` (§15.247(c)(1)(iii))
- S-005-8. For digitally modulated systems the conducted PSD shall not exceed 8 dBm in any 3 kHz during continuous transmission. A 1 W, 500 kHz uniform chirp sits at 7.8 dBm/3 kHz, i.e. at the limit; any narrower emission at 1 W exceeds it. `[S]`+`[D]` (§15.247(e); SPEC-003 S-003-5)
- S-005-9. Out-of-band emissions in any 100 kHz outside the band shall be ≥ 20 dB (30 dB under the alternative measurement) below the in-band 100 kHz peak, and emissions in §15.205 restricted bands shall meet §15.209 limits. `[S]` (§15.247(d))
- S-005-10. Hoppers may incorporate adaptive channel avoidance but shall not coordinate hop sets among transmitters to avoid simultaneous occupancy. `[S]` (§15.247(h))

## Part 15, other paragraphs

- S-005-11. Outside §15.247, an intentional radiator in 902–928 MHz is limited by §15.249 to a fundamental field strength of 50 mV/m at 3 m (harmonics 500 µV/m). 50 mV/m at 3 m corresponds to an EIRP of E²d²/30 = 0.75 mW = −1.2 dBm. `[S]`+`[D]` (§15.249(a))
- S-005-12. Home-built devices, ≤ 5 units, not marketed and not from a kit, need no equipment authorisation but must meet the technical standards using good engineering practice. `[S]` (§15.23)
- S-005-13. An intentional radiator shall use a permanently attached antenna or a unique coupling; replacement antennas must not exceed the gain of the authorised antenna (relevant when swapping antennas on certified modules). `[S]` (§15.203)

## Part 97 (amateur), 33 cm and 5 cm bands

- S-005-14. Transmitter power shall not exceed 1.5 kW PEP, and the minimum power necessary shall be used; no antenna-gain restriction is stated in the power rule. `[S]` (§97.313(a), (b))
- S-005-15. In the 33 cm band amateurs are secondary to US Government stations, the FCC Location and Monitoring Service, and foreign fixed stations, and must accept ISM interference. Transmission is prohibited only from a defined region of Texas/New Mexico and, in four narrow segments, a defined region of Colorado/Wyoming. Washington and Oregon are outside both. `[S]`+`[D]` (§97.303(n), (e); coordinates in the rule)
- S-005-16. Spread-spectrum emissions shall not be used to obscure meaning; a licensee must be able to produce a record convertible to the original information on request, and must cease or restrict SS on a Regional Director's instruction. No SS-specific power limit remains in §97.311. `[S]` (§97.311(a), (c))
- S-005-17. No amateur station shall transmit messages encoded to obscure their meaning, nor broadcasts (one-way transmissions to the general public), nor communications for compensation. Meshtastic's default AES channel encryption is incompatible with Part 97 operation unless disabled (the firmware's licensed mode does so). `[S]`+`[D]` (§97.113(a)(4), (b); SPEC-004 S-004-4)
- S-005-18. The station must identify by call sign at least every 10 minutes during and at the end of a communication. `[S]` (§97.119(a))

## RF exposure (both services)

- S-005-19. General-population MPE: 300–1500 MHz, f/1500 mW/cm² (0.61 mW/cm² at 915 MHz); 1500–100 000 MHz, 1.0 mW/cm². Far-field compliance distances: 36 dBm EIRP at 915 MHz, 0.23 m; 52 dBm, 1.44 m; 64.6 dBm, 6.1 m; 54.6 dBm at 5.8 GHz, 1.5 m. `[S]` (47 CFR 1.1310(e)(1) Table 1; resources/regulatory/cfr-47-1.1310.html; vendor/cfr47/47-CFR-1.1310.txt) + `[D]` (analysis T11)

## Revision 2 additions

- S-005-20. The general-population MPE of §1.1310(e)(1) Table 1 is f/1500 mW/cm² for 300–1500 MHz and 1.0 mW/cm² for 1500–100 000 MHz, averaged over 30 minutes; the occupational limits are f/300 and 5 mW/cm², averaged over 6 minutes; the MPE route may be used in place of SAR for 300 kHz–6 GHz except for portable devices. `[S]` (§1.1310(d)(2), (e)(1); vendor/cfr47/47-CFR-1.1310.txt)
- S-005-21. An amateur licensee must ensure compliance with the exposure rules of §§1.1307(b), 2.1091 and 2.1093 before transmitting from any place where the §1.1310 limits could be exceeded, may evaluate household members against the occupational limits only with training, and must act to prevent exposure where the evaluation shows an exceedance. `[S]` (§97.13(c)(1)–(2); vendor/cfr47/47-CFR-97.13.txt)
- S-005-22. A reported FCC grant for a Seeed Wio-SX1262 module (FCC ID Z4T-WIO-SX1262, 2024-10-16) lists equipment class DSS and frequency rows 902.3–914.9 MHz, which would exclude the Meshtastic US slots above 915 MHz from that module's authorisation; the record has not been read at the FCC database. `[C]` (vendor/summary/fcc-grants.md; closes at backlog V-04)

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-005-1 | Under which §15.247 paragraph the grantee of each Meshtastic module in hand obtained certification, and what host obligations the grant imposes (e.g. hopping) | G02 (read the FCC grant for each module) |
| U-005-2 | Current ecfr.gov text vs LII mirror | G02 (verify before any transmission campaign) |
| U-005-4 | Whether each module's grant covers the full 902–928 MHz band or only a sub-range, and under which class (DSS/DTS) | backlog V-04 |
| U-005-3 | ISED RSS-247 equivalence for paths facing British Columbia | G06 if a BC-facing path is planned |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 1 | 2026-10-08 | First reviewed version |
| 2 | 2026-10-08 | §1.1310 and §97.13 retrieved; S-005-19 retagged `[S]`; S-005-20…22 added; U-005-4 added |
