//! The device-node ABI of the vendor kernel modules, restated as Rust types.
//!
//! Implements: SPEC-002 S-002-14…17; SPEC-008 S-008-2. Nothing here is copied
//! from the GPL-2.0 driver sources: an ioctl number is a function of its
//! direction, magic character, sequence number and argument size, and a
//! `#[repr(C)]` struct is the argument layout the kernel expects. Both are the
//! public interface a userspace program must match (`vendor/scalerf/ATTRIBUTION.md`).
//! The numbers and sizes were checked against the vendor header by compiling
//! it with the system C compiler on this VM (aarch64, 2026-10-08) and the
//! results are the constants `tests::REFERENCE` asserts.

/// Linux generic ioctl encoding (`asm-generic/ioctl.h`): 8 bits of sequence
/// number, 8 bits of type, 14 bits of size, 2 bits of direction. aarch64,
/// x86_64 and riscv64 all use it.
const NR_BITS: u32 = 8;
const TYPE_BITS: u32 = 8;
const SIZE_BITS: u32 = 14;
const NR_SHIFT: u32 = 0;
const TYPE_SHIFT: u32 = NR_SHIFT + NR_BITS;
const SIZE_SHIFT: u32 = TYPE_SHIFT + TYPE_BITS;
const DIR_SHIFT: u32 = SIZE_SHIFT + SIZE_BITS;
const DIR_NONE: u32 = 0;
const DIR_WRITE: u32 = 1;
const DIR_READ: u32 = 2;

/// Encode one ioctl request number.
pub const fn ioc(dir: u32, magic: u8, nr: u8, size: usize) -> u32 {
    assert!(size < (1 << SIZE_BITS));
    (dir << DIR_SHIFT) | ((size as u32) << SIZE_SHIFT) | ((magic as u32) << TYPE_SHIFT) | ((nr as u32) << NR_SHIFT)
}
/// `_IO(magic, nr)`: no argument.
pub const fn io(magic: u8, nr: u8) -> u32 {
    ioc(DIR_NONE, magic, nr, 0)
}
/// `_IOR(magic, nr, T)`: the kernel writes a `T` to userspace.
pub const fn ior<T>(magic: u8, nr: u8) -> u32 {
    ioc(DIR_READ, magic, nr, size_of::<T>())
}
/// `_IOW(magic, nr, T)`: the kernel reads a `T` from userspace.
pub const fn iow<T>(magic: u8, nr: u8) -> u32 {
    ioc(DIR_WRITE, magic, nr, size_of::<T>())
}
/// `_IOWR(magic, nr, T)`: both directions.
pub const fn iowr<T>(magic: u8, nr: u8) -> u32 {
    ioc(DIR_READ | DIR_WRITE, magic, nr, size_of::<T>())
}

/// Magic character of the receive node `/dev/csi_stream0` (S-002-14).
pub const CSI_MAGIC: u8 = b'C';
/// Magic character of the transmit node `/dev/dsi_stream0` (S-002-17).
pub const DSI_MAGIC: u8 = b'D';
/// Virtual channels the CSI-2 receiver distinguishes in its discard counters.
pub const CSI_VC_MAX: usize = 4;

/// Ring geometry and positions (S-002-14). Positions are byte offsets into the
/// mapped ring; the driver masks them with `ring_size - 1`, so `ring_size` is
/// a power of two and the ring holds at most `ring_size - 1` bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CsiRingInfo {
    /// Bytes that `mmap()` exposes, page aligned.
    pub ring_size: u32,
    /// The producer appends whole spans of this many bytes (one CSI frame).
    pub span_bytes: u32,
    /// Producer position: the next byte the driver will write.
    pub head: u32,
    /// Consumer position: the next byte userspace has not consumed.
    pub tail: u32,
}

impl CsiRingInfo {
    /// Bytes available to the consumer.
    pub fn used(&self) -> u32 {
        if self.head >= self.tail { self.head - self.tail } else { self.ring_size - self.tail + self.head }
    }
    /// Whole spans available to the consumer.
    pub fn spans_available(&self) -> u32 {
        if self.span_bytes == 0 { 0 } else { self.used() / self.span_bytes }
    }
    /// Spans the ring can hold at once (the driver keeps one byte free).
    pub fn capacity_spans(&self) -> u32 {
        if self.span_bytes == 0 { 0 } else { (self.ring_size - 1) / self.span_bytes }
    }
}

/// Cumulative capture counters since module load (S-002-15).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CsiStats {
    /// Bytes copied from DMA buffers into the ring.
    pub dma_bytes: u64,
    /// Bytes consumed by userspace (`read()` or `CONSUME_BYTES`).
    pub bytes_out: u64,
    /// CSI-2 receiver overflow interrupts (hardware side, not the ring).
    pub overflows: u64,
    /// DMA spans completed.
    pub frame_count: u32,
    pub _pad0: u32,
    pub ch_irq_total: u64,
    pub ch_irq_fe: u64,
    pub discards_overflow: [u32; CSI_VC_MAX],
    pub discards_len_limit: [u32; CSI_VC_MAX],
    pub discards_unmatched: [u32; CSI_VC_MAX],
    pub discards_inactive: [u32; CSI_VC_MAX],
    pub discards_overflow_dt: u8,
    pub discards_len_limit_dt: u8,
    pub discards_unmatched_dt: u8,
    pub discards_inactive_dt: u8,
    pub _pad1: u32,
    /// Complete spans lost between DMA and the ring (the software overflow).
    pub overflows_ring: u64,
}

impl CsiStats {
    /// Sum of the per-virtual-channel discard counters.
    pub fn discards_total(&self) -> u64 {
        let s = |a: &[u32; CSI_VC_MAX]| a.iter().map(|&x| u64::from(x)).sum::<u64>();
        s(&self.discards_overflow) + s(&self.discards_len_limit) + s(&self.discards_unmatched) + s(&self.discards_inactive)
    }
}

/// Frame-event and DMA-queue counters (S-002-15).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CsiEventStats {
    pub irq_fs: u64,
    pub irq_fe: u64,
    pub irq_both: u64,
    /// Frame ends inferred from the following frame start.
    pub inferred_fe: u64,
    /// Channel re-primes after a fatal discard.
    pub recoveries: u64,
    /// Times the driver could not keep two DMA addresses posted.
    pub no_buffer: u64,
    pub inactive_irqs: u64,
}

/// D-PHY and CSI-2 register snapshot for diagnostics.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CsiLinkInfo {
    pub dphy_version: u32,
    pub dphy_n_lanes: u32,
    pub dphy_resetn: u32,
    pub dphy_shutdownz: u32,
    pub dphy_rstz: u32,
    pub dphy_phy_rx: u32,
    pub dphy_stopstate: u32,
    pub csi2_status: u32,
    pub csi2_discards_overflow: u32,
    pub csi2_discards_inactive: u32,
    pub csi2_discards_unmatched: u32,
    pub csi2_discards_len_limit: u32,
    pub mipic_cfg: u32,
    pub mipic_intr: u32,
    pub mipic_inte: u32,
    pub mipic_ints: u32,
    pub mipic_irq_total: u64,
    pub mipic_irq_dma: u64,
    pub mipic_irq_host: u64,
    pub mipic_irq_other: u64,
    pub ch_irq_total: u64,
    pub ch_irq_fe: u64,
}

/// Two PHY status registers.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CsiDebug {
    pub phy_rx: u32,
    pub stopstate: u32,
}

/// Virtual-channel and data-type filter.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CsiFilterCfg {
    pub enable_vc_filter: u8,
    pub vc: u8,
    pub enable_dt_filter: u8,
    pub dt: u8,
}

/// Frame geometry; accepted and ignored by the driver (fixed from the device tree).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CsiGeometry {
    pub bytes_per_line: u32,
    pub lines: u32,
}

/// One FPGA register transaction over JTAG (S-002-16): 8-bit address, 16-bit value.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CsiJtagReg {
    pub addr: u8,
    pub _pad0: u8,
    pub value: u16,
    pub _pad1: u16,
}

/// A batch of register writes with a per-write delay (S-002-16).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CsiJtagBatch {
    /// Userspace address of a `CsiJtagReg` array.
    pub regs_ptr: u64,
    pub count: u32,
    pub delay_us: u32,
}

/// DSI staging geometry (S-002-17).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DsiFbInfo {
    /// Bytes per staging frame.
    pub fb_bytes: u64,
    /// Staging frames in the mapping.
    pub fb_count: u32,
    pub head: u32,
    pub tail: u32,
    pub queued: i32,
    pub _pad: u32,
}

// Receive node requests (S-002-14…16).
pub const CSI_IOC_SET_FILTER: u32 = iow::<CsiFilterCfg>(CSI_MAGIC, 0x01);
pub const CSI_IOC_GET_STATS: u32 = ior::<CsiStats>(CSI_MAGIC, 0x02);
pub const CSI_IOC_GET_LINK: u32 = ior::<CsiLinkInfo>(CSI_MAGIC, 0x03);
pub const CSI_IOC_DBG_PHY: u32 = ior::<CsiDebug>(CSI_MAGIC, 0x04);
pub const CSI_IOC_SET_GEOMETRY: u32 = iow::<CsiGeometry>(CSI_MAGIC, 0x05);
pub const CSI_IOC_RESET: u32 = io(CSI_MAGIC, 0x06);
pub const CSI_IOC_GET_EVENTS: u32 = ior::<CsiEventStats>(CSI_MAGIC, 0x07);
pub const CSI_IOC_JTAG_SETUP: u32 = io(CSI_MAGIC, 0x10);
pub const CSI_IOC_JTAG_RELEASE: u32 = io(CSI_MAGIC, 0x11);
pub const CSI_IOC_JTAG_REG_WRITE: u32 = iow::<CsiJtagReg>(CSI_MAGIC, 0x12);
pub const CSI_IOC_JTAG_REG_READ: u32 = iowr::<CsiJtagReg>(CSI_MAGIC, 0x13);
pub const CSI_IOC_JTAG_BATCH_WRITE: u32 = iow::<CsiJtagBatch>(CSI_MAGIC, 0x14);
pub const CSI_IOC_JTAG_ACQUIRE_LEASE: u32 = io(CSI_MAGIC, 0x15);
pub const CSI_IOC_JTAG_RELEASE_LEASE: u32 = io(CSI_MAGIC, 0x16);
pub const CSI_IOC_GET_RING_INFO: u32 = ior::<CsiRingInfo>(CSI_MAGIC, 0x40);
pub const CSI_IOC_CONSUME_BYTES: u32 = iow::<u32>(CSI_MAGIC, 0x41);
// Transmit node requests (S-002-17).
pub const DSI_IOC_GET_FB_INFO: u32 = ior::<DsiFbInfo>(DSI_MAGIC, 0x10);
pub const DSI_IOC_QUEUE_NEXT: u32 = io(DSI_MAGIC, 0x11);

#[cfg(test)]
mod tests {
    use super::*;

    /// Printed by a C program that included the vendor header on this VM
    /// (aarch64, gcc, 2026-10-08): the ABI we must match.
    const REFERENCE: &[(&str, u32, u32)] = &[
        ("CSI_IOC_SET_FILTER", CSI_IOC_SET_FILTER, 0x4004_4301),
        ("CSI_IOC_GET_STATS", CSI_IOC_GET_STATS, 0x8080_4302),
        ("CSI_IOC_GET_LINK", CSI_IOC_GET_LINK, 0x8070_4303),
        ("CSI_IOC_DBG_PHY", CSI_IOC_DBG_PHY, 0x8008_4304),
        ("CSI_IOC_SET_GEOMETRY", CSI_IOC_SET_GEOMETRY, 0x4008_4305),
        ("CSI_IOC_RESET", CSI_IOC_RESET, 0x0000_4306),
        ("CSI_IOC_GET_EVENTS", CSI_IOC_GET_EVENTS, 0x8038_4307),
        ("CSI_IOC_GET_RING_INFO", CSI_IOC_GET_RING_INFO, 0x8010_4340),
        ("CSI_IOC_CONSUME_BYTES", CSI_IOC_CONSUME_BYTES, 0x4004_4341),
        ("CSI_IOC_JTAG_SETUP", CSI_IOC_JTAG_SETUP, 0x0000_4310),
        ("CSI_IOC_JTAG_RELEASE", CSI_IOC_JTAG_RELEASE, 0x0000_4311),
        ("CSI_IOC_JTAG_REG_WRITE", CSI_IOC_JTAG_REG_WRITE, 0x4006_4312),
        ("CSI_IOC_JTAG_REG_READ", CSI_IOC_JTAG_REG_READ, 0xc006_4313),
        ("CSI_IOC_JTAG_BATCH_WRITE", CSI_IOC_JTAG_BATCH_WRITE, 0x4010_4314),
        ("CSI_IOC_JTAG_ACQUIRE_LEASE", CSI_IOC_JTAG_ACQUIRE_LEASE, 0x0000_4315),
        ("CSI_IOC_JTAG_RELEASE_LEASE", CSI_IOC_JTAG_RELEASE_LEASE, 0x0000_4316),
        ("DSI_IOC_GET_FB_INFO", DSI_IOC_GET_FB_INFO, 0x8020_4410),
        ("DSI_IOC_QUEUE_NEXT", DSI_IOC_QUEUE_NEXT, 0x0000_4411),
    ];

    #[test]
    fn ioctl_numbers_match_the_compiled_header() {
        for (name, ours, theirs) in REFERENCE {
            assert_eq!(ours, theirs, "{name}");
        }
    }

    #[test]
    fn struct_sizes_match_the_compiled_header() {
        assert_eq!(size_of::<CsiRingInfo>(), 16);
        assert_eq!(size_of::<CsiStats>(), 128);
        assert_eq!(size_of::<CsiEventStats>(), 56);
        assert_eq!(size_of::<CsiLinkInfo>(), 112);
        assert_eq!(size_of::<CsiDebug>(), 8);
        assert_eq!(size_of::<CsiFilterCfg>(), 4);
        assert_eq!(size_of::<CsiGeometry>(), 8);
        assert_eq!(size_of::<CsiJtagReg>(), 6);
        assert_eq!(size_of::<CsiJtagBatch>(), 16);
        assert_eq!(size_of::<DsiFbInfo>(), 32);
        assert_eq!(std::mem::offset_of!(CsiStats, frame_count), 24);
        assert_eq!(std::mem::offset_of!(CsiStats, discards_overflow), 48);
        assert_eq!(std::mem::offset_of!(CsiStats, discards_overflow_dt), 112);
        assert_eq!(std::mem::offset_of!(CsiStats, overflows_ring), 120);
    }

    #[test]
    fn ring_arithmetic_wraps() {
        let r = CsiRingInfo { ring_size: 1 << 23, span_bytes: 1 << 17, head: 1 << 17, tail: (1 << 23) - (1 << 17) };
        assert_eq!(r.used(), 2 << 17);
        assert_eq!(r.spans_available(), 2);
        assert_eq!(r.capacity_spans(), 63);
        let e = CsiRingInfo { ring_size: 1 << 23, span_bytes: 1 << 17, head: 5, tail: 5 };
        assert_eq!(e.used(), 0);
    }
}
