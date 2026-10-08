# J. Tapparel et al. — gr-lora_sdr

| Field | Value |
| --- | --- |
| Copyright holder | Joachim Tapparel and contributors, EPFL Telecommunication Circuits Laboratory |
| Licence | GPL-3.0 (`LICENSE`) |
| URL | https://github.com/tapparelj/gr-lora_sdr ; paper: J. Tapparel, O. Afisiadis, P. Mayoraz, A. Balatsoukas-Stimming, A. Burg, "An Open-Source LoRa Physical Layer Prototype on GNU Radio", IEEE SPAWC 2020, arXiv:2002.08208 |
| Revision read | commit `862746dd1cf635c9c8a4bfbaa2c3a0ec3a5306c9` (2026-01-05); README retrieved 2026-10-08; paper PDF in `resources/lora/` |
| Files read | `README.md`, `CITATION.cff`; the paper |

## What we use

The published description of the LoRa CSS modulation and demodulation
(SPEC-003), and the capability list of the GNU Radio implementation as a
reference decoder for bench work. Restated in SPEC-003 and
`vendor/summary/tapparel-gr-lora_sdr.md`. Our own LoRa demodulator in Rust is
written from the paper and from first principles, not from the GPL code.
