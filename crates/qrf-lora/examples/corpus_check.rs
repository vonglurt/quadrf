//! The R-08 check: every cell of the LoRa corpus (docs/lora-corpus.md) through `qrf_lora::Demodulator`,
//! with the oracle's result on the same cell beside ours.
//!
//!     cargo run --release -p qrf-lora --example corpus_check -- [CORPUS_DIR] [--cells a,b] [--json OUT]
//!
//! Prints a Markdown table and a summary; exits 1 when an R-08 criterion fails: PER > 0.10 on either
//! `r08_*` cell, or any frame whose `crc_ok` verdict disagrees with the CRC recomputed over the payload it
//! delivered. A frame whose CRC passes with a wrong payload is a CRC collision (the LoRa CRC only XORs the
//! last two payload bytes into the check); it is printed and counted, not a failure of the receiver.

use num_complex::Complex;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Deserialize)]
struct Truth {
    preset: String,
    bw_hz: u32,
    sf: u8,
    cr: u8,
    coding_rate: String,
    ldro: bool,
    sync_word: u8,
    preamble_symbols: u16,
    crc: bool,
    oversampling: u8,
    snr_db_in_bandwidth: f64,
    n_frames: usize,
    frames: Vec<TruthFrame>,
}

#[derive(Deserialize)]
struct TruthFrame {
    index: usize,
    payload_hex: String,
    start: u64,
    cfo_hz: f64,
}

#[derive(Deserialize)]
struct Oracle {
    frames_with_header: usize,
    crc_ok: usize,
    correct: usize,
    per_strict: f64,
}

#[derive(Default, serde::Serialize)]
struct CellResult {
    cell: String,
    frames: usize,
    headers: usize,
    crc_ok: usize,
    correct: usize,
    crc_ok_but_wrong: usize,
    crc_failed: usize,
    /// Frames whose `crc_ok` verdict disagrees with recomputing the CRC over the delivered payload (must be 0).
    verdict_inconsistent: usize,
    per: f64,
    oracle_correct: Option<usize>,
    oracle_per: Option<f64>,
    cfo_err_hz_mean: f64,
    start_err_samples_mean: f64,
    snr_db_mean: f64,
    seconds: f64,
    detections: u64,
    sync_failures: u64,
    header_errors: u64,
}

fn hex(s: &str) -> Vec<u8> {
    (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()).collect()
}

fn preset_rank(name: &str) -> (usize, f64) {
    const ORDER: [&str; 7] = ["SHORT_TURBO", "SHORT_FAST", "MEDIUM_FAST", "LONG_TURBO", "LONG_FAST", "LONG_MODERATE", "LONG_SLOW"];
    let r08 = name.starts_with("r08_");
    let body = name.trim_start_matches("r08_");
    let (preset, snr) = body.rsplit_once("_snr").unwrap_or((body, "0"));
    let p = ORDER.iter().position(|&o| o == preset).unwrap_or(ORDER.len());
    (p + if r08 { 100 } else { 0 }, snr.parse().unwrap_or(0.0))
}

fn run_cell(dir: &Path) -> Result<CellResult, String> {
    let name = dir.file_name().unwrap().to_string_lossy().to_string();
    let truth: Truth = serde_json::from_reader(BufReader::new(File::open(dir.join("truth.json")).map_err(|e| e.to_string())?)).map_err(|e| e.to_string())?;
    let oracle: Option<Oracle> = File::open(dir.join("oracle.json")).ok().and_then(|f| serde_json::from_reader(BufReader::new(f)).ok());
    let mut cfg = qrf_lora::Config::meshtastic(truth.sf, truth.bw_hz, truth.cr);
    cfg.sync_word = truth.sync_word;
    cfg.preamble_len = truth.preamble_symbols;
    cfg.has_crc = truth.crc;
    cfg.oversampling = truth.oversampling;
    if cfg.ldro_enabled() != truth.ldro {
        return Err(format!("{name}: LDRO rule disagrees with the truth ({})", truth.ldro));
    }
    let mut demod = qrf_lora::Demodulator::new(cfg).map_err(|e| e.to_string())?;
    let want: HashMap<usize, (Vec<u8>, u64, f64)> = truth.frames.iter().map(|f| (f.index, (hex(&f.payload_hex), f.start, f.cfo_hz))).collect();
    let t0 = Instant::now();
    let mut file = BufReader::with_capacity(1 << 20, File::open(dir.join("iq.cs8")).map_err(|e| e.to_string())?);
    let mut raw = vec![0u8; 2 << 20];
    let mut frames = Vec::new();
    loop {
        let n = file.read(&mut raw).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        let samples: Vec<Complex<f32>> = qrf_lora::sim::from_cs8(&raw[..n - n % 2], 128.0);
        frames.extend(demod.process(&samples));
    }
    let seconds = t0.elapsed().as_secs_f64();
    let mut r = CellResult { cell: name, frames: truth.n_frames, headers: frames.len(), ..Default::default() };
    let mut correct = std::collections::BTreeSet::new();
    let (mut cfo_err, mut start_err, mut snr_sum) = (0.0, 0.0, 0.0);
    for f in &frames {
        if let Some(rx) = f.crc_received {
            if f.crc_ok != Some(rx == qrf_lora::coding::payload_crc(&f.payload)) {
                r.verdict_inconsistent += 1;
            }
        }
        match f.crc_ok {
            Some(true) => {
                r.crc_ok += 1;
                let idx = if f.payload.len() >= 2 { usize::from(f.payload[0]) << 8 | usize::from(f.payload[1]) } else { usize::MAX };
                match want.get(&idx) {
                    Some((p, start, cfo)) if *p == f.payload => {
                        if correct.insert(idx) {
                            cfo_err += (f.cfo_hz - cfo).abs();
                            start_err += (f.start_sample as f64 - *start as f64).abs();
                            snr_sum += f64::from(f.snr_db);
                        }
                    }
                    other => {
                        r.crc_ok_but_wrong += 1;
                        let truth_hex = other.map(|(p, _, _)| p.iter().map(|b| format!("{b:02x}")).collect::<String>()).unwrap_or_default();
                        let overlapping: Vec<String> = truth
                            .frames
                            .iter()
                            .filter(|t| f.start_sample + 1_000 >= t.start && f.start_sample < t.start + 8 * u64::from(truth.oversampling) * (1u64 << truth.sf) * 4)
                            .map(|t| format!("{}@{}", t.index, t.start))
                            .collect();
                        eprintln!(
                            "CRC-passing wrong payload in {}: index {idx} start {} end {} header {:?} snr {:.1} dB cfo {:.0} Hz errors {} corrections {}\n  got   {}\n  truth {}\n  nearby truth frames {:?}\n  symbols {:?}",
                            r.cell, f.start_sample, f.end_sample, f.header, f.snr_db, f.cfo_hz, f.codeword_errors, f.codeword_corrections,
                            f.payload.iter().map(|b| format!("{b:02x}")).collect::<String>(), truth_hex, overlapping, f.symbols
                        );
                    }
                }
            }
            Some(false) => r.crc_failed += 1,
            None => {}
        }
    }
    r.correct = correct.len();
    r.per = 1.0 - r.correct as f64 / r.frames.max(1) as f64;
    if r.correct > 0 {
        r.cfo_err_hz_mean = cfo_err / r.correct as f64;
        r.start_err_samples_mean = start_err / r.correct as f64;
        r.snr_db_mean = snr_sum / r.correct as f64;
    }
    r.oracle_correct = oracle.as_ref().map(|o| o.correct);
    r.oracle_per = oracle.as_ref().map(|o| o.per_strict);
    let _ = oracle.as_ref().map(|o| (o.frames_with_header, o.crc_ok));
    r.seconds = seconds;
    r.detections = demod.stats.detections;
    r.sync_failures = demod.stats.sync_failures;
    r.header_errors = demod.stats.header_errors;
    let _ = (&truth.preset, &truth.coding_rate, truth.snr_db_in_bandwidth);
    Ok(r)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut dir = PathBuf::from("resources/corpus/qrf-lora-v1");
    let mut cells: Option<Vec<String>> = None;
    let mut json_out: Option<PathBuf> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--cells" => {
                cells = Some(args[i + 1].split(',').map(str::to_string).collect());
                i += 1;
            }
            "--json" => {
                json_out = Some(PathBuf::from(&args[i + 1]));
                i += 1;
            }
            a => dir = PathBuf::from(a),
        }
        i += 1;
    }
    let mut names: Vec<String> = match cells {
        Some(c) => c,
        None => std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .filter_map(|e| e.ok())
            .filter(|e| e.path().join("truth.json").exists())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect(),
    };
    names.sort_by(|a, b| preset_rank(a).partial_cmp(&preset_rank(b)).unwrap());
    let mut truths: BTreeMap<String, Truth> = BTreeMap::new();
    println!("| Cell | SF | BW kHz | CR | LDRO | SNR dB | Frames | qrf-lora correct | CRC ok but wrong | PER | Oracle correct | Oracle PER | CFO err Hz | start err samples | s |");
    println!("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |");
    let mut results = Vec::new();
    let mut failed = false;
    for name in &names {
        let cell_dir = dir.join(name);
        let truth: Truth = serde_json::from_reader(BufReader::new(File::open(cell_dir.join("truth.json")).unwrap())).unwrap();
        match run_cell(&cell_dir) {
            Ok(r) => {
                let (oc, op) = match (r.oracle_correct, r.oracle_per) {
                    (Some(c), Some(p)) => (c.to_string(), format!("{p:.3}")),
                    _ => ("—".into(), "—".into()),
                };
                println!(
                    "| `{}` | {} | {} | {} | {} | {:+.1} | {} | {} | {} | {:.3} | {} | {} | {:.1} | {:.1} | {:.1} |",
                    r.cell, truth.sf, truth.bw_hz / 1000, truth.coding_rate, if truth.ldro { "on" } else { "off" }, truth.snr_db_in_bandwidth,
                    r.frames, r.correct, r.crc_ok_but_wrong, r.per, oc, op, r.cfo_err_hz_mean, r.start_err_samples_mean, r.seconds
                );
                if r.verdict_inconsistent > 0 {
                    failed = true;
                }
                if name.starts_with("r08_") && r.per > 0.10 {
                    failed = true;
                }
                results.push(r);
            }
            Err(e) => {
                eprintln!("{e}");
                failed = true;
            }
        }
        truths.insert(name.clone(), truth);
    }
    let total: usize = results.iter().map(|r| r.frames).sum();
    let ours: usize = results.iter().map(|r| r.correct).sum();
    let theirs: usize = results.iter().filter_map(|r| r.oracle_correct).sum();
    let wrong: usize = results.iter().map(|r| r.crc_ok_but_wrong).sum();
    let failed_crc: usize = results.iter().map(|r| r.crc_failed).sum();
    let inconsistent: usize = results.iter().map(|r| r.verdict_inconsistent).sum();
    println!();
    println!(
        "{} cells, {total} frames: qrf-lora {ours} correct, oracle {theirs} correct; {failed_crc} frames reported with a failed CRC, {inconsistent} CRC verdicts inconsistent with the delivered payload (criterion: 0), {wrong} CRC collisions (CRC passed, payload wrong; details above)",
        results.len()
    );
    for r in results.iter().filter(|r| r.cell.starts_with("r08_")) {
        println!("R-08 cell `{}`: PER {:.3} (criterion ≤ 0.100), oracle {}", r.cell, r.per, r.oracle_per.map_or("—".to_string(), |p| format!("{p:.3}")));
    }
    if let Some(p) = json_out {
        std::fs::write(&p, serde_json::to_string_pretty(&results).unwrap()).unwrap();
        println!("wrote {}", p.display());
    }
    println!("{}", if failed { "R-08: FAIL" } else { "R-08: PASS" });
    if failed {
        std::process::exit(1);
    }
}
