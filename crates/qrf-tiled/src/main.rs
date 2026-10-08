//! QuadRF tile capture and control daemon
//!
//! Implements: SPEC-008 S-008-2/3/4/10. Skeleton created 2026-10-08 (backlog R-01). It does nothing
//! yet except identify itself; it never transmits (SPEC-008 S-008-9).
#![forbid(unsafe_code)]

fn main() {
    println!("qrf-tiled 0.1.0 (skeleton; see specs/SPEC-008-system-architecture-rust.md) core={}", qrf_core::NAME);
}
