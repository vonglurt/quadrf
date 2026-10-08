#!/bin/sh
# Re-create the gitignored resources/ tree. Idempotent. See docs/resources-manifest.md.
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
R="$ROOT/resources"
mkdir -p "$R/scalerf" "$R/regulatory" "$R/datasheets" "$R/lora" "$R/repos"

get() { # url dest
  [ -s "$2" ] && return 0
  curl -sSL --max-time 120 -A "Mozilla/5.0" -o "$2" "$1" || echo "FAILED: $1"
}

# ScaleRF QuadRF documentation and schematics
for f in QuadRF_schematics.pdf detailed.svg chained.svg jtag_cable_pinout.drawio.svg; do
  get "https://scalerf.com/docs/$f" "$R/scalerf/$f"
done
get https://scalerf.com/docs/            "$R/scalerf/docs.html"
get https://scalerf.com/updates/         "$R/scalerf/updates.html"
get https://scalerf.com/cals/antennas.html "$R/scalerf/cal_antennas.html"
get https://scalerf.com/cals/txqec.html  "$R/scalerf/cal_txqec.html"
[ -s "$R/scalerf/schematics.txt" ] || pdftotext -layout "$R/scalerf/QuadRF_schematics.pdf" "$R/scalerf/schematics.txt" || true

# 47 CFR text (LII mirror of e-CFR)
for s in 15.23 15.203 15.205 15.209 15.247 15.249 97.3 97.113 97.119 97.303 97.311 97.313; do
  get "https://www.law.cornell.edu/cfr/text/47/$s" "$R/regulatory/cfr-47-$s.html"
done

# LoRa / Meshtastic references
get https://meshtastic.org/docs/overview/radio-settings/            "$R/lora/meshtastic-radio-settings.html"
get https://meshtastic.org/docs/linux/hardware/boards/raspberry-pi/ "$R/lora/meshtastic-linux-rpi.html"
get https://meshtastic.org/docs/configuration/radio/lora/           "$R/lora/meshtastic-lora-config.html"
get https://meshtastic.org/docs/configuration/region-by-country/    "$R/lora/meshtastic-region-us.html"
get https://raw.githubusercontent.com/tapparelj/gr-lora_sdr/master/README.md "$R/lora/gr-lora_sdr-README.md"
get https://arxiv.org/pdf/2002.08208 "$R/lora/lora-phy-paper-tapparel.pdf"

# Datasheets: analog.com and semtech.com block scripted download; fetch manually
# into resources/datasheets/ : MAX2850.pdf MAX2851.pdf MAX2871.pdf SX1262.pdf
# SE5004L.pdf SKY65404-31.pdf

# Reference repositories (shallow)
clone() { [ -d "$2/.git" ] || git clone -q --depth 1 "$1" "$2"; }
clone https://github.com/open-space-sdr/main.git      "$R/repos/quadrf-open-space-sdr"
clone https://github.com/radioroy/quadrf-mesh.git     "$R/repos/quadrf-mesh"
clone https://github.com/tapparelj/gr-lora_sdr.git    "$R/repos/gr-lora_sdr"
clone https://github.com/meshtastic/firmware.git      "$R/repos/meshtastic-firmware"
echo "resources/ populated under $R"
