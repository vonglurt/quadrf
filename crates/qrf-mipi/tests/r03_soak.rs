//! Backlog R-03 check, part 2, shortened: the mock at the T14 cadence (one
//! 131 072-byte frame per 630 µs, 208 MB/s) through `Reader` and the NEON
//! de-interleave for 3 s with no frame-counter gap, and every element's
//! phase within 0.1° of the injected one on every frame. The 60-s run with
//! the same consumer is `make mipi-check` (`examples/mipi_soak.rs`), whose
//! result is recorded in `lab/LR-009`.

use std::time::{Duration, Instant};

use qrf_mipi::deinterleave::{buffers, views, deinterleave};
use qrf_mipi::frame::{ELEMENTS, FRAME_BYTES, SAMPLES_PER_FRAME};
use qrf_mipi::mock::{frame_counter, measure_phase, phase_error_deg};
use qrf_mipi::{LossMonitor, MockConfig, MockDevice, Reader, SpanSource};

#[test]
fn t14_cadence_for_three_seconds_without_a_gap() {
    let seconds = 3u64;
    let cfg = MockConfig { frames: Some(seconds * 1_000_000 / 630), ..Default::default() };
    assert_eq!(cfg.span_bytes as usize, FRAME_BYTES);
    let mut dev = MockDevice::new(cfg);
    dev.start();
    let mut r = Reader::new(dev).unwrap();
    let tones = *r.source().tones();
    let mut b = buffers(SAMPLES_PER_FRAME);
    let mut loss = LossMonitor::new();
    let mut expect = 0u64;
    let mut worst = 0.0f64;
    let mut max_backlog = 0u32;
    let t0 = Instant::now();
    let mut last_stats = t0;
    while let Some(sp) = r.next(Duration::from_millis(200)).unwrap() {
        assert_eq!(frame_counter(sp.data), expect, "frame counter gap at span {}", sp.seq);
        expect += 1;
        deinterleave(sp.data, &mut views(&mut b));
        for e in 0..ELEMENTS {
            worst = worst.max(phase_error_deg(tones[e].phase_rad, measure_phase(&b[e], tones[e].bin)).abs());
        }
        let seq = sp.seq;
        if last_stats.elapsed() >= Duration::from_secs(1) {
            last_stats = Instant::now();
            max_backlog = max_backlog.max(r.backlog().unwrap());
            let (s, e) = (r.source().stats().unwrap(), r.source().events().unwrap());
            assert!(loss.observe(seq, s, e).is_none(), "loss event reported (S-008-3)");
        }
    }
    let elapsed = t0.elapsed();
    let src = r.into_source();
    eprintln!("{} frames in {:.2} s ({:.1} MB/s), worst phase error {:.4}°, max backlog {} spans, dropped {}, late {:?}", expect, elapsed.as_secs_f64(), expect as f64 * FRAME_BYTES as f64 / elapsed.as_secs_f64() / 1e6, worst, max_backlog, src.dropped(), src.lateness());
    assert_eq!(expect, src.produced());
    assert_eq!(src.dropped(), 0);
    assert!(worst < 0.1, "worst phase error {worst}°");
}
