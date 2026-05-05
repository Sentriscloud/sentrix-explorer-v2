//! Runtime Network detection — host-based.
//!
//! Distinct from the compile-time `crate::config::Network` (which is
//! baked from `SENTRIX_NETWORK` at build time). This module reads
//! the active host at runtime so a single binary could in principle
//! serve both networks. The current deploy keeps one binary per
//! network (compile-time guard against cross-network leaks), so the
//! two values agree by construction — but the runtime path is the
//! source of truth for any consumer that programs against context.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Network {
    pub fn from_host(host: &str) -> Self {
        // Substring check — covers every variant the Caddy block
        // forwards (`scan-testnet.sentriscloud.com`, future
        // `testnet.foo.bar`, dev `127.0.0.1:3051` won't match,
        // defaults to Mainnet which is the safe surface).
        if host.contains("testnet") {
            Self::Testnet
        } else {
            Self::Mainnet
        }
    }

    pub fn rpc_url(&self) -> &'static str {
        match self {
            Self::Mainnet => "https://rpc.sentrixchain.com",
            Self::Testnet => "https://testnet-rpc.sentrixchain.com",
        }
    }

    pub fn chain_id(&self) -> u64 {
        match self {
            Self::Mainnet => 7119,
            Self::Testnet => 7120,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Mainnet => "Mainnet",
            Self::Testnet => "Testnet",
        }
    }
}

/// Provide an `RwSignal<Network>` via context. Call once at App root.
/// The signal is mutable so a future "switch network" UI affordance
/// could update it without a full page reload — though the current
/// production deploy uses cross-subdomain redirects (different
/// binary per network) for stronger isolation.
pub fn provide_network_signal() {
    let initial = detect_network();
    provide_context(RwSignal::new(initial));
}

pub fn use_network() -> RwSignal<Network> {
    use_context::<RwSignal<Network>>().expect("Network signal context not provided")
}

#[cfg(target_arch = "wasm32")]
fn detect_network() -> Network {
    let host = web_sys::window()
        .and_then(|w| w.location().host().ok())
        .unwrap_or_default();
    Network::from_host(&host)
}

#[cfg(not(target_arch = "wasm32"))]
fn detect_network() -> Network {
    // SSR pre-render — no `window`. Read from the compile-time bake
    // so the server-rendered HTML matches what the hydrated bundle
    // will compute on the client (same binary = same network).
    // For multi-network from a single binary, swap to:
    //   `use_context::<leptos_axum::RequestParts>().host()` and
    //   parse the host from the inbound request headers.
    match option_env!("SENTRIX_NETWORK") {
        Some(s) if s.eq_ignore_ascii_case("testnet") => Network::Testnet,
        _ => Network::Mainnet,
    }
}
