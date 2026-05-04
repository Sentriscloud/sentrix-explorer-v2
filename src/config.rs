//! Compile-time configuration — network selector + endpoint dispatch.
//!
//! ## Why compile-time, not runtime
//!
//! Browser-side WASM has no `std::env::var`. We could ship a runtime
//! `<meta name="network">` shim, but that pushes config into HTML and
//! splits the source of truth across two files. `option_env!` bakes the
//! choice into the bundle at build time — one artifact per network,
//! verifiable by hashing.
//!
//! Default is `mainnet` so a forgotten env var ships safe-by-default
//! (testnet is the surprise; mainnet is the expectation).

/// Which Sentrix network this build targets. Set via `SENTRIX_NETWORK`
/// at build time (mainnet | testnet); defaults to mainnet.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    /// Resolve once at compile time. Bake into a `const` callsite if you
    /// need it for branch-elimination on the WASM hot path.
    pub const fn current() -> Self {
        match option_env!("SENTRIX_NETWORK") {
            Some(s) if matches_testnet(s) => Self::Testnet,
            _ => Self::Mainnet,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Mainnet => "mainnet",
            Self::Testnet => "testnet",
        }
    }

    pub const fn display(self) -> &'static str {
        match self {
            Self::Mainnet => "Sentrix · Mainnet",
            Self::Testnet => "Sentrix · Testnet",
        }
    }
}

/// `const fn` string compare — `str::eq` isn't const yet on stable.
const fn matches_testnet(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() != 7 {
        return false;
    }
    let target = b"testnet";
    let mut i = 0;
    while i < 7 {
        if bytes[i].to_ascii_lowercase() != target[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Resolved gRPC-Web endpoint for the active network. Per-call override
/// via `SENTRIX_GRPC_MAINNET` / `SENTRIX_GRPC_TESTNET` env vars at build
/// time — useful for staging or local-validator dev.
pub const fn grpc_endpoint() -> &'static str {
    match Network::current() {
        Network::Mainnet => match option_env!("SENTRIX_GRPC_MAINNET") {
            Some(s) => s,
            None => "https://grpc.sentrixchain.com",
        },
        Network::Testnet => match option_env!("SENTRIX_GRPC_TESTNET") {
            Some(s) => s,
            None => "https://grpc-testnet.sentrixchain.com",
        },
    }
}

/// EVM JSON-RPC endpoint — `eth_*` methods served at port 8545,
/// fronted by Caddy at the public RPC subdomain.
pub const fn evm_rpc_endpoint() -> &'static str {
    match Network::current() {
        Network::Mainnet => match option_env!("SENTRIX_EVM_MAINNET") {
            Some(s) => s,
            None => "https://rpc.sentrixchain.com/rpc",
        },
        Network::Testnet => match option_env!("SENTRIX_EVM_TESTNET") {
            Some(s) => s,
            None => "https://testnet-rpc.sentrixchain.com/rpc",
        },
    }
}
