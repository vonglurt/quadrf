# KrakenRF — KrakenSDR: restatement of the facts we rely on

Attribution: `vendor/krakenrf/ATTRIBUTION.md`. Sources in
`resources/krakensdr/` retrieved 2026-10-08: `wiki-Home.md`,
`wiki-04-Antenna-Array-Setup.md`, `heimdall_daq_fw-README.md`,
`krakensdr_doa-README.md`. Facts `[S]` with line numbers of those files.

## Hardware

- Five receive channels, each an R820T2 tuner and an RTL2832U 8-bit ADC; tuning 24–1766 MHz; maximum channel bandwidth 2.56 MHz; oscillator stability 1 ppm; one clock source feeds all five; one switched wideband noise source; an internal USB hub. (`wiki-Home.md` 49–55, 101–105)
- Coherence is obtained in software: at start the noise source is switched in and each channel is correlated against the master (CH0); sample timing and phase offsets are corrected in software; the vendor states the system is not naturally coherent from hardware alone. (`wiki-Home.md` 107–109)
- Power: a separate USB-C power port, 5 V 2.4 A supply recommended, 2.2 A nominal draw; the USB-C data port carries no power; data cable ≤ 1 m. (`wiki-Home.md` 63–71)
- The device cannot transmit; the noise source is enclosed and isolated, with leakage "well below regulatory compliance thresholds" as stated by the vendor. (`wiki-Home.md` 133)

## Array geometry rules

- Inter-element spacing I_e = s·λ with s ≤ 0.5 to avoid ambiguity, ≥ 0.2 for usable accuracy, typical 0.33; UCA radius r = s·λ / √(2(1 − cos(360°/n))); ULA covers 180° only with aperture (n − 1)·s·λ; cable-length mismatch of 1 cm ≈ 14° at 800 MHz, 7° at 400 MHz. (`wiki-04-Antenna-Array-Setup.md` 12, 52–67, 71–79, 90)
- Vendor resolution estimate: Rayleigh θ = 1.22 λ/D; 5-element UCA at s = 0.5 has D = 0.85 λ → 83°, improved "by a very approximate factor of 10" by MUSIC → 8.3°; ULA → 34° → 3.4°. (`wiki-04-Antenna-Array-Setup.md` 150–158)

## Software

- `heimdall_daq_fw`: the coherent DAQ chain; C core built with `make` plus a conda Python environment (numba, scipy); ZeroMQ and Python shared memory; "Tested on the Raspberry Pi 4"; ready-made images for "Raspberry Pi 4/5 or Orange Pi 5B"; GPL-3.0. (`heimdall_daq_fw-README.md` 4, 11, 42, 161–167, 257)
- `krakensdr_doa`: DoA demonstration software; web UI on port 8080, settings served on 8081, middleware API on 8042; GPL-3.0; a Pi 4/5 image quick start; recommends cooling and overclocking the Pi 4 to ≥ 2000 MHz; numba JIT warm-up 1–2 min. (`krakensdr_doa-README.md`)

## Derived consequences for this project (`analysis/linkbudget.py`)

- At 915 MHz: s = 0.33 → spacing 108.1 mm, radius 92.0 mm, aperture 184 mm; s = 0.5 → spacing 163.8 mm, radius 139.4 mm, aperture 279 mm, Rayleigh 82° (vendor: 83°). (T15) `[D]`
- 5 × 2.4 MSPS × 2 B = 192 Mbit/s, 40 % of one USB 2.0 link; 2.2 A exceeds the Pi 5's 1.6 A USB peripheral budget, so the unit must be powered separately. (T14, T20) `[D]`
- A 2.4 MHz channel sees 9–10 of the 104 US LONG_FAST slots at once; a whole-band occupancy map needs 11 retunes with recalibration risk at each. `[D]` (26 MHz / 2.4 MHz)
