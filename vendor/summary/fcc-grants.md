# FCC equipment authorisations of LoRa modules: what was found, and its status

FCC grants are public records at https://www.fcc.gov/oet/ea/fccid. The facts
below came from a search-engine summary of the third-party mirror fccid.io on
2026-10-08; the mirror itself serves a JavaScript challenge to scripted
clients and is not in `resources/`. They are therefore `[C]` until read at the
FCC's own database (SPEC-005 U-005-1).

- Seeed Technology "Wio-SX1262", FCC ID Z4T-WIO-SX1262: original grant dated 2024-10-16, equipment class DSS (Part 15 spread-spectrum transmitter), modular approval with OEM-integrator conditions, listed frequency rows 902.3–914.9 MHz and 903–914.2 MHz; test laboratory Shenzhen BALUN Technology.
- RHF0M62H SX1262 module, FCC ID 2AJUZ0M62H (RuiXingHengFang Network): a re-label of the above dated 2024-10-23, listed as class DTS (digital transmission system), same frequency rows.
- No grant records were found in that search for Heltec, RAK4631 or EBYTE E22-900M30S modules; they must be looked up by FCC ID printed on each module.

## Why it matters

If a certified module's grant covers only 902.3–914.9 MHz, operating it on
Meshtastic slots above 915 MHz (slots 53–103 of the US LONG_FAST plan) is
outside its authorisation even though the band is 902–928 MHz. The DSS/DTS
class also decides whether the host must hop or whether a 500 kHz digitally
modulated emission was the basis of the grant (G02 F.02.5). Reading the actual
grants and their test reports is backlog entry V-04.
