//! A mock of the receive node: a ring in memory filled by a producer thread
//! with frames in the vendor layout from a seeded generator (backlog R-03).
//!
//! The frames are the tile's format (`frame.rs`): one tone per element at a
//! known bin and phase, 8-bit I/Q, elements interleaved. Two things are mock
//! only, so that a consumer can prove it lost nothing and read the right
//! element: time sample 0 of every frame (eight bytes) is replaced by a
//! little-endian 64-bit frame counter, and the tone bins are whole numbers
//! of cycles per frame so that every frame is the same template and the
//! phase at the start of each frame equals the injected phase.
//!
//! Overflow policy: the driver's default (`drop_oldest = 0`): a frame that
//! finds no room is dropped, `overflows_ring` counts it, and the counter
//! still advances, so the consumer sees a gap. The `drop_oldest = 1` policy
//! is not reproduced because it moves `tail` under a consumer that may be
//! reading a mapped span, which `qrf-tiled` must therefore never rely on.

use std::cell::UnsafeCell;
use std::io;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crate::abi::{CsiEventStats, CsiRingInfo, CsiStats};
use crate::frame::{BYTES_PER_SAMPLE, ELEMENTS, FRAME_BYTES, SAMPLES_PER_FRAME};
use crate::ring::SpanSource;
use crate::rng::Rng64;

/// One element's test tone: `bin` whole cycles per frame, phase at sample 0.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tone {
    pub bin: u32,
    pub phase_rad: f64,
}

impl Tone {
    /// Frequency in Hz at sample rate `fs`.
    pub fn freq_hz(&self, fs: f64) -> f64 {
        f64::from(self.bin) * fs / SAMPLES_PER_FRAME as f64
    }
}

/// Mock geometry and pace.
#[derive(Clone, Debug)]
pub struct MockConfig {
    /// Generator seed: tone bins and phases.
    pub seed: u64,
    /// Ring bytes; the driver's 8 MiB on 16 KiB pages (`RING_ORDER 9`).
    pub ring_size: u32,
    /// Span bytes; one CSI frame.
    pub span_bytes: u32,
    /// Time between frames; T14's 630 µs at 4 × 26 MSPS.
    pub frame_period: Duration,
    /// Tone amplitude in LSB of the 8-bit converter.
    pub amplitude: f64,
    /// Stop after this many frames (`None`: until `stop`).
    pub frames: Option<u64>,
    /// Functional-test mode: when the ring is full, wait for the consumer
    /// instead of dropping the frame (the real device cannot wait; T14).
    pub block_when_full: bool,
}

impl Default for MockConfig {
    fn default() -> Self {
        Self { seed: 20261008, ring_size: 8 << 20, span_bytes: FRAME_BYTES as u32, frame_period: Duration::from_micros(630), amplitude: 100.0, frames: None, block_when_full: false }
    }
}

/// Tone bins and phases from a seed; the same seed, the same tones.
pub fn tones_from_seed(seed: u64) -> [Tone; ELEMENTS] {
    let mut r = Rng64::new(seed);
    let mut bins = Vec::new();
    while bins.len() < ELEMENTS {
        let b = 64 + r.below(SAMPLES_PER_FRAME as u64 / 2 - 128) as u32;
        if !bins.contains(&b) {
            bins.push(b);
        }
    }
    std::array::from_fn(|e| Tone { bin: bins[e], phase_rad: (r.unit() * 2.0 - 1.0) * std::f64::consts::PI })
}

/// The frame template: `tones` at `amplitude`, interleaved, without the counter.
pub fn template(tones: &[Tone; ELEMENTS], amplitude: f64) -> Vec<u8> {
    let n = SAMPLES_PER_FRAME;
    let mut f = vec![0u8; FRAME_BYTES];
    for (e, t) in tones.iter().enumerate() {
        let w = 2.0 * std::f64::consts::PI * f64::from(t.bin) / n as f64;
        for i in 0..n {
            let ph = w * i as f64 + t.phase_rad;
            let o = i * BYTES_PER_SAMPLE + 2 * e;
            f[o] = (amplitude * ph.cos()).round() as i8 as u8;
            f[o + 1] = (amplitude * ph.sin()).round() as i8 as u8;
        }
    }
    f
}

/// The frame counter a mock frame carries in its first time sample.
pub fn frame_counter(span: &[u8]) -> u64 {
    u64::from_le_bytes(span[..8].try_into().expect("a span has at least 8 bytes"))
}

/// Phase of the tone at `bin` in one element's CS8 samples, measured over
/// samples 1..n (sample 0 holds the counter): the argument of the single-bin
/// DFT, rotated by a recurrence so that no trigonometric call is made per
/// sample. Quantisation to 8 bits leaves an error far below 0.1° at 16 383
/// samples (the loopback test asserts 0.1°).
pub fn measure_phase(elem_cs8: &[u8], bin: u32) -> f64 {
    let n = elem_cs8.len() / 2;
    let w = -2.0 * std::f64::consts::PI * f64::from(bin) / n as f64;
    let (dc, ds) = (w.cos(), w.sin());
    let (mut c, mut s) = (dc, ds); // e^{jw·1}
    let (mut re, mut im) = (0.0f64, 0.0f64);
    for i in 1..n {
        let x = elem_cs8[2 * i] as i8 as f64;
        let y = elem_cs8[2 * i + 1] as i8 as f64;
        // (x + jy)(c + js)
        re += x * c - y * s;
        im += x * s + y * c;
        let nc = c * dc - s * ds;
        s = c * ds + s * dc;
        c = nc;
        if i % 4096 == 0 {
            let m = (c * c + s * s).sqrt();
            c /= m;
            s /= m;
        }
    }
    im.atan2(re)
}

/// Smallest signed angle from `a` to `b`, in degrees.
pub fn phase_error_deg(a: f64, b: f64) -> f64 {
    let d = (b - a).rem_euclid(2.0 * std::f64::consts::PI);
    let d = if d > std::f64::consts::PI { d - 2.0 * std::f64::consts::PI } else { d };
    d.to_degrees()
}

struct Shared {
    buf: UnsafeCell<Box<[u8]>>,
    ring_size: u32,
    span_bytes: u32,
    head: AtomicU32,
    tail: AtomicU32,
    stop: AtomicBool,
    produced: AtomicU64,
    dropped: AtomicU64,
    bytes_out: AtomicU64,
    late_frames: AtomicU64,
    max_late_ns: AtomicU64,
    gate: Mutex<()>,
    cv: Condvar,
}

// SAFETY: the buffer is written only by the producer, only in
// `[head, head + span)`, and published with a Release store of `head`;
// readers (through `SpanSource::base`) touch only `[tail, head)` after an
// Acquire load. That is the kernel driver's protocol, restated.
unsafe impl Sync for Shared {}
unsafe impl Send for Shared {}

impl Shared {
    fn used(&self) -> u32 {
        let h = self.head.load(Ordering::Acquire);
        let t = self.tail.load(Ordering::Acquire);
        if h >= t { h - t } else { self.ring_size - t + h }
    }
}

/// The mock node. Construct, `start`, then read through `Reader`.
pub struct MockDevice {
    sh: Arc<Shared>,
    cfg: MockConfig,
    tones: [Tone; ELEMENTS],
    template: Arc<Vec<u8>>,
    producer: Option<JoinHandle<()>>,
}

impl MockDevice {
    pub fn new(cfg: MockConfig) -> Self {
        assert!(cfg.ring_size > cfg.span_bytes && cfg.span_bytes >= 8, "ring {} span {}", cfg.ring_size, cfg.span_bytes);
        let tones = tones_from_seed(cfg.seed);
        let mut tpl = template(&tones, cfg.amplitude);
        tpl.resize(cfg.span_bytes as usize, 0);
        let sh = Arc::new(Shared {
            buf: UnsafeCell::new(vec![0u8; cfg.ring_size as usize].into_boxed_slice()),
            ring_size: cfg.ring_size,
            span_bytes: cfg.span_bytes,
            head: AtomicU32::new(0),
            tail: AtomicU32::new(0),
            stop: AtomicBool::new(false),
            produced: AtomicU64::new(0),
            dropped: AtomicU64::new(0),
            bytes_out: AtomicU64::new(0),
            late_frames: AtomicU64::new(0),
            max_late_ns: AtomicU64::new(0),
            gate: Mutex::new(()),
            cv: Condvar::new(),
        });
        Self { sh, cfg, tones, template: Arc::new(tpl), producer: None }
    }

    pub fn config(&self) -> &MockConfig {
        &self.cfg
    }
    pub fn tones(&self) -> &[Tone; ELEMENTS] {
        &self.tones
    }
    /// Frames generated, delivered or dropped.
    pub fn produced(&self) -> u64 {
        self.sh.produced.load(Ordering::Relaxed)
    }
    /// Frames that found no room (the mock's `overflows_ring`).
    pub fn dropped(&self) -> u64 {
        self.sh.dropped.load(Ordering::Relaxed)
    }
    /// Frames the producer wrote later than their schedule by more than one period, and the worst lateness.
    pub fn lateness(&self) -> (u64, Duration) {
        (self.sh.late_frames.load(Ordering::Relaxed), Duration::from_nanos(self.sh.max_late_ns.load(Ordering::Relaxed)))
    }

    /// Start the producer thread at the configured pace.
    pub fn start(&mut self) {
        assert!(self.producer.is_none(), "already started");
        let sh = Arc::clone(&self.sh);
        let tpl = Arc::clone(&self.template);
        let period = self.cfg.frame_period;
        let limit = self.cfg.frames;
        let block = self.cfg.block_when_full;
        self.producer = Some(std::thread::Builder::new().name("mock-csi-producer".into()).spawn(move || produce(sh, tpl, period, limit, block)).expect("spawn"));
    }

    /// Stop the producer and wait for it.
    pub fn stop(&mut self) {
        self.sh.stop.store(true, Ordering::Release);
        if let Some(h) = self.producer.take() {
            let _ = h.join();
        }
    }

    /// True while the producer runs.
    pub fn running(&self) -> bool {
        self.producer.as_ref().is_some_and(|h| !h.is_finished())
    }
}

impl Drop for MockDevice {
    fn drop(&mut self) {
        self.stop();
    }
}

fn produce(sh: Arc<Shared>, tpl: Arc<Vec<u8>>, period: Duration, limit: Option<u64>, block: bool) {
    let span = sh.span_bytes as usize;
    let t0 = Instant::now();
    let mut n: u64 = 0;
    loop {
        if sh.stop.load(Ordering::Acquire) || limit.is_some_and(|l| n >= l) {
            break;
        }
        let deadline = t0 + period * (n as u32);
        pace(deadline);
        let late = deadline.elapsed();
        if late > period {
            sh.late_frames.fetch_add(1, Ordering::Relaxed);
            sh.max_late_ns.fetch_max(late.as_nanos() as u64, Ordering::Relaxed);
        }
        let counter = n;
        n += 1;
        sh.produced.fetch_add(1, Ordering::Relaxed);
        if sh.ring_size - 1 - sh.used() < sh.span_bytes {
            if !block {
                sh.dropped.fetch_add(1, Ordering::Relaxed);
                continue;
            }
            while sh.ring_size - 1 - sh.used() < sh.span_bytes {
                if sh.stop.load(Ordering::Acquire) {
                    return;
                }
                std::thread::sleep(Duration::from_micros(50));
            }
        }
        let head = sh.head.load(Ordering::Acquire) as usize;
        // SAFETY: `[head, head + span)` (modulo the size) is free: `space >= span`
        // and the consumer reads only `[tail, head)`. The writes complete
        // before the Release store of `head` publishes them.
        unsafe {
            let base = (*sh.buf.get()).as_mut_ptr();
            let first = span.min(sh.ring_size as usize - head);
            std::ptr::copy_nonoverlapping(tpl.as_ptr(), base.add(head), first);
            if first < span {
                std::ptr::copy_nonoverlapping(tpl.as_ptr().add(first), base, span - first);
            }
            let c = counter.to_le_bytes();
            for (k, b) in c.iter().enumerate() {
                *base.add((head + k) % sh.ring_size as usize) = *b;
            }
        }
        sh.head.store(((head + span) % sh.ring_size as usize) as u32, Ordering::Release);
        {
            let _g = sh.gate.lock().unwrap_or_else(|e| e.into_inner());
            sh.cv.notify_all();
        }
    }
    sh.stop.store(true, Ordering::Release);
    let _g = sh.gate.lock().unwrap_or_else(|e| e.into_inner());
    sh.cv.notify_all();
}

/// Sleep, then spin, until `deadline`: the sleep leaves the last 300 µs to
/// the spin because a VM's timer slack is of that order.
fn pace(deadline: Instant) {
    let now = Instant::now();
    if deadline > now {
        let remaining = deadline - now;
        if remaining > Duration::from_micros(300) {
            std::thread::sleep(remaining - Duration::from_micros(300));
        }
        while Instant::now() < deadline {
            std::hint::spin_loop();
        }
    }
}

impl SpanSource for MockDevice {
    fn ring_info(&self) -> io::Result<CsiRingInfo> {
        Ok(CsiRingInfo {
            ring_size: self.sh.ring_size,
            span_bytes: self.sh.span_bytes,
            head: self.sh.head.load(Ordering::Acquire),
            tail: self.sh.tail.load(Ordering::Acquire),
        })
    }
    fn wait(&self, timeout: Duration) -> io::Result<bool> {
        let sh = &self.sh;
        let deadline = Instant::now() + timeout;
        let mut g = sh.gate.lock().unwrap_or_else(|e| e.into_inner());
        loop {
            if sh.used() > 0 {
                return Ok(true);
            }
            if sh.stop.load(Ordering::Acquire) {
                return Ok(false);
            }
            let now = Instant::now();
            if now >= deadline {
                return Ok(false);
            }
            let (ng, _) = sh.cv.wait_timeout(g, deadline - now).unwrap_or_else(|e| e.into_inner());
            g = ng;
        }
    }
    fn consume(&self, bytes: u32) -> io::Result<()> {
        if bytes > self.sh.used() {
            return Err(io::Error::from_raw_os_error(libc::EINVAL));
        }
        let t = self.sh.tail.load(Ordering::Acquire);
        self.sh.tail.store((t + bytes) % self.sh.ring_size, Ordering::Release);
        self.sh.bytes_out.fetch_add(u64::from(bytes), Ordering::Relaxed);
        Ok(())
    }
    fn stats(&self) -> io::Result<CsiStats> {
        let produced = self.sh.produced.load(Ordering::Relaxed);
        let dropped = self.sh.dropped.load(Ordering::Relaxed);
        Ok(CsiStats {
            dma_bytes: (produced - dropped) * u64::from(self.sh.span_bytes),
            bytes_out: self.sh.bytes_out.load(Ordering::Relaxed),
            frame_count: produced as u32,
            overflows_ring: dropped,
            ..Default::default()
        })
    }
    fn events(&self) -> io::Result<CsiEventStats> {
        Ok(CsiEventStats { irq_fs: self.sh.produced.load(Ordering::Relaxed), ..Default::default() })
    }
    fn base(&self) -> *const u8 {
        // SAFETY: a shared view of the buffer's address only; readers obey the contract in `Shared`.
        unsafe { (*self.sh.buf.get()).as_ptr() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::deinterleave::{buffers, deinterleave, views};
    use crate::ring::Reader;

    #[test]
    fn tones_are_seeded_and_distinct() {
        let a = tones_from_seed(1);
        assert_eq!(a, tones_from_seed(1));
        assert_ne!(a, tones_from_seed(2));
        for i in 0..ELEMENTS {
            for j in 0..i {
                assert_ne!(a[i].bin, a[j].bin);
            }
            assert!(a[i].phase_rad.abs() <= std::f64::consts::PI);
        }
    }

    #[test]
    fn template_phases_measure_back() {
        let tones = tones_from_seed(20261008);
        let t = template(&tones, 100.0);
        let mut b = buffers(SAMPLES_PER_FRAME);
        deinterleave(&t, &mut views(&mut b));
        for e in 0..ELEMENTS {
            let m = measure_phase(&b[e], tones[e].bin);
            assert!(phase_error_deg(tones[e].phase_rad, m).abs() < 0.01, "element {e}: {m} vs {}", tones[e].phase_rad);
        }
    }

    #[test]
    fn frames_arrive_in_order_with_phases_within_0_1_deg() {
        let cfg = MockConfig { ring_size: 1 << 20, frame_period: Duration::ZERO, frames: Some(200), block_when_full: true, ..Default::default() };
        let mut dev = MockDevice::new(cfg);
        dev.start();
        let mut r = Reader::new(dev).unwrap();
        let tones = *r.source().tones();
        let mut b = buffers(SAMPLES_PER_FRAME);
        let mut expect = 0u64;
        let mut worst = 0.0f64;
        while let Some(sp) = r.next(Duration::from_millis(500)).unwrap() {
            assert_eq!(sp.data.len(), FRAME_BYTES);
            assert_eq!(frame_counter(sp.data), expect, "frame counter gap at span {}", sp.seq);
            expect += 1;
            deinterleave(sp.data, &mut views(&mut b));
            for e in 0..ELEMENTS {
                worst = worst.max(phase_error_deg(tones[e].phase_rad, measure_phase(&b[e], tones[e].bin)).abs());
            }
        }
        assert_eq!(expect, 200);
        assert!(worst < 0.1, "worst phase error {worst}°");
        assert_eq!(r.source().dropped(), 0);
    }

    #[test]
    fn a_slow_consumer_sees_a_counter_gap_and_an_overflow() {
        // 1 MiB ring, 7 spans of 128 KiB fit; produce 20 without consuming.
        let cfg = MockConfig { ring_size: 1 << 20, frame_period: Duration::from_micros(100), frames: Some(20), ..Default::default() };
        let mut dev = MockDevice::new(cfg);
        dev.start();
        while dev.running() {
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(dev.produced(), 20);
        assert_eq!(dev.dropped(), 13);
        assert_eq!(dev.stats().unwrap().overflows_ring, 13);
        let mut r = Reader::new(dev).unwrap();
        let mut seen = Vec::new();
        while let Some(sp) = r.next(Duration::from_millis(10)).unwrap() {
            seen.push(frame_counter(sp.data));
        }
        assert_eq!(seen, (0..7).collect::<Vec<_>>());
    }

    #[test]
    fn wrapped_spans_are_copied_when_the_span_does_not_divide_the_ring() {
        let cfg = MockConfig { ring_size: 3 * FRAME_BYTES as u32 + 4096, frame_period: Duration::ZERO, frames: Some(9), block_when_full: true, ..Default::default() };
        let mut dev = MockDevice::new(cfg);
        dev.start();
        let mut r = Reader::new(dev).unwrap();
        let tpl = Arc::clone(&r.source().template);
        let mut expect = 0u64;
        while let Some(sp) = r.next(Duration::from_millis(200)).unwrap() {
            assert_eq!(frame_counter(sp.data), expect);
            assert_eq!(&sp.data[8..], &tpl[8..]);
            expect += 1;
        }
        assert_eq!(expect, 9);
        assert!(r.copied() >= 1, "at least one span straddled the end");
    }
}
