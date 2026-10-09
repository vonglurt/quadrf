#!/usr/bin/env python3
"""Figures for lab/LR-009 from the R-03 soak record (backlog R-03).

Reads resources/measurements/R-03/mipi-soak-60s.json and frame0.cs8 (both
written by `make mipi-check`) and writes PNGs under lab/figures/LR-009/.
Nothing here is a derived number for the documents; the numbers in LR-009
are quoted from the JSON. Run from the repository root: python3 -I analysis/plot-lr009.py
"""
import json, pathlib, sys
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
from matplotlib.patches import Rectangle

ROOT = pathlib.Path(__file__).resolve().parent.parent
MEAS = ROOT / "resources/measurements/R-03"
OUT = ROOT / "lab/figures/LR-009"
# Categorical slots 1–4 of the validated reference palette (light surface), one per element.
ELEM = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100"]
INK, MUTED, GRID = "#1a1a19", "#6b6b66", "#e4e4e0"
plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 9, "axes.edgecolor": MUTED, "axes.labelcolor": INK,
                     "xtick.color": INK, "ytick.color": INK, "axes.spines.top": False, "axes.spines.right": False,
                     "grid.color": GRID, "axes.grid": True, "axes.axisbelow": True, "figure.facecolor": "white"})

FRAME_BYTES, N, EL = 131072, 16384, 4

def frame_layout(frame: np.ndarray, tones):
    """Figure 1: the interleave as bytes, then the de-interleaved waveforms and spectra."""
    fig = plt.figure(figsize=(9.5, 8.2))
    gs = fig.add_gridspec(3, 2, height_ratios=[0.9, 1.3, 1.4], hspace=0.8, wspace=0.28)
    # (a) byte map of samples 1–8 (sample 0 carries the mock's counter)
    ax = fig.add_subplot(gs[0, :])
    for s in range(1, 5):
        for e in range(EL):
            for k, comp in enumerate("IQ"):
                b = s * 8 + 2 * e + k
                x = (s - 1) * 8 + 2 * e + k
                ax.add_patch(Rectangle((x, 0), 1, 1, facecolor=ELEM[e], edgecolor="white", linewidth=1.5, alpha=0.95 if k == 0 else 0.6))
                ax.text(x + 0.5, 0.5, f"{comp}{e}\n{int(np.int8(frame[b]))}", ha="center", va="center", fontsize=7.5, color="white" if e != 3 else INK)
        ax.text((s - 1) * 8 + 4, 1.12, f"time sample {s}", ha="center", fontsize=8, color=MUTED)
    ax.set_xlim(0, 32); ax.set_ylim(0, 1.35); ax.set_yticks([]); ax.grid(False)
    ax.set_xticks(range(0, 33, 8)); ax.set_xticklabels([str(8 + 8 * i) for i in range(5)])
    ax.set_xlabel("byte offset in the frame (bytes 8–39 shown: time samples 1–4)\neach 8-byte group is one time sample, I then Q for elements 0–3; cell values are the 8-bit codes")
    ax.set_title("(a) the tile's interleave: consecutive 16-bit (I, Q) words, elements 0–3 repeating (SPEC-002 S-002-21)", loc="left", fontsize=9.5, color=INK)
    # de-interleave as the crate does
    el = [frame.reshape(N, 8)[:, 2 * e:2 * e + 2].astype(np.int8) for e in range(EL)]
    # (b) waveforms, first 120 samples after the counter
    ax = fig.add_subplot(gs[1, :])
    t = np.arange(1, 33)
    for e in range(EL):
        ax.plot(t, el[e][1:33, 0], color=ELEM[e], linewidth=1.2, marker="o", markersize=3.5, label=f"element {e}: I (solid), Q (dotted); {tones[e]['bin']} cycles per frame")
        ax.plot(t, el[e][1:33, 1], color=ELEM[e], linewidth=0.9, linestyle=":", marker="o", markersize=2.5, markerfacecolor="white")
    ax.set_xlim(1, 32); ax.set_ylim(-128, 127); ax.set_yticks([-128, -64, 0, 64, 127])
    ax.set_xlabel("time sample within the frame (the tones are 0.15–0.49 cycles per sample, so the eye sees points, not sinusoids)"); ax.set_ylabel("8-bit code")
    ax.set_title("(b) de-interleaved CS8 samples 1–32 of frame 0 (amplitude 100 LSB; sample 0 holds the mock's frame counter)", loc="left", fontsize=9.5, color=INK)
    ax.legend(loc="upper center", bbox_to_anchor=(0.5, -0.32), fontsize=7, frameon=False, ncol=2)
    # (c) spectra
    ax = fig.add_subplot(gs[2, :])
    for e in range(EL):
        x = el[e][:, 0].astype(float) + 1j * el[e][:, 1].astype(float)
        x[0] = 0
        X = np.fft.fft(x) / (N - 1) / 100.0
        p = 20 * np.log10(np.maximum(np.abs(X), 1e-7))
        ax.plot(np.arange(N), p, color=ELEM[e], linewidth=0.8, alpha=0.9)
    order = sorted(range(EL), key=lambda e: tones[e]["bin"])
    for rank, e in enumerate(order):
        ax.annotate(f"e{e}: bin {tones[e]['bin']} ({tones[e]['freq_hz_at_26msps']/1e6:.3f} MHz at 26 MSPS), phase {tones[e]['phase_deg']:+.2f}°",
                    (tones[e]["bin"], 0), xytext=(tones[e]["bin"] + 300, 10 + 8 * rank), fontsize=7, color=INK, ha="left",
                    arrowprops=dict(arrowstyle="-", color=MUTED, linewidth=0.6))
    ax.set_xlim(0, N); ax.set_ylim(-70, 44); ax.set_xlabel("DFT bin (16 384-point, one frame; bin spacing 1.587 kHz at 26 MSPS)")
    ax.set_ylabel("dB re tone amplitude")
    ax.set_title("(c) one-frame spectrum per element: one tone each, 8-bit quantisation floor near −60 dB", loc="left", fontsize=9.5, color=INK)
    fig.savefig(OUT / "frame-layout.png", dpi=130, bbox_inches="tight")
    plt.close(fig)

def soak(rep):
    """Figure 2: ring occupancy over the 60-s run and the inter-arrival histogram."""
    fig, (a, b) = plt.subplots(1, 2, figsize=(9.5, 3.4), gridspec_kw={"width_ratios": [1.6, 1], "wspace": 0.3})
    tr = np.array(rep["occupancy_trace"])
    cap = rep["ring_capacity_spans"]
    a.plot(tr[:, 0], tr[:, 1], color=ELEM[0], linewidth=0.9)
    a.axhline(cap, color="#b3261e", linewidth=1, linestyle="--")
    a.text(rep["seconds_elapsed"], cap + 1.2, f"ring capacity {cap} spans (drop above)", ha="right", fontsize=7.5, color="#b3261e")
    a.axhline(rep["max_backlog_spans"], color=MUTED, linewidth=0.7, linestyle=":")
    a.text(0.3, rep["max_backlog_spans"] + 1.2, f"max backlog {rep['max_backlog_spans']} spans = {rep['max_backlog_spans'] * rep['frame_period_us'] / 1000:.1f} ms", fontsize=7.5, color=MUTED)
    a.set_xlim(0, rep["seconds_elapsed"]); a.set_ylim(0, cap + 8)
    a.set_xlabel("time in the run, s"); a.set_ylabel("spans waiting in the ring")
    a.set_title(f"(a) occupancy sampled every 100 spans; {rep['frames_consumed']} frames, {rep['frames_dropped']} dropped, {rep['counter_gaps']} gaps", loc="left", fontsize=9, color=INK)
    h = np.array(rep["interarrival_hist_us"])
    b.bar(h[:, 0] + 25, np.maximum(h[:, 1], 0.5), width=46, color=ELEM[0], linewidth=0)
    b.set_yscale("log"); b.set_xlim(0, 2000); b.set_ylim(0.5, max(h[:, 1]) * 2)
    b.axvline(rep["frame_period_us"], color=MUTED, linewidth=0.8, linestyle=":")
    b.text(rep["frame_period_us"] + 20, max(h[:, 1]) * 1.2, f"T14 period {rep['frame_period_us']:.0f} µs", fontsize=7.5, color=MUTED)
    ia = rep["interarrival_us"]
    b.set_title(f"(b) span inter-arrival at the consumer; p50 {ia['p50']:.0f}, p99 {ia['p99']:.0f}, max {ia['max']:.0f} µs", loc="left", fontsize=9, color=INK)
    b.set_xlabel("inter-arrival, µs (50 µs bins; last bin ≥ 1 950 µs)"); b.set_ylabel("spans")
    fig.savefig(OUT / "soak-60s.png", dpi=130, bbox_inches="tight")
    plt.close(fig)

def ring_protocol(rep):
    """Figure 3: the ring as the driver and qrf-mipi see it."""
    cap = rep["ring_capacity_spans"]; slots = cap + 1
    fig, ax = plt.subplots(figsize=(9.5, 2.9))
    tail, held, filled, head = 20, 21, 30, 31
    for i in range(slots):
        if i == tail: c, lab = "#1baf7a", "held by\nconsumer"
        elif tail < i < head: c, lab = "#2a78d6", ""
        elif i == slots - 1: c, lab = "#d9d9d4", "slot 64: never\nwhole (1 byte\nkept free)"
        else: c, lab = "#f0f0ec", ""
        ax.add_patch(Rectangle((i, 0), 1, 1, facecolor=c, edgecolor="white", linewidth=1))
        if lab: ax.text(i + 0.5, -0.35, lab, ha="center", va="top", fontsize=7, color=INK)
    ax.annotate("tail (consumer)\nCONSUME_BYTES advances it", (tail, 1), xytext=(tail - 6, 1.9), fontsize=7.5, color=INK, arrowprops=dict(arrowstyle="->", color=INK, linewidth=0.8))
    ax.annotate("head (producer)\nadvanced after the memcpy, Release", (head, 1), xytext=(head + 2, 1.9), fontsize=7.5, color=INK, arrowprops=dict(arrowstyle="->", color=INK, linewidth=0.8))
    ax.text((tail + head) / 2 + 0.5, 0.5, f"{head - tail - 1} spans waiting", ha="center", va="center", fontsize=7.5, color="white")
    ax.text(head + 0.5 + (slots - head) / 2, 0.5, f"free: ring_size − 1 − used", ha="center", va="center", fontsize=7.5, color=MUTED)
    ax.set_xlim(0, slots); ax.set_ylim(-1.1, 2.9); ax.axis("off")
    ax.set_title(f"ring of {rep['ring_bytes'] >> 20} MiB = {slots} span slots of {rep['frame_bytes'] >> 10} KiB; the driver keeps one byte free, so {cap} spans fit ({cap * rep['frame_period_us'] / 1000:.1f} ms at {rep['frame_period_us']:.0f} µs per span); a frame with no room is dropped and counted in overflows_ring",
                 loc="left", fontsize=8.5, color=INK)
    fig.savefig(OUT / "ring-protocol.png", dpi=130, bbox_inches="tight")
    plt.close(fig)

def main():
    rep = json.loads((MEAS / "mipi-soak-60s.json").read_text())
    frame = np.fromfile(MEAS / "frame0.cs8", dtype=np.uint8)
    assert frame.size == FRAME_BYTES, frame.size
    OUT.mkdir(parents=True, exist_ok=True)
    frame_layout(frame, rep["tones"])
    soak(rep)
    ring_protocol(rep)
    for p in sorted(OUT.glob("*.png")):
        print(f"{p.relative_to(ROOT)} {p.stat().st_size} B")

if __name__ == "__main__":
    sys.exit(main())
