# radioroy — quadrf-mesh

| Field | Value |
| --- | --- |
| Copyright holder | radioroy (GitHub) and contributors; the project states it is not an official Scale RF or Meshtastic product |
| Licence | GPL-3.0 (`LICENSE`); the PHY adapts bit-level chains from `gr-lora_sdr` and `gr-lora` (both GPL-3.0) |
| URL | https://github.com/radioroy/quadrf-mesh |
| Revision read | commit `ad3ed3109c7d6cfd7db673ffde4ba6e02639fa28` ("debian: 0.1.9", 2026-10-08) |
| Files read | `README.md`, `docs/air-ipc-v1.md`, `docs/mesh-monitor.md`, `debian/quadrf-lora-phy.default`, `include/phy/config.hpp`, `include/quadrf/air_ipc.hpp` (header only), `src/lora/wander.cpp` (method only), source tree listing |

## What we use

The Air-IPC v1 socket protocol (a wire format, which we re-implement
independently from its documentation), the measured LO-wander results, the
preset table, and the configuration keys. Restated in SPEC-006 and
`vendor/summary/radioroy-quadrf-mesh.md`.

## What we do not do

Copy or link any of its code. Our Rust components speak Air-IPC v1 to its
processes over `/run/quadrf/phy.sock`; either side can be replaced.
