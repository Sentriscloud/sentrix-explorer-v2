//! Sentrix Resolver — per-network external service URLs.
//!
//! Goal: zero cross-network contamination. Every UI surface that links
//! out to faucet / wallet / coinblast picks the URL matching the active
//! network from a single registry. A missed update becomes a compile
//! error, not a silent click that drops the user on the wrong chain.
//!
//! ## Single source of truth
//!
//! `Network::current()` from `crate::config` — compile-time baked from
//! `SENTRIX_NETWORK` at build time. The blueprint suggested URL-based
//! detection (`if url.contains("scan-testnet")`); we use the bake
//! instead so a misconfigured vhost can't make the binary lie about its
//! network. The cross-subdomain toggle in `components::navbar` does a
//! full-document navigation, so the *new* binary's bake takes over
//! after the redirect.

use crate::config::Network;

/// Resolved external service URLs for the active network.
///
/// `sibling_explorer` is the target of the network toggle button — the
/// other deployment's hostname.
#[derive(Clone, Copy, Debug)]
pub struct ServiceRegistry {
    pub explorer: &'static str,
    pub faucet: &'static str,
    pub wallet: &'static str,
    pub coinblast: &'static str,
    pub sibling_explorer: &'static str,
}

const MAINNET: ServiceRegistry = ServiceRegistry {
    explorer: "https://scan.sentriscloud.com",
    faucet: "https://faucet.sentrixchain.com",
    wallet: "https://wallet.sentrixchain.com",
    coinblast: "https://coinblast.id",
    sibling_explorer: "https://scan-testnet.sentriscloud.com",
};

const TESTNET: ServiceRegistry = ServiceRegistry {
    explorer: "https://scan-testnet.sentriscloud.com",
    faucet: "https://faucet-testnet.sentrixchain.com",
    wallet: "https://wallet-testnet.sentrixchain.com",
    coinblast: "https://testnet.coinblast.id",
    sibling_explorer: "https://scan.sentriscloud.com",
};

/// Services for the network this binary was built for.
pub const fn services() -> ServiceRegistry {
    match Network::current() {
        Network::Mainnet => MAINNET,
        Network::Testnet => TESTNET,
    }
}
