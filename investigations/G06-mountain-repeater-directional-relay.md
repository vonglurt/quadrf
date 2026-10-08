# G06 — Mountain-top repeaters: focused, highly directional relay

| Field | Value |
| --- | --- |
| Status | OPEN — desk recommendations complete; depends on G05 bench for the 915 MHz array branch |
| Opened | 2026-10-08 |
| Closed | — |
| Author | project |
| Inherits from | G05 |

## 1. Question

Given line of sight to mountain-top LoRa (Meshtastic) repeaters in the
Pacific Northwest, what does a highly directional, legal, "focused" link to
them require, and what role can the tile play?

## 2. Inherited facts

- F.01.6, F.01.8, F.01.9, F.02.3, F.02.4, F.02.7, F.02.8, F.02.10–F.02.13, F.04.2, F.04.3, F.05.5, F.05.7, C5, C6, C7, C9.

## 3. Method

Compute link budgets for representative PNW geometries (valley or city floor
to a summit at 300–2000 m, 10–80 km) under each lawful regime, include the
channel impairments that dominate in this region (conifer foliage, knife-edge
terrain, Fresnel clearance), and derive what directionality buys in each
regime from the rule structure, not from intuition.

## 4. Findings

Geometry and channel:

- F.06.1 Radio horizon (4/3 earth): user at 2 m to a 300 m summit, 77 km; to a 1200 m summit, 148 km. Visible PNW summits are within the horizon by construction. `[D]` (analysis T6)
- F.06.2 Free-space loss at 40 km: 123.7 dB at 915 MHz, 139.8 dB at 5800 MHz; the 16.0 dB difference is frequency-independent of distance and is exactly recovered by fixed-aperture antennas at both ends, whose gain scales as f². `[D]` (analysis T2; G = 4πA/λ²)
- F.06.3 Fresnel first-zone radius at mid-path, 40 km: 57 m at 915 MHz, 23 m at 5.8 GHz. A "visible" summit with the optical line grazing a ridge may still be obstructed at 915 MHz; the 5.8 GHz link needs less clearance. `[D]` (analysis T5)
- F.06.4 Conifer foliage (Weissberger): 100 m of canopy costs 19 dB at 915 MHz and 33 dB at 5.8 GHz; 20 m costs 7.5 and 13 dB. A 915 MHz path tolerates a tree line; a 5.8 GHz path must clear it. `[D]` (analysis T3)
- F.06.5 Single knife-edge: 6 dB at grazing (v = 0), 14 dB at v = 1, 22 dB at v = 3. `[D]` (analysis T4)

Link budgets at 40 km LOS, repeater with 6 dBi omni (analysis T7):

- F.06.6 Part 15 at 915 MHz, 36 dBm EIRP (rule ceiling, any antenna): received −81.7 dBm; margin 49.8 dB at LONG_FAST, 36.8 dB at SHORT_TURBO against an SX1262 receiver. The uplink to a visible summit closes with tens of dB to spare on an omni; directivity is not needed for the LOS budget. `[D]`
- F.06.7 Part 15 at 5.8 GHz, fixed P2P, tile-to-tile: 1 W + 12 dBi → 42 dBm EIRP, margin 47.6 dB at LONG_FAST; with a 72-element far end (24.6 dBi) 72.8 dB. A tile pair closes a 40 km 5.8 GHz LoRa link with margin for 30 dB of Fresnel/foliage impairment. `[D]`
- F.06.8 Part 97 at 915 MHz, 10 W + 12 dBi Yagi: 52 dBm EIRP, margin 71.8 dB; 10 W + 24.6 dBi array: 97 dB. Power beyond 10 W is unnecessary for any LOS PNW path and conflicts with §97.313(a). `[D]`+`[S]`

What directionality buys, by regime:

- F.06.9 Unlicensed 915 MHz: transmit EIRP is pinned at 36 dBm (F.02.4), so a 12 dBi antenna on a 30 dBm module requires backing off to 24 dBm. The gain is realised (i) on receive, in full: +12 dB downlink margin from the repeater, (ii) as spatial filtering: emitters outside the main lobe are suppressed by the pattern's sidelobe level (−13 dB for a uniform aperture, −20 to −30 dB for a tapered Yagi/panel), raising the repeater's SIR against the urban LoRa floor, and (iii) as reduced self-interference to the mesh in other directions. The hidden-node and airtime-collision problems of a dense mesh are mitigated by (ii), not by power. `[D]` (reciprocity; array sidelobes)
- F.06.10 Unlicensed 5.8 GHz: directionality is rewarded by rule (F.02.7). A tile pair, one at the summit site, one at the user, is a lawful high-EIRP directed LoRa or OFDM link; its throughput at SHORT_TURBO is 21.9 kbit/s (LoRa) or megabits (OFDM TUN demo). It requires a cooperating node at the summit; it cannot talk to existing 915 MHz repeaters. `[D]`+`[S]` (F.04.2; SPEC-003 S-003-6; SPEC-002 S-002-9)
- F.06.11 Licensed 915 MHz: any gain, 1.5 kW PEP ceiling, no encryption, ID every 10 min. This is the only regime in which a transmit phased array at 915 MHz (C4) has lawful purpose; the receive array (C3) is useful in both regimes. `[S]`+`[D]` (F.02.10–F.02.12)
- F.06.12 Polarisation: Meshtastic repeaters use vertical linear whips. A vertical Yagi or vertical monopole array matches; the tile's circular patches at 5.8 GHz are irrelevant to that link (F.06.10 is tile-to-tile, RHCP both ends, matched). `[D]` (F.01.8)
- F.06.13 Exposure: a 12 dBi Yagi at 24 dBm conducted (36 dBm EIRP) needs 0.23 m clearance; at Part 97 power levels with 24.6 dBi, 6 m in the main lobe; mount the main lobe above head height toward the summit. `[D]` (analysis T11)

## 5. Analysis

Directionality at 915 MHz is a receive-side and interference-side instrument
under Part 15, and a transmit instrument only under Part 97. The tile's
native strength is at 5.8 GHz, where the rule rewards directivity and the
tile already carries a LoRa PHY. The physically and legally coherent
architecture for a "focused relay" is therefore layered: a 915 MHz Meshtastic
node with a vertical Yagi (receive gain, spatial filtering) at the user, and,
if a cooperating summit site exists, a tile-to-tile 5.8 GHz directed link
carrying the mesh over IP or LoRa-at-5.8. The receive LoRa array (G05)
adds bearing knowledge: which summit is actually delivering frames, from
which direction the urban floor arrives, and where to point the Yagi.

## 6. Answer (provisional)

Yes, a focused, highly directional relay is achievable: at 915 MHz under
Part 15 only as a receive/filtering gain with a 36 dBm EIRP transmit ceiling;
at 915 MHz under Part 97 with full transmit directivity and up to 10 W (more
is unnecessary); at 5.8 GHz under Part 15 as a tile-to-tile point-to-point
link with unlimited antenna gain, provided a node is placed at the summit.

## 7. Recommendation (opinion)

Phase 1 (no tile, now): SX1262 node on the Pi 5 (or the Pico HAT) with a
10–12 dBi vertical Yagi aimed at the summit; conducted power set to 30 − (G − 6)
dBm; preset SHORT_TURBO or LONG_TURBO for rule-text compliance, LONG_FAST to
match the existing mesh; log RSSI/SNR per repeater. Phase 2 (G05 passed):
FTFE + 164 mm monopole array for bearing and occupancy; confirm Phase 1
pointing and quantify the sidelobe-filtered SIR. Phase 3 (if a summit site
is available): tile pair at 5.8 GHz, 500 kHz LoRa preset or OFDM TUN, with
Meshtastic bridged over IP; 1 W aggregate, fixed mounting, documented as
point-to-point per §15.247(c)(1)(iii). If licensed: Meshtastic licensed mode,
call sign in `CALLSIGN`, encryption off, 10 W maximum, exposure evaluation
on file.

## 8. Gate record

| Field | Value |
| --- | --- |
| Pass criterion | Each of (a)–(d) in the charter is answered with a number, a rule part and a hardware list; the answer for (b) is conditional on G05 criteria (1)–(5). |
| Evidence | F.06.1–F.06.13 |
| Decision | OPEN (desk complete; signs when G05 signs) |
| Date / signatory | — |
| Open conjectures carried as risk | existence of a cooperating summit site (Phase 3); U-005-3 if the path faces BC |

## 9. Carry-forward (provisional)

- Phase 1–3 plan; F.06.6–F.06.11 as the numbers G07 must falsify.
