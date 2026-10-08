# analysis/

`linkbudget.py` is the single source for every `[D]` number in the documents.

```
python3 analysis/linkbudget.py
```

Tables are numbered T1…T20 and are referenced by that number from the
investigations, specs and lab reports. No third-party dependencies.

| Table | Content | Cited by |
| --- | --- | --- |
| T1 | LoRa receiver sensitivity per preset, NF 6 and 4.2 dB | SPEC-003, G05, G06 |
| T2 | Free-space path loss 915 / 5800 MHz | G06 |
| T3 | Weissberger foliage loss | G06 |
| T4 | Knife-edge diffraction | G06 |
| T5 | First Fresnel radius | G06 |
| T6 | Radio horizon | G06 |
| T7 | Regulatory EIRP cases and 40 km margins | G02, G04, G06 |
| T8 | §15.247(b)(4) power reduction vs antenna gain | G02 |
| T9 | Tile array geometry (45.5 mm pitch) and 915 MHz pitches | G01, G05 |
| T10 | Narrowband steering validity | SPEC-003, G05 |
| T11 | RF exposure compliance distances | G02, G06 |
| T12 | LO wander vs LoRa bin width | SPEC-003 |
| T13 | 8-bit quantisation penalty | SPEC-001 |
| T14 | Data-path budgets: CSI lanes, RP1, PCIe, USB, frame timing, KrakenSDR | SPEC-001 rev 2, SPEC-008, SPEC-011, LR-003, LR-006 |
| T15 | 915 MHz apertures: 4-element square and 5-element UCA | SPEC-011, LR-005 |
| T16 | Bearing precision CRLB vs calibration | G05 addendum, LR-003 |
| T17 | MAX2851 phase noise and spurs vs 8-bit floor | SPEC-001 rev 2, G05 addendum, LR-006 |
| T18 | Receive cascade NF with the datasheet value; FTFE cascade | SPEC-001 rev 2, G05 addendum, LR-006 |
| T19 | CPU budget for the whole-band channeliser | SPEC-008, LR-003 |
| T20 | USB power budget on the Pi 5 | SPEC-009, SPEC-011, LR-005 |

A Rust port of these tables (`qrf-analysis`) is backlog entry A-02; its
acceptance check is byte-identical output to this script.
