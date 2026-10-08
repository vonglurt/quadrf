# J. Tapparel et al. — gr-lora_sdr: restatement of the facts we rely on

Attribution: `vendor/tapparel-gr-lora_sdr/ATTRIBUTION.md`. Sources: README at
commit `862746d` (2026-01-05); paper arXiv:2002.08208 (SPAWC 2020). Facts `[S]`.

- The LoRa symbol is a linear chirp of duration 2^SF / BW sweeping BW, cyclically shifted by one of 2^SF starting frequencies; demodulation multiplies by the conjugate base chirp and takes a 2^SF-point DFT; the bin index is the symbol value. (paper §II)
- The transceiver implements SF 5–12, coding rates 0–4, selectable bandwidth, sync word, explicit/implicit header, CRC, low-data-rate optimisation, and operates near the SX127x sensitivity limit; it is hardware-agnostic through GNU Radio sources/sinks (SoapySDR/UHD). (README)
- GNU Radio 3.10 is required; the module is GPL-3.0. (README, `LICENSE`)
- Citation: J. Tapparel, O. Afisiadis, P. Mayoraz, A. Balatsoukas-Stimming, A. Burg, "An Open-Source LoRa Physical Layer Prototype on GNU Radio", IEEE SPAWC 2020. (`CITATION.cff`)
