# SPEC-009 — Sensor plug-ins (USB, HAT, tile) and the feed bus that joins them in one overlay

| Field | Value |
| --- | --- |
| Status | Draft |
| Revision | 1 |
| Date | 2026-10-08 |
| Subject | How a physical sensor (the tile, a USB SDR, a LoRa HAT or stick, a GNSS receiver) becomes a plug-in process; the typed messages every plug-in publishes; the time base and coordinate frame that let two sensors' outputs be drawn on one overlay |
| Primary sources | SPEC-001, SPEC-002, SPEC-004 rev 2, SPEC-008, SPEC-011; `analysis/linkbudget.py` T14, T20; `lab/LR-003`, `lab/LR-005` |
| Depends on | SPEC-008 |

## Scope

Discovery, supervision, schema, transport, time, geometry, conformance.
Statements are design decisions (`[D]`) unless they restate a platform fact.

## Behaviour-goal statements

Discovery and supervision:

- S-009-1. A sensor plug-in is a process that owns exactly one physical device and publishes typed messages; it is declared by a TOML file in `/etc/qrf/sensors.d/` naming the executable, the device match (USB vendor:product and optional serial, a device node, or `tile`), the mounting transform, the regulatory role (`rx-only` or `tx-capable`), and rate limits. `[D]`
- S-009-2. `qrf-sensord` supervises plug-ins: it watches device arrival and removal (USB hot-plug through `nusb`'s hotplug API, or the platform's device manager events), starts the matching plug-in as user `qrf` with device access granted by device-manager rules, restarts a crashed plug-in with exponential back-off capped at 30 s, and never runs two plug-ins on one device. `[D]`
- S-009-3. Plug-in classes in scope: (a) the tile (`qrf-tiled`, SPEC-008 S-008-2), natively at 4.9–6.0 GHz or behind the FTFE with a declared `f_translate`; (b) an RTL2832U dongle through `rtlsdr-nusb` for coarse 24–1766 MHz scanning and single-slot decoding; (c) a HackRF through `seify` for 1 MHz–6 GHz sweeps; (d) a KrakenSDR-class receiver (SPEC-011) through its DAQ output or a native coherent capture; (e) a Meshtastic node bridge over TCP 4403 (a GPL plug-in process); (f) a GNSS/PPS time source. `[D]` (SPEC-004 S-004-11; SPEC-011; user requirement for USB and plug-in items)

Schema (protobuf package `qrf.v1`, append-only; a breaking change is a new package):

- S-009-4. Every message starts with `Header { sensor_id: string, seq: u64, t_tai_ns: i64, t_mono_ns: i64, clock_quality: enum {PPS, NTP, FREE}, power_ref: enum {DBM, DBFS} }`; `seq` increases by one per message of the sensor across all types; `power_ref` is the flag S-009-5 requires. `[D]`
- S-009-5. Message types: `Spectrum { f_start_hz, f_step_hz, n_avg, bins_dbm: [f32] }`; `Occupancy { plan: string, slots: [ { power_dbm, duty, az_deg, az_sigma_deg } ] }` for the 104-slot US plan or any declared plan; `Bearing { f_hz, az_deg, el_deg, sigma_deg, snr_db, burst_id }`; `LoraFrame { f_hz, sf, bw_hz, cr, snr_db, rssi_dbm, cfo_hz, bearing: Bearing, crc_ok, payload: bytes, t_start_ns }`; `Scatter { points: [ { az_deg, el_deg, power_dbm, f_hz } ] }` for the tile's swept-LO view; `Calibration { element_pos_m: [[x,y,z]], element_gain: [[re,im]], f_hz }`; `Health { cpu_pct, temp_c, loss_events, msgs_per_s, device_present, throttled }` (`throttled`: messages the S-009-10 limiter dropped since start). Angles are degrees, true-north azimuth clockwise, elevation positive up; powers are dBm at the antenna port unless `Calibration` is absent, in which case they are dBFS and flagged. `[D]` (SPEC-003 S-003-8 slot plan)

Time and geometry:

- S-009-6. `t_tai_ns` is CLOCK_TAI disciplined by chrony (NTP) or by gpsd with PPS; a plug-in without a disciplined clock reports `clock_quality = FREE` and the overlay widens its association window from 100 ms to 1 s for that sensor. `[D]`
- S-009-7. Each sensor declares a mounting transform (yaw, pitch, roll relative to true north and the horizon) measured at installation; the tile's element-relative azimuth/elevation and a 915 MHz array's bearings are mapped through their transforms before they are drawn; a compass-derived yaw is corrected for magnetic declination at the site. `[D]`
- S-009-8. For the FTFE-fed tile, bearings are computed with the true 915 MHz wavelength and the external aperture's surveyed positions, never with the tile's native pitch. `[D]` (G05 F.05.8; SPEC-007 S-007-11)

Transport:

- S-009-9. Each plug-in publishes on ZeroMQ `ipc:///run/qrf/<sensor_id>.pub` and may additionally bind `tcp://<configured address>`; topic strings are `qrf.v1.<Type>/<sensor_id>`; subscribers filter by prefix. Raw I/Q between `qrf-tiled` and `qrf-dspd` uses a shared-memory ring, not ZeroMQ. `[D]` (SPEC-008 S-008-2, S-008-7)
- S-009-10. Rates: `Health` 1 Hz; `Occupancy` ≤ 10 Hz; `Bearing` and `LoraFrame` per event; `Spectrum` ≤ 5 Hz at ≤ 4096 bins; `Scatter` ≤ 30 Hz at ≤ 4096 points; a plug-in exceeding its declared rate is throttled by the supervisor and reports it in `Health`. `[D]`

Power and data budgets:

- S-009-11. USB plug-ins shall draw ≤ 1.6 A total from the Pi 5 (with a 5 A supply) or carry their own supply; a KrakenSDR-class device (2.2 A) must be self-powered; USB 3.0 data capacity (4 Gbit/s per port) is not a constraint for any sensor in scope. `[D]` (analysis T20, T14)

Conformance:

- S-009-12. Every plug-in passes the conformance test before it is enabled: publishes `Health` at 1 Hz for 10 min; stays within declared rates and sizes; survives 10 unplug/replug cycles without a crash, a leaked file descriptor, or a stale device handle; reports `device_present = false` within 2 s of removal. `[D]`
- S-009-13. The parallel-feed requirement is met when the tile (5 GHz) and a 915 MHz coherent receiver publish concurrently and the overlay draws both layers aligned to ≤ 100 ms in time and ≤ 2° in azimuth after transforms against one surveyed test emitter radiating at both 915 MHz (LoRa node) and 5.8 GHz (second tile or a CW source). `[D]` (user requirement 2026-10-08; G05 criterion 4)

## Interfaces we depend on

- `nusb` hotplug events; the platform device manager's rules (SPEC-010).
- SPEC-011 DAQ output (U-011-1) for class (d).
- `meshtasticd` TCP 4403 for class (e).

## Known unknowns

| Id | Unknown | Closing gate |
| --- | --- | --- |
| U-009-1 | KrakenSDR DAQ output format (= U-011-1) | backlog R-12 |
| U-009-2 | Which device manager copal runs on the Pi 5 (mdev, udev, or eudev) and how to express per-device group ownership there | backlog P-02 |
| U-009-3 | Availability of a PPS-capable GNSS receiver in the field kit | backlog H-08 |

## Revision history

| Rev | Date | Change |
| --- | --- | --- |
| 0 | 2026-10-08 | Draft |
| 1 | 2026-10-08 | S-009-4: `power_ref` in `Header` (the flag S-009-5 requires) and the `seq` rule; S-009-5: `throttled` in `Health` (the count S-009-10 requires). Both found while writing the wire schema `crates/qrf-bus/proto/qrf/v1/qrf.proto` (backlog R-02) |
