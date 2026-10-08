# LR-006 — MAX2851 and RP1 datasheet review: what changed, what it means for the 915 MHz translation, what is still missing

*Lab report. 2026-10-08.*

<!-- SPDX-License-Identifier: MIT -->

| Field | Value |
| --- | --- |
| Status | Final |
| Author | project |
| Feeds | SPEC-001 rev 2 S-001-26…34; G01 and G05 addenda; `analysis/linkbudget.py` T14, T17, T18; backlog V-02, H-01 |

## Abstract

The user dropped the MAX2851 receiver datasheet (Maxim 19-5121 Rev 1, 3/2010,
37 pages) into the UTM share during this session, and the RP1 Peripherals
datasheet (Raspberry Pi, 2023-11-07, 93 pages) was retrieved by script.
Thirty-one receiver and synthesiser parameters were restated into
`vendor/summary/analog-devices-max2851.md` and nine behaviour statements were
added to SPEC-001. Three consequences matter for the investigation: the
device's 4.5 dB noise figure fixes what the tile's external LNA must be
(NF ≤ 1 dB, gain ≥ 13.5 dB) for the vendor's 1.2 dB system figure to hold;
the fractional-N spurs at −42 dBc lie above the 8-bit converter's SFDR, so
strong in-band emitters produce deterministic replicas in a whole-band
915 MHz map; and the selectable baseband high-pass corner (600 kHz / 10 kHz /
0.1 kHz) explains the third-party PHY's low-IF choice and constrains how the
translated band is placed. The RP1 datasheet confirms the data-path margins
(CSI 2.8 Gbit/s raw per 4-lane port; 8 Gbit/s MIPI aggregate; PCIe 2.0 x4).
Five datasheets remain to be dropped in the share.

## I. Objective

1. File the datasheets with provenance and extract the parameters the specs depend on.
2. Replace secondary-sourced statements with datasheet-sourced ones.
3. Derive the consequences for G05 and record them as tagged addenda.
4. List what is still missing and how to get it.

## II. Materials

| Item | Detail |
| --- | --- |
| MAX2851 | `resources/datasheets/MAX2851.pdf`, sha256 `4b6bb5e303586f611ff85384298787639ec126949bdbb84266a489e66f6c97db`, 37 pages, arrived in the share as "EV Bible.pdf" (identified by its first page), renamed by part number |
| RP1 | `resources/datasheets/RP1-peripherals.pdf`, "Raspberry Pi RP1 Peripherals", RP-008370-DS-1, created 2023-11-07, 93 pages |
| Extraction | `pdftotext -layout`; `MAX2851.txt` 2 925 lines; `RP1-peripherals.txt` 6 224 lines |
| Script | `analysis/linkbudget.py` T14, T17, T18 |

## III. Method

The electrical-characteristics tables (datasheet pp. 3–7) were read column
by column; each parameter was restated with its conditions; those that the
specs or gates depend on became statements; three were pushed through the
analysis script to obtain the derived consequences.

## IV. Results

### A. Parameters that changed statements (SPEC-001 rev 2)

| Statement | Parameter | Value `[S]` |
| --- | --- | --- |
| S-001-26 | RF range; LO coherence; reference; step | 4.9–5.9 GHz; coherent among channels; 40 MHz; 76.294 Hz |
| S-001-27 | DSB NF; gain | 4.5 dB (max gain), 15 dB (max − 16 dB); −2 … 68 dB |
| S-001-28 | linearity | P1dB −34 … −1 dBm; OOB IIP3 −13 … +11 dBm; blocker −24 dBm |
| S-001-29 | baseband filters | LPF 9.5 / 19 MHz; HPF 600 k / 10 k / 0.1 kHz; IQ 0.1 dB / 0.2°; 40 dB sideband |
| S-001-30 | synthesiser | −35 dBc integrated; spurs −42 dBc (0–19 MHz), −66 dBc (40 MHz); LO leakage −75 dBm/MHz |
| S-001-31 | flatness | ≤ 4.2 dB p-p (1.8 typ) over 4.9–5.9 GHz |
| S-001-34 (RP1 part) | data path | PCIe 2.0 x4; 2 × xHCI/USB 3.0, > 10 Gbit/s; 2 × 4-lane MIPI, 8 Gbit/s |

### B. Derived consequences (`analysis/linkbudget.py`)

- T17: −35 dBc → 1.44° rms LO phase. Against the G05 criterion of ≤ 5° rms inter-channel phase this is small, and it is common-mode across the four channels of one IC (coherent LO) so it does not enter the bearing at all. `[D]`
- T17: −42 dBc spurs vs 8-bit SFDR ≈ 49.9 dB: a −30 dBm in-band emitter at the antenna produces replicas at −72 dBm, 58 dB above a −130 dBm LoRa signal in whichever slot the replica lands; a −60 dBm emitter's replicas are still 28 dB above it. Replicas are deterministic in offset (the synthesiser's fractional spur set at the chosen LO) and share the parent's bearing. `[D]`
- T18: 4.5 dB device NF with a 0.8–1.0 dB, 13.5 dB LNA and ≤ 0.5 dB ahead gives 1.1–1.8 dB system NF, consistent with the vendor's 1.2 dB; the FTFE chain (LNA 1 dB/20 dB, 7 dB conversion loss, tile 1.2 dB) is 1.19 dB, 3.19 dB with a 2 dB SAW ahead of the LNA. `[D]`
- T14: four interleaved channels at 26 MSPS use 59 % of one CSI port's raw rate; the RP1's 8 Gbit/s aggregate and 16 Gbit/s PCIe link leave the host side unconstrained. `[D]`
- HPF corner: at zero-IF, the 600 kHz or 10 kHz high-pass corners would notch or distort a 125–500 kHz chirp centred on DC; the 0.1 kHz corner or a low-IF offset (the third-party PHY uses 500 kHz) avoids it; for the whole-band FTFE capture, the LO is placed at the band centre (5500 MHz) and the slot at DC (slot 52, 914.875 MHz) must be treated with the HPF set to 0.1 kHz or discarded. `[D]` (S-001-29; SPEC-006 S-006-4)

### C. Mitigations for the spur replicas (for G05 criterion 6)

1. Choose the translator LO so that the tile's known spur offsets map outside the 104 slots or onto slots that are already excluded (desk, once the spur offsets are measured at the bench).
2. Flag replicas in `qrf-dspd`: a detection whose bearing equals a strong emitter's within σ and whose offset matches a measured spur offset is labelled `replica` in `Occupancy`.
3. Keep the tile's RF gain below maximum when a strong emitter is present; P1dB improves from −34 dBm to −18 dBm at max − 16 dB (S-001-28) while the NF rises to 15 dB; the FTFE's own LNA gain sets the overall NF, so the trade is affordable for mapping (not for weakest-signal decoding).

### D. Still missing

| Datasheet | Needed for | Route |
| --- | --- | --- |
| MAX2850 (4-channel transmitter) | SPEC-001 transmit statements; C4 later | analog.com in a browser → share → `import-shared.sh` |
| MAX2871 (synthesiser) | FTFE LO candidate (SPEC-007 S-007-9) | same |
| SX1262 | SPEC-003 S-003-2 thresholds, bandwidths, SF range | semtech.com (login/interstitial) → share |
| SE5004L (PA) | S-001-9 rating and linearity | skyworksinc.com → share |
| SKY65404-31 (LNA) | T18 placeholders → `[S]` | same |

## V. Discussion

The datasheet confirms every first-pass physics statement about the tile and
adds two terms the first pass lacked. The spur term is the only one that
changes bench work: criterion 6 (measure replica offsets and levels with a
−30 dBm CW) is added to G05 so that the overlay's replica flagging is built
on measured offsets rather than on the datasheet's bound.

What would refute §B: measured spurs below −60 dBc at the tile's output (the
datasheet's −42 dBc is a typical at one LO; a different fractional word may be
quieter), in which case the replica logic is unnecessary.

## VI. Recommendations and best practice

1. Drop the five remaining datasheets in the share; run the import; the manifest rows print themselves (V-02).
2. Re-read SPEC-001 S-001-2 and S-001-9 against the MAX2850 and SE5004L datasheets when they arrive and retag.
3. Add G05 criterion 6 and the replica-flagging rule to the DSP backlog (R-13).

## VII. References

`resources/datasheets/MAX2851.pdf` pp. 1, 3–7; `resources/datasheets/RP1-peripherals.pdf` ch. 1; `vendor/summary/analog-devices-max2851.md`; `vendor/summary/raspberry-pi-rp1.md`; SPEC-001 rev 2; SPEC-006; `analysis/linkbudget.py` T14, T17, T18.
