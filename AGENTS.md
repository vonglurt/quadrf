# AGENTS.md — quadrf

Investigation repository with a design track: can the ScaleRF QuadRF 4x4 MIMO
tile (4.9–6.0 GHz) be used in a LoRa configuration, what is the legal,
physical, and software envelope for doing so in the US Pacific Northwest, and
how is the resulting system built in Rust on copal (Alpine Linux, Pi 5).

## Rules for agents working here

- Read `index.md`, then `docs/00-process.md`. Work is sequential and gated.
  Do not open investigation Gn+1 while Gn's gate record is not PASS or
  WAIVED. Do not edit a signed gate; append a dated "Errata and addenda".
- Every factual statement carries an evidence tag: `[S]` sourced, `[D]`
  derived (reproduced by `analysis/linkbudget.py` or traced to a stated
  requirement), `[M]` measured by this project, `[C]` conjecture (names what
  closes it). Run `make check` before committing; it must exit 0 (tag linter,
  link and citation checks, analysis, Rust build and parity). Search-engine summaries and third-party mirrors are `[C]`.
- `specs/` are clean-room: written in our own words as behaviour-goal
  statements. Never paste vendor text, datasheet tables, or regulation text
  into `specs/`. Quote regulations verbatim only in investigations or lab
  reports, with the CFR section cited; the public-domain CFR text lives in
  `vendor/cfr47/`.
- `resources/` is gitignored. Every file placed there must be listed in
  `docs/resources-manifest.md` with URL (or share-folder provenance),
  retrieval date, purpose, and commit hash or sha256, and must be
  reproducible by `scripts/fetch-resources.sh` or `scripts/import-shared.sh`.
  Datasheets the publishers block arrive through `~/Downloads/SharedVM/quadrf/`.
- Third-party licences are ledgered in `vendor/`. Nothing GPL, CC BY-SA or
  proprietary is copied into this repository; it is attributed in
  `vendor/<source>/ATTRIBUTION.md` and restated in `vendor/summary/`. GPL
  software is used only as a separate process behind documented IPC.
- Numbers in prose must come from `analysis/` scripts. Change the script,
  re-run, then update the document; never hand-edit a derived number. The
  Rust port `crates/qrf-analysis` must stay byte-identical (`make parity`),
  so a new table goes into both in the same change.
- New software is Rust (stable, edition 2024, musl target) built with Cargo,
  `Cargo.lock` committed, `cargo deny` licence allow-list; C only for kernel
  modules and unmodified vendor rebuilds; Python only in `analysis/` and
  `scripts/`, run with `python3 -I`.
- Design work goes through `backlog.md`: an entry is done when its check
  passes; an entry resting on an open conjecture waits for the gate that
  closes it.
- The platform is a Raspberry Pi 5 running copal (OpenRC, apk, musl). The
  vendor's DietPi image is a bring-up aid, not a dependency.
- No binary here transmits without an explicit, logged unlock naming the
  regulatory profile (SPEC-005 regime, callsign if Part 97).
- Shell: use absolute paths (the working directory resets between calls);
  ignore `._*` AppleDouble files; commit with the configured git identity and
  only when asked.
