//! `/dev/csi_stream0`: the receive ring and the JTAG register path.
//!
//! Implements: SPEC-002 S-002-14 (read-only mapping, ring info, consume,
//! poll), S-002-15 (stats and events), S-002-16 (JTAG ioctls and the lease);
//! SPEC-008 S-008-2. `unsafe` is confined to the ioctl and mmap wrappers,
//! each with its safety argument. This path has no device to run against on
//! the development VM; it is exercised at the bench (backlog P-03/R-10) and
//! until then is `[C]` beyond the ABI tests in `abi.rs`.

use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::path::Path;
use std::ptr::NonNull;
use std::time::Duration;

use crate::abi::*;
use crate::ring::SpanSource;

/// The receive node, opened read-only with its ring mapped.
pub struct CsiDevice {
    file: File,
    base: NonNull<u8>,
    len: usize,
    info: CsiRingInfo,
}

// SAFETY: the mapping is shared memory the kernel writes beyond `head`; the
// `SpanSource` contract restricts readers to `[tail, head)`, and the ioctls
// are thread-safe system calls, so the handle may move between threads.
unsafe impl Send for CsiDevice {}

/// One ioctl with no argument.
///
/// # Safety
/// `req` must be a request the node accepts without an argument.
unsafe fn ioctl_none(fd: &impl AsRawFd, req: u32) -> io::Result<()> {
    // SAFETY: delegated to the caller.
    let r = unsafe { libc::ioctl(fd.as_raw_fd(), req as _) };
    if r < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
}

/// One ioctl whose argument the kernel fills in.
///
/// # Safety
/// `req` must be encoded for exactly `T`, a `#[repr(C)]` type the kernel writes whole.
unsafe fn ioctl_read<T: Default>(fd: &impl AsRawFd, req: u32) -> io::Result<T> {
    let mut v = T::default();
    // SAFETY: `v` is a valid, writable `T` for the duration of the call.
    let r = unsafe { libc::ioctl(fd.as_raw_fd(), req as _, &mut v as *mut T) };
    if r < 0 { Err(io::Error::last_os_error()) } else { Ok(v) }
}

/// One ioctl whose argument the kernel reads.
///
/// # Safety
/// `req` must be encoded for exactly `T`, a `#[repr(C)]` type the kernel reads whole.
unsafe fn ioctl_write<T>(fd: &impl AsRawFd, req: u32, v: &T) -> io::Result<()> {
    // SAFETY: `v` is a valid `T` for the duration of the call.
    let r = unsafe { libc::ioctl(fd.as_raw_fd(), req as _, v as *const T) };
    if r < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
}

/// One ioctl whose argument the kernel reads and then updates.
///
/// # Safety
/// As `ioctl_write`, for a request encoded with both directions.
unsafe fn ioctl_read_write<T>(fd: &impl AsRawFd, req: u32, v: &mut T) -> io::Result<()> {
    // SAFETY: `v` is a valid, writable `T` for the duration of the call.
    let r = unsafe { libc::ioctl(fd.as_raw_fd(), req as _, v as *mut T) };
    if r < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
}

/// Wait for `POLLIN` on `fd` for at most `timeout`.
pub(crate) fn poll_in(fd: &impl AsRawFd, timeout: Duration) -> io::Result<bool> {
    let mut p = libc::pollfd { fd: fd.as_raw_fd(), events: libc::POLLIN, revents: 0 };
    let ms = timeout.as_millis().min(i32::MAX as u128) as i32;
    // SAFETY: `p` is one valid pollfd and the count says so.
    let r = unsafe { libc::poll(&mut p, 1, ms) };
    if r < 0 {
        let e = io::Error::last_os_error();
        if e.kind() == io::ErrorKind::Interrupted { return Ok(false) }
        return Err(e);
    }
    Ok(r > 0 && (p.revents & libc::POLLIN) != 0)
}

/// Map `len` bytes of `fd` at offset 0, shared, with `prot`.
///
/// # Safety
/// The caller unmaps with `munmap(ptr, len)` exactly once and never reads
/// outside the contract of the device that backs the mapping.
unsafe fn map(fd: &impl AsRawFd, len: usize, prot: i32) -> io::Result<NonNull<u8>> {
    // SAFETY: a fresh anonymous-address mapping of an open descriptor; the
    // kernel validates length and protection against the node's mmap handler.
    let p = unsafe { libc::mmap(std::ptr::null_mut(), len, prot, libc::MAP_SHARED, fd.as_raw_fd(), 0) };
    if p == libc::MAP_FAILED { Err(io::Error::last_os_error()) } else { Ok(NonNull::new(p.cast::<u8>()).expect("mmap returned null")) }
}

impl CsiDevice {
    /// The node the vendor driver creates (S-002-14).
    pub const PATH: &'static str = "/dev/csi_stream0";

    /// Open the node read-only, read the ring geometry and map the ring.
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).open(path)?;
        // SAFETY: the request is `_IOR` of `CsiRingInfo` (abi.rs, tested against the header).
        let info: CsiRingInfo = unsafe { ioctl_read(&file, CSI_IOC_GET_RING_INFO)? };
        if info.ring_size == 0 || info.span_bytes == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "ring geometry is zero"));
        }
        let len = info.ring_size as usize;
        // SAFETY: unmapped once in `Drop`; readers go through `SpanSource::base`.
        let base = unsafe { map(&file, len, libc::PROT_READ)? };
        Ok(Self { file, base, len, info })
    }

    /// Geometry read at open.
    pub fn geometry(&self) -> CsiRingInfo {
        self.info
    }

    pub fn link(&self) -> io::Result<CsiLinkInfo> {
        // SAFETY: `_IOR` of `CsiLinkInfo`.
        unsafe { ioctl_read(&self.file, CSI_IOC_GET_LINK) }
    }
    pub fn debug_phy(&self) -> io::Result<CsiDebug> {
        // SAFETY: `_IOR` of `CsiDebug`.
        unsafe { ioctl_read(&self.file, CSI_IOC_DBG_PHY) }
    }
    pub fn set_filter(&self, cfg: &CsiFilterCfg) -> io::Result<()> {
        // SAFETY: `_IOW` of `CsiFilterCfg`.
        unsafe { ioctl_write(&self.file, CSI_IOC_SET_FILTER, cfg) }
    }

    // ---- JTAG register path (S-002-16); `qrf-jtag` builds its sequences on these ----

    /// Take the lease (the driver waits up to 100 ms for the holder).
    pub fn jtag_acquire_lease(&self) -> io::Result<()> {
        // SAFETY: `_IO`, no argument.
        unsafe { ioctl_none(&self.file, CSI_IOC_JTAG_ACQUIRE_LEASE) }
    }
    pub fn jtag_release_lease(&self) -> io::Result<()> {
        // SAFETY: `_IO`, no argument.
        unsafe { ioctl_none(&self.file, CSI_IOC_JTAG_RELEASE_LEASE) }
    }
    pub fn jtag_setup(&self) -> io::Result<()> {
        // SAFETY: `_IO`, no argument.
        unsafe { ioctl_none(&self.file, CSI_IOC_JTAG_SETUP) }
    }
    pub fn jtag_release(&self) -> io::Result<()> {
        // SAFETY: `_IO`, no argument.
        unsafe { ioctl_none(&self.file, CSI_IOC_JTAG_RELEASE) }
    }
    pub fn jtag_write(&self, addr: u8, value: u16) -> io::Result<()> {
        let r = CsiJtagReg { addr, value, ..Default::default() };
        // SAFETY: `_IOW` of `CsiJtagReg`.
        unsafe { ioctl_write(&self.file, CSI_IOC_JTAG_REG_WRITE, &r) }
    }
    pub fn jtag_read(&self, addr: u8) -> io::Result<u16> {
        let mut r = CsiJtagReg { addr, ..Default::default() };
        // SAFETY: `_IOWR` of `CsiJtagReg`.
        unsafe { ioctl_read_write(&self.file, CSI_IOC_JTAG_REG_READ, &mut r)? };
        Ok(r.value)
    }
    /// Write `regs` in order with `delay_us` between writes.
    pub fn jtag_batch_write(&self, regs: &[CsiJtagReg], delay_us: u32) -> io::Result<()> {
        let count = u32::try_from(regs.len()).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "too many registers"))?;
        let b = CsiJtagBatch { regs_ptr: regs.as_ptr() as usize as u64, count, delay_us };
        // SAFETY: `_IOW` of `CsiJtagBatch`; `regs` outlives the call and the
        // kernel copies `count` entries from `regs_ptr`.
        unsafe { ioctl_write(&self.file, CSI_IOC_JTAG_BATCH_WRITE, &b) }
    }
}

impl SpanSource for CsiDevice {
    fn ring_info(&self) -> io::Result<CsiRingInfo> {
        // SAFETY: `_IOR` of `CsiRingInfo`.
        unsafe { ioctl_read(&self.file, CSI_IOC_GET_RING_INFO) }
    }
    fn wait(&self, timeout: Duration) -> io::Result<bool> {
        poll_in(&self.file, timeout)
    }
    fn consume(&self, bytes: u32) -> io::Result<()> {
        // SAFETY: `_IOW` of `u32`.
        unsafe { ioctl_write(&self.file, CSI_IOC_CONSUME_BYTES, &bytes) }
    }
    fn stats(&self) -> io::Result<CsiStats> {
        // SAFETY: `_IOR` of `CsiStats`.
        unsafe { ioctl_read(&self.file, CSI_IOC_GET_STATS) }
    }
    fn events(&self) -> io::Result<CsiEventStats> {
        // SAFETY: `_IOR` of `CsiEventStats`.
        unsafe { ioctl_read(&self.file, CSI_IOC_GET_EVENTS) }
    }
    fn base(&self) -> *const u8 {
        self.base.as_ptr()
    }
}

impl Drop for CsiDevice {
    fn drop(&mut self) {
        // SAFETY: the pointer and length are those `mmap` returned, unmapped once.
        unsafe { libc::munmap(self.base.as_ptr().cast(), self.len) };
    }
}
