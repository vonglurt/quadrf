# G05 — 915 MHz frequency-translating front end (FTFE): feasibility

| Field | Value |
| --- | --- |
| Status | OPEN — desk analysis complete; bench gate scheduled after kit delivery (vendor ship date 2026-11-30) |
| Opened | 2026-10-08 |
| Closed | — |
| Author | project |
| Inherits from | G04 |

## 1. Question

Can a four-way coherent block up-converter from 902–928 MHz to the tile's
band, with an external 915 MHz aperture, turn the tile into a receive LoRa
phased array whose sensitivity, dynamic range and bearing accuracy are
sufficient to map Meshtastic traffic and to beamform on a repeater?

## 2. Inherited facts

- F.01.4, F.01.5, F.01.6, F.01.7, F.01.8, F.01.11, F.01.12, F.03.6, F.03.8, F.04.1, C12.

## 3. Method

Desk: derive the translation plan (LO, image, spurious), the cascade noise
figure, the aperture geometry, the beamformer coordinate transform, the data
and CPU budgets, and the calibration transfer. Bench (after delivery): the
measurements in §8 close U-001-1, U-001-2, U-001-3, U-001-5, F.03.8.

## 4. Findings (desk)

Translation plan:

- F.05.1 Choose non-inverting up-conversion f_tile = f_LO + f_RF with f_LO = 4585 MHz, mapping 902–928 MHz onto 5487–5513 MHz, centred at 5500 MHz, inside the tile's band and within one 40 MHz channel. The image band f_LO − f_RF = 3657–3683 MHz is 1.8 GHz from the wanted RF and is removed by the 915 MHz band-pass filter ahead of the mixer. LO leakage at 4585 MHz lies 915 MHz below the tile's LO and outside its 40 MHz baseband. `[D]`
- F.05.2 Any common LO frequency error or drift appears identically on all four channels and is therefore invisible to phase-difference DoA and to the FPGA beamformer; it is seen by the LoRa demodulator as CFO, which Meshtastic-class demodulators and `quadrf-mesh` estimate and track (the tile pair already tolerates ±1 kHz wander at 500 kHz presets). A free-running TCXO LO at 1 ppm (4.6 kHz) is acceptable for SHORT_TURBO/LONG_TURBO and marginal for LONG_FAST unless the demodulator's acquisition range exceeds it; locking the translator LO to an exported tile reference removes the term. `[D]`+`[S]` (SPEC-003 S-003-9; SPEC-006 S-006-6; U-001-2)
- F.05.3 Inter-channel phase coherence requires the four LO ports to be driven from one synthesiser through a 4-way splitter with equal electrical lengths; static phase and amplitude differences among the four paths are constants and are absorbed into the per-element calibration table, which the vendor method already solves for. `[D]`+`[S]` (F.01.12)
- F.05.4 Cascade noise figure, per channel: LNA NF 1.0 dB / G 20 dB; filter and mixer conversion loss 7 dB; tile NF 1.2 dB. F_total = F1 + (F2 − 1)/G1 + (F3 − 1)/(G1·G2) = 1.26 + (5.01 − 1)/100 + (1.32 − 1)/(100 × 0.2) = 1.26 + 0.040 + 0.016 = 1.32 → NF ≈ 1.2 dB. An inter-stage amplifier is unnecessary; a 915 MHz SAW filter before the LNA adds its insertion loss (≈ 2 dB) directly. Expected system NF ≈ 3 dB with a pre-LNA filter, ≈ 1.3 dB without. `[D]` (Friis cascade)
- F.05.5 Sensitivity through the FTFE at NF 3 dB: LONG_FAST −134.5 dBm, SHORT_TURBO −121.5 dBm, 3 dB better than an SX1262 node; the receive array then adds up to 6 dB of coherent gain for a source in the beam. `[D]` (SPEC-003 S-003-3 with NF = 3)
- F.05.6 Dynamic range is bounded by the 8-bit converters: a nearby 22 dBm node at 10 m (−30 dBm at the antenna, FSPL 51.7 dB) against a −130 dBm wanted signal is a 100 dB spread, far beyond 8 bits. Mitigation is procedural: the local node is muted during captures, or an RF attenuator is switched in; this is the same constraint as any receiver co-sited with a transmitter. `[D]`

Aperture geometry:

- F.05.7 The external aperture must be re-pitched for 915 MHz. Half-wave pitch is 164 mm (unambiguous DoA ±90°, no grating lobes for any steer); the pitch that preserves the tile's own d/λ = 0.880 is 288 mm (same 33° two-element HPBW, same ±34.6° ambiguity, same 7.8° grating-lobe steer limit). A 2×2 of 164 mm fits a 40 × 40 cm panel; quarter-wave monopoles over a ground plane, or 915 MHz patches (≈ 10 cm on FR4, εr ≈ 4.4), are the element options. Linear elements lose the circular-polarisation multipath rejection of F.01.8 but match Meshtastic whips without the 3 dB penalty. `[D]` (analysis T9)
- F.05.8 Beamformer transform: the FPGA computes steering phases at the tile LO frequency; presenting scaled coordinates d' = d × (f_tile / f_RF) = d × 6.01 makes k'·d' = k·d. With 164 mm true pitch the beamformer is told 986 mm; with manual `p1..p4` phases no transform is needed. The RF-vision demo instead needs its two constants changed (F.03.6). `[D]` (SPEC-001 S-001-18)
- F.05.9 Narrowband steering holds: 0.33 m aperture transit 1.1 ns versus 2 µs at 500 kHz (ratio 5.5 × 10⁻⁴). `[D]` (analysis T10)

Budgets:

- F.05.10 Full-band interleaved capture: 4 × 26 MSPS × 2 B = 208 MB/s, 30 % of the link. `[D]` (SPEC-001 S-001-23)
- F.05.11 CPU: a 104-slot polyphase channeliser at 26 MSPS is ≈ 4 × 26e6 × log₂(128) ≈ 0.7 GFLOP/s per channel class, feasible on four A76 cores for energy detection and per-burst DoA; continuous demodulation of all slots is not. Target: occupancy + bearing for all slots, decode for ≤ 4 slots. `[D]`+`[C]` (F.03.8; to be measured)

Calibration:

- F.05.12 The vendor transponder method transfers if the transponder emits at 915 MHz and the unwrapping condition (path change per sample < λ/2 = 164 mm) holds; at 1 kHz phase sampling and 1 m/s motion the change per sample is 1 mm, so unwrapping is trivial at 915 MHz. A LoRa node emitting a continuous preamble, or a CW at 915 MHz, serves as the transponder. `[D]`+`[S]` (SPEC-001 S-001-20)

Cost and parts (indicative, `[C]` until quoted):

- F.05.13 Per channel: 915 MHz SAW/ceramic BPF, LNA (0.6–1 dB NF, +20 dB), double-balanced mixer rated to 6 GHz, 5.5 GHz BPF, attenuator pad; shared: 4.585 GHz synthesiser (a 23.5 MHz–6 GHz PLL evaluation board suffices), 4-way power divider, phase-matched cables, 5 V regulator. Order of US$ 300–500 in evaluation-board form. The comparator C12 (5-channel coherent receiver, native 24–1766 MHz, 8-bit, Rx-only) is of the same order and needs no front end, but gives no tile integration and no 5.8 GHz path. `[C]`

## 5. Analysis

Every desk-closable assumption closes in favour of C3. The residual risks
are all mechanical/electrical and are listed as bench items: the element-port
connector (U-001-1), the tile's gain flatness and NF around 5500 MHz
(U-001-3), reference export (U-001-2), the factory bitstream's behaviour with
a translated band (U-001-5), and CPU (F.05.11). None is a physics obstacle.

## 6. Answer (provisional)

Yes on paper: the FTFE turns the tile into a four-channel coherent 902–928 MHz
receiver with NF ≈ 1.3–3 dB, whole-band capture, and a 915 MHz aperture
geometry of our choosing, with bearing and beamforming maths unchanged apart
from a coordinate scale. The bench gate decides.

## 7. Recommendation (opinion)

Build a single-channel FTFE first, verify tile-side NF and spurious-free
operation at 5500 MHz, then replicate to four channels. Order a 915 MHz
quarter-wave monopole ground-plane array at 164 mm pitch for the first field
test; it is the cheapest unambiguous aperture.

## 8. Gate record

| Field | Value |
| --- | --- |
| Pass criterion (bench) | (1) Element-port pigtail connection with ≤ 1 dB loss at 5500 MHz, `[M]`. (2) Translated single channel: measured NF ≤ 4 dB and spurious ≤ −60 dBc in 5487–5513 MHz, `[M]`. (3) Four channels: inter-channel phase stable to ≤ 5° rms over 10 min at fixed temperature, `[M]`. (4) DoA of a 915 MHz LoRa node at 20 m: bearing error ≤ 5° rms over ±60°, `[M]`. (5) Full-band 26 MSPS interleaved capture sustained ≥ 60 s with occupancy for all 104 slots at ≤ 80 % of four cores, `[M]`. |
| Evidence | pending |
| Decision | OPEN |
| Date / signatory | — |
| Open conjectures carried as risk | U-001-1, U-001-2, U-001-3, U-001-5, F.05.11, F.05.13 |

## 9. Carry-forward (provisional, confirmed at bench)

- F.05.1–F.05.12; the pass/fail of criteria (1)–(5).

## 10. Addenda (2026-10-08, second pass; gate remains OPEN)

Desk findings from the MAX2851 datasheet (LR-006), the bearing-precision
bound (T16) and the plug-in architecture (LR-003/LR-005):

- F.05.14 The tile's receiver synthesiser has −42 dBc fractional spurs at 0–19 MHz offsets, above the 8-bit single-tone SFDR (≈ 49.9 dB); through the FTFE a −30 dBm in-band emitter yields replicas at −72 dBm (58 dB above a −130 dBm LoRa signal) at deterministic offsets sharing the parent's bearing. Mitigations: choose the translator LO so measured spur offsets fall outside the 104 slots; flag detections that match a strong emitter's bearing and a measured offset as replicas; lower the tile RF gain when a strong emitter is present (P1dB −34 → −18 dBm at max − 16 dB). `[S]`+`[D]` (SPEC-001 S-001-28/30/32; analysis T17; LR-006 §C)
- F.05.15 The tile LO's integrated phase noise is 1.44° rms and is common-mode across the four channels of one IC; it does not enter phase-difference bearings. `[S]`+`[D]` (S-001-30; T17)
- F.05.16 The thermal-noise bound on a two-element phase-difference bearing at half-wave pitch is 0.57° at 0 dB SNR with 1024 samples and 0.057° at 20 dB; criterion (4)'s 5° rms is therefore a calibration and multipath budget, not a sensitivity one. `[D]` (analysis T16)
- F.05.17 The FTFE cascade from the script: 1.19 dB NF without a pre-filter, 3.19 dB with a 2 dB SAW ahead of the LNA, reproducing F.05.4 and F.05.5 (which used ≈ 1.3 / ≈ 3 dB). `[D]` (analysis T18)
- F.05.18 The tile IC's gain flatness is ≤ 4.2 dB peak-to-peak over the whole 4.9–5.9 GHz span, which bounds U-001-3's flatness term over the 26 MHz window; the NF term stays a bench item. `[S]`+`[D]` (S-001-31)
- F.05.19 Baseband placement: with the LO at 5500 MHz the slot at DC (slot 52, 914.875 MHz) sits in the receiver's high-pass region unless the 0.1 kHz corner is selected; the capture daemon selects it or the slot is excluded from the map. `[S]`+`[D]` (S-001-29)
- F.05.20 The FTFE-fed tile is one sensor plug-in among several; its `Bearing` messages are computed with the true 915 MHz wavelength and the surveyed external aperture. `[D]` (SPEC-009 S-009-8)

Bench criterion added to §8: (6) LO-spur replicas: with a −30 dBm CW at the
915 MHz input, record every product within 5487–5513 MHz above −100 dBm with
its offset and level, `[M]`; the replica-flagging rule (F.05.14) uses this
table.

Open conjecture added: U-001-6 (FPGA part) is read at first power-up (backlog
H-01).
