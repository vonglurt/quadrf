# quadrf — index

<!-- SPDX-License-Identifier: MIT -->

One investigation, two tracks, one rule: every statement is sourced, derived,
measured or declared a conjecture, and nothing passes a gate on a conjecture.

**Question (G00).** Can a ScaleRF QuadRF 4-channel coherent 4.9–6.0 GHz tile
on a Raspberry Pi 5 be used, directly or with a defined modification, for
LoRa/Meshtastic work in the US Pacific Northwest: mapping LoRa emitters with
bearings, forming a LoRa phased array, transmitting at the legal limit, and
relaying to visible mountain-top repeaters with a highly directional link?

**Extension (2026-10-08).** The software is Rust on copal (the user's Alpine
Linux installer for the Pi); the system accepts USB and plug-in sensors; a
second data feed (a 915 MHz coherent receiver) is drawn on the same overlay
as the tile; prototypes will be tested on the bench; the approach must be
secure. The extension does not change the gates; it adds a design track.

## Start here

1. [`docs/00-process.md`](docs/00-process.md) — the practice: evidence tags, document kinds, gates, vendoring, implementation rules. Ten minutes.
2. [`README.md`](README.md) — headline findings with their gates.
3. [`investigations/README.md`](investigations/README.md) — the gate ledger and the risk register.
4. [`backlog.md`](backlog.md) — what to do next, with the check that says when it is done.
5. The spec or lab report your task names (map below).

## Map

### Investigation track (synchronous gates)

| Gate | Document | Question | Status |
| --- | --- | --- | --- |
| G00 | [`investigations/G00-charter.md`](investigations/G00-charter.md) | Is the question well-posed? | PASS |
| G01 | [`investigations/G01-quadrf-hardware-survey.md`](investigations/G01-quadrf-hardware-survey.md) | What does the tile do? | PASS + addenda |
| G02 | [`investigations/G02-regulatory-envelope-pnw.md`](investigations/G02-regulatory-envelope-pnw.md) | What is lawful at 915 MHz and 5.8 GHz in WA/OR? | PASS + addenda |
| G03 | [`investigations/G03-pi-software-stack.md`](investigations/G03-pi-software-stack.md) | What runs on the Pi; what do we write? | PASS (desk) + addenda |
| G04 | [`investigations/G04-lora-options-inventory.md`](investigations/G04-lora-options-inventory.md) | Which LoRa architectures survive? | PASS + addenda |
| G05 | [`investigations/G05-915mhz-translation-frontend.md`](investigations/G05-915mhz-translation-frontend.md) | Does the 915 MHz translating front end work? | OPEN (desk done; bench after kit delivery, vendor date 2026-11-30) |
| G06 | [`investigations/G06-mountain-repeater-directional-relay.md`](investigations/G06-mountain-repeater-directional-relay.md) | What should the directional relay be? | OPEN (desk done; signs with G05) |
| G07 | [`investigations/G07-field-test-plan.md`](investigations/G07-field-test-plan.md) | How do we falsify G05/G06 in the field? | DRAFT (T-1…T-10) |

### Clean-room specifications (what things do, in our words)

| Spec | Subject | Status |
| --- | --- | --- |
| [`SPEC-001`](specs/SPEC-001-quadrf-tile.md) | The QuadRF tile: frequency, converters (now datasheet-backed), geometry, beamforming, interfaces, data path, licences | Reviewed; rev 2 additions await a second read |
| [`SPEC-002`](specs/SPEC-002-quadrf-host-software.md) | The vendor's Pi 5 software: services, device-node ABI (ring, ioctls, stats), boot, bring-up | Reviewed; rev 2 additions await a second read |
| [`SPEC-003`](specs/SPEC-003-lora-css-phy.md) | LoRa chirp-spread-spectrum PHY as Meshtastic uses it | Reviewed |
| [`SPEC-004`](specs/SPEC-004-meshtastic-radio-layer.md) | Meshtastic radio layer, Linux daemon, HAT and USB-radio pin tables | Reviewed; rev 2 additions await a second read |
| [`SPEC-005`](specs/SPEC-005-us-regulatory-envelope.md) | 47 CFR Parts 15 and 97 as behaviour goals; exposure limits | Reviewed; rev 2 additions await a second read |
| [`SPEC-006`](specs/SPEC-006-quadrf-mesh-lora-phy.md) | The third-party LoRa PHY and Meshtastic daemon on the tile (Air-IPC) | Reviewed |
| [`SPEC-011`](specs/SPEC-011-krakensdr-class-coherent-receiver.md) | KrakenSDR-class five-channel coherent receiver (comparator; candidate plug-in) | Draft |

### Implementation specifications (what we build; same form, we are the vendor)

| Spec | Subject | Status |
| --- | --- | --- |
| [`SPEC-007`](specs/SPEC-007-ftfe-implementation.md) | 915 MHz frequency-translating front end and external aperture | Draft (becomes Reviewed at G05 bench) |
| [`SPEC-008`](specs/SPEC-008-system-architecture-rust.md) | System architecture: Rust processes, capture, DSP, bus, overlay, security, workspace | Draft |
| [`SPEC-009`](specs/SPEC-009-sensor-plugins-and-feed-bus.md) | Sensor plug-ins (USB, HAT, tile), message schema, time base, azimuth frame, conformance, the parallel-feed test | Draft |
| [`SPEC-010`](specs/SPEC-010-copal-platform.md) | copal platform: kernel, modules, boot, OpenRC services, tuning options, security, packaging | Draft |

### Lab reports (practice, procedure, analysis of options)

| Report | Subject |
| --- | --- |
| [`LR-001`](lab/LR-001-second-pass-review.md) | Audit of the first pass: chain consistency, 15 defects and their disposition, linter metrics, rules adopted |
| [`LR-002`](lab/LR-002-operating-procedures-and-tools.md) | Operating procedures: acquiring blocked sources via the UTM share, PDFs, tags and the linter, derived numbers, vendoring, git, shell quirks, bench-day checklist |
| [`LR-003`](lab/LR-003-rust-system-architecture.md) | Rust architecture: decisions and rejected alternatives, budgets, security model, comparison with the vendor stack |
| [`LR-004`](lab/LR-004-copal-platform-and-kernel.md) | Porting to copal/Alpine: package mapping, bring-up sequence, kernel tuning options and how each is admitted |
| [`LR-005`](lab/LR-005-coherent-915mhz-receiver-options-and-parallel-feed.md) | FTFE vs KrakenSDR vs dongles vs SX1262 nodes; the "phased array for LoRa itself"; wider spectrum; the parallel-feed test |
| [`LR-006`](lab/LR-006-max2851-and-rp1-datasheet-review.md) | What the MAX2851 and RP1 datasheets changed; LO-spur replicas; what is still missing |

### Analysis, sources, vendoring, tools

| Path | Content |
| --- | --- |
| [`analysis/linkbudget.py`](analysis/linkbudget.py) ([README](analysis/README.md)) | Every `[D]` number, tables T1–T20 |
| [`docs/resources-manifest.md`](docs/resources-manifest.md) | What is in gitignored `resources/`, from where, when, which commit, which sha256 |
| [`vendor/README.md`](vendor/README.md) | Licence classes and the rule for each; per-upstream `ATTRIBUTION.md` under `vendor/<source>/` |
| [`vendor/summary/README.md`](vendor/summary/README.md) | Own-words restatements of everything we may not copy (datasheets, GPL code, CC BY-SA files, vendor pages) |
| [`vendor/cfr47/README.md`](vendor/cfr47/README.md) | Public-domain text of the fourteen 47 CFR sections we rely on |
| [`scripts/fetch-resources.sh`](scripts/fetch-resources.sh) | Re-creates `resources/` (everything fetchable by script) |
| [`scripts/import-shared.sh`](scripts/import-shared.sh) | Imports datasheets dropped in `~/Downloads/SharedVM/quadrf/` by part number with sha256 |
| [`scripts/vendor-cfr.py`](scripts/vendor-cfr.py) | Regenerates `vendor/cfr47/` from the LII pages |
| [`scripts/lint-tags.py`](scripts/lint-tags.py) | Finds untagged statements; exit 1 blocks a commit |
| [`scripts/check-links.py`](scripts/check-links.py) | Checks that every relative Markdown link resolves |
| [`docs/templates/`](docs/templates/) | Templates: investigation, gate record, spec, measurement record, lab report |
| [`AGENTS.md`](AGENTS.md) | Rules for agents working here |
| [`LICENSE`](LICENSE) | MIT |

## The two tracks and how they join

```
investigation track   G00 → G01 → G02 → G03 → G04 → G05 → G06 → G07
                      PASS  PASS  PASS  PASS  PASS  OPEN  OPEN  DRAFT
                                                     │
                                       bench, after kit delivery (≥ 2026-11-30)
                                                     │
design track          P0 ─ P1 ─ P2 ───────────── P3 ─ P4 ─ P5 ─ P6 ─ P7
                      docs  Rust  field tools    copal tile 915   relay Tx-array
                      now   sim   (dongle, stick, bring  on   MHz  (G06) (Part 97)
                                   nodes, Pico)   -up   copal layer
```

P0–P2 need no tile and run now. P3 onward needs the kit. A design-track
entry that rests on an open conjecture (listed in the risk register) cannot
be marked done until the gate that closes the conjecture is signed.

## Conventions on one screen

- Tags: `[S]` sourced (file and line), `[D]` derived (script table or stated requirement), `[M]` measured (dated record), `[C]` conjecture (names what closes it). Untagged statements fail `scripts/lint-tags.py`.
- Numbers come from `analysis/linkbudget.py`; prose cites "analysis Tnn".
- Signed gates are not edited; evidence that arrives later goes into a dated "Errata and addenda" section.
- Sources: raw in `resources/` (ignored), listed in the manifest; licences in `vendor/`; restatements in `vendor/summary/`.
- Software: Rust, Cargo, stable toolchain, MIT; GPL components as separate processes; no transmit without an unlock naming the regulatory profile.
- Platform: Raspberry Pi 5 on copal (Alpine, musl, OpenRC); the vendor's Debian image is a bring-up aid.

## State on 2026-10-08

- Gates: G00–G04 PASS; G05 and G06 OPEN on bench work; G07 DRAFT.
- Backlog: 57 open, 9 done, 0 dropped; current phase P0 (ledger hygiene), P1 (Rust workspace with simulated feeds) and P2 (field tools) can run before the kit arrives.
- Sources in hand: vendor repository at commit `8b61ae5`, schematic export 2026-05-22, MAX2851 and RP1 datasheets, fourteen CFR sections, KrakenSDR documentation, Meshtastic firmware at `364a111`, `quadrf-mesh` at `ad3ed31`. Missing: MAX2850, MAX2871, SX1262, SE5004L, SKY65404-31 datasheets (drop them in the share; see LR-002 P1).
- Headline numbers: the tile cannot tune below 4900 MHz (datasheet-backed); the Part 15 EIRP ceiling at 915 MHz is 36 dBm for any antenna; the tile LO's −42 dBc spurs sit above the 8-bit floor; four interleaved channels at 26 MSPS use 59 % of the CSI link; the whole-band channeliser costs about 0.73 of one core.
