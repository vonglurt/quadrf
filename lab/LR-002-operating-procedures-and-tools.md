# LR-002 — Operating procedures and tools for this repository

*Lab report. 2026-10-08.*

<!-- SPDX-License-Identifier: MIT -->

| Field | Value |
| --- | --- |
| Status | Final |
| Author | project |
| Feeds | `docs/00-process.md` §5–6; `scripts/`; `AGENTS.md`; backlog D-04, V-02 |

## Abstract

This report records how the repository is operated: how sources are
acquired when their publishers refuse scripted download, how datasheets
cross from the Mac host into the guest VM, how PDFs are read and cited, how
evidence tags are linted, how derived numbers are produced, how third-party
material is vendored, and how git is used here. Each procedure was exercised
in this session; the counts in §IV are what it produced. The procedures are
written so that a second operator, or a later automated run, repeats them
without rediscovering the obstacles.

## I. Objective

1. Make every acquisition, extraction and citation step reproducible by command.
2. State the obstacles met (blocked sites, redirects, bot challenges, shell quirks) and the working route around each.
3. Give a bench-day checklist for the day the kit arrives.

## II. Materials

| Item | Detail |
| --- | --- |
| Guest VM | Alpine Linux 3.24.1 aarch64 under UTM/QEMU, kernel 6.18.52-0-lts, 4 cores, 5.9 GB RAM |
| Toolchain | rustc/cargo 1.98.1 (rustup, host `aarch64-unknown-linux-musl`, targets also `armv7-unknown-linux-musleabihf`, `x86_64-unknown-linux-musl`); python3 3.14.7; poppler `pdftotext` 25.12; curl; git |
| Host share | `~/Downloads/SharedVM/quadrf/` (UTM share from the Mac); the Mac writes `._*` AppleDouble files beside every file |
| Repository | `/home/user/code/quadrf`, remote `git@github.com:vonglurt/quadrf.git`, SSH key auth |

## III. Method (procedures)

### P1 — Acquiring sources

- Run `sh scripts/fetch-resources.sh`; it is idempotent and skips files already present.
- Sites that refuse scripted retrieval, as observed 2026-10-08: analog.com (HTTP/2 stream reset), semtech.com (datasheet link returns an HTML interstitial), skyworksinc.com (HTML instead of PDF), ecfr.gov (302 to an "unblock" page), fccid.io (JavaScript challenge). `[M]`
- Working routes: the LII mirror for CFR text; `raw.githubusercontent.com/wiki/<org>/<repo>/<Page>.md` for GitHub wiki pages; the crates.io JSON API with a descriptive `User-Agent`; `pip-assets.raspberrypi.com` after two redirects for the RP1 datasheet (the WebFetch tool saved the binary to its tool-results directory, from where it was copied). `[M]`
- Datasheets from blocked publishers: download in a browser on the Mac, drop into `~/Downloads/SharedVM/quadrf/`, run `sh scripts/import-shared.sh`. The script identifies the part from the first page, names the file `<PART>.pdf`, writes `<PART>.txt`, prints the sha256 and the manifest row. Dry-run with `--dry-run`. `[M]` (MAX2851 imported this way; the file arrived named "EV Bible.pdf")

### P2 — Reading PDFs

- `pdftotext -layout file.pdf file.txt`; search the text with `grep -n`; cite the datasheet's own page number (the page footer "Maxim Integrated │ 3" style), not the text line, in `[S]` citations. The line numbers of the `.txt` are used only inside `vendor/summary/` restatements where that helps a reader with the same extraction.
- Tables in datasheets are extracted with the `sed 's/[[:space:]]\{3,\}/ | /g'` idiom to see columns.

### P3 — Evidence tags and the linter

- Every `- F.nn.k` or `- S-nnn-k` line carries `[S]`, `[D]`, `[M]` or `[C]`; lines under "Inherited facts" and "Carry-forward" are reference lists and are exempt.
- `python3 -I scripts/lint-tags.py` prints per-file counts and untagged lines; exit 1 means fix before committing.

### P4 — Derived numbers

- Add or change a table in `analysis/linkbudget.py`, run `python3 analysis/linkbudget.py`, then cite "analysis Tnn" in the document. Placeholders inside the script that await a datasheet are commented `[C]`.
- Never type a derived number into prose that the script does not print.

### P5 — Vendoring third-party material

- Three classes (`vendor/README.md`): carried verbatim (public domain, MIT-compatible), attributed-and-summarised (GPL, CC BY-SA), summarised-only (datasheets, proprietary pages).
- Adding an upstream: a row in `vendor/README.md`, `vendor/<source>/ATTRIBUTION.md`, `vendor/summary/<source>.md`, and a manifest row.
- `python3 -I scripts/vendor-cfr.py` regenerates `vendor/cfr47/` from the LII pages in `resources/regulatory/`.

### P6 — Git

- Commit with the configured global identity; do not pass `-c user.name`/`-c user.email`. Commit and push only when asked.
- `resources/` is ignored; check `git status` for files that landed there by a relative-path write.
- Ignore `._*` files; they are the Mac's metadata.

### P7 — Shell quirks in this environment

- A leading `cd DIR &&` does not reliably apply to later lines of a compound command, and the working directory resets between calls; write and read files by absolute path.
- Scripts that read downloaded files run with `python3 -I` so that a planted module in the download directory is not imported.
- Heredocs use a quoted delimiter (`<<'EOF'`) so that backticks and `$` in documents are not expanded.

### P8 — Writing

- Gates from `docs/templates/investigation.md`; specs from `docs/templates/spec.md`; lab reports from `docs/templates/lab-report.md`; measurements from `docs/templates/measurement-record.md`.
- A signed gate gets an "Errata and addenda" section, never an edit.

### P9 — Bench-day checklist (kit delivery, vendor date 2026-11-30)

1. Record package versions on the kit SD card (`dpkg -l | grep quadrf`), the bitstream file's sha256, and copy `quadrf.svf` to the copal build host (outside git).
2. Read the FPGA IDCODE with OpenOCD (U-001-6).
3. `CSI_IOC_GET_RING_INFO` → ring and span sizes (U-002-3); baseline `csi_stats` at rest and at 4 × 26 MSPS for 60 s (G05 criterion 5 precondition).
4. Photograph and measure the antenna-module connector; identify the mating part (U-001-1).
5. Probe GPIO 22 during `quadrf-load` to see whether TRST is driven (U-004-2).
6. Inject a −30 dBm CW at the element port and record LO-spur replicas (G05 criterion 6).
7. File one `M-nnn` record per item under `investigations/records/`.

## IV. Results (what the procedures produced this session)

| Procedure | Output |
| --- | --- |
| P1 | 14 CFR sections (2 new), the RP1 datasheet (93 pp.), 4 KrakenSDR pages, the SX1262 product page; 1 datasheet via the share (MAX2851); 5 datasheets still missing (MAX2850, MAX2871, SX1262, SE5004L, SKY65404-31) |
| P2 | `MAX2851.txt` 2 925 lines; `RP1-peripherals.txt` 6 224 lines; `schematics.txt` designator histogram |
| P3 | 0 untagged statements across 11 specs and 8 gates (271 statements, 320 tags) |
| P4 | 7 new tables T14–T20 |
| P5 | 11 attribution files, 9 summaries, 14 public-domain CFR texts |

`[M]` (this session's shell logs)

## V. Discussion

The slowest step is still the human one: datasheets that only a browser can
fetch. The share-folder route costs one minute per file and keeps the
provenance (sha256, date) in the manifest, which is acceptable. The bot
challenge at fccid.io means FCC grants must be read at the FCC's own site,
also by hand (backlog V-04). The shell working-directory reset cost the first
pass misplaced files; absolute paths remove the risk entirely.

## VI. Recommendations and best practice

1. Keep `scripts/fetch-resources.sh` as the single acquisition record; a URL that is not in it is not a source.
2. Run the linter and the analysis script before every commit (backlog D-04 adds a `make check`).
3. Treat the share folder as an inbox: import, then delete from the share, so the manifest is the only record.
4. Before the bench day, print P9 and the G05 pass criteria.

## VII. References

`scripts/fetch-resources.sh`, `scripts/import-shared.sh`, `scripts/vendor-cfr.py`, `scripts/lint-tags.py`, `docs/resources-manifest.md`, `docs/00-process.md`, memory notes on git identity and shell quirks (project memory, 2026-10-08).
