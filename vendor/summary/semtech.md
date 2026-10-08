# Semtech — SX1262 and LR1121 product data: restatement

Attribution: `vendor/semtech/ATTRIBUTION.md`.

## SX1262 (product page retrieved 2026-10-08, `resources/lora/semtech-sx1262-product.html`) `[S]`

- Continuous frequency coverage 150–960 MHz; up to +22 dBm output (SX1261: +15 dBm) with a high-efficiency PA; sensitivity "down to −148 dBm"; 170 dB maximum link budget (SX1262/68); 88 dB blocking immunity at 1 MHz offset; up to 62.5 kbit/s LoRa and 300 kbit/s FSK. The page does not list LoRa bandwidths or spreading factors; the datasheet is needed for those and for the SNR thresholds used in SPEC-003 S-003-2 (currently sourced from the Meshtastic link-budget table).

## LR1121 and the absence of LoRa silicon above 2.5 GHz (search summary of Semtech and distributor pages, 2026-10-08) `[C]` until the product page is in `resources/`

- LR1121 covers sub-GHz 150–960 MHz (+22 dBm), the 2.4 GHz ISM band (+11.5 dBm), satellite S-band around 1.9–2.1 GHz and L-band around 1.55 GHz; distributor listings give 150 MHz–2.5 GHz overall. SX1280/SX1281 are 2.4 GHz parts. No Semtech LoRa transceiver lists any band above 2.5 GHz; a 5.8 GHz LoRa link exists only as an SDR-generated waveform (G04 C10).
