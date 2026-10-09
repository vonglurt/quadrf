//! Backlog R-03 check, the 60-s run: the mock at the T14 cadence through
//! `Reader`, the NEON de-interleave and a per-frame phase check on all four
//! elements; CPU time of the consumer thread and of the process; the
//! de-interleave kernels' throughput; a JSON record for the lab report.
//!
//!   mipi_soak [--seconds 60] [--period-us 630] [--json out.json] [--dump-frame frame0.cs8] [--occupancy-every N]

use std::fs;
use std::time::{Duration, Instant};

use qrf_mipi::deinterleave::{buffers, views, deinterleave, deinterleave_scalar, kernel_name};
use qrf_mipi::frame::{ELEMENTS, FRAME_BYTES, SAMPLES_PER_FRAME};
use qrf_mipi::mock::{frame_counter, measure_phase, phase_error_deg};
use qrf_mipi::{LossMonitor, MockConfig, MockDevice, Reader, SpanSource};
use serde::Serialize;

#[derive(Serialize)]
struct Report {
    seconds_requested: f64,
    seconds_elapsed: f64,
    frame_period_us: f64,
    frame_bytes: usize,
    ring_bytes: u32,
    ring_capacity_spans: u32,
    frames_produced: u64,
    frames_consumed: u64,
    frames_dropped: u64,
    counter_gaps: u64,
    loss_events: u64,
    throughput_mb_s: f64,
    producer_late_frames: u64,
    producer_max_late_us: f64,
    max_backlog_spans: u32,
    interarrival_us: Percentiles,
    consumer_service_us: Percentiles,
    worst_phase_error_deg: f64,
    mean_phase_error_deg: f64,
    tones: Vec<ToneRec>,
    consumer_cpu_s: f64,
    consumer_cpu_cores: f64,
    process_cpu_s: f64,
    process_cpu_cores: f64,
    kernel: &'static str,
    deinterleave_scalar_gb_s: f64,
    deinterleave_neon_gb_s: f64,
    occupancy_trace: Vec<(f64, u32)>,
    interarrival_hist_us: Vec<(f64, u64)>,
    host: String,
}

#[derive(Serialize)]
struct ToneRec {
    element: usize,
    bin: u32,
    freq_hz_at_26msps: f64,
    phase_deg: f64,
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

fn process_cpu() -> f64 {
    let mut ts = libc::timespec { tv_sec: 0, tv_nsec: 0 };
    // SAFETY: as above.
    unsafe { libc::clock_gettime(libc::CLOCK_PROCESS_CPUTIME_ID, &mut ts) };
    ts.tv_sec as f64 + ts.tv_nsec as f64 * 1e-9
}

fn bench(f: fn(&[u8], &mut [&mut [u8]; ELEMENTS]), src: &[u8]) -> f64 {
    let mut out = buffers(SAMPLES_PER_FRAME);
    let reps = 2000;
    for _ in 0..50 {
        f(src, &mut views(&mut out));
    }
    let t = Instant::now();
    for _ in 0..reps {
        f(src, &mut views(&mut out));
    }
    reps as f64 * src.len() as f64 / t.elapsed().as_secs_f64() / 1e9
}

fn main() {
    let mut args = std::env::args().skip(1);
    let (mut seconds, mut period_us, mut json, mut dump, mut occ_every) = (60.0f64, 630.0f64, None, None, 100u64);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--seconds" => seconds = args.next().unwrap().parse().unwrap(),
            "--period-us" => period_us = args.next().unwrap().parse().unwrap(),
            "--json" => json = args.next(),
            "--dump-frame" => dump = args.next(),
            "--occupancy-every" => occ_every = args.next().unwrap().parse().unwrap(),
            other => panic!("unknown argument {other}"),
        }
    }
    let period = Duration::from_nanos((period_us * 1000.0) as u64);
    let frames = (seconds * 1e6 / period_us) as u64;
    let cfg = MockConfig { frame_period: period, frames: Some(frames), ..Default::default() };
    let ring_bytes = cfg.ring_size;
    let mut dev = MockDevice::new(cfg);
    let tones = *dev.tones();
    let template = qrf_mipi::mock::template(&tones, dev.config().amplitude);
    let scalar_gb_s = bench(deinterleave_scalar, &template);
    let fast_gb_s = bench(deinterleave, &template);
    eprintln!("deinterleave: scalar {scalar_gb_s:.2} GB/s, {} {fast_gb_s:.2} GB/s", kernel_name());

    dev.start();
    let mut r = Reader::new(dev).unwrap();
    let capacity = r.source().ring_info().unwrap().capacity_spans();
    let mut b = buffers(SAMPLES_PER_FRAME);
    let mut loss = LossMonitor::new();
    let (mut expect, mut gaps) = (0u64, 0u64);
    let (mut worst, mut sum_err) = (0.0f64, 0.0f64);
    let mut max_backlog = 0u32;
    let mut inter = Vec::with_capacity(frames as usize);
    let mut service = Vec::with_capacity(frames as usize);
    let mut occupancy = Vec::new();
    let mut hist = vec![0u64; 40]; // 50 µs bins up to 2 ms
    let cpu0 = thread_cpu();
    let pcpu0 = process_cpu();
    let t0 = Instant::now();
    let mut last_arrival: Option<Instant> = None;
    let mut last_stats = t0;
    let mut dumped = dump.is_none();
    while let Some(sp) = r.next(Duration::from_millis(500)).unwrap() {
        let arrived = Instant::now();
        if let Some(p) = last_arrival {
            let us = (arrived - p).as_secs_f64() * 1e6;
            inter.push(us);
            let bin = ((us / 50.0) as usize).min(hist.len() - 1);
            hist[bin] += 1;
        }
        last_arrival = Some(arrived);
        let c = frame_counter(sp.data);
        if c != expect {
            gaps += 1;
            eprintln!("counter gap: expected {expect}, got {c} at span {}", sp.seq);
            expect = c;
        }
        expect += 1;
        if !dumped {
            fs::write(dump.as_ref().unwrap(), sp.data).unwrap();
            dumped = true;
        }
        deinterleave(sp.data, &mut views(&mut b));
        for e in 0..ELEMENTS {
            let err = phase_error_deg(tones[e].phase_rad, measure_phase(&b[e], tones[e].bin)).abs();
            worst = worst.max(err);
            sum_err += err;
        }
        service.push(arrived.elapsed().as_secs_f64() * 1e6);
        let seq = sp.seq;
        if seq % occ_every == 0 {
            let bl = r.backlog().unwrap();
            max_backlog = max_backlog.max(bl);
            occupancy.push((t0.elapsed().as_secs_f64(), bl));
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
    let pcpu = process_cpu() - pcpu0;
    let consumed = r.delivered();
    let src = r.into_source();
    let (late, max_late) = src.lateness();
    let rep = Report {
        seconds_requested: seconds,
        seconds_elapsed: elapsed,
        frame_period_us: period_us,
        frame_bytes: FRAME_BYTES,
        ring_bytes,
        ring_capacity_spans: capacity,
        frames_produced: src.produced(),
        frames_consumed: consumed,
        frames_dropped: src.dropped(),
        counter_gaps: gaps,
        loss_events: loss.events(),
        throughput_mb_s: consumed as f64 * FRAME_BYTES as f64 / elapsed / 1e6,
        producer_late_frames: late,
        producer_max_late_us: max_late.as_secs_f64() * 1e6,
        max_backlog_spans: max_backlog,
        interarrival_us: percentiles(&mut inter),
        consumer_service_us: percentiles(&mut service),
        worst_phase_error_deg: worst,
        mean_phase_error_deg: sum_err / (consumed as f64 * ELEMENTS as f64).max(1.0),
        tones: tones.iter().enumerate().map(|(e, t)| ToneRec { element: e, bin: t.bin, freq_hz_at_26msps: t.freq_hz(26e6), phase_deg: t.phase_rad.to_degrees() }).collect(),
        consumer_cpu_s: cpu,
        consumer_cpu_cores: cpu / elapsed,
        process_cpu_s: pcpu,
        process_cpu_cores: pcpu / elapsed,
        kernel: kernel_name(),
        deinterleave_scalar_gb_s: scalar_gb_s,
        deinterleave_neon_gb_s: fast_gb_s,
        occupancy_trace: occupancy,
        interarrival_hist_us: hist.iter().enumerate().map(|(i, &n)| (i as f64 * 50.0, n)).collect(),
        host: format!("{} {}", std::env::consts::ARCH, fs::read_to_string("/proc/cpuinfo").ok().and_then(|s| s.lines().find(|l| l.starts_with("CPU part") || l.starts_with("model name")).map(|l| l.trim().to_string())).unwrap_or_default()),
    };
    println!("frames produced {} consumed {} dropped {} gaps {} loss events {}", rep.frames_produced, rep.frames_consumed, rep.frames_dropped, rep.counter_gaps, rep.loss_events);
    println!("elapsed {:.2} s, {:.1} MB/s, max backlog {} of {} spans, producer late {} (max {:.0} µs)", rep.seconds_elapsed, rep.throughput_mb_s, rep.max_backlog_spans, rep.ring_capacity_spans, rep.producer_late_frames, rep.producer_max_late_us);
    println!("inter-arrival µs: min {:.0} p50 {:.0} p99 {:.0} p99.9 {:.0} max {:.0}", rep.interarrival_us.min, rep.interarrival_us.p50, rep.interarrival_us.p99, rep.interarrival_us.p999, rep.interarrival_us.max);
    println!("consumer service µs: p50 {:.1} p99 {:.1} max {:.1}", rep.consumer_service_us.p50, rep.consumer_service_us.p99, rep.consumer_service_us.max);
    println!("phase error: worst {:.4}° mean {:.4}°", rep.worst_phase_error_deg, rep.mean_phase_error_deg);
    println!("cpu: consumer {:.2} core, process {:.2} core; deinterleave scalar {:.2} GB/s, {} {:.2} GB/s", rep.consumer_cpu_cores, rep.process_cpu_cores, rep.deinterleave_scalar_gb_s, rep.kernel, rep.deinterleave_neon_gb_s);
    if let Some(p) = json {
        if let Some(dir) = std::path::Path::new(&p).parent() {
            fs::create_dir_all(dir).unwrap();
        }
        fs::write(&p, serde_json::to_string_pretty(&rep).unwrap()).unwrap();
        println!("wrote {p}");
    }
    let ok = rep.counter_gaps == 0 && rep.frames_dropped == 0 && rep.worst_phase_error_deg < 0.1 && rep.frames_consumed == rep.frames_produced;
    println!("R-03 check: {}", if ok { "PASS" } else { "FAIL" });
    if !ok {
        std::process::exit(1);
    }
}
