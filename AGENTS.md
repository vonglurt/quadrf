# AGENTS.md — quadrf

Investigation repository: can the ScaleRF QuadRF 4x4 MIMO tile (4.9–6.0 GHz) be
used in a LoRa configuration, and what is the legal, physical, and software
envelope for doing so in the US Pacific Northwest.

## Rules for agents working here

- Read `docs/00-process.md` first. Work is sequential and gated. Do not open
  investigation Gn+1 while Gn's gate record is not PASS or WAIVED.
- Every factual statement carries an evidence tag: `[S]` sourced, `[D]` derived
  (reproduced by `analysis/linkbudget.py`), `[M]` measured by this project,
  `[C]` conjecture. Untagged statements are defects.
- `specs/` are clean-room: written in our own words as behaviour-goal
  statements. Never paste vendor text, datasheet tables, or regulation text
  into `specs/`. Quote regulations verbatim only in `investigations/`, with the
  CFR section cited.
- `resources/` is gitignored. Every file placed there must be listed in
  `docs/resources-manifest.md` with URL, retrieval date, and purpose, and must be
  reproducible by `scripts/fetch-resources.sh`.
- Numbers in prose must come from `analysis/` scripts. Change the script, re-run,
  then update the document; never hand-edit a derived number.
- Commit and push only when the user asks.
