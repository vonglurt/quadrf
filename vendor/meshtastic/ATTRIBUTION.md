# Meshtastic

| Field | Value |
| --- | --- |
| Copyright holder | Meshtastic LLC and contributors |
| Licences | Firmware repository: GPL-3.0 (`LICENSE`). Protobuf definitions (`protobufs/`): GPL-3.0. The `meshtastic` Rust client crate: GPL-3.0. Documentation at meshtastic.org: per the site's notice (not retrieved as a licence file; treated as all rights reserved for copying purposes). "Meshtastic" is a registered trademark of Meshtastic LLC. |
| URLs | https://github.com/meshtastic/firmware ; https://meshtastic.org/docs/ |
| Revision read | firmware commit `364a111f4a5601708f3b9d60527aca02b6ac73ef` (2026-10-08); documentation pages retrieved 2026-10-08 |
| Files read | `src/mesh/RadioInterface.{h,cpp}`, `src/platform/portduino/SimRadio.{h,cpp}`, `bin/config-dist.yaml`, `bin/config.d/*.yaml` (pin assignments), documentation pages listed in the manifest |

## What we use

Radio-layer behaviour (SPEC-004): backend abstraction, power clamping,
licensed mode, slot plan, Linux daemon configuration, and the per-HAT pin
tables. Restated in SPEC-004 and `vendor/summary/meshtastic.md`.

## What we do not do

Link the GPL-3.0 `meshtastic` crate or the protobufs into MIT binaries. Where
our software must talk to a Meshtastic node it does so over the daemon's TCP
API from a separate process, or by re-implementing the public wire format from
the published documentation.
