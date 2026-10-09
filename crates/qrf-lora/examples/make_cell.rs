//! A corpus cell made with `qrf_lora::Modulator` and the corpus channel model (docs/lora-corpus.md §2), in
//! the layout `scripts/make-corpus.py` reads, so that the oracle receiver can be run over frames this crate
//! synthesised (`make-corpus.py verify --out DIR --cells NAME`): the modulator half of the R-08 check.
//!
//!     cargo run --release -p qrf-lora --example make_cell -- OUT_DIR --preset LONG_FAST --snr -5 [--frames 50] [--seed 1] [--os 4] [--name NAME]

use num_complex::Complex;
use qrf_lora::sim::{apply_cfo, channel_filter, to_cs8, Fir, Rng};
use qrf_lora::{Config, Modulator};
use std::f64::consts::TAU;
use std::io::Write;
use std::path::PathBuf;

const PRESETS: [(&str, u32, u8, u8); 7] = [
    ("SHORT_TURBO", 500_000, 7, 1),
    ("SHORT_FAST", 250_000, 7, 1),
    ("MEDIUM_FAST", 250_000, 9, 1),
    ("LONG_TURBO", 500_000, 11, 4),
    ("LONG_FAST", 250_000, 11, 1),
    ("LONG_MODERATE", 125_000, 11, 4),
    ("LONG_SLOW", 125_000, 12, 4),
];
const NOISE_LSB: f64 = 20.0;
const CFO_PPM: f64 = 2.0;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut out = PathBuf::from("resources/corpus/qrf-lora-rs");
    let (mut preset, mut snr, mut frames, mut seed, mut os, mut name) = ("LONG_FAST".to_string(), -5.0f64, 50usize, 1u64, 4u8, None::<String>);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--preset" => preset = args[i + 1].clone(),
            "--snr" => snr = args[i + 1].parse().unwrap(),
            "--frames" => frames = args[i + 1].parse().unwrap(),
            "--seed" => seed = args[i + 1].parse().unwrap(),
            "--os" => os = args[i + 1].parse().unwrap(),
            "--name" => name = Some(args[i + 1].clone()),
            a => {
                out = PathBuf::from(a);
                i += 1;
                continue;
            }
        }
        i += 2;
    }
    let (_, bw, sf, cr) = *PRESETS.iter().find(|p| p.0 == preset).unwrap_or_else(|| panic!("unknown preset {preset}"));
    let name = name.unwrap_or_else(|| format!("rs_{preset}_snr{snr:+.1}"));
    let mut cfg = Config::meshtastic(sf, bw, cr);
    cfg.oversampling = os;
    let modem = Modulator::new(cfg.clone()).unwrap();
    let fs = cfg.sample_rate_hz();
    let sps = cfg.symbol_samples();
    let mut rng = Rng::new(seed.wrapping_mul(1_000_003).wrapping_add(u64::from(sf)));
    let snr_lin = 10f64.powf(snr / 10.0);
    let sigma_pre_sq = f64::from(os) / snr_lin; // unit signal power; noise power per sample before the filter
    let taps = channel_filter(usize::from(os));
    let sum_h2: f64 = taps.iter().map(|v| v * v).sum();
    let noise_out = (sigma_pre_sq * sum_h2).sqrt();
    let amp = NOISE_LSB / noise_out;
    let mut fir = Fir::new(taps.clone());
    let delay = fir.group_delay();
    let dir = out.join(&name);
    std::fs::create_dir_all(&dir).unwrap();
    let mut file = std::io::BufWriter::new(std::fs::File::create(dir.join("iq.cs8")).unwrap());
    let mut truth_frames = Vec::new();
    let mut pos: u64 = 0;
    let mut clipped = 0usize;
    let mut emit = |block: Vec<Complex<f64>>, file: &mut std::io::BufWriter<std::fs::File>, clipped: &mut usize| {
        let y = fir.process(&block);
        let (bytes, c) = to_cs8(&y, amp);
        *clipped += c;
        file.write_all(&bytes).unwrap();
    };
    for i in 0..frames {
        let len = rng.int(8, 32) as usize;
        let mut payload = vec![(i >> 8) as u8, i as u8];
        payload.extend((2..len).map(|_| rng.int(0, 255) as u8));
        let gap = (rng.range(0.5, 2.0) * sps as f64).round() as usize;
        let cfo = rng.range(-CFO_PPM, CFO_PPM) * 1e-6 * 915e6;
        let phase = rng.range(0.0, TAU);
        let noise_gap: Vec<Complex<f64>> = (0..gap).map(|_| rng.complex_noise(sigma_pre_sq)).collect();
        emit(noise_gap, &mut file, &mut clipped);
        pos += gap as u64;
        let clean = modem.modulate(&payload);
        let mut x = apply_cfo(&clean, cfo, fs, phase);
        for v in &mut x {
            *v += rng.complex_noise(sigma_pre_sq);
        }
        let length = x.len();
        emit(x, &mut file, &mut clipped);
        truth_frames.push(serde_json::json!({
            "index": i, "payload_hex": payload.iter().map(|b| format!("{b:02x}")).collect::<String>(),
            "start": pos + delay as u64, "length": length, "cfo_hz": cfo, "phase_rad": phase, "gap_before": gap,
        }));
        pos += length as u64;
    }
    let tail = sps + delay;
    let noise_tail: Vec<Complex<f64>> = (0..tail).map(|_| rng.complex_noise(sigma_pre_sq)).collect();
    emit(noise_tail, &mut file, &mut clipped);
    pos += tail as u64;
    file.flush().unwrap();
    let truth = serde_json::json!({
        "cell": name, "preset": preset, "bw_hz": bw, "sf": sf, "cr": cr, "coding_rate": format!("4/{}", cr + 4), "ldro": cfg.ldro_enabled(),
        "sync_word": cfg.sync_word, "preamble_symbols": cfg.preamble_len, "explicit_header": true, "crc": true,
        "sample_rate_hz": fs, "oversampling": os, "format": "CS8 interleaved I,Q, little endian int8",
        "snr_db_in_bandwidth": snr, "snr_db_per_sample_before_filter": snr - 10.0 * f64::from(os).log10(),
        "channel_filter": {"type": "kaiser low-pass on signal plus noise, unity DC gain", "taps": taps.len(), "group_delay_samples": delay, "cutoff_6db_bw": 0.6, "stopband_from_bw": 0.7, "stopband_db": 60, "enbw_bw": f64::from(os) * sum_h2},
        "noise_rms_lsb_per_complex_sample": NOISE_LSB, "lsb_per_unit_amplitude": amp, "clean_signal_power": 1.0,
        "clipped_samples": clipped, "total_samples": pos, "n_frames": frames, "cfo_ppm_range": CFO_PPM,
        "payload_len_range": [8, 32], "gap_symbols_range": [0.5, 2.0], "seed": seed,
        "generator": "crates/qrf-lora/examples/make_cell.rs (qrf_lora::Modulator)", "frames": truth_frames,
    });
    std::fs::write(dir.join("truth.json"), serde_json::to_string_pretty(&truth).unwrap() + "\n").unwrap();
    println!("wrote {} ({} frames, {:.1} MB, clipped {clipped})", dir.display(), frames, pos as f64 * 2.0 / 1e6);
}
