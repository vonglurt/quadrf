//! Compiles `proto/qrf/v1/qrf.proto` into Rust at build time with `protox`
//! (a pure-Rust protobuf compiler, so no `protoc` is needed on the build host
//! or on copal) and `prost-build`. The `.proto` file is the normative wire
//! schema (SPEC-009 S-009-4/5); the generated module is `qrf_bus::v1`.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto = "proto/qrf/v1/qrf.proto";
    println!("cargo:rerun-if-changed={proto}");
    let fds = protox::compile([proto], ["proto"])?;
    prost_build::Config::new().compile_fds(fds)?;
    Ok(())
}
