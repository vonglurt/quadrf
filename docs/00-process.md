# 00 — Documentation practice: from specification to implementation, gated

## 1. Purpose

This repository records an engineering investigation, not a product. Its
documents must let a second communications engineer reproduce every conclusion
from the cited sources and the scripts in `analysis/`, without talking to us.
The practice below is the minimum that achieves that.

## 2. Epistemic rule: every statement is tagged

A technical document is a set of claims. Each claim is one of four kinds, and
the kind is written next to it:

| Tag | Meaning | Obligation |
| --- | --- | --- |
| `[S]` | Sourced. Stated by a primary document (vendor schematic, source code, datasheet, CFR text). | Cite the file in `resources/` or the URL in `docs/resources-manifest.md`, and the line, page or section. |
| `[D]` | Derived. Follows from physics or arithmetic applied to `[S]` inputs. | The computation exists in `analysis/` and the table number is cited. |
| `[M]` | Measured. Observed by this project on our hardware. | A dated measurement record exists in the investigation, with instrument, setup, and raw data location. |
| `[C]` | Conjecture. Plausible but not yet one of the above. | Must name the gate that will close it. A conjecture cannot be carried forward through a gate as a fact. |

Untagged factual sentences are defects. Opinions and recommendations are
labelled as such and are permitted only in the "Recommendation" section of an
investigation.

The philosophical commitment is Popperian: a design claim is admitted only if
it states what observation would refute it, and the gate is where that
observation is attempted. Vendor marketing text is `[C]` until the schematic,
source, or a measurement confirms it.

## 3. Three document kinds

### 3.1 Clean-room specification (`specs/SPEC-nnn-*.md`)

A specification is our own restatement of what a thing *does*, written as
numbered behaviour-goal statements, so that we can (a) verify the thing against
it, and (b) design against it without carrying the vendor's wording, licence,
or errors. Rules:

- One subject per spec (a tile, a PHY, a regulation, a daemon).
- Each statement is of the form "`<subject>` `<shall|does|exposes|limits>`
  `<observable behaviour>` `<quantity with unit>` `[tag] (source)`".
- Statements are falsifiable. "High performance" is not a statement;
  "receiver noise figure ≤ 1.5 dB at 5.5 GHz, 290 K" is.
- No copied text. Regulation and datasheet text may be quoted only in
  investigations, inside a quotation block, with the section cited.
- The spec records the *interface* we depend on, not the implementation we
  don't. If a vendor changes a proprietary bitstream and our statements still
  hold, the spec is unchanged.
- Status header: `Draft` → `Reviewed` (a second reader checked every tag) →
  `Baselined` (a gate depends on it; changes require a new revision number).

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

## 4. From specification to implementation

The sequence this repository follows, with the deliverable of each stage:

| Stage | Deliverable | Gate asks |
| --- | --- | --- |
| Charter | `G00` | Is the question well-posed and bounded? |
| Survey | Clean-room specs of the things we depend on | Do we know what the hardware and software actually do, from primary sources? |
| Envelope | Regulatory and physical limits, as numbers | What is permitted and what is possible, independent of our design? |
| Inventory | Enumerated candidate architectures, each with a kill criterion | Which candidates survive the envelope? |
| Feasibility | Per-candidate desk analysis, then bench measurement | Does the candidate's critical assumption hold when measured? |
| Design | Implementation spec (interfaces, numbers, test points) | Could a third party build it from the document alone? |
| Build and test | Code, hardware, test records | Does the artefact meet its spec, measured? |
| Field | Field-test plan and records | Does it meet the charter's success criterion in the real channel? |

An implementation spec is written in the same behaviour-goal form as a
clean-room spec; the difference is only that we are the vendor.

## 5. Sources and reproducibility

- Raw sources live in `resources/` (gitignored, may contain third-party
  licensed material). `docs/resources-manifest.md` lists every item with URL,
  date, and what we used it for. `scripts/fetch-resources.sh` re-creates the
  folder.
- Our understanding of each source is written in our own words into `specs/`
  or an investigation. If `resources/` is deleted, the repository still makes
  sense; if the vendor's site disappears, our conclusions still stand on the
  manifest's record of what was read and when.
- Every derived number is produced by a script in `analysis/`. The documents
  cite the script and table. Numbers are never typed by hand into prose.
- Dates are absolute (ISO 8601). "Current firmware" is a defect; "firmware
  packaged 2026-07-15" is not.

## 6. Writing style

Dense, declarative, quantitative. Units on every quantity. dB quantities
state their reference (dBm, dBi, dBFS, dBc). Frequencies in MHz unless a band
name is conventional. No adjectives of quality without a number behind them.
Regulatory statements name the CFR section and the paragraph.
