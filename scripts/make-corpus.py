#!/usr/bin/env python3
"""LoRa I/Q test corpus with a recorded truth and the oracle's own decode result (backlog V-08).

The oracle is gr-lora_sdr (GPL-3.0, Tapparel et al.), built by scripts/build-oracle.sh into
resources/oracle/ and run here as a separate process (docs/00-process.md §6). It is used for
two things only: to synthesise clean Meshtastic-style LoRa frames (SPEC-003 S-003-7: sync word
0x2B, 16-symbol preamble, explicit header, CRC, LDRO by the 16 ms rule) and to re-decode the
finished corpus so that qrf-lora (backlog R-08) is compared with it on the same frames.

The channel is applied in numpy from a seeded generator, so every byte of the corpus follows
from (seed, cell): per frame a payload (8-32 bytes, the first two the frame index), a carrier
offset within ±2 ppm of 915 MHz, an initial phase and a noise-only gap before it; complex white
Gaussian noise at the cell's SNR defined in the signal bandwidth (the datasheet convention,
S-003-2), i.e. per-sample SNR is lower by 10 log10(oversampling) before filtering; a channel
filter (Kaiser low-pass, 60 dB, passband ±0.5 BW, stopband from ±0.7 BW) on signal plus noise,
as a receiver front end or the qrf channeliser would apply, so that a demodulator without its
own filter sees the stated SNR; 8-bit quantisation to CS8 at 4 × BW with the filtered noise
fixed at NOISE_LSB rms per complex sample. Cells: the seven Meshtastic presets at five SNRs
around the S-003-2 threshold (100 frames each) and the two backlog R-08 cells (1 000 frames).

    python3 -I scripts/make-corpus.py generate [--cells all|r08|sweep|NAME,NAME] [--frames N]
                                               [--frames-r08 N] [--seed N] [--os N] [--no-verify]
    python3 -I scripts/make-corpus.py verify   [--cells ...]     # oracle receiver over existing cells
    python3 -I scripts/make-corpus.py check                       # sha256 of every file vs MANIFEST.json
    python3 -I scripts/make-corpus.py list
    python3 -I scripts/make-corpus.py report                      # Markdown table of every cell with the oracle's result

Layout: resources/corpus/qrf-lora-v1/<cell>/{iq.cs8, truth.json, oracle.json}; MANIFEST.json at the top.
"""
import argparse
import glob
import hashlib
import json
import math
import os
import pathlib
import sys
import tempfile
import time

ROOT = pathlib.Path(__file__).resolve().parent.parent
PREFIX = ROOT / "resources" / "oracle"
OUT_DEFAULT = ROOT / "resources" / "corpus" / "qrf-lora-v1"
TMP_DIR = ROOT / "resources" / "tmp"          # intermediates (hundreds of MB per cell); never /tmp, which is a small tmpfs on the VM

# --- LoRa parameters (SPEC-003) ---------------------------------------------------------------
SYNC_WORD = 0x2B            # S-003-7
PREAMBLE_LEN = 16           # S-003-7 (symbols)
CENTER_HZ = 906_875_000     # Meshtastic US LONG_FAST default slot (the value only labels the oracle's frame_sync)
SNR_MIN = {7: -7.5, 8: -10.0, 9: -12.5, 10: -15.0, 11: -17.5, 12: -20.0}   # S-003-2 (dB, in BW)
# Meshtastic presets (analysis/linkbudget.py PRESETS): bandwidth Hz, SF, coding rate 1..4 = 4/5..4/8
PRESETS = {
    "SHORT_TURBO":   (500_000, 7, 1),
    "SHORT_FAST":    (250_000, 7, 1),
    "MEDIUM_FAST":   (250_000, 9, 1),
    "LONG_TURBO":    (500_000, 11, 4),
    "LONG_FAST":     (250_000, 11, 1),
    "LONG_MODERATE": (125_000, 11, 4),
    "LONG_SLOW":     (125_000, 12, 4),
}
SNR_OFFSETS_DB = (-5.0, -2.5, 0.0, 2.5, 5.0)              # sweep cells, relative to SNR_MIN[sf]
R08_CELLS = {"SHORT_TURBO": -5.0, "LONG_FAST": -15.0}      # backlog R-08 check cells, absolute dB in BW
CFO_PPM = 2.0               # carrier offset drawn uniformly in ±2 ppm of 915 MHz per frame (TCXO-class nodes; the oracle's reliable range, docs/lora-corpus.md)
PAYLOAD_LEN = (8, 32)       # bytes, inclusive, uniform
GAP_SYMBOLS = (0.5, 2.0)    # noise-only gap before each frame, in symbol durations, uniform
NOISE_LSB = 20.0            # noise rms per complex sample after quantisation (LSB)
# Pacing of the oracle receiver (see verify_cell): one LoRa symbol per chunk, SYMBOL_INTERVAL_MS of
# wall-clock time between chunks, whatever the spreading factor. The receiver's frame_sync must not
# receive the first payload symbol before the header's message has come back through five blocks;
# a chunk bounded to one symbol and a 2 ms gap give that path its time. At SF7 this is ~0.13x real
# time, at SF12 ~16x; --throttle N instead paces at N x real time with GNU Radio's default chunks.
SYMBOL_INTERVAL_MS = 2.0
THROTTLE_X = 0.0

def cell_table():
    """Every cell in a stable order: (name, preset, snr_db_inband, n_frames_kind)."""
    cells = []
    for preset in PRESETS:
        sf = PRESETS[preset][1]
        for off in SNR_OFFSETS_DB:
            cells.append((f"{preset}_snr{SNR_MIN[sf] + off:+.1f}", preset, SNR_MIN[sf] + off, "sweep"))
    for preset, snr in R08_CELLS.items():
        cells.append((f"r08_{preset}_snr{snr:+.1f}", preset, snr, "r08"))
    return cells

def select_cells(spec, out_dir=None):
    table = cell_table()
    if spec in ("all", None):
        return table
    if spec == "r08":
        return [c for c in table if c[3] == "r08"]
    if spec == "sweep":
        return [c for c in table if c[3] == "sweep"]
    wanted = set(spec.split(","))
    chosen = [c for c in table if c[0] in wanted]
    missing = wanted - {c[0] for c in chosen}
    if missing and out_dir is not None:
        # A cell that is not in the table but exists under --out with a truth.json (for example one made by
        # crates/qrf-lora/examples/make_cell.rs) can still be verified; its parameters come from the sidecar.
        for name in sorted(missing):
            tr = out_dir / name / "truth.json"
            if tr.exists():
                t = json.loads(tr.read_text())
                chosen.append((name, t["preset"], float(t["snr_db_in_bandwidth"]), "external"))
                missing.discard(name)
    if missing:
        sys.exit(f"unknown cells: {sorted(missing)}; see `list`")
    return chosen

# --- the oracle ---------------------------------------------------------------------------------
def load_oracle():
    """Imports GNU Radio and the privately installed gr-lora_sdr. The binding's shared library lives
    in the private prefix, which the loader only searches through LD_LIBRARY_PATH, and that must be
    set before the interpreter starts (musl ignores a library pre-loaded under another path), so the
    script re-executes itself once with the variable set; no caller needs to know."""
    libdir = PREFIX / "lib"
    sites = glob.glob(str(PREFIX / "lib" / "python3*" / "site-packages"))
    if not libdir.is_dir() or not sites:
        sys.exit(f"oracle not built under {PREFIX}: run `make oracle` (scripts/build-oracle.sh)")
    if str(libdir) not in os.environ.get("LD_LIBRARY_PATH", "").split(":"):
        env = dict(os.environ)
        env["LD_LIBRARY_PATH"] = str(libdir) + (":" + env["LD_LIBRARY_PATH"] if env.get("LD_LIBRARY_PATH") else "")
        os.execve(sys.executable, [sys.executable, "-I", os.path.abspath(__file__), *sys.argv[1:]], env)
    sys.path.insert(0, sites[0])
    os.environ.setdefault("GR_CONF_LOG_LOG_LEVEL", "warn")
    from gnuradio import gr, blocks, lora_sdr  # noqa: E402
    import pmt  # noqa: E402
    return gr, blocks, lora_sdr, pmt

def versions(gr):
    v = {"gnuradio": gr.version()}
    vt = PREFIX / "VERSION.txt"
    if vt.exists():
        for ln in vt.read_text().splitlines():
            k, _, val = ln.partition(" ")
            if k in ("gr-lora_sdr", "gnuradio"):
                v[k] = val
    return v

def ldro_on(sf, bw):
    return (1 << sf) * 1e3 / bw > 16.0   # gr-lora_sdr AUTO rule (LDRO_MAX_DURATION_MS = 16); Meshtastic's is the same threshold

def channel_filter(os_factor):
    """Kaiser-window low-pass for the generated stream: -6 dB at 0.6 BW, 60 dB stopband from 0.7 BW,
    unity DC gain. Returns (taps, equivalent noise bandwidth in units of BW)."""
    import numpy as np
    fs_bw = float(os_factor)                 # sample rate in units of BW
    delta = 0.2 / fs_bw                      # transition 0.5 BW -> 0.7 BW, normalised to fs
    n = int(math.ceil(52.0 / (2.285 * 2.0 * math.pi * delta))) + 1
    n |= 1                                   # odd length, integer group delay
    m = (n - 1) // 2
    k = np.arange(n) - m
    fc = 0.6 / fs_bw                         # cutoff in cycles per sample
    h = 2.0 * fc * np.sinc(2.0 * fc * k) * np.kaiser(n, 5.65)
    h /= h.sum()
    enbw_bw = fs_bw * float(np.sum(h * h))   # fs * sum(h^2) / (sum h)^2, in units of BW
    return h, enbw_bw

def synthesise_frames(gr, blocks, lora_sdr, pmt, preset, payloads, os_factor, workdir):
    """Clean frames from the oracle's transmitter: returns a complex64 memmap and [(offset, length)]."""
    import numpy as np
    bw, sf, cr = PRESETS[preset]
    fs = os_factor * bw
    sps = (1 << sf) * os_factor
    padd = sps                      # one symbol of zeros after each frame, stripped below; keeps the last frame complete
    hexfile = workdir / "payloads.txt"
    hexfile.write_text("".join(p.hex() + "," for p in payloads))
    cf32 = workdir / "tx.cf32"
    tb = gr.top_block("tx")
    src = blocks.file_source(gr.sizeof_char, str(hexfile), False)
    wh = lora_sdr.whitening(True, False, ",", "packet_len")
    hdr = lora_sdr.header(False, True, cr)
    crc = lora_sdr.add_crc(True)
    enc = lora_sdr.hamming_enc(cr, sf)
    il = lora_sdr.interleaver(cr, sf, 2, bw)
    gd = lora_sdr.gray_demap(sf)
    mod = lora_sdr.modulate(sf, fs, bw, [SYNC_WORD], padd, PREAMBLE_LEN)
    sink = blocks.file_sink(gr.sizeof_gr_complex, str(cf32), False)
    tags = blocks.tag_debug(gr.sizeof_gr_complex, "frames", "frame_len")
    tags.set_display(False)
    tags.set_save_all(True)
    for a, b in ((src, wh), (wh, hdr), (hdr, crc), (crc, enc), (enc, il), (il, gd), (gd, mod), (mod, sink), (mod, tags)):
        tb.connect(a, b)
    tb.run()
    sink.close()
    found = [(int(t.offset), int(pmt.to_long(t.value))) for t in tags.current_tags()]
    if len(found) != len(payloads):
        sys.exit(f"{preset}: oracle transmitter produced {len(found)} frames for {len(payloads)} payloads")
    x = np.memmap(cf32, dtype=np.complex64, mode="r")
    frames = [(off, length - padd) for off, length in found]
    last_off, last_len = frames[-1]
    if last_off + last_len > len(x):
        sys.exit(f"{preset}: last frame truncated ({last_off + last_len} > {len(x)} samples)")
    return x, frames, fs, sps

def generate_cell(ctx, name, preset, snr_db, n_frames, seed, os_factor, out_dir, cell_index, use_filter=True):
    import numpy as np
    gr, blocks, lora_sdr, pmt = ctx
    bw, sf, cr = PRESETS[preset]
    rng = np.random.default_rng([seed, cell_index])
    lengths = rng.integers(PAYLOAD_LEN[0], PAYLOAD_LEN[1] + 1, n_frames)
    payloads = [bytes([i >> 8, i & 0xFF]) + rng.integers(0, 256, int(l) - 2, dtype=np.uint8).tobytes() for i, l in enumerate(lengths)]
    cfo_hz = rng.uniform(-CFO_PPM * 1e-6 * 915e6, CFO_PPM * 1e-6 * 915e6, n_frames)
    phase = rng.uniform(0.0, 2.0 * math.pi, n_frames)
    gaps_sym = rng.uniform(GAP_SYMBOLS[0], GAP_SYMBOLS[1], n_frames)
    TMP_DIR.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="qrf-corpus-", dir=TMP_DIR) as tmp:
        x, frames, fs, sps = synthesise_frames(gr, blocks, lora_sdr, pmt, preset, payloads, os_factor, pathlib.Path(tmp))
        p_sig = float(np.mean(np.abs(x[frames[0][0]:frames[0][0] + frames[0][1]].astype(np.complex64)) ** 2))
        snr_lin = 10 ** (snr_db / 10)
        # White noise before the filter: PSD N0 such that the noise power in BW is P_sig / SNR, i.e.
        # sigma_pre^2 = N0 * fs = P_sig * os / SNR per complex sample (unit signal scale).
        sigma_pre = math.sqrt(p_sig * os_factor / snr_lin)
        snr_sample_db = snr_db - 10 * math.log10(os_factor)
        h, enbw_bw = (np.ones(1), float(os_factor)) if not use_filter else channel_filter(os_factor)
        ntaps = len(h)
        delay = (ntaps - 1) // 2
        noise_out = sigma_pre * math.sqrt(float(np.sum(h * h)))      # filtered noise rms, unit scale
        amp = NOISE_LSB / noise_out                                  # LSB per unit of signal amplitude
        snr_sample_after_db = 10 * math.log10(p_sig / noise_out ** 2)
        gaps = [int(round(g * sps)) for g in gaps_sym]
        tail = sps + delay                                           # flushes the filter
        total = sum(gaps) + sum(l for _, l in frames) + tail
        cell_dir = out_dir / name
        cell_dir.mkdir(parents=True, exist_ok=True)
        iq_path = cell_dir / "iq.cs8"
        clipped = 0
        pos = 0
        truth_frames = []
        sig_r = sigma_pre / math.sqrt(2.0)
        carry = np.zeros(ntaps - 1, dtype=np.complex128)             # overlap-save state
        with open(iq_path, "wb") as f:
            def emit(block):
                """Filters one region (continuing from the previous one), scales, quantises, writes."""
                nonlocal clipped, carry
                if ntaps > 1:
                    xcat = np.concatenate((carry, block))
                    y = np.convolve(xcat, h, mode="valid")
                    carry = xcat[-(ntaps - 1):]
                else:
                    y = block
                q = np.empty((len(y), 2))
                q[:, 0] = np.rint(y.real * amp)
                q[:, 1] = np.rint(y.imag * amp)
                clipped += int(np.count_nonzero((q > 127) | (q < -128)))
                np.clip(q, -128, 127, out=q)
                f.write(q.astype(np.int8).tobytes())
            def noise(n):
                z = rng.standard_normal((n, 2)) * sig_r
                return z[:, 0] + 1j * z[:, 1]
            for i, ((off, length), gap) in enumerate(zip(frames, gaps)):
                emit(noise(gap))
                pos += gap
                n = np.arange(length, dtype=np.float64)
                rot = np.exp(1j * (2.0 * math.pi * cfo_hz[i] / fs * n + phase[i]))
                emit(x[off:off + length].astype(np.complex128) * rot + noise(length))
                truth_frames.append({"index": i, "payload_hex": payloads[i].hex(), "start": pos + delay, "length": int(length), "cfo_hz": float(cfo_hz[i]), "phase_rad": float(phase[i]), "gap_before": gap})
                pos += length
            emit(noise(tail))
            pos += tail
        del x
    assert pos == total
    truth = {
        "cell": name, "preset": preset, "bw_hz": bw, "sf": sf, "cr": cr, "coding_rate": f"4/{cr + 4}", "ldro": ldro_on(sf, bw),
        "sync_word": SYNC_WORD, "preamble_symbols": PREAMBLE_LEN, "explicit_header": True, "crc": True,
        "sample_rate_hz": fs, "oversampling": os_factor, "format": "CS8 interleaved I,Q, little endian int8",
        "snr_db_in_bandwidth": snr_db, "snr_db_per_sample_before_filter": snr_sample_db, "snr_db_per_sample_after_filter": snr_sample_after_db, "snr_min_db_s_003_2": SNR_MIN[sf],
        "channel_filter": {"type": "kaiser low-pass on signal plus noise, unity DC gain", "taps": ntaps, "group_delay_samples": delay, "cutoff_6db_bw": 0.6, "stopband_from_bw": 0.7, "stopband_db": 60, "enbw_bw": enbw_bw} if use_filter else None,
        "noise_rms_lsb_per_complex_sample": NOISE_LSB, "lsb_per_unit_amplitude": amp, "clean_signal_power": p_sig,
        "clipped_samples": clipped, "total_samples": total, "n_frames": n_frames,
        "cfo_ppm_range": CFO_PPM, "payload_len_range": list(PAYLOAD_LEN), "gap_symbols_range": list(GAP_SYMBOLS),
        "seed": seed, "cell_index": cell_index, "generator": "scripts/make-corpus.py", "oracle": versions(gr),
        "frames": truth_frames,
    }
    (cell_dir / "truth.json").write_text(json.dumps(truth, indent=1) + "\n")
    return truth

def verify_cell(ctx, out_dir, name):
    """Runs the oracle receiver over iq.cs8 and compares with truth.json; writes oracle.json.

    The receiver is timing-sensitive by construction (its frame_sync learns the frame length from
    header_decoder through an asynchronous message), so it is paced at THROTTLE_X real time and run
    twice; identical results are accepted, otherwise a third run decides and the record says so.
    Verification therefore belongs on an otherwise idle machine."""
    gr, blocks, lora_sdr, pmt = ctx
    cell_dir = out_dir / name
    truth = json.loads((cell_dir / "truth.json").read_text())
    bw, sf, cr = truth["bw_hz"], truth["sf"], truth["cr"]
    os_factor = truth["oversampling"]
    fs = truth["sample_rate_hz"]
    sps = (1 << sf) * os_factor
    if THROTTLE_X > 0:
        pace_sps, chunk = THROTTLE_X * fs, 0
    else:
        pace_sps, chunk = sps * 1000.0 / SYMBOL_INTERVAL_MS, sps
    throttle_x = pace_sps / fs
    want = {f["index"]: bytes.fromhex(f["payload_hex"]) for f in truth["frames"]}
    n = truth["n_frames"]

    def run_once():
        TMP_DIR.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="qrf-oracle-", dir=TMP_DIR) as tmp:
            payload_bin = pathlib.Path(tmp) / "payload.bin"
            tb = gr.top_block("rx")
            # CS8 straight from the corpus file: interleaved int8 pairs to complex, scaled back to unit signal amplitude.
            raw = blocks.file_source(gr.sizeof_char, str(cell_dir / "iq.cs8"), False)
            src = blocks.interleaved_char_to_complex(False, 1.0 / truth["lsb_per_unit_amplitude"])
            src.set_min_output_buffer(int((1 << sf) * os_factor * 2))
            # Upstream's own simulation paces the stream with a throttle; without it frame_sync outruns
            # the header message (frame_sync_impl.cc, m_received_head) and drops most payloads.
            throttle = blocks.throttle(gr.sizeof_gr_complex, float(pace_sps), True, int(chunk))
            throttle.set_min_output_buffer(int((1 << sf) * os_factor * 2))
            if chunk:
                throttle.set_max_noutput_items(int(chunk))
            fsync = lora_sdr.frame_sync(CENTER_HZ, bw, sf, False, [SYNC_WORD], os_factor, PREAMBLE_LEN)
            demod = lora_sdr.fft_demod(False, True)
            gmap = lora_sdr.gray_mapping(False)
            deil = lora_sdr.deinterleaver(False)
            hdec = lora_sdr.hamming_dec(False)
            hdr = lora_sdr.header_decoder(False, cr, 255, True, 2, False)
            dewh = lora_sdr.dewhitening()
            crc = lora_sdr.crc_verif(0, True)
            sink = blocks.file_sink(gr.sizeof_char, str(payload_bin), False)
            flags = blocks.null_sink(gr.sizeof_char)
            tags = blocks.tag_debug(gr.sizeof_char, "rx", "frame_info")
            tags.set_display(False)
            tags.set_save_all(True)
            for a, b in ((raw, src), (src, throttle), (throttle, fsync), (fsync, demod), (demod, gmap), (gmap, deil), (deil, hdec), (hdec, hdr), (hdr, dewh), (dewh, crc), (crc, sink), (crc, tags)):
                tb.connect(a, b)
            tb.connect((crc, 1), (flags, 0))
            tb.msg_connect((hdr, "frame_info"), (fsync, "frame_info"))
            t0 = time.monotonic()
            tb.run()
            runtime = time.monotonic() - t0
            sink.close()
            data = pathlib.Path(payload_bin).read_bytes()
            decoded = []
            for t in tags.current_tags():
                d = t.value
                key = lambda k: pmt.dict_ref(d, pmt.intern(k), pmt.PMT_NIL)
                pay_len = pmt.to_long(key("pay_len")) if not pmt.is_null(key("pay_len")) else 0
                crc_valid = pmt.to_bool(key("crc_valid")) if not pmt.is_null(key("crc_valid")) else False
                err = pmt.to_long(key("err")) if not pmt.is_null(key("err")) else 0
                decoded.append((data[int(t.offset):int(t.offset) + pay_len], crc_valid, err))
        correct = set()
        crc_ok = 0
        false_pass = 0
        for payload, crc_valid, _err in decoded:
            if crc_valid:
                crc_ok += 1
                idx = int.from_bytes(payload[:2], "big") if len(payload) >= 2 else -1
                if want.get(idx) == payload:
                    correct.add(idx)
                else:
                    false_pass += 1
        return {
            "frames_with_header": len(decoded), "crc_ok": crc_ok, "correct": len(correct), "crc_ok_but_wrong": false_pass,
            "missed": n - len(decoded), "header_errors": sum(1 for _p, _c, e in decoded if e), "correct_indices": sorted(correct),
        }, runtime

    runs = []
    runtimes = []
    for _ in range(2):
        r, rt = run_once()
        runs.append(r)
        runtimes.append(rt)
    stable = runs[0] == runs[1]
    if not stable:
        r, rt = run_once()
        runs.append(r)
        runtimes.append(rt)
    best = max(runs, key=lambda r: (r["correct"], r["crc_ok"], r["frames_with_header"]))
    result = {
        "cell": name, "n_frames": n, **best,
        "per_by_crc": 1.0 - best["crc_ok"] / n, "per_strict": 1.0 - best["correct"] / n,
        "stable": stable, "runs": [r["correct"] for r in runs],
        "receiver": "gr-lora_sdr frame_sync/fft_demod(hard)/gray_mapping/deinterleaver/hamming_dec/header_decoder/dewhitening/crc_verif",
        "oracle": versions(gr), "pacing": {"x_realtime": throttle_x, "items_per_chunk": int(chunk), "symbol_interval_ms": SYMBOL_INTERVAL_MS if chunk else None},
    }
    (cell_dir / "oracle.json").write_text(json.dumps(result, indent=1) + "\n")
    result["realtime_factor"] = (truth["total_samples"] / fs) / (sum(runtimes) / len(runtimes))
    return result

# --- manifest -----------------------------------------------------------------------------------
def sha256(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for chunk in iter(lambda: f.read(1 << 20), b""):
            h.update(chunk)
    return h.hexdigest()

def update_manifest(out_dir, names, seed, os_factor):
    mpath = out_dir / "MANIFEST.json"
    m = json.loads(mpath.read_text()) if mpath.exists() else {"corpus": "qrf-lora-v1", "cells": {}}
    m.update({"generator": "scripts/make-corpus.py", "seed": seed, "oversampling": os_factor, "noise_rms_lsb": NOISE_LSB})
    for name in names:
        cell_dir = out_dir / name
        entry = {}
        for fn in ("iq.cs8", "truth.json", "oracle.json"):
            p = cell_dir / fn
            if p.exists():
                entry[fn] = {"sha256": sha256(p), "bytes": p.stat().st_size}
        m["cells"][name] = entry
    mpath.write_text(json.dumps(m, indent=1, sort_keys=True) + "\n")
    return mpath

def check(out_dir):
    mpath = out_dir / "MANIFEST.json"
    if not mpath.exists():
        sys.exit(f"no manifest at {mpath}")
    m = json.loads(mpath.read_text())
    bad = 0
    n = 0
    for name, files in sorted(m["cells"].items()):
        for fn, rec in files.items():
            n += 1
            p = out_dir / name / fn
            if not p.exists():
                print(f"missing {name}/{fn}")
                bad += 1
            elif sha256(p) != rec["sha256"]:
                print(f"changed {name}/{fn}")
                bad += 1
    print(f"{n} files in {len(m['cells'])} cells; {bad} differ from MANIFEST.json")
    return 1 if bad else 0

def report(out_dir):
    """Markdown table of every cell with the oracle's result (pasted into docs/lora-corpus.md)."""
    mpath = out_dir / "MANIFEST.json"
    if not mpath.exists():
        sys.exit(f"no manifest at {mpath}")
    m = json.loads(mpath.read_text())
    order = [c[0] for c in cell_table()]
    print("| Cell | Preset | BW kHz | SF | CR | LDRO | SNR dB (in BW) | Frames | MB | Oracle: headers | CRC ok | correct | missed | PER | Repeatable |")
    print("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |")
    for name in sorted(m["cells"], key=lambda n: order.index(n) if n in order else 1e9):
        d = out_dir / name
        if not (d / "truth.json").exists() or not (d / "oracle.json").exists():
            continue
        tr = json.loads((d / "truth.json").read_text())
        o = json.loads((d / "oracle.json").read_text())
        print(f"| `{name}` | {tr['preset']} | {tr['bw_hz'] // 1000} | {tr['sf']} | {tr['coding_rate']} | {'on' if tr['ldro'] else 'off'} | {tr['snr_db_in_bandwidth']:+.1f} | {tr['n_frames']} | {m['cells'][name]['iq.cs8']['bytes'] / 1e6:.1f} | {o['frames_with_header']} | {o['crc_ok']} | {o['correct']} | {o['missed']} | {o['per_strict']:.3f} | {'yes' if o.get('stable') else 'no: ' + str(o.get('runs'))} |")
    return 0

# --- main ---------------------------------------------------------------------------------------
def main():
    global CFO_PPM, THROTTLE_X
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("command", choices=("generate", "verify", "check", "list", "report"))
    ap.add_argument("--out", type=pathlib.Path, default=OUT_DEFAULT)
    ap.add_argument("--cells", default="all")
    ap.add_argument("--frames", type=int, default=100, help="frames per sweep cell")
    ap.add_argument("--frames-r08", type=int, default=1000, help="frames per R-08 cell")
    ap.add_argument("--seed", type=int, default=20261008)
    ap.add_argument("--os", dest="os_factor", type=int, default=4, help="integer oversampling of the bandwidth (4: the oracle's tested regime)")
    ap.add_argument("--cfo-ppm", type=float, default=CFO_PPM, help="carrier-offset range per frame, ± ppm of 915 MHz")
    ap.add_argument("--no-verify", action="store_true")
    ap.add_argument("--no-filter", action="store_true", help="white noise over the whole sample band (no channel filter)")
    ap.add_argument("--throttle", type=float, default=0.0, help="pace the oracle at N x real time with default chunks instead of one symbol per 2 ms")
    a = ap.parse_args()
    CFO_PPM = a.cfo_ppm
    THROTTLE_X = a.throttle

    if a.command == "list":
        for i, (name, preset, snr, kind) in enumerate(cell_table()):
            print(f"{i:3} {name:32} {preset:14} SNR {snr:+6.1f} dB in BW  {kind}")
        return 0
    if a.command == "check":
        return check(a.out)
    if a.command == "report":
        return report(a.out)

    ctx = load_oracle()
    table = cell_table()
    chosen = select_cells(a.cells, a.out)
    a.out.mkdir(parents=True, exist_ok=True)
    names = []
    for name, preset, snr, kind in chosen:
        idx = [c[0] for c in table].index(name) if kind != "external" else 0
        if kind == "external" and a.command == "generate":
            sys.exit(f"{name}: not a table cell; only `verify` applies to external cells")
        if a.command == "generate":
            n = a.frames_r08 if kind == "r08" else a.frames
            t0 = time.monotonic()
            truth = generate_cell(ctx, name, preset, snr, n, a.seed, a.os_factor, a.out, idx, not a.no_filter)
            mb = truth["total_samples"] * 2 / 1e6
            print(f"generated {name:32} {n:5} frames {mb:8.1f} MB  clipped {truth['clipped_samples']}  {time.monotonic() - t0:6.1f} s", flush=True)
        if a.command == "verify" or (a.command == "generate" and not a.no_verify):
            r = verify_cell(ctx, a.out, name)
            rtf = r["realtime_factor"]
            print(f"oracle    {name:32} headers {r['frames_with_header']:5} crc_ok {r['crc_ok']:5} correct {r['correct']:5} missed {r['missed']:5}  PER {r['per_strict']:.3f}  runs {r['runs']} {'stable' if r['stable'] else 'UNSTABLE'}  ({rtf:.1f}x realtime)", flush=True)
        names.append(name)
    mpath = update_manifest(a.out, names, a.seed, a.os_factor)
    print(f"manifest {mpath}")
    return 0

if __name__ == "__main__":
    sys.exit(main())
