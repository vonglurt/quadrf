# Raspberry Pi Ltd — RP1 Peripherals datasheet (RP-008370-DS-1, 2023-11-07): restatement

Attribution: `vendor/raspberry-pi/ATTRIBUTION.md`. Source:
`resources/datasheets/RP1-peripherals.pdf` (93 pages; `RP1-peripherals.txt`
lines cited). Facts `[S]`.

- RP1 is the Pi 5's peripheral controller, connected to the BCM2712 application processor over a PCIe 2.0 x4 link. (Chapter 1, text lines 155–157)
- USB: two independent XHCI controllers, each with one USB 3.0 PHY and one USB 2.0 PHY, "more than 10 Gbps" of downstream USB traffic together. (lines 175–176)
- MIPI: two CSI-2 camera controllers and two DSI display controllers share two 4-lane MIPI D-PHY transceivers; together 8 Gbit/s of downstream traffic to two cameras, two displays, or one of each; each camera controller has an ISP front end. (lines 178–181)
- Gigabit Ethernet MAC over RGMII; 28 GPIO; the internal fabric prioritises real-time camera/display traffic over USB and Ethernet, and PCIe QoS signalling prioritises RP1 traffic against the AP's. (lines 183–192)
- Eight-channel DMAC for low-speed peripherals; three PLLs; a 5-input 12-bit (9.5-bit ENOB) 500 kSPS ADC; 64 kB shared SRAM; timebase tick generators. (lines 203–211)
- PCIe link latency is "typically 1 μs"; reads cost a request and a response, so posted writes are preferred; ASPM adds wake latency. (lines 2257–2290)

## Consequences used in this project

- CSI-2 raw capacity on one 4-lane port at the tile's 700 Mbit/s lane rate is 2.8 Gbit/s; the 4-channel 26 MSPS CS8 stream is 1.66 Gbit/s (59 %) and the 40 MSPS stream 2.56 Gbit/s (91 %) (T14). The PCIe 2.0 x4 link (16 Gbit/s nominal) is not the bottleneck. `[D]`
- USB 3.0 sensor plugins have 4 Gbit/s of payload per port available, far above any SDR dongle's need; the Pi's USB *power* budget (1.6 A with a 5 A supply) is the practical limit (T20). `[D]`
