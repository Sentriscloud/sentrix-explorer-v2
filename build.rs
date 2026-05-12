//! Compile `proto/sentrix.proto` into prost types + tonic stubs.
//!
//! ## Codegen scoping — why server stubs are SSR-only
//!
//! The browser bundle only needs the client. The SSR axum binary picks
//! up server stubs on top so we can later:
//!
//!   1. Reverse-proxy gRPC-Web → upstream gRPC at the explorer's edge,
//!      mirroring the `lb_try_duration` resilience already shipped on
//!      the JSON-RPC edge for transient validator failover.
//!   2. Mock/canary handlers under a future `stub-grpc` feature for CI
//!      integration tests that don't want a live chain.
//!
//! Tonic's server stubs depend on the `transport` feature (hyper),
//! which can't compile on `wasm32-unknown-unknown`. So we read
//! `CARGO_FEATURE_SSR` in build.rs and skip server codegen for the
//! browser build.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let build_server = std::env::var_os("CARGO_FEATURE_SSR").is_some();

    let mut config = prost_build::Config::new();
    // Older apt-installed protoc on Ubuntu 22.04 still treats proto3
    // `optional` as experimental. Match the chain crate's build.rs so
    // the proto compiles on the same hosts.
    config.protoc_arg("--experimental_allow_proto3_optional");

    tonic_prost_build::configure()
        .build_client(true)
        .build_server(build_server)
        .compile_with_config(config, &["proto/sentrix.proto"], &["proto"])?;

    println!("cargo:rerun-if-changed=proto/sentrix.proto");
    println!("cargo:rerun-if-env-changed=CARGO_FEATURE_SSR");
    println!("cargo:rerun-if-env-changed=SENTRIX_NETWORK");
    println!("cargo:rerun-if-env-changed=SENTRIX_GRPC_MAINNET");
    println!("cargo:rerun-if-env-changed=SENTRIX_GRPC_TESTNET");
    Ok(())
}
