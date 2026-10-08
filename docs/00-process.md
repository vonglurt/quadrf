# 00 — Documentation practice: from specification to implementation, gated

## 1. Purpose

This repository records an engineering investigation and the design that
follows from it. Its documents must let a second communications engineer
reproduce every conclusion from the cited sources and the scripts in
`analysis/`, without talking to us, and must let a third party build the
designed system from `specs/` and `backlog.md` alone. The practice below is the
minimum that achieves that.

## 2. Epistemic rule: every statement is tagged

A technical document is a set of claims. Each claim is one of four kinds, and
the kind is written next to it:

| Tag | Meaning | Obligation |
| --- | --- | --- |
| `[S]` | Sourced. Stated by a primary document (vendor schematic, source code, datasheet, CFR text). | Cite the file in `resources/` or the URL in `docs/resources-manifest.md`, and the line, page or section. |
| `[D]` | Derived. Follows from physics or arithmetic applied to `[S]` inputs, or is a design decision that follows from stated requirements. | The computation exists in `analysis/` and the table number is cited; a design decision cites the requirement it satisfies. |
| `[M]` | Measured. Observed by this project on our hardware. | A dated measurement record exists in the investigation, with instrument, setup, and raw data location. |
| `[C]` | Conjecture. Plausible but not yet one of the above. | Must name the gate or backlog check that will close it. A conjecture cannot be carried forward through a gate as a fact. |

Untagged factual sentences are defects. `scripts/lint-tags.py` finds untagged
numbered statements; it runs clean before any commit. Opinions and
recommendations are labelled as such and are permitted only in the
"Recommendation" section of an investigation or the "Discussion" section of a
lab report.

The philosophical commitment is Popperian: a design claim is admitted only if
it states what observation would refute it, and the gate is where that
observation is attempted. Vendor marketing text is `[C]` until the schematic,
source, datasheet, or a measurement confirms it. A search-engine summary or a
third-party mirror is not a primary source; facts taken from one stay `[C]`
until the primary document is in `resources/`.

## 3. Document kinds

### 3.1 Clean-room specification (`specs/SPEC-nnn-*.md`)

A specification is our own restatement of what a thing *does*, written as
numbered behaviour-goal statements, so that we can (a) verify the thing against
it, and (b) design against it without carrying the vendor's wording, licence,
or errors. Rules:

- One subject per spec (a tile, a PHY, a regulation, a daemon, a crate).
- Each statement is of the form "`<subject>` `<shall|does|exposes|limits>`
  `<observable behaviour>` `<quantity with unit>` `[tag] (source)`".
- Statements are falsifiable. "High performance" is not a statement;
  "receiver noise figure ≤ 1.5 dB at 5.5 GHz, 290 K" is.
- No copied text. Regulation and datasheet text may be quoted only in
  investigations or lab reports, inside a quotation block, with the section
  cited. The public-domain CFR text lives verbatim in `vendor/cfr47/`.
- The spec records the *interface* we depend on, not the implementation we
  don't. If a vendor changes a proprietary bitstream and our statements still
  hold, the spec is unchanged.
- Status header: `Draft` → `Reviewed` (a second reader checked every tag) →
  `Baselined` (a gate or a backlog phase depends on it; changes require a new
  revision number).
- Implementation specs (SPEC-007 onward) use the same form; the difference is
  only that we are the vendor. Their statements are `[D]` design decisions
  traced to a requirement, or `[S]` platform facts.

### 3.2 Investigation (`investigations/Gnn-*.md`)

One question per investigation. Sections in fixed order:

1. **Question** — one sentence, answerable yes/no or by a number.
2. **Inherited facts** — the numbered facts carried forward from prior gates.
   Nothing else may be assumed.
3. **Method** — what was read, computed, or measured, and why that suffices.
4. **Findings** — numbered, tagged statements.
5. **Analysis** — the derivations that connect findings to the answer.
6. **Answer** — the answer to the question, with the residual uncertainty.
7. **Recommendation** — the opinion of the author, labelled.
8. **Gate record** — see 3.3.
9. **Carry-forward** — the numbered facts the next investigation inherits.

A signed investigation is not rewritten. New evidence that arrives after the
gate record is signed goes into a dated **Errata and addenda** section at the
end, with its own tagged statements; if it would change the decision, the gate
is re-opened explicitly in the ledger.

### 3.3 Gate record

A gate is a synchronous checkpoint. The next investigation is not opened until
the current gate record is signed. The record states:

- **Pass criterion**, written *before* the work starts.
- **Evidence** that the criterion is met: the finding numbers.
- **Decision**: `PASS`, `FAIL` (investigation re-planned), `BLOCKED` (names
  the external dependency), or `WAIVED` (criterion not met, proceeding
  knowingly; states the risk accepted and who accepted it).
- **Date and signatory.**
- **Open conjectures** carried as risks, each with the gate that will close it.

Gates are strictly ordered because each inherits the carried-forward facts of
the previous one. Parallel desk work is allowed; parallel *conclusions* are not.

### 3.4 Lab report (`lab/LR-nnn-*.md`)

A lab report records a piece of practice: how a tool was used, what procedure
worked, what an audit found, what an architecture trade was decided and why.
It is the place for best practice and operating procedure. Sections, in the
IEEE-style form the user's other repositories use: Abstract, Objective,
Materials, Method, Results, Discussion, Recommendations, References. Facts in
a lab report carry evidence tags like any other document; recommendations are
opinions and say so. A lab report never closes a gate; it feeds findings into
an investigation's addenda or into `backlog.md` entries.

### 3.5 Backlog (`backlog.md`)

The backlog is the ledger of the design and build track. Each entry names the
deliverable, the rule or gate it serves, and a **check** that decides whether
it is done. An entry is done when its check passes, not when the change is
made. Phases are ordered; a phase is entered when the previous phase's exit
check passes, which is the same synchronous discipline as the gates.

## 4. From specification to implementation

The sequence this repository follows, with the deliverable of each stage:

| Stage | Deliverable | Gate asks |
| --- | --- | --- |
| Charter | `G00` | Is the question well-posed and bounded? |
| Survey | Clean-room specs of the things we depend on | Do we know what the hardware and software actually do, from primary sources? |
| Envelope | Regulatory and physical limits, as numbers | What is permitted and what is possible, independent of our design? |
| Inventory | Enumerated candidate architectures, each with a kill criterion | Which candidates survive the envelope? |
| Feasibility | Per-candidate desk analysis, then bench measurement | Does the candidate's critical assumption hold when measured? |
| Design | Implementation specs (interfaces, numbers, test points) and the backlog | Could a third party build it from the documents alone? |
| Build and test | Code, hardware, test records; backlog phase checks | Does the artefact meet its spec, measured? |
| Field | Field-test plan and records | Does it meet the charter's success criterion in the real channel? |

The investigation track (G00–G07) and the design track (`backlog.md` phases
P0–P7, specs SPEC-007 onward) run in parallel as desk work. They join at the
bench: no design-track entry that depends on a conjecture is marked done
before the gate that closes the conjecture is signed.

## 5. Sources, vendored material, and reproducibility

Three places hold third-party material, with different rules:

| Place | Tracked in git | Content | Rule |
| --- | --- | --- | --- |
| `resources/` | No (gitignored) | Raw downloads, cloned repositories, datasheets | Every item listed in `docs/resources-manifest.md` with URL, date, purpose; re-created by `scripts/fetch-resources.sh` or `scripts/import-shared.sh` |
| `vendor/<source>/` | Yes | An `ATTRIBUTION.md` per upstream naming the copyright holder, licence, revision, and what we use; verbatim material only where its licence permits redistribution (today: public-domain CFR text in `vendor/cfr47/`) | Nothing under a licence incompatible with this repository's MIT licence is copied here |
| `vendor/summary/` | Yes | Our own-words summaries of sources whose licence does not permit copying (datasheets, GPL code, CC BY-SA design files, vendor web pages), each with its attribution | The catch-all for every copyright that is not MIT-compatible: restated, attributed, never reproduced |

Our understanding of each source is written in our own words into `specs/`,
an investigation, or a `vendor/summary/` file. If `resources/` is deleted, the
repository still makes sense; if the vendor's site disappears, our conclusions
still stand on the manifest's record of what was read and when.

Every derived number is produced by a script in `analysis/`. The documents
cite the script and table. Numbers are never typed by hand into prose.

Dates are absolute (ISO 8601). "Current firmware" is a defect; "firmware
packaged 2026-07-15" is not. Revisions of cloned repositories are recorded as
commit hashes in the manifest.

## 6. Implementation rules

- **Language.** New software is written in Rust (edition 2024, stable
  toolchain pinned in `rust-toolchain.toml`) and built with Cargo. C is
  accepted only for Linux kernel modules and for unmodified vendor sources
  rebuilt from their own trees. Python is accepted for `analysis/` and for
  one-off tools under `scripts/`, run with `python3 -I`.
- **Licence boundary.** This repository is MIT. GPL-licensed components
  (vendor drivers, SoapySDR module, `quadrf-mesh`, `gr-lora_sdr`, Meshtastic,
  KrakenSDR software) are used as separate processes behind documented IPC
  (sockets, ZeroMQ, TCP), never linked into our binaries and never copied.
  `cargo deny` enforces an allow-list of crate licences.
- **Supply chain.** `Cargo.lock` is committed; `cargo deny check` and
  `cargo audit` run before a release; releases are tagged and signed with
  the same `ssh-keygen -Y` scheme copal uses.
- **Platform.** The target is a Raspberry Pi 5 running copal (Alpine Linux,
  musl, OpenRC, `linux-rpi` kernel). Anything that only runs on the vendor's
  Debian image is a bring-up aid, not a dependency.
- **Transmit safety.** No binary in this repository transmits unless an
  explicit, logged unlock names the regulatory profile (SPEC-005 regime,
  callsign if Part 97) under which it operates.

## 7. Writing style

Dense, declarative, quantitative. Units on every quantity. dB quantities
state their reference (dBm, dBi, dBFS, dBc). Frequencies in MHz unless a band
name is conventional. No adjectives of quality without a number behind them.
Regulatory statements name the CFR section and the paragraph.
