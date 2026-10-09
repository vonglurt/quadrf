#!/bin/sh
# Builds the LoRa test oracle (backlog V-08): gr-lora_sdr (GPL-3.0, Tapparel et
# al.) from the clone in resources/repos/gr-lora_sdr against the system GNU
# Radio 3.10, installed into the gitignored prefix resources/oracle/. Nothing
# from it is linked into any qrf binary; it runs as a separate process
# (docs/00-process.md §6). Re-runnable; no root needed.
#
#   sh scripts/build-oracle.sh            # configure, build, install
#   sh scripts/build-oracle.sh --clean    # remove the build tree and prefix first
#
# Requires (Alpine): cmake, gnuradio-dev, py3-pybind11-dev, boost-dev, gmp-dev,
# spdlog-dev, fmt-dev, python3-dev, py3-numpy (all present on the VM 2026-10-08).
set -eu
ROOT=$(cd "$(dirname "$0")/.." && pwd)
SRC="$ROOT/resources/repos/gr-lora_sdr"
BUILD="$ROOT/resources/build/gr-lora_sdr"
PREFIX="$ROOT/resources/oracle"
EXPECT_REV=862746dd1cf635c9c8a4bfbaa2c3a0ec3a5306c9   # docs/resources-manifest.md row for gr-lora_sdr

if [ "${1:-}" = "--clean" ]; then rm -rf "$BUILD" "$PREFIX"; fi
[ -d "$SRC/.git" ] || { echo "clone missing: run sh scripts/fetch-resources.sh first" >&2; exit 1; }
REV=$(git -C "$SRC" rev-parse HEAD)
[ "$REV" = "$EXPECT_REV" ] || echo "warning: gr-lora_sdr is at $REV, manifest says $EXPECT_REV" >&2

mkdir -p "$BUILD" "$PREFIX"
cmake -S "$SRC" -B "$BUILD" -DCMAKE_INSTALL_PREFIX="$PREFIX" -DCMAKE_BUILD_TYPE=Release -DENABLE_DOXYGEN=OFF -Wno-dev > "$BUILD/configure.log" 2>&1 || { tail -30 "$BUILD/configure.log" >&2; exit 1; }
cmake --build "$BUILD" -j"$(nproc)" > "$BUILD/build.log" 2>&1 || { tail -30 "$BUILD/build.log" >&2; exit 1; }
cmake --install "$BUILD" > "$BUILD/install.log" 2>&1 || { tail -30 "$BUILD/install.log" >&2; exit 1; }

GRV=$(python3 -I -c 'from gnuradio import gr; print(gr.version())' 2>/dev/null || echo unknown)
{
  echo "gr-lora_sdr $REV"
  echo "gnuradio $GRV"
  echo "built $(date -u +%Y-%m-%dT%H:%M:%SZ) on $(uname -m) $(uname -r)"
  echo "prefix $PREFIX"
} > "$PREFIX/VERSION.txt"
SITE=$(find "$PREFIX/lib" -maxdepth 3 -type d -name site-packages | head -1)
echo "oracle installed: $PREFIX (python package under $SITE)"
cat "$PREFIX/VERSION.txt"
