//! Dual-engine provider abstraction.
//!
//! ## Why two providers
//!
//! Sentrix Chain exposes two read interfaces:
//!
//! 1. **Native gRPC** (`sentrix.v1.Sentrix`) — block index, native
//!    transactions, staking ops, RWA registry. Streaming via
//!    `StreamEvents`. Used for everything that's *Sentrix-shaped*.
//!
//! 2. **EVM JSON-RPC** (`eth_*` at port 8545) — Ethereum-standard
//!    interface used by every Solidity tool, wagmi, viem, MetaMask.
//!    Used for `eth_call`, contract storage reads, log filters.
//!
//! Splitting along trait lines means feature code never has to ask
//! "which transport am I on" — the address shape decides:
//!
//! - `0x` + 40 hex chars → H160 → `EVMProvider`
//! - 42-char Sentrix-native (TBD canonical encoding) → `NativeProvider`
//!
//! ## Status
//!
//! Native side is fully wired against `crates/sentrix-grpc` (live
//! today). EVM side ships as a trait + stub impl that returns
//! `Unimplemented`; it slots in the real `alloy-providers` /
//! `jsonrpsee` calls once the bundle-size budget for that dep tree
//! is approved (≈ +400 KB WASM gzipped, last measured).

pub mod address;
pub mod evm;
pub mod native;

pub use address::{classify_address, AddressKind};
#[cfg(target_arch = "wasm32")]
pub use evm::HttpEvmProvider;
pub use evm::{EvmError, EvmProvider, LogEntry, NoopEvmProvider};
pub use native::{NativeError, NativeProvider, SentrixNativeProvider};
