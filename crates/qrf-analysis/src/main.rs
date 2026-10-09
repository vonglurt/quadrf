//! Rust port of `analysis/linkbudget.py` (backlog A-02).
//!
//! The acceptance check is byte-identical output:
//!     diff <(python3 analysis/linkbudget.py) <(cargo run -q -p qrf-analysis)
//! Every expression keeps the Python evaluation order so that the IEEE-754
//! results, and therefore the printed digits, agree. Powers use `powf`
//! because CPython's float `**` calls the C library `pow` that this binary
//! links as well; `log10`, `sqrt`, `acos`, `asin` and `cos` are the same
//! libm functions in both. Python's `e` format is reproduced by `pye`.

use std::f64::consts::PI;

const C: f64 = 299_792_458.0;
const K_DBM_HZ: f64 = -174.0; // kT at 290 K, dBm/Hz

/// LoRa demodulator SNR thresholds (Semtech SX126x datasheet, CR 4/5).
fn snr_min(sf: u32) -> f64 {
    match sf {
        7 => -7.5,
        8 => -10.0,
        9 => -12.5,
        10 => -15.0,
        11 => -17.5,
        12 => -20.0,
        _ => unreachable!("SF outside 7..=12"),
    }
}

/// Meshtastic presets (bandwidth kHz, SF, CR), in the Python dict order.
const PRESETS: &[(&str, i64, u32, &str)] = &[
    ("SHORT_TURBO", 500, 7, "4/5"),
    ("SHORT_FAST", 250, 7, "4/5"),
    ("MEDIUM_FAST", 250, 9, "4/5"),
    ("LONG_TURBO", 500, 11, "4/8"),
    ("LONG_FAST", 250, 11, "4/5"),
    ("LONG_MODERATE", 125, 11, "4/8"),
    ("LONG_SLOW", 125, 12, "4/8"),
];

fn sens(bw_hz: f64, sf: u32, nf_db: f64) -> f64 {
    K_DBM_HZ + 10.0 * bw_hz.log10() + nf_db + snr_min(sf)
}

fn fspl(d_km: f64, f_mhz: f64) -> f64 {
    32.45 + 20.0 * d_km.log10() + 20.0 * f_mhz.log10()
}

/// Modified exponential decay foliage model, 230 MHz-95 GHz, valid 14..400 m.
fn weissberger(d_m: f64, f_ghz: f64) -> f64 {
    if d_m <= 14.0 {
        0.45 * f_ghz.powf(0.284) * d_m
    } else {
        1.33 * f_ghz.powf(0.284) * d_m.powf(0.588)
    }
}

/// ITU-R P.526 single knife-edge approximation, valid v > -0.78.
fn knife_edge_j(v: f64) -> f64 {
    6.9 + 20.0 * (((v - 0.1).powf(2.0) + 1.0).sqrt() + v - 0.1).log10()
}

fn fresnel_r1(f_mhz: f64, d1_km: f64, d2_km: f64) -> f64 {
    let lam = C / (f_mhz * 1e6);
    let (d1, d2) = (d1_km * 1e3, d2_km * 1e3);
    (lam * d1 * d2 / (d1 + d2)).sqrt()
}

fn radio_horizon_km(h1_m: f64, h2_m: f64) -> f64 {
    4.12 * (h1_m.sqrt() + h2_m.sqrt()) // 4/3 effective earth
}

fn mpe_distance_m(eirp_w: f64, s_limit_w_cm2: f64) -> f64 {
    (eirp_w / (4.0 * PI * s_limit_w_cm2)).sqrt() / 100.0
}

/// Half-power beamwidth of a 2-element broadside pair, isotropic elements.
/// ln Γ(n) = ln((n−1)!) for integer n ≥ 1, from a cumulative table of logs (`lg_int` in the Python script).
fn lg_int(n: usize) -> f64 {
    thread_local! { static LG: std::cell::RefCell<Vec<f64>> = std::cell::RefCell::new(vec![0.0, 0.0]); }
    LG.with(|t| {
        let mut t = t.borrow_mut();
        while t.len() <= n {
            let v = t[t.len() - 1] + ((t.len() - 1) as f64).ln();
            t.push(v);
        }
        t[n]
    })
}

/// Continued fraction for the incomplete beta function (Numerical Recipes).
fn betacf(a: f64, b: f64, x: f64) -> f64 {
    let (qab, qap, qam) = (a + b, a + 1.0, a - 1.0);
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < 1e-300 {
        d = 1e-300;
    }
    d = 1.0 / d;
    let mut h = d;
    for m in 1..=1000 {
        let mf = m as f64;
        let m2 = 2.0 * mf;
        let mut aa = mf * (b - mf) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < 1e-300 {
            d = 1e-300;
        }
        c = 1.0 + aa / c;
        if c.abs() < 1e-300 {
            c = 1e-300;
        }
        d = 1.0 / d;
        h *= d * c;
        aa = -(a + mf) * (qab + mf) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < 1e-300 {
            d = 1e-300;
        }
        c = 1.0 + aa / c;
        if c.abs() < 1e-300 {
            c = 1e-300;
        }
        d = 1.0 / d;
        let de = d * c;
        h *= de;
        if (de - 1.0).abs() < 3e-16 {
            break;
        }
    }
    h
}

/// Regularised incomplete beta I_x(a, b) for integer a, b.
fn incbeta(a: usize, b: usize, x: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    let (af, bf) = (a as f64, b as f64);
    let bt = (lg_int(a + b) - lg_int(a) - lg_int(b) + af * x.ln() + bf * (1.0 - x).ln()).exp();
    if x < (af + 1.0) / (af + bf + 2.0) { bt * betacf(af, bf, x) / af } else { 1.0 - bt * betacf(bf, af, 1.0 - x) / bf }
}

/// CA-CFAR threshold over the training mean (ratio law): Pfa = I_{K/(K + αk)}(K, k), solved by bisection.
fn cfar_alpha_ca(k: usize, kk: usize, pfa: f64) -> f64 {
    let (mut lo, mut hi) = (1.0f64, 1000.0f64);
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        if incbeta(kk, k, kk as f64 / (kk as f64 + mid * k as f64)) > pfa {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

fn bessel_i0(x: f64) -> f64 {
    let (mut s, mut t, q) = (1.0, 1.0, x * x / 4.0);
    for k in 1..200 {
        t *= q / (k as f64 * k as f64);
        s += t;
        if t < s * 1e-17 {
            break;
        }
    }
    s
}

/// Kaiser-windowed sinc, m·p taps, cutoff fs/(2m), unit DC gain (qrf-dsp `filter::prototype`).
fn pfb_prototype(m: usize, p: usize, beta: f64) -> Vec<f64> {
    let l = m * p;
    let c = (l as f64 - 1.0) / 2.0;
    let mut h = Vec::with_capacity(l);
    for n in 0..l {
        let x = (n as f64 - c) / m as f64;
        let sinc = if x.abs() < 1e-12 { 1.0 } else { (PI * x).sin() / (PI * x) };
        let r = 2.0 * n as f64 / (l as f64 - 1.0) - 1.0;
        h.push(sinc * bessel_i0(beta * (1.0 - r * r).max(0.0).sqrt()) / bessel_i0(beta));
    }
    let tot: f64 = h.iter().sum();
    h.iter().map(|v| v / tot).collect()
}

/// Independent looks equivalent to `n_avg` block outputs of `nbins` adjacent bins of a critically sampled PFB on white noise.
fn pfb_effective_looks(h: &[f64], m: usize, p: usize, nbins: usize, n_avg: usize) -> f64 {
    let e2: f64 = h.iter().map(|v| v * v).sum();
    let mut tot = 0.0;
    for dk in -(nbins as i64 - 1)..nbins as i64 {
        let cnt = (nbins as i64 - dk.abs()) as f64;
        for tau in -(p as i64 - 1)..p as i64 {
            let (mut re, mut im) = (0.0f64, 0.0f64);
            for n in 0..h.len() {
                let j = n as i64 + tau * m as i64;
                if j >= 0 && (j as usize) < h.len() {
                    let a = 2.0 * PI * dk as f64 * n as f64 / m as f64;
                    re += h[n] * h[j as usize] * a.cos();
                    im += h[n] * h[j as usize] * a.sin();
                }
            }
            tot += cnt * (n_avg as f64 - tau.abs() as f64) * (re * re + im * im) / (e2 * e2);
        }
    }
    ((n_avg * nbins) as f64).powi(2) / tot
}

/// Regularised upper incomplete gamma Q(a, x) for integer a (Numerical Recipes gser/gcf); ln Gamma(a) as a sum of logs.
fn gamma_q(a: i64, x: f64) -> f64 {
    let af = a as f64;
    let mut lg = 0.0;
    for i in 1..a {
        lg += (i as f64).ln();
    }
    let pre = (-x + af * x.ln() - lg).exp();
    if x < af + 1.0 {
        let (mut ap, mut s, mut d) = (af, 1.0 / af, 1.0 / af);
        for _ in 0..500 {
            ap += 1.0;
            d *= x / ap;
            s += d;
            if d.abs() < s.abs() * 1e-15 {
                break;
            }
        }
        return 1.0 - s * pre;
    }
    let mut b = x + 1.0 - af;
    let mut c = 1.0 / 1e-300;
    let mut d = 1.0 / b;
    let mut h = d;
    for i in 1..500 {
        let an = -(i as f64) * (i as f64 - af);
        b += 2.0;
        d = an * d + b;
        if d.abs() < 1e-300 {
            d = 1e-300;
        }
        c = b + an / c;
        if c.abs() < 1e-300 {
            c = 1e-300;
        }
        d = 1.0 / d;
        let de = d * c;
        h *= de;
        if (de - 1.0).abs() < 1e-15 {
            break;
        }
    }
    pre * h
}

/// Threshold over the noise mean for a power averaged over k exponential looks: Q(k, k*alpha) = pfa, by bisection.
fn cfar_alpha(k: i64, pfa: f64) -> f64 {
    let (mut lo, mut hi) = (1.0f64, 1000.0f64);
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        if gamma_q(k, k as f64 * mid) > pfa {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

fn hpbw_2el_deg(d_over_lambda: f64) -> f64 {
    let x = (1.0 / 2f64.sqrt()).acos() / (PI * d_over_lambda);
    2.0 * x.min(1.0).asin().to_degrees()
}

/// Max steer angle before a grating lobe enters visible space.
fn grating_steer_limit_deg(d_over_lambda: f64) -> f64 {
    let s = 1.0 / d_over_lambda - 1.0;
    if s >= 1.0 { 90.0 } else { s.max(0.0).asin().to_degrees() }
}

fn unambiguous_doa_deg(d_over_lambda: f64) -> f64 {
    let s = 1.0 / (2.0 * d_over_lambda);
    if s >= 1.0 { 90.0 } else { s.asin().to_degrees() }
}

/// KrakenSDR-style uniform circular array: inter-element spacing s*lambda.
fn uca_radius_m(s: f64, lam_m: f64, n: f64) -> f64 {
    s * lam_m / (2.0 * (1.0 - (2.0 * PI / n).cos())).sqrt()
}

fn db(x: f64) -> f64 {
    10.0 * x.log10()
}
fn lin(x_db: f64) -> f64 {
    10f64.powf(x_db / 10.0)
}

fn line() {
    println!("{}", "-".repeat(78));
}

/// Python's `f"{x:.{prec}e}"`: mantissa with `prec` decimals, exponent sign and at least two digits.
fn pye(x: f64, prec: usize) -> String {
    let s = format!("{:.*e}", prec, x);
    let (m, e) = s.split_once('e').expect("exponent");
    let (sign, digits) = match e.strip_prefix('-') {
        Some(d) => ("-", d),
        None => ("+", e),
    };
    let digits = if digits.len() < 2 { format!("0{digits}") } else { digits.to_string() };
    format!("{m}e{sign}{digits}")
}

fn main() {
    println!("T1  LoRa receiver sensitivity (dBm) = -174 + 10log10(BW) + NF + SNRmin");
    println!("{:14}{:>8}{:>4}{:>5}{:>12}{:>14}", "preset", "BW kHz", "SF", "CR", "SX1262 NF6", "QuadRF NF4.2");
    for &(name, bw, sf, cr) in PRESETS {
        let bw_hz = bw as f64 * 1e3;
        println!("{name:14}{bw:8}{sf:4}{cr:>5}{:12.1}{:14.1}", sens(bw_hz, sf, 6.0), sens(bw_hz, sf, 4.2));
    }
    line();
    println!("T2  Free-space path loss (dB)");
    println!("{:>6}{:>10}{:>10}{:>8}", "d km", "915 MHz", "5800 MHz", "delta");
    for d in [5i64, 10, 25, 40, 60, 80, 120] {
        let (a, b) = (fspl(d as f64, 915.0), fspl(d as f64, 5800.0));
        println!("{d:6}{a:10.1}{b:10.1}{:8.1}", b - a);
    }
    line();
    println!("T3  Foliage excess loss, Weissberger (dB)");
    println!("{:>8}{:>10}{:>10}", "depth m", "915 MHz", "5800 MHz");
    for dm in [10i64, 20, 50, 100, 200, 400] {
        println!("{dm:8}{:10.1}{:10.1}", weissberger(dm as f64, 0.915), weissberger(dm as f64, 5.8));
    }
    line();
    println!("T4  Single knife-edge diffraction loss J(v) (dB), ITU-R P.526");
    for v in [-0.5f64, 0.0, 0.5, 1.0, 2.0, 3.0, 5.0] {
        println!("  v={v:5.1}  J={:5.1}", knife_edge_j(v));
    }
    line();
    println!("T5  First Fresnel radius at midpoint (m)");
    for d in [10i64, 25, 40, 80] {
        let h = d as f64 / 2.0;
        println!("  d={d:3} km  915: {:5.1}   5800: {:5.1}", fresnel_r1(915.0, h, h), fresnel_r1(5800.0, h, h));
    }
    line();
    println!("T6  Radio horizon, 4/3 earth (km)");
    for (h1, h2) in [(2i64, 300i64), (2, 1200), (10, 1200), (30, 2000)] {
        println!("  h_user={h1:3} m  h_summit={h2:5} m  ->  {:6.1} km", radio_horizon_km(h1 as f64, h2 as f64));
    }
    line();
    println!("T7  Regulatory EIRP ceilings and worked link margins, 40 km LOS, Rx 6 dBi omni");
    let cases: [(&str, f64, f64, f64); 5] = [
        ("Part15 902-928 any antenna (36 dBm cap)", 36.0, 915.0, 6.0),
        ("Part15 5725-5850 P2P, 1 W + 12 dBi tile", 42.0, 5800.0, 12.0),
        ("Part15 5725-5850 P2P, 1 W + 72-el 24.6 dBi", 54.6, 5800.0, 24.6),
        ("Part97 33cm SS 10 W PEP + 12 dBi yagi", 52.0, 915.0, 12.0),
        ("Part97 33cm SS 10 W PEP + 24.6 dBi array", 64.6, 915.0, 24.6),
    ];
    for (name, eirp, f, grx) in cases {
        let pl = fspl(40.0, f);
        let prx = eirp - pl + grx;
        let s_lf = if f < 2000.0 { sens(250e3, 11, 6.0) } else { sens(250e3, 11, 4.2) };
        let s_st = if f < 2000.0 { sens(500e3, 7, 6.0) } else { sens(500e3, 7, 4.2) };
        println!("  {name:46} EIRP {eirp:5.1} dBm  Prx {prx:7.1} dBm  margin LF {:5.1}  ST {:5.1}", prx - s_lf, prx - s_st);
    }
    line();
    println!("T8  Part 15.247(b)(4) conducted-power reduction, 902-928 MHz: Pc_max = 30 - max(0, G-6)");
    for g in [2i64, 6, 9, 12, 15, 24] {
        let pc = 30 - 0.max(g - 6);
        println!("  G={g:2} dBi  Pc_max={:5.1} dBm  EIRP={:5.1} dBm", pc as f64, (30.min(pc) + g) as f64);
    }
    line();
    println!("T9  Array geometry, QuadRF tile pitch 45.5 mm");
    for f in [4900i64, 5500, 5800, 6000] {
        let lam = C / (f as f64 * 1e6);
        let dl = 0.0455 / lam;
        println!(
            "  f={f} MHz  lambda={:5.1} mm  d/lambda={dl:5.3}  HPBW2el={:5.1} deg  steer-limit(no GL)={:5.1} deg  DoA-unambiguous=+-{:5.1} deg",
            lam * 1000.0,
            hpbw_2el_deg(dl),
            grating_steer_limit_deg(dl),
            unambiguous_doa_deg(dl)
        );
    }
    let lam915 = C / 915e6;
    println!(
        "  915 MHz lambda={:5.1} mm; pitch for identical d/lambda as 5800: {:5.0} mm; half-wave pitch: {:5.0} mm",
        lam915 * 1000.0,
        0.0455 * 5800.0 / 915.0 * 1000.0,
        lam915 / 2.0 * 1000.0
    );
    println!(
        "  Array gain over element, N=4: {:4.1} dB; N=72: {:4.1} dB; N=240: {:4.1} dB",
        10.0 * 4f64.log10(),
        10.0 * 72f64.log10(),
        10.0 * 240f64.log10()
    );
    line();
    println!("T10 Narrowband (phase-steer) validity: aperture transit time vs 1/BW");
    for (ap_m, bw) in [(0.1f64, 40e6f64), (0.3, 500e3), (0.6, 500e3), (1.0, 125e3)] {
        let tau = ap_m / C;
        println!(
            "  aperture {ap_m:3.1} m  tau={:5.2} ns  1/BW={:8.2} us  ratio={:>9}",
            tau * 1e9,
            1.0 / bw * 1e6,
            pye(tau * bw, 2)
        );
    }
    line();
    println!("T11 RF exposure (47 CFR 1.1310 Table 1, general population MPE): 300-1500 MHz -> f/1500 = 0.61 mW/cm2 at 915; 1500-100000 MHz -> 1.0 mW/cm2");
    for (name, eirp_dbm, s) in [
        ("36 dBm @915", 36.0f64, 0.61e-3f64),
        ("52 dBm @915", 52.0, 0.61e-3),
        ("64.6 dBm @915", 64.6, 0.61e-3),
        ("42 dBm @5800", 42.0, 1.0e-3),
        ("54.6 dBm @5800", 54.6, 1.0e-3),
    ] {
        println!(
            "  {name:16} compliance distance (far-field formula) = {:5.2} m",
            mpe_distance_m(10f64.powf(eirp_dbm / 10.0) / 1000.0, s)
        );
    }
    line();
    println!("T12 LoRa symbol/chip timing vs measured QuadRF-pair LO wander (800 Hz RMS)");
    for &(name, bw, sf, _cr) in PRESETS {
        let chips = (1u64 << sf) as f64;
        let ts = chips / (bw as f64 * 1e3);
        let binw = bw as f64 * 1e3 / chips;
        println!("  {name:14} Tsym={:7.2} ms  bin={binw:7.1} Hz  800Hz/bin={:6.2}", ts * 1e3, 800.0 / binw);
    }
    line();
    println!("T13 8-bit ADC quantisation vs thermal floor: with noise rms = k LSB, added noise power (dB)");
    for (k_str, k) in [("0.5", 0.5f64), ("  1", 1.0), ("  2", 2.0), ("  4", 4.0)] {
        let q = (1.0 / 12f64.sqrt()).powf(2.0);
        let n = k.powf(2.0);
        println!("  k={k_str}  10log10(1+q/n) = {:5.2} dB", 10.0 * (1.0 + q / n).log10());
    }
    line();
    // ------------------------------------------------------------------
    // Second-pass tables (2026-10-08): data paths, 915 MHz apertures, LO
    // quality, cascade NF from the MAX2851 datasheet, CPU budget, USB power.
    // ------------------------------------------------------------------
    println!("T14 Data-path budgets (bit/s), Pi 5 + QuadRF tile");
    let lane_mbps = 700.0f64; // [S] fpga-csi.dts: 350 MHz DDR -> 700 Mbps per lane
    let csi_gbps = 4.0 * lane_mbps / 1e3;
    println!(
        "  CSI-2 4 lanes x {lane_mbps:.0} Mbit/s = {csi_gbps:.2} Gbit/s raw; DSI same -> {:.1} Gbit/s aggregate (vendor: 5.6)",
        2.0 * csi_gbps
    );
    println!(
        "  RP1 MIPI aggregate (datasheet): 8 Gbit/s; PCIe 2.0 x4 to BCM2712: 4 x 5 GT/s x 8/10 = {:.0} Gbit/s; USB 3.0 per port 5 GT/s x 8/10 = 4 Gbit/s",
        4.0 * 5.0 * 0.8
    );
    for (label, ch, msps) in [("1 ch, 40 MSPS", 1i64, 40i64), ("4 ch interleaved, 26 MSPS", 4, 26), ("4 ch interleaved, 40 MSPS", 4, 40), ("PhaseGaze 1 ch, 38 MSPS", 1, 38)] {
        let gbps = (ch * msps) as f64 * 1e6 * 16.0 / 1e9;
        println!("  {label:28} CS8: {gbps:5.2} Gbit/s = {:6.0} MB/s  ({:4.0} % of CSI raw)", gbps / 8.0 * 1000.0, 100.0 * gbps / csi_gbps);
    }
    let frame_bytes: i64 = 1024 * 128; // [S] fpga-csi.dts geometry, RAW8
    let rate_bps = 4.0 * 26e6 * 2.0;
    let frame_us = frame_bytes as f64 / rate_bps * 1e6;
    println!(
        "  CSI frame = 1024 B x 128 lines = {frame_bytes} B; at 4 ch x 26 MSPS one frame every {frame_us:5.0} us; DMA_BUF_COUNT=16 -> {:4.1} ms of kernel-side buffering before loss",
        16.0 * frame_us / 1000.0
    );
    let kr = 5.0 * 2.4e6 * 2.0 * 8.0 / 1e6;
    println!("  KrakenSDR 5 ch x 2.4 MSPS x 2 B = {kr:5.0} Mbit/s ({:3.0} % of one USB 2.0 link)", 100.0 * kr / 480.0);
    line();
    println!("T15 915 MHz receive apertures: 4-element square (ours) and 5-element UCA (KrakenSDR rule s<=0.5, typical 0.33)");
    for pitch_mm in [164i64, 288] {
        let dl = pitch_mm as f64 / 1000.0 / lam915;
        println!(
            "  square pitch {pitch_mm} mm  d/lambda={dl:5.3}  HPBW2el={:5.1} deg  steer-limit={:5.1} deg  unambiguous=+-{:5.1} deg  diagonal={:4.0} mm",
            hpbw_2el_deg(dl),
            grating_steer_limit_deg(dl),
            unambiguous_doa_deg(dl),
            pitch_mm as f64 * 2f64.sqrt()
        );
    }
    for s in [0.33f64, 0.5] {
        let r = uca_radius_m(s, lam915, 5.0);
        let d = 2.0 * r;
        println!(
            "  UCA n=5 s={s:4.2}: spacing {:5.1} mm  radius {:5.1} mm  aperture {:5.1} mm  Rayleigh 1.22*lambda/D = {:5.1} deg (MUSIC ~10x finer per vendor)",
            s * lam915 * 1000.0,
            r * 1000.0,
            d * 1000.0,
            (1.22 * lam915 / d).to_degrees()
        );
    }
    line();
    println!("T16 Bearing precision (CRLB, two-element phase difference) vs calibration: sigma_theta = 1/(sqrt(N*SNR) * k d cos(theta))");
    let kd = 2.0 * PI * 0.5; // half-wave pitch
    for (snr_db, n) in [(0i64, 1024i64), (10, 1024), (20, 1024), (20, 16384)] {
        let sig = 1.0 / (n as f64 * lin(snr_db as f64)).sqrt() / kd;
        println!("  SNR {snr_db:3} dB, N={n:5} samples (broadside): sigma_theta = {:6.3} deg", sig.to_degrees());
    }
    println!("  -> thermal noise is not what limits the G05 5-degree criterion; static phase calibration and multipath are.");
    line();
    println!("T17 MAX2851 LO quality (datasheet 19-5121 Rev 1) vs 8-bit converter floor");
    let ipn_dbc = -35.0f64; // [S] integrated phase noise, 1 kHz-10 MHz, loop BW 200 kHz
    let rms_rad = (2.0 * lin(ipn_dbc)).sqrt();
    println!("  integrated phase noise {ipn_dbc:.0} dBc -> rms phase {rms_rad:.4} rad = {:.2} deg", rms_rad.to_degrees());
    let sfdr8 = 6.02 * 8.0 + 1.76;
    println!("  fractional spur level (0-19 MHz offset) -42 dBc typ [S]; 8-bit single-tone SFDR ~ {sfdr8:.1} dB -> spur is above the quantiser floor for a near-full-scale signal");
    for p_in in [-30.0f64, -60.0] {
        println!(
            "  strong in-band emitter at {p_in:4.0} dBm -> LO-spur replica at {:5.0} dBm, i.e. {:3.0} dB above a -130 dBm LoRa signal in the replica's slot",
            p_in - 42.0,
            p_in - 42.0 - (-130.0)
        );
    }
    let wander = 800.0f64;
    println!(
        "  measured pair wander 800 Hz rms at 5800 MHz = {:.3} ppm; PLL step 40e6/2^19 = {:.3} Hz [S]",
        wander / 5.8e9 * 1e6,
        40e6 / (1u64 << 19) as f64
    );
    line();
    println!("T18 Receive cascade NF: SKY65404-31 LNA (datasheet 201512K: NF 0.8/1.0/1.5 dB min/typ/max, gain 11/13/16 dB) ahead of the MAX2851 (4.5 dB DSB) [S]");
    for (label, lna_nf, lna_g, pre_loss) in [
        ("typ, no loss ahead", 1.0f64, 13.0f64, 0.0f64),
        ("typ, 0.5 dB switch ahead", 1.0, 13.0, 0.5),
        ("best corner, no loss", 0.8, 16.0, 0.0),
        ("worst corner, 0.5 dB", 1.5, 11.0, 0.5),
    ] {
        let f = lin(pre_loss) * (lin(lna_nf) + (lin(4.5) - 1.0) / lin(lna_g));
        println!("  {label:26} LNA NF {lna_nf:3.1} dB G {lna_g:4.1} dB -> system NF {:4.2} dB", db(f));
    }
    println!("  vendor states ~1.2 dB [S]; the datasheet-typical chain gives 1.30 dB (1.80 dB with 0.5 dB ahead); 1.2 dB is reached only near the best-case corner");
    for tile_nf in [1.3f64, 1.8] {
        let f = lin(1.0) + (lin(7.0) - 1.0) / lin(20.0) + (lin(tile_nf) - 1.0) / (lin(20.0) * lin(-7.0));
        println!(
            "  FTFE (915 MHz): LNA 1.0 dB/20 dB, mixer+filter loss 7 dB, tile {tile_nf:3.1} dB -> NF {:4.2} dB (no pre-filter); + 2 dB SAW ahead -> {:4.2} dB",
            db(f),
            db(f) + 2.0
        );
    }
    line();
    println!("T19 CPU budget for a whole-band channeliser on the Pi 5 (4x Cortex-A76, 2.4 GHz)");
    let (m, p) = (128i64, 8i64); // 128 bins over 26 MHz -> 203 kHz bins; P taps per phase
    let flops_per_sample = (4 * p) as f64 + 5.0 * (m as f64).log2();
    let per_ch = 26e6 * flops_per_sample;
    let peak = 2.4e9 * 16.0; // 2 FP pipes x 4-lane FMA x 2 flop
    println!(
        "  M={m} bins, P={p} taps: {flops_per_sample:.0} flop/sample -> {:4.2} GFLOP/s per channel, {:4.1} GFLOP/s for 4 channels",
        per_ch / 1e9,
        4.0 * per_ch / 1e9
    );
    println!(
        "  A76 NEON peak {:4.0} GFLOP/s per core; at 25 % efficiency {:4.0} GFLOP/s -> 4-ch channeliser = {:4.2} cores",
        peak / 1e9,
        0.25 * peak / 1e9,
        4.0 * per_ch / (0.25 * peak)
    );
    println!("  quadrf-mesh measured PHY cost 0.35 core at 8 MSPS single channel [S]; decoding <=4 selected 250 kHz slots from channeliser outputs is < 0.1 core each [C]");
    line();
    println!("T20 USB power on the Pi 5: peripherals budget 1.6 A at 5 V with a 5 A PD supply (0.6 A otherwise) [S Pi documentation]");
    for (name, a) in [("KrakenSDR (needs own supply)", 2.2f64), ("RTL-SDR v4", 0.3), ("HackRF One", 0.5), ("USB LoRa stick (CH341+SX1262, 22 dBm)", 0.2)] {
        println!("  {name:40} {a:3.1} A -> {} the 1.6 A budget", if a > 1.6 { "exceeds" } else { "within" });
    }
    line();
    println!("T21 Antenna-referred compression of the tile receive chain, and FTFE level plan");
    let lna_g = 13.0f64; // [S] SKY65404-31 typ
    println!("  LNA input P1dB -4 dBm [S]; MAX2851 input P1dB -34 dBm at max gain, -18 at max-16 dB, -1 at max-32 dB [S]");
    for (setting, p1) in [("max gain", -34.0f64), ("max - 16 dB", -18.0), ("max - 32 dB", -1.0)] {
        let ant = (p1 - lna_g).min(-4.0);
        println!("  tile RF gain {setting:12}: chain compresses at {ant:6.1} dBm referred to the element port (IC-limited unless the LNA's -4 dBm is lower)");
    }
    for (p_node_dbm, d_m) in [(22.0f64, 10.0f64), (22.0, 100.0), (30.0, 10.0)] {
        let p_ant = p_node_dbm + 2.15 - fspl(d_m / 1000.0, 915.0) + 2.15; // dipole-class antennas both ends
        let p_port = p_ant + 10.0; // FTFE net gain +10 dB (SPEC-007 S-007-5)
        let need = if p_port > -31.0 { "32" } else if p_port > -47.0 { "16" } else { "0" };
        println!(
            "  co-sited node {p_node_dbm:4.1} dBm at {d_m:5.1} m -> {p_ant:6.1} dBm at the 915 MHz antenna, {p_port:6.1} dBm at the element port after +10 dB FTFE gain -> needs tile RF gain <= max-{need} dB or the 30 dB FTFE pad"
        );
    }
    line();
    println!("T22 FTFE LO candidate: MAX2871 (datasheet 19-7106 Rev 4) at the translation LO; in-band floor = -230 + 20log10(N) + 10log10(fPFD) [S]+[D]");
    for (label, f_lo, f_pfd, mode) in [
        ("frac-N, 40 MHz PFD, 4585 MHz", 4585e6f64, 40e6f64, "fractional"),
        ("int-N, 20 MHz PFD, 4580 MHz", 4580e6, 20e6, "integer"),
        ("int-N, 5 MHz PFD, 4585 MHz", 4585e6, 5e6, "integer"),
        ("int-N, 40 MHz PFD, 4600 MHz", 4600e6, 40e6, "integer"),
    ] {
        let n = f_lo / f_pfd;
        let floor = -230.0 + 20.0 * n.log10() + 10.0 * f_pfd.log10();
        let onef_10k = -122.0 + 20.0 * (f_lo / 1e9).log10(); // 1/f term at 10 kHz offset (Note 7)
        let total_10k = db(lin(floor) + lin(onef_10k));
        println!(
            "  {label:32} N={n:8.3} ({mode:10}) in-band floor {floor:7.1} dBc/Hz; 1/f at 10 kHz {onef_10k:7.1}; total at 10 kHz {total_10k:7.1} dBc/Hz; band 915 MHz -> {:.0}-{:.0} MHz",
            (f_lo + 902e6) / 1e6,
            (f_lo + 928e6) / 1e6
        );
    }
    // rough integrated phase noise 1 kHz-10 MHz with a 150 kHz loop and the 4500 MHz VCO curve (-106 dBc/Hz at 100 kHz, -20 dB/dec)
    let floor = -230.0 + 20.0 * (4585e6 / 40e6f64).log10() + 10.0 * 40e6f64.log10();
    let inband = lin(floor) * (150e3 - 1e3);
    let vco_100k = -106.0f64;
    let outband = lin(vco_100k) * 100e3f64.powf(2.0) * (1.0 / 150e3 - 1.0 / 10e6);
    let ipn = db(inband + outband);
    println!(
        "  integrated 1 kHz-10 MHz (150 kHz loop, VCO -106 dBc/Hz at 100 kHz): ~{ipn:5.1} dBc -> rms {:5.2} deg, vs tile LO -35 dBc = 1.44 deg (T17)",
        (2.0 * lin(ipn)).sqrt().to_degrees()
    );
    println!("  PFD spurs -88 dBc at 50 kHz loop [S] vs tile LO fractional spurs -42 dBc [S]: the translator's own spurs are 46 dB below the tile's");
    println!("  REF_IN 10-210 MHz accepts the tile's 40 MHz reference if it can be exported (U-001-2) [S]; supply 3.3 V, <= 200 mA both outputs [S]");
    line();
    println!("T23 SX1261/2 datasheet (DS.SX1261-2.W.APP Rev 1.1, Table 3-8, Rx boosted gain) vs the T1 model; implied NF = P_sens + 174 - 10log10(BW) - SNRmin [S]+[D]");
    // sorted by (bw, sf) as Python's sorted(dict.items()) does
    let ds: [(f64, u32, f64); 8] = [
        (10.4e3, 7, -134.0),
        (10.4e3, 12, -148.0),
        (125e3, 7, -124.0),
        (125e3, 12, -137.0),
        (250e3, 7, -121.0),
        (250e3, 12, -134.0),
        (500e3, 7, -117.0),
        (500e3, 12, -129.0),
    ];
    for (bw, sf, ps) in ds {
        let nf_impl = ps + 174.0 - 10.0 * bw.log10() - snr_min(sf);
        println!(
            "  BW {:6.1} kHz SF{sf:2}: datasheet {ps:7.1} dBm; T1 model NF 6 dB {:7.1} dBm; implied NF+impl. loss {nf_impl:4.1} dB",
            bw / 1e3,
            sens(bw, sf, 6.0)
        );
    }
    println!("  -> the SX126x sensitivity model should use NF ~ 7 dB (6.5-8.0 implied) rather than 6 dB; T1/T7 margins for SX1262 receivers are 0.5-2 dB optimistic");
    println!(
        "  Meshtastic presets by SF-interpolation of the datasheet rows (2.6 dB per SF step at 250 kHz): LONG_FAST (250k/SF11) ~ {:6.1} dBm; SHORT_TURBO (500k/SF7) {:6.1} dBm; LONG_TURBO (500k/SF11) ~ {:6.1} dBm",
        -121.0 + (-134.0 + 121.0) / 5.0 * 4.0,
        -117.0f64,
        -117.0 + (-129.0 + 117.0) / 5.0 * 4.0
    );
    println!("  Tolerated Tx-Rx frequency offset: +/-25 % of BW (all SF); tighter ppm limits SF12 +/-50, SF11 +/-100, SF10 +/-200 ppm [S]:");
    for &(name, bw, sf, _cr) in PRESETS {
        let lim_bw = 0.25 * bw as f64 * 1e3;
        let lim_ppm: Option<i64> = match sf {
            12 => Some(50),
            11 => Some(100),
            10 => Some(200),
            _ => None,
        };
        let (lim, rule) = match lim_ppm {
            Some(ppm) => {
                let l = (ppm * 915) as f64;
                (lim_bw.min(l), if l < lim_bw { "ppm rule" } else { "25 % BW rule" })
            }
            None => (lim_bw, "25 % BW rule"),
        };
        println!(
            "    {name:14} limit {:6.2} kHz ({rule}); a 1 ppm free-running translator LO at 4585 MHz (4.6 kHz error, G05 F.05.2) uses {:4.1} % of it",
            lim / 1e3,
            100.0 * 4585.0 / lim
        );
    }
    println!("  SX126x synthesiser phase noise at 868/915 MHz: -75/-95/-100/-120/-135 dBc/Hz at 1k/10k/100k/1M/10M [S]; step 0.95 Hz; LDRO recommended for Tsym >= 16.38 ms [S]");
    line();
    println!("T24 SoapyRemote fallback (backlog P-03b): CS8 streamed from the vendor image over the Pi 5's Gigabit Ethernet (RP1 datasheet: GEM_GXL MAC, 10/100/1000 Mbps) [S]+[D]");
    let gbe = 1e9f64;
    // TCP payload per Ethernet wire slot: 1500 B MTU - 40 B IP+TCP headers, over 1500 + 14 + 4 + 8 + 12 B framing [D]
    let eff = 1460.0 / 1538.0f64;
    println!(
        "  usable TCP payload at 1500 B MTU: {:4.0} Mbit/s ({:4.1} % of line rate); a stream fits when its rate is below that",
        eff * gbe / 1e6,
        100.0 * eff
    );
    for (label, ch, rate) in [
        ("4 ch, 26 MSPS (hardware rate)", 4i64, 26e6f64),
        ("4 ch, 13 MSPS", 4, 13e6),
        ("4 ch, 6.5 MSPS", 4, 6.5e6),
        ("2 ch, 26 MSPS", 2, 26e6),
        ("1 ch, 26 MSPS", 1, 26e6),
    ] {
        let bps = ch as f64 * rate * 2.0 * 8.0;
        let fits = if bps <= eff * gbe { "fits" } else { "does not fit" };
        println!(
            "  {label:30} CS8 {:5.2} Gbit/s = {:4.0} % of line rate; band seen per channel {:5.1} MHz = {:4.0} of the 104 slots; {fits}",
            bps / 1e9,
            100.0 * bps / gbe,
            rate / 1e6,
            rate / 250e3
        );
    }
    println!("  -> whole-band coherent capture (4 ch x 26 MSPS) never crosses the LAN; the fallback gives the whole band on two channels or half of it on all four (83 %, marginal), or a quarter of it on all four with margin: enough for bring-up (P-04), the register observation (R-04) and a bearing check on a CW source, not for the G05 criterion (5) load test");

    line();
    println!("T25 Channeliser and detector design for qrf-dsp (backlog R-07): M=128 bins over 26 MHz, P=8 taps, 104 slots of 250 kHz, CA-CFAR [D]");
    let (m, p) = (128i64, 8i64);
    let fs = 26e6f64;
    let bin_hz = fs / m as f64;
    println!(
        "  bin width {:7.3} kHz; slot width 250 kHz = {:5.3} bins; block of M samples = {:5.3} us; {:7.3} k blocks/s per channel",
        bin_hz / 1e3,
        250e3 / bin_hz,
        m as f64 / fs * 1e6,
        fs / m as f64 / 1e3
    );
    // slot i centre: 902.125 + 0.25 i MHz (S-003-8: 104 slots, slot 20 at 906.875); capture centred on 915 MHz, bin k centre at k*bin_hz
    let mut counts: std::collections::BTreeMap<i64, i64> = std::collections::BTreeMap::new();
    for i in 0..104i64 {
        let lo = (902.125 + 0.25 * i as f64 - 0.125 - 915.0) * 1e6;
        let hi = lo + 250e3;
        let n = (-m / 2..m / 2).filter(|&k| lo <= k as f64 * bin_hz && k as f64 * bin_hz < hi).count() as i64;
        *counts.entry(n).or_insert(0) += 1;
    }
    let parts: Vec<String> = counts.iter().map(|(n, c)| format!("{c} slots with {n} bin{}", if *n != 1 { "s" } else { "" })).collect();
    println!("  bins whose centre falls in a slot: {}; every slot has >= 1 (S-008-6)", parts.join(", "));
    let l = m * p;
    let beta = 0.1102 * (60.0 - 8.7); // Kaiser beta for 60 dB sidelobes
    println!(
        "  prototype low-pass: windowed sinc, L = M*P = {l} taps, cutoff fs/(2M), Kaiser beta {beta:5.3} (60 dB sidelobes); main-lobe half-width ~ (beta^2+pi^2)^0.5/(pi*L) = {:5.3} bins",
        (beta * beta + PI * PI).sqrt() / (PI * l as f64) * m as f64
    );
    let n16 = 16i64;
    let a16 = n16 as f64 * (1e-3f64.powf(-1.0 / n16 as f64) - 1.0);
    println!(
        "  CA-CFAR, {n16} training slots of one look, Pfa 1e-03: alpha = {a16:6.3} = {:5.2} dB over the training mean (closed form N (Pfa^(-1/N) - 1); the ratio law below gives {:6.3})",
        db(a16),
        cfar_alpha_ca(1, 16, 1e-3)
    );
    for pfa in [1e-3f64, 1e-4] {
        let parts: Vec<String> = [1i64, 32, 203, 406].iter().map(|&k| { let a = cfar_alpha(k, pfa); format!("k={k:3}: {a:6.3} ({:5.2} dB)", db(a)) }).collect();
        println!("  CA-CFAR, slot power averaged over k looks, training mean taken as exact (N*k >> k), Pfa {}: {}", pye(pfa, 0), parts.join("; "));
    }
    println!("  one look = one block of M samples; 203 blocks = {:5.3} ms, the detector's look; a 2-bin slot averages 406", 203.0 * m as f64 / fs * 1e3);
    let h = pfb_prototype(m as usize, p as usize, beta);
    let keff = [0.0, pfb_effective_looks(&h, m as usize, p as usize, 1, 203), pfb_effective_looks(&h, m as usize, p as usize, 2, 203)];
    let lag1: f64 = (0..h.len() - m as usize).map(|n| h[n] * h[n + m as usize]).sum::<f64>().powi(2) / h.iter().map(|v| v * v).sum::<f64>().powi(2);
    println!(
        "  block outputs of one bin are correlated through the prototype (lag-1 |rho|^2 = {lag1:5.3}) and adjacent bins overlap: 203 outputs of a 1-bin slot = {:6.1} independent looks, 406 of a 2-bin slot = {:6.1}",
        keff[1], keff[2]
    );
    let nb: Vec<usize> = (0..104i64)
        .map(|i| {
            let lo = (902.125 + 0.25 * i as f64 - 0.125 - 915.0) * 1e6;
            (-m / 2..m / 2).filter(|&k| lo <= k as f64 * bin_hz && k as f64 * bin_hz < lo + 250e3).count()
        })
        .collect();
    let train_shape = |i: usize| -> (usize, f64) {
        let (guard, half) = (1usize, 8usize);
        let cells: Vec<usize> = (i.saturating_sub(guard + half)..i.saturating_sub(guard)).chain((i + guard + 1).min(104)..(i + guard + half + 1).min(104)).collect();
        let n = cells.len();
        (n, (n * n) as f64 / cells.iter().map(|&j| 1.0 / keff[nb[j]]).sum::<f64>())
    };
    let two = (20..80).find(|&i| nb[i] == 2).unwrap();
    let parts: Vec<String> = [("interior 1-bin slot 50", 50usize), ("interior 2-bin slot", two), ("edge slot 0", 0)]
        .iter()
        .map(|&(label, i)| {
            let (n, kk) = train_shape(i);
            let k = keff[nb[i]].round() as usize;
            let a = cfar_alpha_ca(k, kk.round() as usize, 1e-3);
            format!("{label} (k={k}, {n} training cells, K={kk:6.0}): alpha {a:5.3} ({:4.2} dB)", db(a))
        })
        .collect();
    println!("  the detector's threshold accounts for both the cell's looks and the training mean's spread (ratio law, Pfa 1e-03): {}", parts.join("; "));
    let kd = 2.0 * PI * 0.5;
    for (n, snr_db) in [(1024i64, 20i64), (4096, 20), (1024, 10)] {
        let s2 = 1.0 / (n as f64 * lin(snr_db as f64)).sqrt() / kd;
        println!(
            "  CRLB sigma at broadside, half-wave pitch, N={n:5}, SNR {snr_db:2} dB: one pair {:6.3} deg; two parallel pairs of a 2x2 square averaged {:6.3} deg (T16 formula)",
            s2.to_degrees(),
            (s2 / 2f64.sqrt()).to_degrees()
        );
    }
    println!(
        "  R-07 function check: bearing error (rms over trials) <= 0.1 deg at 20 dB SNR and N=1024 against the one-pair bound {:5.3} deg; a tone at a bin centre through the channeliser within 0.1 dB of its input power",
        (1.0 / (1024.0 * lin(20.0)).sqrt() / kd).to_degrees()
    );}
