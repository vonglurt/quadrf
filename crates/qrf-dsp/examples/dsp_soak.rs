//! Backlog R-07 check: (1) the function record on simulated signals (the
//! prototype response and the channeliser's gain on tones, the detector's
//! false-alarm rate and sensitivity, the two bearing estimators against the
//! T16/T25 bound, timings); (2) the 60-s budget run: the R-03 mock tile at
//! the T14 cadence through `Reader`, the NEON de-interleave, the CS8
//! conversion, four channelisers, the four-element power sum, the CA-CFAR
//! detector and the duty estimator, with the consumer's CPU from the thread
//! clock and from `/proc/self/task/<tid>/stat`, the process's from
//! `/proc/self/stat`, and the channeliser's own share from its timed calls.
//!
//!   dsp_soak [--seconds 60] [--period-us 630] [--trials 1000] [--noise-dbfs -30] [--json out.json]
//!
//! The mock's frames are noiseless, so a seeded Gaussian floor of
//! `--noise-dbfs` per sample (a 64-frame buffer, cycled) is added after the
//! CS8 conversion to give the detector the statistics it is designed for.

use std::fs;
use std::time::{Duration, Instant};

use num_complex::Complex;
use qrf_dsp::cfar::{Detector, DetectorConfig, Duty, SlotMap, SlotPlan};
use qrf_dsp::channeliser::{Channeliser, FS_HZ, Kernel, M, P};
use qrf_dsp::convert::{CS8_SCALE, cs8_to_c32};
use qrf_dsp::filter::{KAISER_BETA_60DB, prototype, response_db};
use qrf_dsp::rng::Rng;
use qrf_dsp::{Array, Music, phase_difference};
use qrf_mipi::deinterleave::{buffers, deinterleave, views};
use qrf_mipi::frame::{ELEMENTS, FRAME_BYTES, SAMPLES_PER_FRAME};
use qrf_mipi::mock::frame_counter;
use qrf_mipi::{LossMonitor, MockConfig, MockDevice, Reader, SpanSource};
use serde::Serialize;

#[derive(Serialize)]
struct Report {
    host: String,
    kernel: &'static str,
    function: Function,
    soak: Soak,
}

#[derive(Serialize)]
struct Function {
    prototype: Prototype,
    cfar: Cfar,
    bearing: Vec<BearingTrials>,
    music_two_sources: TwoSources,
    timing: Timing,
}

#[derive(Serialize)]
struct Prototype {
    m: usize,
    p: usize,
    kaiser_beta: f64,
    response_db: Vec<(f64, f64)>,
    /// (bin, offset in bins, measured gain dB, prototype response dB)
    tones: Vec<(usize, f64, f64, f64)>,
    worst_centre_gain_db: f64,
    worst_off_centre_mismatch_db: f64,
}

#[derive(Serialize)]
struct Cfar {
    pfa_design: f64,
    n_avg: usize,
    looks: usize,
    slots: usize,
    false_alarms: u64,
    pfa_measured: f64,
    alpha_interior_one_bin: f32,
    alpha_interior_two_bin: f32,
    alpha_edge: f32,
    looks_interior_one_bin: (usize, usize),
    /// (tone power re the per-element noise power, dB; duty over the looks)
    sensitivity: Vec<(f64, f64)>,
    noise_per_bin_db_re_input: f64,
}

#[derive(Serialize)]
struct BearingTrials {
    method: &'static str,
    snr_db: f64,
    n: usize,
    trials: usize,
    theta_max_deg: f64,
    rms_deg: f64,
    max_deg: f64,
    mean_sigma_theta_deg: f64,
    crlb_one_pair_deg: f64,
    crlb_two_pairs_deg: f64,
    errors_deg: Vec<f32>,
}

#[derive(Serialize)]
struct TwoSources {
    sources: Vec<(f64, f64)>,
    n: usize,
    snr_db: f64,
    estimates: Vec<(f64, f64)>,
    spectrum_phi_at_theta: f64,
    spectrum_phi: Vec<(f64, f64)>,
}

#[derive(Serialize)]
struct Timing {
    block_scalar_ns: f64,
    block_fastest_ns: f64,
    frame_four_elements_us: f64,
    msps_per_core_four_elements: f64,
    convert_frame_us: f64,
    phase_difference_1024_us: f64,
    music_1024_us: f64,
}

#[derive(Serialize)]
struct Soak {
    seconds_requested: f64,
    seconds_elapsed: f64,
    frame_period_us: f64,
    frames_produced: u64,
    frames_consumed: u64,
    frames_dropped: u64,
    counter_gaps: u64,
    loss_events: u64,
    throughput_mb_s: f64,
    max_backlog_spans: u32,
    ring_capacity_spans: u32,
    interarrival_us: Percentiles,
    service_us: Percentiles,
    channeliser_us: Percentiles,
    consumer_cpu_cores_threadclock: f64,
    consumer_cpu_cores_procstat: f64,
    process_cpu_cores_procstat: f64,
    channeliser_cores: f64,
    convert_cores: f64,
    deinterleave_cores: f64,
    detector_cores: f64,
    /// (element, cycles per frame, nearest bin, offset in bins, measured dBFS, expected dBFS from the amplitude and the prototype response)
    tone_bins: Vec<(usize, u32, usize, f64, f64, f64)>,
    worst_tone_mismatch_db: f64,
    tone_slots: Vec<usize>,
    windows: Vec<Window>,
    tone_slots_min_duty: f32,
    other_slots_max_duty: f32,
    /// Mean duty of the 100 other slots over the full windows: the detector's false-alarm rate in the run.
    other_slots_mean_duty: f64,
    /// Median over the 100 non-tone slots of their mean duty: the false-alarm rate away from the template's spurs.
    other_slots_median_duty: f64,
    /// Non-tone slots at duty > 0.1 with their duty: a tone within half a bin of a slot edge also fills the neighbouring slot's bin (203 kHz bins against 250 kHz slots), and any quantisation spur of the mock's template above the floor.
    leakage_slots: Vec<(usize, f64)>,
    other_slots_detected: Vec<usize>,
    noise_dbfs: f64,
}

#[derive(Serialize)]
struct Window {
    t_s: f64,
    looks: usize,
    detected: Vec<usize>,
    tone_duty: Vec<f32>,
    tone_power_db: Vec<f32>,
    duty: Vec<f32>,
}

#[derive(Serialize, Default)]
struct Percentiles {
    min: f64,
    p50: f64,
    p99: f64,
    p999: f64,
    max: f64,
}

fn percentiles(v: &mut [f64]) -> Percentiles {
    if v.is_empty() {
        return Percentiles::default();
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let at = |q: f64| v[((v.len() - 1) as f64 * q).round() as usize];
    Percentiles { min: v[0], p50: at(0.5), p99: at(0.99), p999: at(0.999), max: v[v.len() - 1] }
}

fn thread_cpu() -> f64 {
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: a valid timespec for a clock every Linux provides.
    unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut ts) };
    ts.tv_sec as f64 + ts.tv_nsec as f64 * 1e-9
}

/// utime + stime in seconds from a `/proc/.../stat` line.
fn proc_stat_cpu(path: &str) -> f64 {
    let s = fs::read_to_string(path).unwrap_or_default();
    let rest = s.rsplit_once(')').map(|(_, r)| r).unwrap_or("");
    let f: Vec<&str> = rest.split_whitespace().collect();
    // after ')' the fields are state(3) ... utime(14) stime(15): indices 11 and 12 here
    let ticks: u64 = f.get(11).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0) + f.get(12).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
    // SAFETY: sysconf has no preconditions.
    let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.max(1) as f64;
    ticks as f64 / hz
}

fn tone(n: usize, cycles_per_block: f64, amp: f32) -> Vec<Complex<f32>> {
    (0..n).map(|i| { let a = 2.0 * std::f64::consts::PI * cycles_per_block * i as f64 / M as f64; Complex::new(amp * a.cos() as f32, amp * a.sin() as f32) }).collect()
}

fn mean_bin_power(ch: &mut Channeliser, x: &[Complex<f32>], k: usize) -> f64 {
    let mut out = vec![Complex::new(0.0, 0.0); x.len()];
    let blocks = ch.process(x, &mut out);
    let s: f64 = (P..blocks).map(|b| out[b * M + k].norm_sqr() as f64).sum();
    ch.reset();
    s / (blocks - P) as f64
}

fn function_record(trials: usize) -> Function {
    let h = prototype(M, P, KAISER_BETA_60DB);
    let response: Vec<(f64, f64)> = (0..=200).map(|i| { let o = -2.0 + i as f64 * 0.02; (o, response_db(&h, M, o)) }).collect();
    let mut ch = Channeliser::new(M, P);
    let mut tones = Vec::new();
    let (mut worst_centre, mut worst_off) = (0.0f64, 0.0f64);
    for (k, off) in [(0usize, 0.0f64), (1, 0.0), (17, 0.0), (63, 0.0), (64, 0.0), (100, 0.0), (127, 0.0), (10, 0.1), (10, 0.25), (10, 0.5), (90, -0.25), (90, -0.5)] {
        let cycles = if k < M / 2 { k as f64 } else { k as f64 - M as f64 } + off;
        let x = tone(64 * M, cycles, 0.5);
        let g = 10.0 * (mean_bin_power(&mut ch, &x, k) / 0.25).log10();
        let want = response_db(&h, M, off);
        tones.push((k, off, g, want));
        if off == 0.0 { worst_centre = worst_centre.max(g.abs()) } else { worst_off = worst_off.max((g - want).abs()) }
    }
    let prototype = Prototype { m: M, p: P, kaiser_beta: KAISER_BETA_60DB, response_db: response, tones, worst_centre_gain_db: worst_centre, worst_off_centre_mismatch_db: worst_off };

    // CFAR: noise only, one element, the default detector; then a tone at slot 19 at several levels
    let mut rng = Rng::new(20_261_008);
    let map = SlotMap::new(SlotPlan::US_104, 915e6, FS_HZ, M);
    let cfg = DetectorConfig::default();
    let looks = 2000usize;
    let mut det = Detector::with_prototype(map.clone(), cfg, &h, M, P);
    let mut out = vec![Complex::new(0.0, 0.0); M];
    let mut false_alarms = 0u64;
    let mut noise_bin = 0.0f64;
    let mut noise_blocks = 0u64;
    for _ in 0..looks * cfg.n_avg {
        let x = rng.complex_gaussian_f32(M, 1.0);
        ch.push_block(&x, &mut out);
        noise_bin += out.iter().map(|z| z.norm_sqr() as f64).sum::<f64>() / M as f64;
        noise_blocks += 1;
        if let Some(l) = det.push_bins(&out) {
            false_alarms += l.detected.iter().filter(|&&d| d).count() as u64;
        }
    }
    let noise_per_bin_db = 10.0 * (noise_bin / noise_blocks as f64).log10();
    let mut sensitivity = Vec::new();
    let slot19 = 19usize;
    let bin19 = map.bins[slot19][0];
    let cycles19 = if bin19 < M / 2 { bin19 as f64 } else { bin19 as f64 - M as f64 };
    for level_db in [-35.0f64, -30.0, -27.0, -25.0, -23.0, -21.0, -19.0, -17.0, -15.0, -10.0] {
        ch.reset();
        let mut det = Detector::with_prototype(map.clone(), cfg, &h, M, P);
        let amp = 10f64.powf(level_db / 20.0) as f32;
        let t = tone(M, cycles19, amp);
        let (mut hits, mut n) = (0u64, 0u64);
        for _ in 0..200 * cfg.n_avg {
            let mut x = rng.complex_gaussian_f32(M, 1.0);
            for (a, b) in x.iter_mut().zip(&t) {
                *a += b;
            }
            ch.push_block(&x, &mut out);
            if let Some(l) = det.push_bins(&out) {
                hits += u64::from(l.detected[slot19]);
                n += 1;
            }
        }
        sensitivity.push((level_db, hits as f64 / n as f64));
    }
    let two = (20..80).find(|&i| map.bins[i].len() == 2).unwrap();
    let cfar = Cfar { pfa_design: cfg.pfa, n_avg: cfg.n_avg, looks, slots: 104, false_alarms, pfa_measured: false_alarms as f64 / (looks * 104) as f64, alpha_interior_one_bin: det.alpha(50), alpha_interior_two_bin: det.alpha(two), alpha_edge: det.alpha(0), looks_interior_one_bin: det.looks(50), sensitivity, noise_per_bin_db_re_input: noise_per_bin_db };

    // bearings: both estimators, SNR 20/10/0 dB, N = 1024, θ within 30° of the normal
    let a = Array::half_wave(915e6);
    let music = Music::new(a, 915e6, 1);
    let kd = std::f64::consts::PI;
    let mut bearing = Vec::new();
    for (method, snr_db) in [("phase_difference", 20.0f64), ("music", 20.0), ("phase_difference", 10.0), ("music", 10.0), ("phase_difference", 0.0)] {
        let n = 1024usize;
        let tr = if method == "music" { trials.min(300) } else { trials };
        let mut errors = Vec::with_capacity(tr);
        let mut sig = 0.0;
        for t in 0..tr {
            let (theta, phi) = (30.0 * (t as f64 + 0.5) / tr as f64, rng.range(-180.0, 180.0));
            let x = a.plane_wave(915e6, theta, phi, n, snr_db, &mut rng);
            let v = [&x[0][..], &x[1][..], &x[2][..], &x[3][..]];
            let b = if method == "music" { music.estimate(v)[0] } else { phase_difference(&a, 915e6, v) };
            errors.push(Array::separation_deg((b.theta_deg, b.phi_deg), (theta, phi)) as f32);
            sig += b.sigma_theta_deg;
        }
        let rms = (errors.iter().map(|&e| (e as f64).powi(2)).sum::<f64>() / tr as f64).sqrt();
        let max = errors.iter().cloned().fold(0.0f32, f32::max) as f64;
        let snr = 10f64.powf(snr_db / 10.0);
        let crlb1 = (1.0 / (n as f64 * snr).sqrt() / kd).to_degrees();
        bearing.push(BearingTrials { method, snr_db, n, trials: tr, theta_max_deg: 30.0, rms_deg: rms, max_deg: max, mean_sigma_theta_deg: sig / tr as f64, crlb_one_pair_deg: crlb1, crlb_two_pairs_deg: crlb1 / 2f64.sqrt(), errors_deg: errors });
    }
    let sources = vec![(20.0f64, -60.0f64), (25.0, 100.0)];
    let x = a.plane_waves(915e6, &[(20.0, -60.0, 1.0), (25.0, 100.0, 1.0)], 4096, 0.01, &mut rng);
    let v = [&x[0][..], &x[1][..], &x[2][..], &x[3][..]];
    let m2 = Music::new(a, 915e6, 2);
    let est = m2.estimate(v).iter().map(|b| (b.theta_deg, b.phi_deg)).collect();
    let spectrum_phi = m2.spectrum_phi(v, 22.5, 0.5);
    let music_two_sources = TwoSources { sources, n: 4096, snr_db: 20.0, estimates: est, spectrum_phi_at_theta: 22.5, spectrum_phi };

    // timings in cache
    let x = rng.complex_gaussian_f32(SAMPLES_PER_FRAME, 0.1);
    let mut outf = vec![Complex::new(0.0, 0.0); SAMPLES_PER_FRAME];
    let time = |f: &mut dyn FnMut(), reps: usize| -> f64 {
        for _ in 0..reps / 10 + 1 {
            f();
        }
        let t = Instant::now();
        for _ in 0..reps {
            f();
        }
        t.elapsed().as_secs_f64() / reps as f64
    };
    let mut chs = Channeliser::new(M, P);
    chs.set_kernel(Kernel::Scalar);
    let block_scalar_ns = time(&mut || chs.push_block(&x[..M], &mut outf[..M]), 20_000) * 1e9;
    let mut chf = Channeliser::new(M, P);
    let block_fastest_ns = time(&mut || chf.push_block(&x[..M], &mut outf[..M]), 20_000) * 1e9;
    let mut four: Vec<Channeliser> = (0..4).map(|_| Channeliser::new(M, P)).collect();
    let frame_four_elements_us = time(&mut || { for c in four.iter_mut() { c.process(&x, &mut outf); } }, 200) * 1e6;
    let src: Vec<u8> = (0..2 * SAMPLES_PER_FRAME).map(|_| rng.next_u64() as u8).collect();
    let convert_frame_us = time(&mut || { cs8_to_c32(&src, CS8_SCALE, &mut outf); }, 2000) * 1e6;
    let xb = a.plane_wave(915e6, 12.0, 5.0, 1024, 20.0, &mut rng);
    let vb = [&xb[0][..], &xb[1][..], &xb[2][..], &xb[3][..]];
    let phase_difference_1024_us = time(&mut || { std::hint::black_box(phase_difference(&a, 915e6, vb)); }, 2000) * 1e6;
    let music_1024_us = time(&mut || { std::hint::black_box(music.estimate(vb)); }, 50) * 1e6;
    let timing = Timing { block_scalar_ns, block_fastest_ns, frame_four_elements_us, msps_per_core_four_elements: 4.0 * SAMPLES_PER_FRAME as f64 / frame_four_elements_us, convert_frame_us, phase_difference_1024_us, music_1024_us };
    Function { prototype, cfar, bearing, music_two_sources, timing }
}

fn soak(seconds: f64, period_us: f64, noise_dbfs: f64) -> Soak {
    let period = Duration::from_nanos((period_us * 1000.0) as u64);
    let frames = (seconds * 1e6 / period_us) as u64;
    let cfg = MockConfig { frame_period: period, frames: Some(frames), ..Default::default() };
    let mut dev = MockDevice::new(cfg);
    let tones = *dev.tones();
    let amplitude = dev.config().amplitude;
    dev.start();
    let mut r = Reader::new(dev).unwrap();
    let capacity = r.source().ring_info().unwrap().capacity_spans();
    let mut bufs = buffers(SAMPLES_PER_FRAME);
    let mut c32: Vec<Vec<Complex<f32>>> = (0..ELEMENTS).map(|_| vec![Complex::new(0.0, 0.0); SAMPLES_PER_FRAME]).collect();
    let mut bins: Vec<Vec<Complex<f32>>> = (0..ELEMENTS).map(|_| vec![Complex::new(0.0, 0.0); SAMPLES_PER_FRAME]).collect();
    let mut chs: Vec<Channeliser> = (0..ELEMENTS).map(|_| Channeliser::new(M, P)).collect();
    let h = prototype(M, P, KAISER_BETA_60DB);
    let map = SlotMap::new(SlotPlan::US_104, 915e6, FS_HZ, M);
    let dcfg = DetectorConfig { looks_per_block: ELEMENTS, ..Default::default() };
    let mut det = Detector::with_prototype(map.clone(), dcfg, &h, M, P);
    let mut duty = Duty::new(&SlotPlan::US_104, 1000);
    let noise_frames = 64usize;
    let noise: Vec<Complex<f32>> = Rng::new(20_261_008).complex_gaussian_f32(noise_frames * SAMPLES_PER_FRAME, 10f64.powf(noise_dbfs / 10.0));
    let mut noise_at = 0usize;
    let tone_slots: Vec<usize> = tones.iter().map(|t| map.slot_for_offset(t.freq_hz(FS_HZ), 915e6).unwrap()).collect();
    let tone_bin: Vec<usize> = tones.iter().map(|t| chs[0].bin_for_offset(t.freq_hz(FS_HZ), FS_HZ)).collect();
    let mut tone_acc = vec![0.0f64; ELEMENTS];
    let mut tone_blocks = 0u64;
    let mut power = vec![0.0f32; M];
    let mut loss = LossMonitor::new();
    let (mut expect, mut gaps) = (0u64, 0u64);
    let mut max_backlog = 0u32;
    let mut inter = Vec::with_capacity(frames as usize);
    let mut service = Vec::with_capacity(frames as usize);
    let mut chan_us = Vec::with_capacity(frames as usize);
    let (mut t_chan, mut t_conv, mut t_deint, mut t_det) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
    let mut windows = Vec::new();
    // SAFETY: gettid has no preconditions.
    let tid = unsafe { libc::gettid() };
    let task_stat = format!("/proc/self/task/{tid}/stat");
    let cpu0 = thread_cpu();
    let (ps0, ts0) = (proc_stat_cpu("/proc/self/stat"), proc_stat_cpu(&task_stat));
    let t0 = Instant::now();
    let mut last_arrival: Option<Instant> = None;
    let mut last_stats = t0;
    while let Some(sp) = r.next(Duration::from_millis(500)).unwrap() {
        let arrived = Instant::now();
        if let Some(p) = last_arrival {
            inter.push((arrived - p).as_secs_f64() * 1e6);
        }
        last_arrival = Some(arrived);
        let c = frame_counter(sp.data);
        if c != expect {
            gaps += 1;
            expect = c;
        }
        expect += 1;
        let t1 = Instant::now();
        deinterleave(sp.data, &mut views(&mut bufs));
        let t2 = Instant::now();
        for e in 0..ELEMENTS {
            cs8_to_c32(&bufs[e], CS8_SCALE, &mut c32[e]);
            let off = (noise_at + e * noise_frames / ELEMENTS) % noise_frames * SAMPLES_PER_FRAME;
            for (a, b) in c32[e].iter_mut().zip(&noise[off..off + SAMPLES_PER_FRAME]) {
                *a += b;
            }
        }
        noise_at = (noise_at + 1) % noise_frames;
        let t3 = Instant::now();
        for e in 0..ELEMENTS {
            chs[e].process(&c32[e], &mut bins[e]);
        }
        let t4 = Instant::now();
        for b in 0..SAMPLES_PER_FRAME / M {
            // sample 0 of the frame carries the mock's counter: skip block 0 in the tone check
            power.iter_mut().for_each(|p| *p = 0.0);
            for e in 0..ELEMENTS {
                let row = &bins[e][b * M..(b + 1) * M];
                for (p, z) in power.iter_mut().zip(row) {
                    *p += z.norm_sqr();
                }
                if b >= P {
                    tone_acc[e] += row[tone_bin[e]].norm_sqr() as f64;
                }
            }
            if b >= P {
                tone_blocks += 1;
            }
            if let Some(look) = det.push_power(&power) {
                if let Some(occ) = duty.update(&look) {
                    let detected: Vec<usize> = occ.slots.iter().enumerate().filter(|(_, s)| s.duty > 0.0).map(|(i, _)| i).collect();
                    windows.push(Window { t_s: t0.elapsed().as_secs_f64(), looks: occ.looks, detected, tone_duty: tone_slots.iter().map(|&s| occ.slots[s].duty).collect(), tone_power_db: tone_slots.iter().map(|&s| occ.slots[s].power_db).collect(), duty: occ.slots.iter().map(|s| s.duty).collect() });
                }
            }
        }
        let t5 = Instant::now();
        t_deint += (t2 - t1).as_secs_f64();
        t_conv += (t3 - t2).as_secs_f64();
        t_chan += (t4 - t3).as_secs_f64();
        t_det += (t5 - t4).as_secs_f64();
        chan_us.push((t4 - t3).as_secs_f64() * 1e6);
        service.push(arrived.elapsed().as_secs_f64() * 1e6);
        let seq = sp.seq;
        if seq % 100 == 0 {
            max_backlog = max_backlog.max(r.backlog().unwrap());
        }
        if last_stats.elapsed() >= Duration::from_secs(1) {
            last_stats = Instant::now();
            let (s, e) = (r.source().stats().unwrap(), r.source().events().unwrap());
            if let Some(ev) = loss.observe(seq, s, e) {
                eprintln!("loss event: {ev:?}");
            }
        }
    }
    let elapsed = t0.elapsed().as_secs_f64();
    let cpu = thread_cpu() - cpu0;
    let (ps, ts) = (proc_stat_cpu("/proc/self/stat") - ps0, proc_stat_cpu(&task_stat) - ts0);
    let consumed = r.delivered();
    let src = r.into_source();
    let tone_bins: Vec<(usize, u32, usize, f64, f64, f64)> = tones
        .iter()
        .enumerate()
        .map(|(e, t)| {
            let cycles_per_block = t.bin as f64 * M as f64 / SAMPLES_PER_FRAME as f64;
            let centre = if tone_bin[e] < M / 2 { tone_bin[e] as f64 } else { tone_bin[e] as f64 - M as f64 };
            let off = cycles_per_block - centre;
            let measured = 10.0 * (tone_acc[e] / tone_blocks.max(1) as f64).log10();
            let expected = 20.0 * (amplitude / 128.0).log10() + response_db(&h, M, off);
            (e, t.bin, tone_bin[e], off, measured, expected)
        })
        .collect();
    let worst_tone = tone_bins.iter().map(|t| (t.4 - t.5).abs()).fold(0.0, f64::max);
    let full: Vec<&Window> = windows.iter().filter(|w| w.t_s > 2.5).collect();
    let tone_slots_min_duty = full.iter().flat_map(|w| w.tone_duty.iter().cloned()).fold(1.0f32, f32::min);
    let mut others: Vec<usize> = full.iter().flat_map(|w| w.detected.iter().cloned()).filter(|s| !tone_slots.contains(s)).collect();
    others.sort_unstable();
    others.dedup();
    let other_max = full.iter().flat_map(|w| w.duty.iter().enumerate().filter(|(i, _)| !tone_slots.contains(i)).map(|(_, &d)| d)).fold(0.0f32, f32::max);
    let other_mean = full.iter().flat_map(|w| w.duty.iter().enumerate().filter(|(i, _)| !tone_slots.contains(i)).map(|(_, &d)| d as f64)).sum::<f64>() / (full.len().max(1) * 100) as f64;
    // per-slot mean duty over the full windows; the median over the 100 non-tone slots is the run's false-alarm rate away from the template's quantisation spurs
    let per_slot: Vec<f64> = (0..104).map(|i| full.iter().map(|w| w.duty[i] as f64).sum::<f64>() / full.len().max(1) as f64).collect();
    let mut others_sorted: Vec<f64> = per_slot.iter().enumerate().filter(|(i, _)| !tone_slots.contains(i)).map(|(_, &d)| d).collect();
    others_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let other_median = others_sorted.get(others_sorted.len() / 2).copied().unwrap_or(0.0);
    let leakage_slots: Vec<(usize, f64)> = per_slot.iter().enumerate().filter(|(i, d)| !tone_slots.contains(i) && **d > 0.1).map(|(i, d)| (i, *d)).collect();
    Soak {
        seconds_requested: seconds,
        seconds_elapsed: elapsed,
        frame_period_us: period_us,
        frames_produced: src.produced(),
        frames_consumed: consumed,
        frames_dropped: src.dropped(),
        counter_gaps: gaps,
        loss_events: loss.events(),
        throughput_mb_s: consumed as f64 * FRAME_BYTES as f64 / elapsed / 1e6,
        max_backlog_spans: max_backlog,
        ring_capacity_spans: capacity,
        interarrival_us: percentiles(&mut inter),
        service_us: percentiles(&mut service),
        channeliser_us: percentiles(&mut chan_us),
        consumer_cpu_cores_threadclock: cpu / elapsed,
        consumer_cpu_cores_procstat: ts / elapsed,
        process_cpu_cores_procstat: ps / elapsed,
        channeliser_cores: t_chan / elapsed,
        convert_cores: t_conv / elapsed,
        deinterleave_cores: t_deint / elapsed,
        detector_cores: t_det / elapsed,
        tone_bins,
        worst_tone_mismatch_db: worst_tone,
        tone_slots,
        windows,
        tone_slots_min_duty,
        other_slots_max_duty: other_max,
        other_slots_mean_duty: other_mean,
        other_slots_median_duty: other_median,
        leakage_slots,
        other_slots_detected: others,
        noise_dbfs,
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (mut seconds, mut period_us, mut json, mut trials, mut noise_dbfs) = (60.0f64, 630.0f64, None, 1000usize, -30.0f64);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--seconds" => seconds = args.next().unwrap().parse().unwrap(),
            "--period-us" => period_us = args.next().unwrap().parse().unwrap(),
            "--json" => json = args.next(),
            "--trials" => trials = args.next().unwrap().parse().unwrap(),
            "--noise-dbfs" => noise_dbfs = args.next().unwrap().parse().unwrap(),
            other => panic!("unknown argument {other}"),
        }
    }
    eprintln!("function record ({trials} bearing trials)…");
    let function = function_record(trials);
    println!("channeliser: worst centre gain {:.4} dB, worst off-centre mismatch {:.4} dB", function.prototype.worst_centre_gain_db, function.prototype.worst_off_centre_mismatch_db);
    println!("cfar: design Pfa {:.0e}, measured {:.2e} ({} false alarms in {} slot-looks); alpha interior 1-bin {:.3} (k, K = {:?}), 2-bin {:.3}, edge {:.3}", function.cfar.pfa_design, function.cfar.pfa_measured, function.cfar.false_alarms, function.cfar.looks * 104, function.cfar.alpha_interior_one_bin, function.cfar.looks_interior_one_bin, function.cfar.alpha_interior_two_bin, function.cfar.alpha_edge);
    for b in &function.bearing {
        println!("bearing {:16} SNR {:2} dB N {}: rms {:.4}° max {:.4}° (reported σθ {:.4}°; CRLB one pair {:.4}°, two pairs {:.4}°)", b.method, b.snr_db, b.n, b.rms_deg, b.max_deg, b.mean_sigma_theta_deg, b.crlb_one_pair_deg, b.crlb_two_pairs_deg);
    }
    println!("music two sources {:?} -> {:?}", function.music_two_sources.sources, function.music_two_sources.estimates);
    let t = &function.timing;
    println!("timing: block scalar {:.0} ns, fastest {:.0} ns; frame x4 {:.0} µs = {:.1} MSPS/core; convert {:.1} µs; phase_difference {:.1} µs; music {:.0} µs", t.block_scalar_ns, t.block_fastest_ns, t.frame_four_elements_us, t.msps_per_core_four_elements, t.convert_frame_us, t.phase_difference_1024_us, t.music_1024_us);
    eprintln!("soak {seconds} s at {period_us} µs per frame…");
    let soak = soak(seconds, period_us, noise_dbfs);
    println!("frames produced {} consumed {} dropped {} gaps {} loss events {}; {:.1} MB/s; max backlog {} of {}", soak.frames_produced, soak.frames_consumed, soak.frames_dropped, soak.counter_gaps, soak.loss_events, soak.throughput_mb_s, soak.max_backlog_spans, soak.ring_capacity_spans);
    println!("service µs p50 {:.0} p99 {:.0} max {:.0}; channeliser µs per frame p50 {:.0} p99 {:.0} max {:.0}", soak.service_us.p50, soak.service_us.p99, soak.service_us.max, soak.channeliser_us.p50, soak.channeliser_us.p99, soak.channeliser_us.max);
    println!("cpu cores: consumer thread {:.3} (procstat {:.3}), process {:.3}; channeliser {:.3}, convert {:.3}, deinterleave {:.3}, detector {:.3}", soak.consumer_cpu_cores_threadclock, soak.consumer_cpu_cores_procstat, soak.process_cpu_cores_procstat, soak.channeliser_cores, soak.convert_cores, soak.deinterleave_cores, soak.detector_cores);
    for t in &soak.tone_bins {
        println!("tone element {} ({} cycles/frame) -> bin {} offset {:+.3} bins: measured {:.3} dBFS, expected {:.3} dBFS", t.0, t.1, t.2, t.3, t.4, t.5);
    }
    println!("tone slots {:?}: min duty {:.3}; other slots: median duty {:.2e} (design Pfa 1e-3), mean {:.2e}, max {:.3}; slots at duty > 0.1 (tone leakage into the neighbouring bin) {:?}; ever detected {:?}", soak.tone_slots, soak.tone_slots_min_duty, soak.other_slots_median_duty, soak.other_slots_mean_duty, soak.other_slots_max_duty, soak.leakage_slots, soak.other_slots_detected);
    let rep = Report { host: format!("{} {}", std::env::consts::ARCH, fs::read_to_string("/proc/cpuinfo").ok().and_then(|s| s.lines().find(|l| l.starts_with("CPU part") || l.starts_with("model name")).map(|l| l.trim().to_string())).unwrap_or_default()), kernel: Kernel::fastest().name(), function, soak };
    if let Some(p) = json {
        if let Some(dir) = std::path::Path::new(&p).parent() {
            fs::create_dir_all(dir).unwrap();
        }
        fs::write(&p, serde_json::to_string_pretty(&rep).unwrap()).unwrap();
        println!("wrote {p}");
    }
    let f = &rep.function;
    let ok_function = f.prototype.worst_centre_gain_db <= 0.1 && f.bearing.iter().filter(|b| b.snr_db == 20.0).all(|b| b.rms_deg <= 0.1);
    let ok_chain = rep.soak.worst_tone_mismatch_db <= 0.1 && rep.soak.tone_slots_min_duty >= 1.0 && rep.soak.other_slots_median_duty < 5e-3;
    let lossless = rep.soak.counter_gaps == 0 && rep.soak.frames_dropped == 0 && rep.soak.frames_consumed == rep.soak.frames_produced;
    println!(
        "R-07 function check: {}; chain on the mock: {}; cadence lossless: {} (R-03's criterion, informative here); budget (Pi 5 only): channeliser {:.3} core, consumer thread {:.3} core on this host",
        if ok_function { "PASS" } else { "FAIL" },
        if ok_chain { "PASS" } else { "FAIL" },
        if lossless { "yes" } else { "no" },
        rep.soak.channeliser_cores,
        rep.soak.consumer_cpu_cores_threadclock
    );
    if !(ok_function && ok_chain) {
        std::process::exit(1);
    }
}
