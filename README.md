# quadrf — LoRa on a 5 GHz phased array: a gated investigation and a Rust design on copal

Start at [`index.md`](index.md).

**Question.** The ScaleRF QuadRF is a 4-channel coherent, full-duplex SDR tile
with four patch antennas, an ECP5 FPGA beamformer, and a Raspberry Pi 5 host,
operating 4.9–6.0 GHz. This repository investigates, one gated step at a time,
whether and how it can be applied to LoRa / Meshtastic work in the Pacific
Northwest: receiving and mapping LoRa emitters with bearings, building a LoRa
phased array, transmitting at the legal limit, and relaying to mountain-top
repeaters with a highly directional link. Since 2026-10-08 it also designs the
system that does it: Rust processes on copal (the user's Alpine Linux
installer for the Pi), USB and plug-in sensors, and a second data feed drawn
on the tile's overlay.

**Method.** Specification-first, synchronous, gated. Each investigation step
asks one question, is closed by an explicit gate record, and feeds a fixed set
of carried-forward facts into the next step. The design track is ledgered in
`backlog.md` with a check per entry. See `docs/00-process.md`.

## Layout

| Path | Content |
| --- | --- |
| `index.md` | Map of every document, both tracks, conventions, state |
| `backlog.md` | Design and build track: phases P0–P7, entries with checks |
| `docs/00-process.md` | The documentation and gating practice (read first) |
| `docs/templates/` | Templates for investigations, clean-room specs, gate records, measurements, lab reports |
| `docs/resources-manifest.md` | What was downloaded into gitignored `resources/`, from where, when, which commit |
| `specs/` | Clean-room specifications (SPEC-001…006, 011) and implementation specs (SPEC-007…010) |
| `investigations/` | The gated sequence G00…G07, the gate ledger and the risk register |
| `lab/` | Lab reports: audit, procedures, architecture, platform, receiver options, datasheet review, backlog review |
| `analysis/` | The script that produces every derived number (T1–T25) |
| `crates/`, `Cargo.toml` | The qrf Cargo workspace; `qrf-analysis` is the byte-identical Rust port of the analysis |
| `Makefile` | `make check` runs every linter, the analysis, the Rust build and the Rust/Python parity test |
| `vendor/` | Licence ledger per upstream, own-words summaries of what we may not copy, public-domain CFR text |
| `scripts/` | Fetch, import, vendor, tag-lint, link-check and citation-check tools; the LoRa oracle build and test-corpus generator |
| `resources/` | Gitignored raw sources (vendor docs, CFR pages, datasheets, cloned repos) |

## State of the investigation (2026-10-08)

| Gate | Title | Status |
| --- | --- | --- |
| G00 | Charter and constraints | PASS |
| G01 | QuadRF hardware survey | PASS (+ datasheet addenda) |
| G02 | Regulatory envelope, US 902–928 / 5725–5850 MHz, PNW | PASS (+ exposure-rule addenda) |
| G03 | Raspberry Pi software stack | PASS (+ HAT, copal, Rust addenda) |
| G04 | Inventory: ways to put LoRa on / through the QuadRF | PASS (+ C12–C14) |
| G05 | 915 MHz frequency-translating front end: feasibility | OPEN (desk analysis done; bench gate after kit delivery, vendor date 2026-11-30) |
| G06 | Mountain-top repeater: directional relay recommendations | OPEN (depends on G05 decision) |
| G07 | Field-test plan with ESP32 / Pico LoRa nodes | DRAFT |

Headline findings, each proved or sourced in the investigations:

1. The tile's RF core cannot tune to 902–928 MHz. The converter IC is specified 4.9–5.9 GHz (datasheet), the LNAs, PAs, patch elements and both software tuning guards are confined to 4.9–6.0 GHz. Native LoRa-band reception is physically excluded; a per-element frequency-translating front end is the only path (G01, G05).
2. A LoRa-waveform PHY plus Meshtastic daemon already runs on the tile at 5.8 GHz (third-party `quadrf-mesh`, packaged into the vendor image); by that project's own measurements two tiles form a LoRa-modulated 5.8 GHz link (G03, G04).
3. Under 47 CFR 15.247 the 902–928 MHz EIRP ceiling is 36 dBm regardless of antenna gain; directional gain there only buys receive margin and spatial filtering. In 5725–5850 MHz, fixed point-to-point links may use unlimited antenna gain at 1 W conducted. The high-EIRP directional relay therefore belongs at 5.8 GHz under Part 15, or at 915 MHz under Part 97 (G02, G06).
4. The 45.5 mm element pitch is 0.88 λ at 5.8 GHz: direction finding is unambiguous only to ±35° and beam steering beyond ~8° admits a grating lobe. A 915 MHz array must be rebuilt at 164–288 mm pitch; the FPGA beamformer accepts it because steering depends only on the product k·d (G01, G05).
5. The receiver IC's fractional-N spurs (−42 dBc) sit above the 8-bit converter's floor: a whole-band 915 MHz map will show replicas of strong emitters at deterministic offsets, which the DSP flags by bearing and offset (G05 addendum, LR-006).
6. Four interleaved channels at 26 MSPS are 59 % of the CSI link and arrive every 630 µs with 10 ms of kernel buffering; a 128-bin channeliser for all 104 slots costs about 0.73 of one A76 core (SPEC-001 rev 2, SPEC-008).
7. Everything below the application layer exists in Alpine for the Pi 5 (kernel 6.18.52 with headers, akms, OpenOCD, SoapySDR, Rust); the vendor's Debian/systemd/Flask layer is replaced by Rust processes on copal, with GPL components isolated behind IPC (SPEC-008, SPEC-010, LR-003, LR-004).
8. Of the Meshtastic Pi HATs, only the MeshAdv-Mini 900M22S and the USB sticks are clear of the tile's JTAG pins; every HAT with reset on GPIO 18 collides (SPEC-004 rev 2).
9. With all six component datasheets in hand: the receive chain's typical noise figure is 1.3 dB (the vendor's 1.2 dB is the LNA's best-case corner), the PA is 26 dBm linear per element (the "1 W" is BOM wording; saturated power unknown), a MAX2871 translator LO would be 21 dB cleaner than the tile's own LO, and the SX1262 node model gains 0.5–2 dB of realism from the datasheet sensitivities (SPEC-001 rev 3, SPEC-003 rev 2, SPEC-007 rev 1).
