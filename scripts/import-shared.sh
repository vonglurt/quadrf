#!/bin/sh
# Import datasheets dropped by the host into the UTM shared folder.
#
# analog.com, semtech.com and skyworksinc.com reject scripted downloads, so the
# PDFs are fetched in a browser on the Mac and placed in
#   ~/Downloads/SharedVM/quadrf/
# This script copies each PDF into resources/datasheets/ (gitignored), names it
# by the part number found on its first page, extracts text with pdftotext,
# and prints the sha256 and a row for docs/resources-manifest.md.
#
#   scripts/import-shared.sh            # import everything new
#   scripts/import-shared.sh --dry-run  # only report
set -eu
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SRC="${QUADRF_SHARE:-$HOME/Downloads/SharedVM/quadrf}"
DST="$ROOT/resources/datasheets"
DRY=0
[ "${1:-}" = "--dry-run" ] && DRY=1
mkdir -p "$DST"

[ -d "$SRC" ] || { echo "no shared folder at $SRC" >&2; exit 1; }

part_of() { # first-page text -> part number, or empty
    page="$(pdftotext -f 1 -l 1 "$1" - 2>/dev/null)"
    case "$page" in *SX1261/2*) echo SX1261-2; return ;; esac
    printf '%s' "$page" | grep -o -E 'MAX28(50|51|71)|SX126[128]|LR1121|SE5004L|SKY65404-31|RP1 Peripherals|BCM2712|RP2040|RP2350' | head -1 | tr ' ' '-'
}

for f in "$SRC"/*.pdf "$SRC"/*.PDF; do
    [ -f "$f" ] || continue
    case "$(basename "$f")" in ._*) continue ;; esac     # AppleDouble metadata
    part="$(part_of "$f")"
    if [ -z "$part" ]; then
        echo "unrecognised: $f (name it by hand into $DST)" >&2
        continue
    fi
    out="$DST/$part.pdf"
    sum="$(sha256sum "$f" | cut -c1-64)"
    if [ -s "$out" ] && [ "$(sha256sum "$out" | cut -c1-64)" = "$sum" ]; then
        echo "have   $out"
        continue
    fi
    if [ "$DRY" -eq 1 ]; then
        echo "would import $f -> $out ($sum)"
        continue
    fi
    cp "$f" "$out"
    pdftotext -layout "$out" "${out%.pdf}.txt" || true
    pages="$(pdfinfo "$out" 2>/dev/null | awk '/^Pages:/{print $2}')"
    echo "import $out  pages=$pages  sha256=$sum"
    echo "manifest row: | \`$part.pdf\` | manual browser fetch, dropped in the UTM share $(date +%F) | sha256 $sum | pages $pages |"
done
