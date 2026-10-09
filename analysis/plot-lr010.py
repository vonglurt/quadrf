#!/usr/bin/env python3
"""Figures for lab/LR-010 from the R-07 record (backlog R-07).

Reads resources/measurements/R-07/dsp-soak-60s.json (written by `make dsp-check`)
and writes PNGs under lab/figures/LR-010/. Nothing here is a derived number for
the documents; the numbers in LR-010 are quoted from the JSON or from analysis
T25. Run from the repository root: python3 -I analysis/plot-lr010.py
"""
import json, pathlib, sys
import numpy as np
import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

ROOT = pathlib.Path(__file__).resolve().parent.parent
MEAS = ROOT / "resources/measurements/R-07"
OUT = ROOT / "lab/figures/LR-010"
# Categorical slots 1–4 of the validated reference palette (light surface).
ELEM = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100"]
INK, MUTED, GRID, RED = "#1a1a19", "#6b6b66", "#e4e4e0", "#b3261e"
plt.rcParams.update({"font.family": "DejaVu Sans", "font.size": 9, "axes.edgecolor": MUTED, "axes.labelcolor": INK,
                     "xtick.color": INK, "ytick.color": INK, "axes.spines.top": False, "axes.spines.right": False,
                     "grid.color": GRID, "axes.grid": True, "axes.axisbelow": True, "figure.facecolor": "white"})

def channeliser(fn):
    """Figure 1: the prototype's response over ±2 bins with the measured tone gains, and the detector's sensitivity."""
    fig, (a, b) = plt.subplots(1, 2, figsize=(9.5, 3.4), gridspec_kw={"width_ratios": [1.4, 1], "wspace": 0.3})
    r = np.array(fn["prototype"]["response_db"])
    a.plot(r[:, 0], r[:, 1], color=ELEM[0], linewidth=1.2, label="prototype |H(δ)|², Kaiser β 5.65, M·P = 1024 taps")
    t = np.array(fn["prototype"]["tones"])
    a.plot(t[:, 1], t[:, 2], linestyle="none", marker="o", markersize=5, markerfacecolor="white", color=ELEM[1], label="tone through the bank: measured gain in its bin")
    for off in (-0.5, 0.5):
        a.axvline(off, color=MUTED, linewidth=0.7, linestyle=":")
    a.text(0.52, -75, "bin edge", fontsize=7.5, color=MUTED)
    a.set_xlim(-2, 2); a.set_ylim(-80, 3)
    a.set_xlabel("tone offset from the bin centre, bins (1 bin = 203.125 kHz)"); a.set_ylabel("gain, dB")
    a.set_title(f"(a) a tone at a bin centre passes at 0 dB (worst {fn['prototype']['worst_centre_gain_db']:.1e} dB);\noff centre it follows the prototype (worst mismatch {fn['prototype']['worst_off_centre_mismatch_db']:.1e} dB)", loc="left", fontsize=8.5, color=INK)
    a.legend(loc="lower center", fontsize=7.5, frameon=False)
    c = fn["cfar"]
    s = np.array(c["sensitivity"])
    b.plot(s[:, 0], s[:, 1], color=ELEM[2], linewidth=1.2, marker="o", markersize=4)
    b.axhline(c["pfa_design"], color=RED, linewidth=0.8, linestyle="--")
    b.text(s[0, 0], c["pfa_design"] * 1.6, f"design Pfa {c['pfa_design']:.0e}; measured {c['pfa_measured']:.2e} on noise alone ({c['false_alarms']} in {c['looks'] * 104})", fontsize=7.5, color=RED)
    b.set_yscale("log"); b.set_ylim(3e-4, 1.5); b.set_xlim(s[0, 0] - 1, s[-1, 0] + 1)
    nb = c["noise_per_bin_db_re_input"]
    b.set_xlabel(f"tone power re the per-element noise power, dB (noise per bin {nb:.1f} dB re input)"); b.set_ylabel("duty of the tone's slot over 200 looks")
    b.set_title(f"(b) detector sensitivity, one element,\n{c['n_avg']}-block looks, α {c['alpha_interior_one_bin']:.3f}", loc="left", fontsize=8.5, color=INK)
    fig.savefig(OUT / "channeliser-cfar.png", dpi=130, bbox_inches="tight")
    plt.close(fig)

def bearing(fn):
    """Figure 2: bearing error histograms of both estimators against the CRLB, and the two-source MUSIC spectrum."""
    runs = fn["bearing"]
    fig = plt.figure(figsize=(9.5, 6.4))
    gs = fig.add_gridspec(2, 3, height_ratios=[1, 1], hspace=0.55, wspace=0.3)
    panels = [(0, "phase_difference", 20.0), (1, "music", 20.0), (2, "phase_difference", 10.0)]
    for col, method, snr in panels:
        ax = fig.add_subplot(gs[0, col])
        run = next(r for r in runs if r["method"] == method and r["snr_db"] == snr)
        e = np.array(run["errors_deg"])
        hi = max(0.3, np.percentile(e, 99.5) * 1.2)
        ax.hist(e, bins=40, range=(0, hi), color=ELEM[col], linewidth=0, alpha=0.9)
        for v, lab, c in ((run["crlb_one_pair_deg"], "T16 one pair", MUTED), (run["crlb_two_pairs_deg"], "two pairs", MUTED), (run["rms_deg"], "rms", RED)):
            ax.axvline(v, color=c, linewidth=0.9, linestyle="--" if c == MUTED else "-")
        ax.text(run["rms_deg"] * 1.05, ax.get_ylim()[1] * 0.92, f"rms {run['rms_deg']:.3f}°\nσθ reported {run['mean_sigma_theta_deg']:.3f}°\nCRLB 1 pair {run['crlb_one_pair_deg']:.3f}°, 2 pairs {run['crlb_two_pairs_deg']:.3f}°", fontsize=7, color=INK, va="top")
        ax.set_xlim(0, hi); ax.set_xlabel("angle between true and\nestimated direction, °"); ax.set_ylabel("trials" if col == 0 else "")
        ax.set_title(f"({'abc'[col]}) {method}, SNR {snr:.0f} dB\nN = {run['n']}, {run['trials']} trials, θ ≤ {run['theta_max_deg']:.0f}°", loc="left", fontsize=8.5, color=INK)
    ax = fig.add_subplot(gs[1, :])
    ts = fn["music_two_sources"]
    sp = np.array(ts["spectrum_phi"])
    ax.plot(sp[:, 0], sp[:, 1] - sp[:, 1].max(), color=ELEM[0], linewidth=1.1)
    for (t, p) in ts["sources"]:
        ax.axvline(p, color=MUTED, linewidth=0.8, linestyle=":")
    for (t, p) in ts["estimates"]:
        ax.axvline(p, color=RED, linewidth=0.8, linestyle="--")
    ax.set_xlim(-180, 180); ax.set_xticks(range(-180, 181, 45))
    ax.set_xlabel(f"φ, ° (cut at θ = {ts['spectrum_phi_at_theta']:.1f}°; dotted: true sources at θ,φ = {ts['sources']}; dashed: estimates)"); ax.set_ylabel("MUSIC pseudo-spectrum, dB re peak")
    est = ", ".join(f"({t:.2f}°, {p:.2f}°)" for t, p in ts["estimates"])
    ax.set_title(f"(d) MUSIC, two equal sources, N = {ts['n']}, SNR {ts['snr_db']:.0f} dB, half-wave 2 × 2 at 915 MHz: estimates {est}", loc="left", fontsize=8.5, color=INK)
    fig.savefig(OUT / "bearing.png", dpi=130, bbox_inches="tight")
    plt.close(fig)

def soak(sk, host):
    """Figure 3: the 60-s run: CPU shares, channeliser time per frame, and the occupancy strip."""
    fig = plt.figure(figsize=(9.5, 6.2))
    gs = fig.add_gridspec(2, 2, height_ratios=[1, 1.15], hspace=0.6, wspace=0.3)
    ax = fig.add_subplot(gs[0, 0])
    parts = [("channeliser ×4", sk["channeliser_cores"]), ("convert + noise", sk["convert_cores"]), ("de-interleave", sk["deinterleave_cores"]), ("detector + duty", sk["detector_cores"])]
    other = max(0.0, sk["consumer_cpu_cores_threadclock"] - sum(v for _, v in parts))
    parts.append(("reader, stamps, rest", other))
    y = np.arange(len(parts))
    ax.barh(y, [v for _, v in parts], color=[ELEM[0], ELEM[1], ELEM[2], ELEM[3], "#9a9a94"], height=0.6)
    for i, (_, v) in enumerate(parts):
        ax.text(v + 0.005, i, f"{v:.3f}", va="center", fontsize=7.5, color=INK)
    ax.set_yticks(y); ax.set_yticklabels([n for n, _ in parts], fontsize=8); ax.invert_yaxis()
    ax.axvline(0.8, color=RED, linewidth=0.9, linestyle="--"); ax.text(0.81, len(parts) - 0.7, "S-008-6 budget\n0.8 core (Pi 5)", fontsize=7.5, color=RED)
    ax.set_xlim(0, 1.0); ax.set_xlabel("cores (CPU seconds per elapsed second)")
    ax.set_title(f"(a) consumer thread {sk['consumer_cpu_cores_threadclock']:.3f} core (/proc task stat {sk['consumer_cpu_cores_procstat']:.3f});\nprocess incl. the mock producer {sk['process_cpu_cores_procstat']:.3f}", loc="left", fontsize=8.3, color=INK)
    ax = fig.add_subplot(gs[0, 1])
    ch = sk["channeliser_us"]
    labels = ["min", "p50", "p99", "p99.9", "max"]
    vals = [ch[k] for k in ("min", "p50", "p99", "p999", "max")]
    ax.bar(labels, vals, color=ELEM[0], width=0.6)
    for i, v in enumerate(vals):
        ax.text(i, v * 1.15, f"{v:.0f}", ha="center", fontsize=7.5, color=INK)
    ax.axhline(sk["frame_period_us"], color=RED, linewidth=0.9, linestyle="--"); ax.text(-0.4, sk["frame_period_us"] * 1.2, f"T14 frame period {sk['frame_period_us']:.0f} µs", fontsize=7.5, color=RED)
    ax.set_yscale("log"); ax.set_ylim(50, max(vals) * 4); ax.set_ylabel("µs per frame (4 elements × 128 blocks)")
    ax.set_title(f"(b) channeliser time per frame;\n{sk['frames_consumed']} frames, {sk['frames_dropped']} dropped, {sk['counter_gaps']} gaps", loc="left", fontsize=8.3, color=INK)
    ax = fig.add_subplot(gs[1, :])
    w = sk["windows"]
    img = np.array([x["duty"] for x in w]).T  # slots × windows
    im = ax.imshow(img, aspect="auto", cmap="Blues", vmin=0, vmax=1, interpolation="nearest", extent=(0, len(w), 103.5, -0.5))
    for s in sk["tone_slots"]:
        ax.text(len(w) + 0.3, s - 1.2, f"tone slot {s}", fontsize=7, color=ELEM[1], va="center")
    for s, d in sk["leakage_slots"]:
        ax.text(len(w) + 0.3, s + 2.2, f"leakage slot {s} (duty {d:.2f})", fontsize=7, color=MUTED, va="center")
    ax.set_xlabel("1-s window of the 60-s run"); ax.set_ylabel("slot of the US-104 plan"); ax.grid(False)
    ax.set_title(f"(c) occupancy duty per slot and 1-s window (noise floor {sk['noise_dbfs']:.0f} dBFS added): tone slots {sk['tone_slots']} at duty ≥ {sk['tone_slots_min_duty']:.2f};\nthe slot above each edge-straddling tone holds its leakage; the other slots' median duty is {sk['other_slots_median_duty']:.1e}", loc="left", fontsize=8.3, color=INK)
    cb = fig.colorbar(im, ax=ax, fraction=0.02, pad=0.09); cb.set_label("duty", fontsize=8)
    fig.savefig(OUT / "soak-60s.png", dpi=130, bbox_inches="tight")
    plt.close(fig)

def main():
    rep = json.loads((MEAS / "dsp-soak-60s.json").read_text())
    OUT.mkdir(parents=True, exist_ok=True)
    channeliser(rep["function"])
    bearing(rep["function"])
    soak(rep["soak"], rep["host"])
    for p in sorted(OUT.glob("*.png")):
        print(f"{p.relative_to(ROOT)} {p.stat().st_size} B")

if __name__ == "__main__":
    sys.exit(main())
