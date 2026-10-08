#!/bin/sh
# Re-create the gitignored resources/ tree. Idempotent. See docs/resources-manifest.md.
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
R="$ROOT/resources"
mkdir -p "$R/scalerf" "$R/regulatory" "$R/datasheets" "$R/lora" "$R/repos" "$R/krakensdr" "$R/misc"
UA="Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0 Safari/537.36"

get() { # url dest
  [ -s "$2" ] && return 0
  curl -sSL --max-time 120 -A "$UA" -o "$2" "$1" || echo "FAILED: $1"
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

# 47 CFR text (LII mirror of e-CFR). ecfr.gov redirects scripted requests.
for s in 1.1310 15.23 15.203 15.205 15.209 15.247 15.249 97.3 97.13 97.113 97.119 97.303 97.311 97.313; do
  get "https://www.law.cornell.edu/cfr/text/47/$s" "$R/regulatory/cfr-47-$s.html"
done

# LoRa / Meshtastic references
get https://meshtastic.org/docs/overview/radio-settings/            "$R/lora/meshtastic-radio-settings.html"
get https://meshtastic.org/docs/linux/hardware/boards/raspberry-pi/ "$R/lora/meshtastic-linux-rpi.html"
get https://meshtastic.org/docs/configuration/radio/lora/           "$R/lora/meshtastic-lora-config.html"
get https://meshtastic.org/docs/configuration/region-by-country/    "$R/lora/meshtastic-region-us.html"
get https://raw.githubusercontent.com/tapparelj/gr-lora_sdr/master/README.md "$R/lora/gr-lora_sdr-README.md"
get https://arxiv.org/pdf/2002.08208 "$R/lora/lora-phy-paper-tapparel.pdf"
get https://www.semtech.com/products/wireless-rf/lora-connect/sx1262 "$R/lora/semtech-sx1262-product.html"
get https://www.semtech.com/products/wireless-rf/lora-connect/lr1121 "$R/lora/semtech-lr1121-product.html"
get https://www.semtech.com/products/wireless-rf/lora-connect/sx1280 "$R/lora/semtech-sx1280-product.html"
# fccid.io serves a JavaScript challenge to scripted clients; read FCC grants at https://www.fcc.gov/oet/ea/fccid by hand

# KrakenSDR (C12 comparator): GitHub wiki pages are served raw from the wiki repo
get https://raw.githubusercontent.com/wiki/krakenrf/krakensdr_docs/Home.md                   "$R/krakensdr/wiki-Home.md"
get "https://raw.githubusercontent.com/wiki/krakenrf/krakensdr_docs/04.-Antenna-Array-Setup.md" "$R/krakensdr/wiki-04-Antenna-Array-Setup.md"
get https://raw.githubusercontent.com/krakenrf/heimdall_daq_fw/main/README.md                  "$R/krakensdr/heimdall_daq_fw-README.md"
get https://raw.githubusercontent.com/krakenrf/krakensdr_doa/main/README.md                    "$R/krakensdr/krakensdr_doa-README.md"

# Raspberry Pi RP1 southbridge datasheet (MIPI, PCIe, USB figures)
get "https://pip-assets.raspberrypi.com/categories/892-raspberry-pi-5/documents/RP-008370-DS-1-rp1-peripherals.pdf" "$R/datasheets/RP1-peripherals.pdf"
[ -s "$R/datasheets/RP1-peripherals.txt" ] || pdftotext -layout "$R/datasheets/RP1-peripherals.pdf" "$R/datasheets/RP1-peripherals.txt" || true

# Datasheets that block scripted download: analog.com (MAX2850, MAX2851, MAX2871),
# semtech.com (SX1262), skyworksinc.com (SE5004L, SKY65404-31). Fetch them in a
# browser on the host, drop them in ~/Downloads/SharedVM/quadrf/, then run
#   scripts/import-shared.sh

# Reference repositories (shallow)
clone() { [ -d "$2/.git" ] || git clone -q --depth 1 "$1" "$2"; }
clone https://github.com/open-space-sdr/main.git      "$R/repos/quadrf-open-space-sdr"
clone https://github.com/radioroy/quadrf-mesh.git     "$R/repos/quadrf-mesh"
clone https://github.com/tapparelj/gr-lora_sdr.git    "$R/repos/gr-lora_sdr"
clone https://github.com/meshtastic/firmware.git      "$R/repos/meshtastic-firmware"
echo "resources/ populated under $R"
