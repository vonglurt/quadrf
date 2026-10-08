# quadrf — LoRa on a 5 GHz phased array: a gated investigation

**Question.** The ScaleRF QuadRF is a 4-channel coherent, full-duplex SDR tile
with four patch antennas, an ECP5 FPGA beamformer, and a Raspberry Pi 5 host,
operating 4.9–6.0 GHz. This repository investigates, one gated step at a time,
whether and how it can be applied to LoRa / Meshtastic work in the Pacific
Northwest: receiving and mapping LoRa emitters, building a LoRa phased array,
transmitting at the legal limit, and relaying to mountain-top repeaters with a
highly directional link.

**Method.** Specification-first, synchronous, gated. Each investigation step
asks one question, is closed by an explicit gate record, and feeds a fixed set
of carried-forward facts into the next step. See `docs/00-process.md`.

## Layout

| Path | Content |
| --- | --- |
| `docs/00-process.md` | The documentation and gating practice (read first) |
| `docs/templates/` | Templates for investigations, clean-room specs, gate records |
| `docs/resources-manifest.md` | What was downloaded into gitignored `resources/`, from where, when |
| `specs/` | Clean-room specifications: behaviour-goal statements in our own words |
| `investigations/` | The gated sequence G00…G07 and the gate ledger |
| `analysis/` | Scripts that produce every derived number quoted in the documents |
| `scripts/fetch-resources.sh` | Re-populates `resources/` |
| `resources/` | Gitignored raw sources (vendor docs, CFR text, cloned repos) |

## State of the investigation (2026-10-08)

| Gate | Title | Status |
| --- | --- | --- |
| G00 | Charter and constraints | PASS |
| G01 | QuadRF hardware survey | PASS |
| G02 | Regulatory envelope, US 902–928 / 5725–5850 MHz, PNW | PASS |
| G03 | Raspberry Pi software stack | PASS |
| G04 | Inventory: ways to put LoRa on / through the QuadRF | PASS |
| G05 | 915 MHz frequency-translating front end: feasibility | OPEN (desk analysis done, bench gate pending) |
| G06 | Mountain-top repeater: directional relay recommendations | OPEN (depends on G05 decision) |
| G07 | Field-test plan with ESP32 / Pico LoRa nodes | DRAFT |

Headline findings, each proved or sourced in the investigations:

1. The tile's RF core cannot tune to 902–928 MHz. The converters, LNAs, PAs,
   patch elements and both software tuning guards are confined to 4.9–6.0 GHz.
   Native LoRa-band reception is physically excluded; a per-element
   frequency-translating front end is the only path (G01, G05).
2. A LoRa-waveform PHY plus Meshtastic daemon already runs on the tile at
   5.8 GHz (third-party `quadrf-mesh`, packaged into the vendor image). Two
   tiles form a LoRa-modulated 5.8 GHz link today (G03, G04).
3. Under 47 CFR 15.247 the 902–928 MHz EIRP ceiling is 36 dBm regardless of
   antenna gain; directional gain there only buys receive margin and spatial
   filtering. In 5725–5850 MHz, fixed point-to-point links may use unlimited
   antenna gain at 1 W conducted. The high-EIRP directional relay therefore
   belongs at 5.8 GHz under Part 15, or at 915 MHz under Part 97 (G02, G06).
4. The 45.5 mm element pitch is 0.88 λ at 5.8 GHz: direction finding is
   unambiguous only to ±35° and beam steering beyond ~8° admits a grating lobe.
   A 915 MHz array must be rebuilt at 164–288 mm pitch; the FPGA beamformer
   accepts it because steering depends only on the product k·d (G01, G05).
