//! `/dev/dsi_stream0`: the transmit staging area.
//!
//! Implements: SPEC-002 S-002-17 (frame geometry, writable staging mapping,
//! queue-next). Like `csi.rs`, untested until the bench; the transmit path
//! also stays behind the control plane of SPEC-008 S-008-9 (backlog R-14):
//! nothing here enables a transmitter, it only moves bytes to the staging
//! area the FPGA scans out.

use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::AsRawFd;
use std::path::Path;
use std::ptr::NonNull;

use crate::abi::*;

/// The transmit node with its staging frames mapped read-write.
pub struct DsiDevice {
    file: File,
    base: NonNull<u8>,
    len: usize,
    info: DsiFbInfo,
}

// SAFETY: as `CsiDevice`; the mapping is ours to write and the ioctls are system calls.
unsafe impl Send for DsiDevice {}

impl DsiDevice {
    /// The node the vendor driver creates (S-002-17).
    pub const PATH: &'static str = "/dev/dsi_stream0";

    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = OpenOptions::new().read(true).write(true).open(path)?;
        let info = Self::query(&file)?;
        if info.fb_bytes == 0 || info.fb_count == 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "staging geometry is zero"));
        }
        let len = usize::try_from(info.fb_bytes * u64::from(info.fb_count)).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "staging too large"))?;
        // SAFETY: a fresh shared mapping of the node, unmapped once in `Drop`.
        let p = unsafe { libc::mmap(std::ptr::null_mut(), len, libc::PROT_READ | libc::PROT_WRITE, libc::MAP_SHARED, file.as_raw_fd(), 0) };
        if p == libc::MAP_FAILED {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { file, base: NonNull::new(p.cast()).expect("mmap returned null"), len, info })
    }

    fn query(file: &File) -> io::Result<DsiFbInfo> {
        let mut v = DsiFbInfo::default();
        // SAFETY: `_IOR` of `DsiFbInfo` (abi.rs); `v` is valid and writable.
        let r = unsafe { libc::ioctl(file.as_raw_fd(), DSI_IOC_GET_FB_INFO as _, &mut v as *mut DsiFbInfo) };
        if r < 0 { Err(io::Error::last_os_error()) } else { Ok(v) }
    }

    /// Geometry read at open.
    pub fn geometry(&self) -> DsiFbInfo {
        self.info
    }
    /// Geometry and positions now.
    pub fn info(&self) -> io::Result<DsiFbInfo> {
        Self::query(&self.file)
    }
    /// Staging frame `i` (0-based), writable.
    pub fn frame_mut(&mut self, i: u32) -> &mut [u8] {
        assert!(i < self.info.fb_count, "frame {i} of {}", self.info.fb_count);
        let n = self.info.fb_bytes as usize;
        // SAFETY: inside the mapping (`i < fb_count`); exclusive through `&mut self`.
        unsafe { std::slice::from_raw_parts_mut(self.base.as_ptr().add(i as usize * n), n) }
    }
    /// The frame userspace fills next: the driver's `head` modulo the count.
    pub fn next_frame_mut(&mut self) -> io::Result<&mut [u8]> {
        let i = self.info()?.head % self.info.fb_count;
        Ok(self.frame_mut(i))
    }
    /// Hand the filled frame to the flip thread.
    pub fn queue_next(&self) -> io::Result<()> {
        // SAFETY: `_IO`, no argument.
        let r = unsafe { libc::ioctl(self.file.as_raw_fd(), DSI_IOC_QUEUE_NEXT as _) };
        if r < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
    }
}

impl Drop for DsiDevice {
    fn drop(&mut self) {
        // SAFETY: the pointer and length `mmap` returned, unmapped once.
        unsafe { libc::munmap(self.base.as_ptr().cast(), self.len) };
    }
}
