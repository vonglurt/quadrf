//! Consuming whole spans from a ring that a producer fills ahead of us.
//!
//! Implements: SPEC-008 S-008-2 (map the ring, consume whole spans, stamp
//! each with CLOCK_MONOTONIC_RAW and CLOCK_TAI) and S-008-3 (loss is a
//! measured property: the counters are compared once per period and any
//! increase is a loss event carrying the span sequence number).
//!
//! `SpanSource` is the contract the kernel node (`CsiDevice`) and the mock
//! (`MockDevice`) both meet, so a consumer is written once against `Reader`.

use std::io;
use std::time::Duration;

use qrf_core::time::Stamp;

use crate::abi::{CsiEventStats, CsiRingInfo, CsiStats};

/// A ring the consumer reads through a mapping and advances by ioctl.
///
/// The contract, which the driver honours and the mock reproduces: the
/// producer writes only at and beyond `head`; the bytes in `[tail, head)` do
/// not change until the consumer advances `tail` with `consume`; positions
/// are byte offsets modulo `ring_size`.
pub trait SpanSource {
    /// Geometry and the current positions.
    fn ring_info(&self) -> io::Result<CsiRingInfo>;
    /// Block until data is available or `timeout` passes; `Ok(true)` when data arrived.
    fn wait(&self, timeout: Duration) -> io::Result<bool>;
    /// Advance `tail` by `bytes` (the driver refuses more than is used).
    fn consume(&self, bytes: u32) -> io::Result<()>;
    /// Capture counters (S-002-15).
    fn stats(&self) -> io::Result<CsiStats>;
    /// Frame-event counters (S-002-15).
    fn events(&self) -> io::Result<CsiEventStats>;
    /// Base address of the mapping, valid for `ring_size` bytes while `self` lives.
    ///
    /// Readers may only dereference `[tail, head)` of it, which is stable by
    /// the contract above; everything else is being written concurrently.
    fn base(&self) -> *const u8;
}

/// One span handed to the consumer. `data` is a view into the mapping when
/// the span is contiguous (always, when `span_bytes` divides `ring_size`), a
/// copy otherwise. It stays valid until the next `Reader::next` or
/// `Reader::release`, which consume it.
#[derive(Debug)]
pub struct Span<'a> {
    /// 1-based count of spans this reader has delivered.
    pub seq: u64,
    /// Both clocks, read when the consumer took the span (S-008-2).
    pub stamp: Stamp,
    /// Byte offset of the span in the ring.
    pub offset: u32,
    pub data: &'a [u8],
}

/// Reads whole spans from a `SpanSource`, one at a time.
pub struct Reader<S> {
    src: S,
    ring_size: u32,
    span_bytes: u32,
    tail: u32,
    seq: u64,
    held: bool,
    scratch: Vec<u8>,
    copied: u64,
    resyncs: u64,
}

impl<S: SpanSource> Reader<S> {
    /// Query the geometry and start at the producer's current `tail`.
    pub fn new(src: S) -> io::Result<Self> {
        let info = src.ring_info()?;
        if info.span_bytes == 0 || info.ring_size < info.span_bytes {
            return Err(io::Error::new(io::ErrorKind::InvalidData, format!("ring {} bytes, span {} bytes", info.ring_size, info.span_bytes)));
        }
        Ok(Self {
            src,
            ring_size: info.ring_size,
            span_bytes: info.span_bytes,
            tail: info.tail,
            seq: 0,
            held: false,
            scratch: Vec::new(),
            copied: 0,
            resyncs: 0,
        })
    }

    pub fn source(&self) -> &S {
        &self.src
    }
    pub fn into_source(self) -> S {
        self.src
    }
    pub fn ring_size(&self) -> u32 {
        self.ring_size
    }
    pub fn span_bytes(&self) -> u32 {
        self.span_bytes
    }
    /// Spans delivered so far.
    pub fn delivered(&self) -> u64 {
        self.seq
    }
    /// Spans that straddled the end of the ring and were copied.
    pub fn copied(&self) -> u64 {
        self.copied
    }
    /// Times the producer's `tail` disagreed with ours (another consumer, or a drop-oldest discard).
    pub fn resyncs(&self) -> u64 {
        self.resyncs
    }
    /// Whole spans waiting now.
    pub fn backlog(&self) -> io::Result<u32> {
        Ok(self.src.ring_info()?.spans_available())
    }

    /// Give the held span back to the producer.
    pub fn release(&mut self) -> io::Result<()> {
        if self.held {
            self.src.consume(self.span_bytes)?;
            self.tail = (self.tail + self.span_bytes) % self.ring_size;
            self.held = false;
        }
        Ok(())
    }

    /// Release the previous span and take the next one, waiting at most `timeout` for it.
    pub fn next(&mut self, timeout: Duration) -> io::Result<Option<Span<'_>>> {
        self.release()?;
        let mut info = self.src.ring_info()?;
        if info.spans_available() == 0 {
            if !self.src.wait(timeout)? {
                return Ok(None);
            }
            info = self.src.ring_info()?;
            if info.spans_available() == 0 {
                return Ok(None);
            }
        }
        if info.tail != self.tail {
            self.resyncs += 1;
            self.tail = info.tail;
        }
        let stamp = Stamp::now();
        let off = self.tail as usize;
        let n = self.span_bytes as usize;
        let base = self.src.base();
        let ptr: *const u8 = if off + n <= self.ring_size as usize {
            // SAFETY: `[tail, head)` is stable by the `SpanSource` contract and
            // holds at least one span (checked above); the mapping is valid
            // for `ring_size` bytes while `self.src` lives, and `self` holds it.
            unsafe { base.add(off) }
        } else {
            let first = self.ring_size as usize - off;
            self.scratch.resize(n, 0);
            // SAFETY: as above; the two pieces are the span's bytes before and after the wrap.
            unsafe {
                std::ptr::copy_nonoverlapping(base.add(off), self.scratch.as_mut_ptr(), first);
                std::ptr::copy_nonoverlapping(base, self.scratch.as_mut_ptr().add(first), n - first);
            }
            self.copied += 1;
            self.scratch.as_ptr()
        };
        // SAFETY: `ptr` is valid for `n` bytes for as long as the returned
        // borrow of `self` lives: the ring bytes until `release` (which needs
        // `&mut self`), the scratch copy until the next resize (same).
        let data = unsafe { std::slice::from_raw_parts(ptr, n) };
        self.held = true;
        self.seq += 1;
        Ok(Some(Span { seq: self.seq, stamp, offset: self.tail, data }))
    }
}

/// An increase in any loss counter between two observations (S-008-3).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct LossEvent {
    /// The last span delivered before the loss was observed.
    pub span_seq: u64,
    /// Spans the driver dropped between DMA and the ring.
    pub ring_overflows: u64,
    /// CSI-2 receiver overflow interrupts.
    pub csi_overflows: u64,
    /// Times the DMA queue ran dry.
    pub no_buffer: u64,
    /// Per-virtual-channel discards, all reasons.
    pub discards: u64,
}

impl LossEvent {
    pub fn total(&self) -> u64 {
        self.ring_overflows + self.csi_overflows + self.no_buffer + self.discards
    }
}

/// Compares successive counter snapshots and reports every increase.
#[derive(Debug, Default)]
pub struct LossMonitor {
    last: Option<(CsiStats, CsiEventStats)>,
    events: u64,
    lost: u64,
}

impl LossMonitor {
    pub fn new() -> Self {
        Self::default()
    }
    /// Loss events reported so far.
    pub fn events(&self) -> u64 {
        self.events
    }
    /// Sum of all counter increases reported so far.
    pub fn lost(&self) -> u64 {
        self.lost
    }
    /// Record a snapshot; the first one only sets the baseline.
    pub fn observe(&mut self, span_seq: u64, s: CsiStats, e: CsiEventStats) -> Option<LossEvent> {
        let out = self.last.map(|(ps, pe)| LossEvent {
            span_seq,
            ring_overflows: s.overflows_ring.saturating_sub(ps.overflows_ring),
            csi_overflows: s.overflows.saturating_sub(ps.overflows),
            no_buffer: e.no_buffer.saturating_sub(pe.no_buffer),
            discards: s.discards_total().saturating_sub(ps.discards_total()),
        });
        self.last = Some((s, e));
        match out {
            Some(ev) if ev.total() > 0 => {
                self.events += 1;
                self.lost += ev.total();
                Some(ev)
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loss_monitor_reports_increases_only() {
        let mut m = LossMonitor::new();
        let s0 = CsiStats::default();
        let e0 = CsiEventStats::default();
        assert_eq!(m.observe(1, s0, e0), None, "baseline");
        assert_eq!(m.observe(2, s0, e0), None, "no change");
        let s1 = CsiStats { overflows_ring: 3, ..s0 };
        let e1 = CsiEventStats { no_buffer: 1, ..e0 };
        let ev = m.observe(7, s1, e1).expect("a loss");
        assert_eq!(ev, LossEvent { span_seq: 7, ring_overflows: 3, csi_overflows: 0, no_buffer: 1, discards: 0 });
        assert_eq!(m.observe(8, s1, e1), None);
        let s2 = CsiStats { discards_len_limit: [0, 2, 0, 0], ..s1 };
        assert_eq!(m.observe(9, s2, e1).unwrap().discards, 2);
        assert_eq!(m.events(), 2);
        assert_eq!(m.lost(), 6);
    }
}
