#!/usr/bin/env python3
"""Stage-by-stage transmitter vectors from the LoRa oracle, for qrf-lora's conformance tests (backlog R-08).

Runs the oracle's (gr-lora_sdr, GPL-3.0, a separate process; see docs/lora-corpus.md) transmitter
chain on chosen payloads and records what every stage emits: whitened nibbles, header nibbles, CRC
nibbles, Hamming codewords, interleaved symbols, Gray-demapped symbols, and the first samples of the
modulated frame. The output is the oracle's observable behaviour, not its code; qrf-lora's encoder
must produce the same symbols for the same bytes, and its decoder must invert them.

    python3 -I scripts/oracle-vectors.py [--out resources/vectors/lora-tx-vectors.json] [--os 4]
"""
import argparse
import glob
import json
import os
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
PREFIX = ROOT / "resources" / "oracle"
SYNC_WORD = 0x2B
PREAMBLE_LEN = 16
PRESETS = {
    "SHORT_TURBO":   (500_000, 7, 1),
    "SHORT_FAST":    (250_000, 7, 1),
    "MEDIUM_FAST":   (250_000, 9, 1),
    "LONG_TURBO":    (500_000, 11, 4),
    "LONG_FAST":     (250_000, 11, 1),
    "LONG_MODERATE": (125_000, 11, 4),
    "LONG_SLOW":     (125_000, 12, 4),
}

def load_oracle():
    libdir = PREFIX / "lib"
    sites = glob.glob(str(PREFIX / "lib" / "python3*" / "site-packages"))
    if not libdir.is_dir() or not sites:
        sys.exit(f"oracle not built under {PREFIX}: run `make oracle`")
    if str(libdir) not in os.environ.get("LD_LIBRARY_PATH", "").split(":"):
        env = dict(os.environ)
        env["LD_LIBRARY_PATH"] = str(libdir) + (":" + env["LD_LIBRARY_PATH"] if env.get("LD_LIBRARY_PATH") else "")
        os.execve(sys.executable, [sys.executable, "-I", os.path.abspath(__file__), *sys.argv[1:]], env)
    sys.path.insert(0, sites[0])
    os.environ.setdefault("GR_CONF_LOG_LOG_LEVEL", "warn")
    from gnuradio import gr, blocks, lora_sdr  # noqa: E402
    import pmt  # noqa: E402
    return gr, blocks, lora_sdr, pmt

def sink_for(gr, blocks, block):
    size = block.output_signature().sizeof_stream_item(0)
    if size == 1:
        return blocks.vector_sink_b(), size
    if size == 2:
        return blocks.vector_sink_s(), size
    if size == 4:
        return blocks.vector_sink_i(), size
    if size == 8:
        return blocks.vector_sink_c(), size
    raise SystemExit(f"unexpected item size {size}")

def run_case(ctx, name, bw, sf, cr, payload, os_factor, n_samples, workdir):
    gr, blocks, lora_sdr, pmt = ctx
    fs = os_factor * bw
    sps = (1 << sf) * os_factor
    hexfile = workdir / f"{name}.txt"
    hexfile.write_text(payload.hex() + ",")
    cf32 = workdir / f"{name}.cf32"
    tb = gr.top_block("tx")
    src = blocks.file_source(gr.sizeof_char, str(hexfile), False)
    stages = [
        ("whitening", lora_sdr.whitening(True, False, ",", "packet_len")),
        ("header", lora_sdr.header(False, True, cr)),
        ("add_crc", lora_sdr.add_crc(True)),
        ("hamming_enc", lora_sdr.hamming_enc(cr, sf)),
        ("interleaver", lora_sdr.interleaver(cr, sf, 2, bw)),
        ("gray_demap", lora_sdr.gray_demap(sf)),
    ]
    mod = lora_sdr.modulate(sf, fs, bw, [SYNC_WORD], sps, PREAMBLE_LEN)
    fsink = blocks.file_sink(gr.sizeof_gr_complex, str(cf32), False)
    tags = blocks.tag_debug(gr.sizeof_gr_complex, "frames", "frame_len")
    tags.set_display(False)
    tags.set_save_all(True)
    sinks = []
    prev = src
    for sname, blk in stages:
        tb.connect(prev, blk)
        vs, size = sink_for(gr, blocks, blk)
        tb.connect(blk, vs)
        sinks.append((sname, vs, size))
        prev = blk
    tb.connect(prev, mod)
    tb.connect(mod, fsink)
    tb.connect(mod, tags)
    tb.run()
    fsink.close()
    import numpy as np
    x = np.fromfile(cf32, dtype=np.complex64)
    frames = [(int(t.offset), int(pmt.to_long(t.value))) for t in tags.current_tags()]
    out = {"bw_hz": bw, "sf": sf, "cr": cr, "ldro": (1 << sf) * 1e3 / bw > 16.0, "payload_hex": payload.hex(),
           "oversampling": os_factor, "frame_len_tag": frames[0][1] if frames else None, "frame_offset": frames[0][0] if frames else None,
           "total_samples": int(len(x))}
    for sname, vs, size in sinks:
        data = [int(v) for v in vs.data()]
        out[sname] = data
    off = frames[0][0] if frames else 0
    head = x[off:off + n_samples]
    out["iq_head"] = [[float(v.real), float(v.imag)] for v in head]
    # the symbol boundary samples: first sample of each symbol through the frame (every sps samples)
    nsym = (frames[0][1] - sps) // sps if frames else 0   # padding of one symbol excluded
    out["n_symbols_incl_quarter"] = ((frames[0][1] - sps) / sps) if frames else None
    return out

def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--out", type=pathlib.Path, default=ROOT / "resources" / "vectors" / "lora-tx-vectors.json")
    ap.add_argument("--os", dest="os_factor", type=int, default=4)
    ap.add_argument("--samples", type=int, default=64, help="modulated samples kept per case")
    a = ap.parse_args()
    ctx = load_oracle()
    gr = ctx[0]
    work = ROOT / "resources" / "tmp" / "oracle-vectors"
    work.mkdir(parents=True, exist_ok=True)
    cases = []
    zeros8 = bytes(8)
    zeros32 = bytes(32)
    short = bytes.fromhex("0000f9aa7872c77b")                    # 8 bytes: corpus frame style (index first)
    long_ = bytes(range(0, 32))                                   # 32 bytes
    for preset, (bw, sf, cr) in PRESETS.items():
        cases.append((f"{preset}_zeros8", bw, sf, cr, zeros8))
        cases.append((f"{preset}_short8", bw, sf, cr, short))
        cases.append((f"{preset}_long32", bw, sf, cr, long_))
    cases.append(("SHORT_TURBO_zeros32", 500_000, 7, 1, zeros32))
    # every CR at SF7 for the Hamming/interleaver variants, and SF8/SF10 for completeness
    for cr in (1, 2, 3, 4):
        cases.append((f"SF7_cr{cr}_short8", 250_000, 7, cr, short))
    cases.append(("SF8_cr1_short8", 250_000, 8, 1, short))
    cases.append(("SF10_cr4_short8", 250_000, 10, 4, short))
    cases.append(("SF12_cr1_long32", 125_000, 12, 1, long_))
    result = {"generator": "scripts/oracle-vectors.py", "oracle": {"gnuradio": gr.version()}, "sync_word": SYNC_WORD, "preamble_len": PREAMBLE_LEN, "cases": {}}
    vt = PREFIX / "VERSION.txt"
    if vt.exists():
        for ln in vt.read_text().splitlines():
            k, _, val = ln.partition(" ")
            if k == "gr-lora_sdr":
                result["oracle"]["gr-lora_sdr"] = val
    for name, bw, sf, cr, payload in cases:
        result["cases"][name] = run_case(ctx, name, bw, sf, cr, payload, a.os_factor, a.samples, work)
        c = result["cases"][name]
        print(f"{name:28} sf{sf} cr{cr} ldro={c['ldro']!s:5} whitened {len(c['whitening'])} nib, hdr {len(c['header'])}, crc {len(c['add_crc'])}, cw {len(c['hamming_enc'])}, il {len(c['interleaver'])}, sym {len(c['gray_demap'])}, frame {c['n_symbols_incl_quarter']} sym", flush=True)
    a.out.parent.mkdir(parents=True, exist_ok=True)
    a.out.write_text(json.dumps(result, indent=1) + "\n")
    print(f"wrote {a.out}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
