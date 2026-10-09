# LR-001 — Second-pass review of the first investigation pass: coherence audit, defects, and the rules adopted

*Lab report. 2026-10-08.*

<!-- SPDX-License-Identifier: MIT -->

| Field | Value |
| --- | --- |
| Status | Final |
| Author | project (second pass, higher-effort run) |
| Feeds | G01–G05 and G07 errata sections; SPEC-001/002/004/005 revision 2; `docs/00-process.md` §3.4–3.5, §5–6; `backlog.md` P0 entries |

## Abstract

The 27 files committed in `810feef` (8 gates, 7 clean-room specs, process
document, templates, manifest, fetch script, analysis script) were re-read
against their sources, against each other, and against the newly available
primary documents (the MAX2851 datasheet supplied through the UTM share, the
RP1 datasheet, two further CFR sections, the Meshtastic per-board templates,
the vendor driver sources and licence files, the KrakenSDR documentation).
The chain of gates is internally consistent: every carried-forward fact is
used by the gate that inherits it and no gate assumes a fact its predecessor
did not carry. Fifteen defects or gaps were found; none reverses a gate
decision, two change a tag (one conjecture became sourced, one sourced claim
became conjecture), and six are new unknowns. An evidence-tag linter now runs
clean over 181 spec statements and 90 gate findings. The rules adopted are:
dated errata sections instead of edits to signed gates, commit hashes in the
manifest, a vendor ledger for licences, and a datasheet drop-folder procedure.

## I. Objective

1. Verify that each of G00–G07 inherits only facts the previous gate carried forward, and that each `[S]` statement names a file in `resources/`.
2. Find statements whose tag is stronger than their source supports, and statements whose source arrived after the first pass.
3. Check the derived numbers against `analysis/linkbudget.py` and find numbers computed inline.
4. Record every defect with its disposition (fixed, routed to a backlog entry, or accepted).

## II. Materials

| Item | Detail |
| --- | --- |
| Repository state reviewed | `810feef` "Add gated investigation framework, clean-room specs and G00–G07", pushed 2026-10-08 |
| New primary sources | `resources/datasheets/MAX2851.pdf` (sha256 `4b6bb5e3…97db`, 37 pp., 19-5121 Rev 1); `resources/datasheets/RP1-peripherals.pdf` (RP-008370-DS-1, 93 pp.); `resources/regulatory/cfr-47-1.1310.html`, `cfr-47-97.13.html`; `resources/krakensdr/*` (4 files); `resources/lora/semtech-sx1262-product.html` |
| Vendor sources re-read | `fpga_csi.h`, `fpga-csi.c`, `fpga-csi.dts`, `fpga-dsi.c`, `jtag.c`, `MipiDevice.cpp/.hpp`, `NEON.cpp`, `quadrf-load`, `10-boot`, `debian/copyright`, `licenses/*`, `updates.html` FAQ; `quadrf-mesh` `air-ipc-v1.md`, `quadrf-lora-phy.default`, `config.hpp`, `wander.cpp`; Meshtastic `bin/config.d/*.yaml` |
| Tools | `scripts/lint-tags.py` (new), `analysis/linkbudget.py` T1–T20, `pdftotext` 25.12, `python3` 3.14.7 |

## III. Method

1. Read every tracked file in full. For each gate, list its inherited facts and check each against the previous gate's carry-forward list.
2. For each `[S]` statement, open the cited file and line; where the line was not found, mark the statement.
3. Run the linter in its first form (every `- F.` / `- S-` line must carry a tag), inspect each hit, then refine the linter to exempt the two reference-list sections.
4. Recompute each `[D]` figure from the script; where the document shows arithmetic not in the script, add a table.
5. Read the new primary sources and list which existing statements they confirm, refine or contradict.

## IV. Results

### A. Chain consistency

- The carry-forward lists of G00…G06 match the inherited-facts lists of G01…G07 exactly (G01 inherits F.00.1, F.00.3; G02 inherits F.01.9, F.01.6, F.00.2; …; G07 inherits F.05.5, F.05.7, G05 criteria, F.06.6, F.06.9). No gate uses an uncarried fact. `[M]` (by reading; this report)
- The decisive chain F.01.1 → F.01.2 → F.01.3 (no native 915 MHz) is now datasheet-backed: the receiver IC's RF input range is 4.9–5.9 GHz. `[S]` (SPEC-001 S-001-26)

### B. Defects and gaps

| Id | Finding | Severity | Disposition |
| --- | --- | --- | --- |
| D1 | S-001-2 sourced the converter frequency range from ADI product pages (secondary). | tag strength | Superseded by S-001-26…31 from the datasheet; S-001-2 kept with its original citation. |
| D2 | S-005-19 cited §1.1310 as "not retrieved, verify". | missing source | §1.1310 and §97.13 retrieved; S-005-19 retagged `[S]`; S-005-20/21 added. |
| D3 | U-001-2 named a "SiT8008 MEMS" reference oscillator; the schematic text contains no such designator and no "40 MHz" string. The source was the vendor BOM page. | over-strong | Noted in G01 addendum; the oscillator identity is `[C]` until the schematic page is read visually. |
| D4 | The schematic's FPGA part (LFE5U-25F-6BG256C) differs from the OpenOCD tap name (lfe5u45f); not noticed in the first pass. | new unknown | U-001-6; backlog H-01 (read IDCODE). |
| D5 | G04 C10 "no LoRa silicon above 2.5 GHz" was tagged `[S] common knowledge, verify`. | over-strong | Semtech SX1262 page (150–960 MHz) is now in `resources/`; LR1121/SX128x facts are from a search summary only. C10 is `[S]` for SX1262, `[C]` for the family statement; backlog V-05. |
| D6 | SPEC-004 S-004-6 said "Waveshare-class HATs use GPIO 18" without listing alternatives, leaving G03's recommendation ("choose a HAT whose reset is not GPIO 18") unactionable. | gap | S-004-9…12 enumerate 14 templates; MeshAdv-Mini 900M22S and the USB sticks are clear of the JTAG pins. |
| D7 | G05 F.05.4 computed the cascade NF inline (1.26 + 0.040 + 0.016); the process forbids inline arithmetic. | process | T18 reproduces it (1.19 dB without pre-filter, 3.19 dB with a 2 dB SAW); the first pass's "≈ 1.2 dB / ≈ 3 dB" stands. |
| D8 | The first pass had no statement about the tile LO's fractional-N spurs; at −42 dBc they exceed the 8-bit SFDR and create deterministic replicas of strong emitters. | missing physics | G05 addendum F.05.14; T17; new bench item (criterion 6). |
| D9 | First linter run flagged 12 lines; all were reference lists in "Inherited facts" / "Carry-forward" sections, not statements. | tool | Linter exempts those sections; 0 untagged statements remain. |
| D10 | The manifest recorded no commit hashes for the four cloned repositories. | reproducibility | Manifest rewritten with hashes and dates. |
| D11 | README headline 2 ("two tiles form a LoRa-modulated 5.8 GHz link today") rests on the third-party project's own measurements, not ours. | wording | README now says so. |
| D12 | The first commit was made with a `-c user.*` identity override instead of the configured global identity. | process | LR-002 §P6; memory note. |
| D13 | The process document had no document kind for practice/architecture writing and no ledger for the design track. | process | §3.4 lab reports, §3.5 backlog, §6 implementation rules added. |
| D14 | `scripts/fetch-resources.sh` did not fetch §1.1310/§97.13, the RP1 datasheet, or the KrakenSDR pages the comparator C12 relies on. | reproducibility | Script extended; `scripts/import-shared.sh` added for the blocked datasheets. |
| D15 | SPEC-003 S-003-2 SNR thresholds come from the Meshtastic documentation's reproduction of Semtech values, not the SX1262 datasheet. | tag strength | Remains `[S]` to the Meshtastic table; backlog V-03 replaces it when the datasheet is imported. |

### C. Linter metrics after the second pass

| File group | Statements | `[S]` | `[D]` | `[M]` | `[C]` |
| --- | --- | --- | --- | --- | --- |
| investigations G00–G07 (incl. addenda) | 90 | 46 | 61 | 0 | 10 |
| specs SPEC-001…007 (rev 2 where revised) | 127 | 101 | 38 | 0 | 4 |
| specs SPEC-008…011 (new) | 54 | 14 | 42 | 0 | 4 |

(Statements are counted once; tag columns count tags, and a statement may carry two. Source: `python3 -I scripts/lint-tags.py`, 2026-10-08, after the addenda and revisions.) `[M]`

### D. What the new sources changed

- Tile receiver: device NF 4.5 dB, gain 68 dB, P1dB −34 dBm, LPF corners 9.5/19 MHz, HPF corners 600 kHz/10 kHz/0.1 kHz, phase noise −35 dBc (1.44° rms), spurs −42 dBc, step 76.294 Hz. `[S]` (SPEC-001 S-001-26…30)
- Data path: 4 × 700 Mbit/s CSI lanes = 2.8 Gbit/s; 4 × 26 MSPS uses 59 %; 16 buffers = 10 ms. `[S]`+`[D]` (S-001-34; T14)
- Regulatory: MPE table and the amateur exposure duty are now sourced. `[S]` (SPEC-005 S-005-19…21)
- A reported module grant covering only 902.3–914.9 MHz raises a new host-obligation question. `[C]` (SPEC-005 S-005-22)
- Fourteen Meshtastic HAT templates give a concrete compatible-HAT list. `[S]` (SPEC-004 S-004-9…11)

## V. Discussion

The first pass was sound where it had sources and honest where it did not;
its weaknesses were in reproducibility (hashes, fetchable sources) and in two
physics terms a datasheet makes visible (LO spurs, HPF corner). Nothing found
changes a PASS decision. The largest practical consequence is D8: a
whole-band 915 MHz map through the FTFE will show ghost emitters 42 dB below
any strong one at fixed offsets; the overlay must either flag them (same
bearing, deterministic offset) or the translator LO must be chosen so the
replicas fall outside the 104 slots. That is a bench item, not a desk one.

What would refute this report: a reader finding a carried-forward fact used
by a gate that did not inherit it, or an `[S]` statement whose cited line
does not say what the statement says.

## VI. Recommendations and best practice

1. Signed gates are never edited; new evidence goes into a dated "Errata and addenda" section (adopted in `docs/00-process.md` §3.2).
2. `python3 -I scripts/lint-tags.py` runs before every commit; a non-zero exit blocks the commit (backlog D-04).
3. The manifest records commit hashes and sha256 sums; `scripts/import-shared.sh` prints the row to paste.
4. Every search-engine fact stays `[C]` until its primary page is in `resources/` (process §2).
5. Rev-2 spec additions get a second read before the status returns to Reviewed (backlog D-05).

## VII. References

`investigations/*.md`, `specs/*.md`, `docs/00-process.md`, `docs/resources-manifest.md`, `analysis/linkbudget.py` (T13, T14, T17, T18), `scripts/lint-tags.py`, `vendor/summary/*.md`.
