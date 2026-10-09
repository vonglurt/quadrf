# LR-007 — Review of the backlog after the first build batch: cross-phase dependencies, checks that could not run, missing entries, and the re-sequencing adopted

*Lab report. 2026-10-08.*

<!-- SPDX-License-Identifier: MIT -->

| Field | Value |
| --- | --- |
| Status | Final |
| Author | project (review of `backlog.md` at `3b09f1a`; applied the same day) |
| Feeds | `backlog.md` *Sequencing* section and phases P0–P7 (eight new entries, two splits, four moves, five rewritten checks); `analysis/linkbudget.py` T24 and its Rust port; `index.md` state |

## Abstract

The 53 open entries of `backlog.md` at commit `3b09f1a` were read for whether
each check can be run on the day its phase is entered, with the tools and
hardware that will exist then, and for dependencies that cross phase
boundaries. Most of P2 depended on P1 crates, so only the three entries that
need no qrf software (F-01, F-02, F-05) run before P1 ends; three checks
could not be run as written (R-03 needed a ring recording that exists only
after the kit, R-07 allowed the VM as a proxy for an A76 core budget, R-P1
passed by construction); five behaviour-goal statements or risks had no
entry (the S-008-9 control plane, the S-008-13 release procedure, a
procurement list, a fallback for the kernel-module port, the Part 97
decision). The oracle for R-08 is half present: GNU Radio 3.10.12 and pyzmq
are installed on the VM, `gr-lora_sdr` is cloned and not built, and no I/Q
corpus exists. A new analysis table (T24) bounds the fallback capture over
Ethernet at two channels of 26 MSPS or four of 13 MSPS. The backlog was
re-sequenced accordingly: P0 is a standing phase concurrent with P1, P1 is
the critical path in dependency order, P2 waits for R-P1 and the purchases;
61 entries are open.

## I. Objective

1. For every open entry, decide whether its check can be run on the day its phase is entered, with the artefacts (recordings, oracles, hardware) that will exist then.
2. Find entries whose phase is wrong because their deliverable depends on work in a later or a parallel phase.
3. Find statements in SPEC-008…010 that constrain a later entry but are delivered by no entry before it.
4. Record the re-sequencing with the counts before and after, so that the change is auditable.

## II. Materials

| Item | Detail |
| --- | --- |
| `backlog.md` | commit `3b09f1a`, 53 open, 16 done, 0 dropped |
| Specifications | SPEC-002 rev 2 (S-002-4, S-002-14…17, S-002-21), SPEC-008 rev 0 (S-008-1…15), SPEC-009 rev 0 (S-009-3, S-009-5…7, S-009-13), SPEC-010 rev 0 (S-010-9, S-010-11) |
| Lab reports | LR-002 §P1 (sites that refuse scripted retrieval), LR-003 §C (budgets), LR-005 §VI (dongle and stick first) |
| Development VM | Alpine 3.24.1, 4 cores, host triple `aarch64-unknown-linux-musl`, under UTM/QEMU on a Mac host (`AGENTS.md`); `gnuradio` 3.10.12.0-r12 and `py3-pyzmq` 27.1.0 installed; `resources/repos/gr-lora_sdr` at `862746d`, not built |
| Analysis | `analysis/linkbudget.py` T14 (data path), T16 (bearing CRLB), T19 (CPU budget), T24 (added by this report) |
| Datasheet | `resources/datasheets/RP1-peripherals.txt`, Ethernet section (Gigabit MAC) |

## III. Method

1. Each row's **Check** column was read for the artefacts it names (a recording, an oracle, a Pi, the kit) and each artefact was traced to the entry that produces it and that entry's phase.
2. Each P2 row's deliverable was traced to the P1 crates and processes it calls (`qrf-bus`, `qrf-lora`, `qrf-overlay`, `qrf-sensord`).
3. SPEC-008 S-008-1…15 and SPEC-010 S-010-9/11 were compared with the set of entries to find statements no entry delivers before the entry that depends on them.
4. The VM was queried (`apk info -e`, `python3 -c "from gnuradio import gr; import zmq"`, `git rev-parse` in the clone) for the oracle tooling.
5. T24 was added to the Python script and the Rust port in the same change; `make parity` confirmed byte-identical output (198 lines).
6. `backlog.md` was rewritten by a script that copies every unchanged row by its ID from the previous file and writes only the new, split, moved or rewritten rows, so that no row was retyped.

## IV. Results

### A. Dependencies across phases

- F-03's check needs `Occupancy` messages (R-02) and decoding through `qrf-lora` (R-08); F-04's check needs `LoraFrame` in the overlay (R-02, R-09) from a supervised plug-in (R-02b); F-P2 needs all of them. Only F-01, F-02 and F-05 name no qrf software. `[D]` (the entry texts; SPEC-009 S-009-3, S-009-5)
- F-06 (Pico firmware) needs no P1 crate; its check needs the dongle and its target needs F-01's confirmation of the HAT's band and radio IC. `[D]` (SPEC-008 S-008-11; G07 §8)

### B. Checks that could not be run as written

- R-03 asked the mock device to replay "a recorded ring"; a ring recording exists only after P-05 (kit on the desk, modules loaded). `[D]`
- R-07 allowed "the VM's 4 cores as a proxy" for the ≤ 0.8-core channeliser budget. T19 defines that budget as a fraction of one Cortex-A76 core at 2.4 GHz with a 38 GFLOP/s NEON peak and an assumed 25 % efficiency; the VM's cores are the Mac host's under UTM/QEMU, with a different peak and efficiency, so a fraction measured there does not transfer to the Pi. `[D]` (analysis T19) The host triple is `aarch64-unknown-linux-musl` and the VM has 4 cores. `[M]` (`rustc -vV`, `nproc`, 2026-10-08)
- R-P1 passed by construction: both simulated feeds were generated from one model with no clock or mounting error between them, so "aligned to ≤ 100 ms and ≤ 2°" exercised neither the mounting transform (S-009-7) nor the time association (S-009-6). `[D]`
- R-08 named `gr-lora_sdr` as the test oracle. On the VM `gnuradio` 3.10.12.0 and `pyzmq` 27.1.0 import; `gr-lora_sdr` is cloned at `862746d` and not built; no I/Q corpus exists. `[M]` (`apk info -e gnuradio`; `python3 -c "from gnuradio import gr; print(gr.version())"` → `3.10.12.0`; `python3 -c "import zmq; print(zmq.__version__)"` → `27.1.0`; `git -C resources/repos/gr-lora_sdr rev-parse --short HEAD` → `862746d`; 2026-10-08)
- R-06 asked for interoperability with "a GNU Radio ZMQ source block", which needs the same GNU Radio install as R-08's oracle; the two entries shared no prerequisite entry. `[D]`

### C. Statements and risks with no entry

- S-008-9 (control sockets owned by group `qrf`; transmit refused without an unlock file naming the regulatory profile; every transmit command logged) was delivered by no entry; R-10 would have added a control socket to `qrf-tiled` in P4 with no test of the refusal path before it. `[D]` (SPEC-008 S-008-9; SPEC-010 S-010-9)
- S-008-13 and S-010-11 (locked dependency set, `cargo deny`, `cargo audit`, release tags signed with `ssh-keygen -Y`, reproducible from a tagged commit) were delivered by no entry, although P-01 installs the workspace on the Pi. `[D]`
- No entry listed the parts the open entries need or their lead times; the risk register points F.05.13 and U-007-3 at "procurement (H-04)" without a list. `[D]` (`investigations/README.md`; SPEC-007 U-007-3)
- P-03 (U-010-1, U-010-2: whether the vendor modules build and bind on `linux-rpi` 6.18) had no fallback; a failure stalled P3, P4 and P5 entirely. `[D]` (SPEC-010 known unknowns)
- V-07 (Part 97 licence and profile) sat in P6, after the bench phases, although P7 and G06's licensed branch have lawful purpose only under Part 97 and a licence has lead time. `[D]` (G06 F.06.11; SPEC-005 S-005-14…18)

### D. The fallback's bound (T24)

- Four channels at 26 MSPS in CS8 are 1.66 Gbit/s, 166 % of Gigabit Ethernet line rate; the usable TCP payload at a 1500 B MTU is 949 Mbit/s (94.9 % of line rate); two channels at 26 MSPS or four at 13 MSPS (0.83 Gbit/s, 83 %) fit, four at 6.5 MSPS or one at 26 MSPS (0.42 Gbit/s) fit with margin; whole-band four-channel capture never crosses the LAN. `[D]` (analysis T24)
- The Pi 5's RP1 provides a 10/100/1000 Mbit/s Ethernet MAC (Cadence GEM_GXL) over RGMII to an external Gigabit PHY. `[S]` (`resources/datasheets/RP1-peripherals.txt`, overview bullet "Gigabit Ethernet" and the Ethernet subsystem section)
- T24 bounds the link, not the server: whether the vendor's SoapyRemote on the Pi sustains 0.83 Gbit/s of CS8 is `[C]` until P-03b measures it.

### E. Human-blocked entries in P0

- V-02c and V-04 need a browser: semtech.com returns an HTML interstitial, apps.fcc.gov "Access Denied" and fccid.io a JavaScript challenge to scripted requests. `[S]` (LR-002 §P1; `backlog.md` V-04)
- D-05 needs a reader other than the author of the statements it reviews. `[S]` (`docs/00-process.md` §3.1: Reviewed means "a second reader checked every tag")

### F. Counts

| | Before (`3b09f1a`) | After |
| --- | --- | --- |
| Open | 53 | 61 |
| Done | 16 | 16 |
| Split | — | D-05 → D-05a (user: SPEC-001, SPEC-005) + D-05b (a session that did not write them: SPEC-002/003/004 rev 2, SPEC-008…011); V-07 → V-07a (decision, P0) + V-07b (profile, P6) |
| New | — | D-09 procurement list (P0); V-08 test oracle and corpus (P1); R-14 control plane and refusal tests (P1); D-10 release procedure (P1); P-03b SoapyRemote fallback (P3); R-03b ring recording and replay (P4) |
| Moved | — | F-01, F-02, F-05: P2 → P0; V-07a: P6 → P0 |
| Rewritten checks | — | R-03 (synthetic frames now, replay in R-03b), R-07 (budget on a Pi 5 only), R-P1 (injected yaw, declination and 300 ms delivery delay; negative control), R-08 (against the V-08 corpus, oracle PER beside ours), R-06 (shares V-08) |

`[M]` (this report; `git diff 3b09f1a -- backlog.md`)

## V. Discussion

What the review did not find wrong: every entry has a check; P1–P7 each end
in an exit check; the kit dependency is explicit at P3; the rule that nothing
closes on an open conjecture is written down. P0 never had an exit check,
and the index already said that P0–P2 "run now", which contradicted the
sequential-phase rule of `docs/00-process.md` §3.5 without saying so. The
re-sequencing makes the deviation explicit: P0 is a standing phase of
independent, mostly human-blocked lookups that P1 must not wait for, and P2
is the first phase entered on an exit check.

The largest practical consequences are two. First, the demodulator (R-08)
now has a prerequisite that produces its oracle and a corpus with a recorded
truth, so that its PER is compared with the oracle's on the same frames
rather than asserted against a threshold alone. Second, the control plane
(R-14) is tested before any binary can command a transmitter, which is the
order `docs/00-process.md` §6 implies but the backlog did not enforce.

Duration (opinion of the reviewer, binding nothing): P1 four to six weeks of
focused work, dominated by R-07 and R-08; P2 one to two weeks after the
purchases arrive; P3 about a week with the kit if P-03 builds cleanly, a week
more if it does not; P4 two to three weeks; P5 three to four weeks plus the
parts' lead time.

What would refute this report: a P2 entry whose check runs without any P1
crate (then its move was wrong); `gr-lora_sdr` failing to build against GNU
Radio 3.10.12 on musl (then V-08's fallback, frames from the SX1262 stick
recorded through the dongle, becomes the oracle and R-08 waits for the
purchases); the vendor modules building unmodified on the first day (then
P-03b is never invoked and costs nothing). The proxy objection to R-07 is
one of method and stands even if a VM fraction and the A76 fraction happen
to agree.

## VI. Recommendations and best practice

1. P0 is standing and concurrent with P1 (`backlog.md` *Sequencing*); its human-blocked entries (D-05a, V-02c, V-04, V-07a, D-09) are the user's next actions, and D-05b is a separate agent session that did not write the statements it reads.
2. P1 runs in dependency order: R-02, V-08, R-08, R-03, R-07, R-09, R-02b, R-06, R-14, D-10, R-P1.
3. P2 is entered on R-P1 plus the D-09 purchases; F-01, F-02 and F-05 run as soon as the nodes are on the desk (P0).
4. A CPU budget stated per A76 core is measured on a Pi 5, never on the VM (R-07); if no Pi 5 is on the desk before the kit, it is a D-09 row.
5. A simulated exit check injects a known error and carries a negative control that must fail (R-P1).
6. A spec statement that constrains a later entry gets its own test entry before that entry (R-14 before R-10; D-10 before P-01).
7. A phase that waits on an unknown with a stop-the-line outcome gets a fallback entry whose capacity is bounded by an `analysis/` table (P-03b, T24).
8. A check that names a tool names the entry that installs it (R-06 and R-08 name V-08).

## VII. References

`backlog.md` (`3b09f1a` and this revision); `docs/00-process.md` §3.1, §3.5, §6; `specs/SPEC-002-quadrf-host-software.md` S-002-4, S-002-14…17, S-002-21; `specs/SPEC-008-system-architecture-rust.md` S-008-9, S-008-10, S-008-11, S-008-13; `specs/SPEC-009-sensor-plugins-and-feed-bus.md` S-009-3, S-009-5, S-009-6, S-009-7, S-009-13; `specs/SPEC-010-copal-platform.md` S-010-9, S-010-11, U-010-1, U-010-2; `investigations/G06-mountain-repeater-directional-relay.md` F.06.11; `investigations/G07-field-test-plan.md` §8; `investigations/README.md` (risk register); `lab/LR-002-operating-procedures-and-tools.md` §P1; `lab/LR-003-rust-system-architecture.md` §C; `lab/LR-005-coherent-915mhz-receiver-options-and-parallel-feed.md` §VI; `analysis/linkbudget.py` T14, T16, T19, T24; `resources/datasheets/RP1-peripherals.txt` (manifest row "RP1 peripherals").
