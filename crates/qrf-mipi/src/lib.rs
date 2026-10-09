//! QuadRF receive ring and DSI staging access: mmap, ioctls, de-interleave, mock device
//!
//! Implements: SPEC-002 S-002-14…17, S-002-21; SPEC-008 S-008-2/3/5. Each
//! public item names the statement it implements. `unsafe` is confined to the
//! ioctl and mmap wrappers (`csi`, `dsi`), the NEON leaf kernel
//! (`deinterleave`, with a scalar reference and a property test) and the
//! mock's shared ring (`mock`, with the driver's own publication protocol).
//!
//! Layout: `abi` (ioctl numbers and argument structs), `frame` (the tile's
//! interleaved CS8 layout), `deinterleave` (scalar and NEON kernels), `ring`
//! (`SpanSource`, `Reader`, `LossMonitor`), `csi` and `dsi` (the kernel
//! nodes), `mock` (a seeded producer in the same layout for tests and
//! budgets without hardware).

pub mod abi;
pub mod csi;
pub mod deinterleave;
pub mod dsi;
pub mod frame;
pub mod mock;
pub mod ring;
pub mod rng;

pub use csi::CsiDevice;
pub use dsi::DsiDevice;
pub use mock::{MockConfig, MockDevice};
pub use ring::{LossEvent, LossMonitor, Reader, Span, SpanSource};

/// The crate's name, for the `Health` message and logs (SPEC-009 S-009-5).
pub const NAME: &str = "qrf-mipi";

#[cfg(test)]
mod tests {
    #[test]
    fn name_is_stable() {
        assert_eq!(super::NAME, "qrf-mipi");
    }
}
