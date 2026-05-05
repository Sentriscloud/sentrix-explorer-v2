//! Static address labels — Solscan-style name tags for premine,
//! governance, and canonical contract addresses.
//!
//! Linear scan over a small const slice (≤ 30 entries today). No
//! HashMap, no `once_cell`: at this size the cache-friendly contiguous
//! scan beats hashing, and we avoid pulling another dep into the
//! WASM bundle. If the registry ever grows past ~200 entries we
//! revisit, but addresses worth a hardcoded tag are rare — most
//! labels should come from a runtime registry feed (validators,
//! tokens) once the indexer surface lands.
//!
//! Source of truth: `apps/scan/lib/labels.tsx::STATIC_LABELS` in the
//! sibling Next.js explorer. Keep the two in sync when seeding new
//! tags. Premine + governance are chain-id-agnostic (same addresses
//! seeded in both genesis files); DEX deployments and SentrixSafes
//! are per-network.

use leptos::prelude::*;

use crate::state::network::Network;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelKind {
    Validator,
    Treasury,
    Token,
    Account,
}

#[derive(Clone, Copy, Debug)]
pub struct Label {
    pub name: &'static str,
    pub kind: LabelKind,
}

impl Label {
    /// Tailwind classes for the pill rendering. Matches the
    /// emerald/amber/violet/sky palette already used elsewhere on the
    /// dashboard so labels feel native, not bolted-on.
    pub const fn pill_classes(&self) -> &'static str {
        match self.kind {
            LabelKind::Validator => "border-emerald-500/30 bg-emerald-500/10 text-emerald-300",
            LabelKind::Treasury => "border-amber-500/30 bg-amber-500/10 text-amber-300",
            LabelKind::Token => "border-violet-500/30 bg-violet-500/10 text-violet-300",
            LabelKind::Account => "border-sky-500/30 bg-sky-500/10 text-sky-300",
        }
    }
}

/// Premine + governance + protocol sentinels. Same addresses on both
/// genesis files, so this table applies regardless of network.
const SHARED: &[(&str, Label)] = &[
    // Premine wallets (v3 — post 2026-04-24 rotation)
    (
        "0x5b5b06688dcdbe532353ac610aaff41af825279d",
        Label {
            name: "Founder v3",
            kind: LabelKind::Treasury,
        },
    ),
    (
        "0xeb70fdefd00fdb768dec06c478f450c351499f14",
        Label {
            name: "Ecosystem Fund",
            kind: LabelKind::Treasury,
        },
    ),
    (
        "0x328d56b8174697ef6c9e40e19b7663797e16fa47",
        Label {
            name: "Validator Incentive Pool",
            kind: LabelKind::Treasury,
        },
    ),
    (
        "0x2578cad17e3e56c2970a5b5eab45952439f5ba97",
        Label {
            name: "Strategic Reserve",
            kind: LabelKind::Treasury,
        },
    ),
    // Governance signing wallet (1-of-1 SentrixSafe owner)
    (
        "0xa25236925bc10954e0519731cc7ba97f4bb5714b",
        Label {
            name: "Authority Wallet",
            kind: LabelKind::Treasury,
        },
    ),
    // Mainnet validator operators — names match the systemd unit
    // identities so logs and dashboards line up.
    (
        "0x753f2f68829fbe76a0132295624f48b27ce2e2d9",
        Label {
            name: "Sentrix Foundation (Validator)",
            kind: LabelKind::Validator,
        },
    ),
    (
        "0x0804a00f53fde72d46abd1db7ee3e97cbfd0a107",
        Label {
            name: "Sentrix Treasury (Validator)",
            kind: LabelKind::Validator,
        },
    ),
    (
        "0x87c9976d4b2e360b9fbb87e4bd5442edce2a7511",
        Label {
            name: "Sentrix Core (Validator)",
            kind: LabelKind::Validator,
        },
    ),
    (
        "0x4cad4793b25b6bb2c927eddfe911996070c7ce68",
        Label {
            name: "Sentrix Beacon (Validator)",
            kind: LabelKind::Validator,
        },
    ),
    // Protocol-reserved sentinels — no private key, consensus-level only.
    (
        "0x0000000000000000000000000000000000000000",
        Label {
            name: "Sentrix Token Op (sentinel)",
            kind: LabelKind::Treasury,
        },
    ),
    (
        "0x0000000000000000000000000000000000000002",
        Label {
            name: "Protocol Treasury (Reward Escrow)",
            kind: LabelKind::Treasury,
        },
    ),
    (
        "0x0000000000000000000000000000000000000100",
        Label {
            name: "Sentrix Staking (sentinel)",
            kind: LabelKind::Treasury,
        },
    ),
];

/// Mainnet (chain 7119) — DEX router/factory + canonical SRX-pegged
/// tokens, plus the SentrixSafe instance.
const MAINNET_ONLY: &[(&str, Label)] = &[
    (
        "0x6272dc0c842f05542f9ff7b5443e93c0642a3b26",
        Label {
            name: "SentrixSafe",
            kind: LabelKind::Treasury,
        },
    ),
    (
        "0xab67e171c0de0cd6dd6fe87e5e399c091f9c9de8",
        Label {
            name: "Sentrix DEX Router",
            kind: LabelKind::Token,
        },
    ),
    (
        "0xc5344f0dde0b9916217449ad9222e446475ad936",
        Label {
            name: "Sentrix DEX Factory",
            kind: LabelKind::Token,
        },
    ),
    (
        "0x4693b113e523a196d9579333c4ab8358e2656553",
        Label {
            name: "WSRX",
            kind: LabelKind::Token,
        },
    ),
    (
        "0xa79fc9015ae30766ab4d24a5d4d3a0c66f371504",
        Label {
            name: "SGC",
            kind: LabelKind::Token,
        },
    ),
];

/// Testnet (chain 7120) — disjoint from mainnet because the testnet
/// DEX deploys against a different deterministic deployer nonce.
const TESTNET_ONLY: &[(&str, Label)] = &[
    (
        "0xc9d7a61d7c2f428f6a055916488041fd00532110",
        Label {
            name: "SentrixSafe (Testnet)",
            kind: LabelKind::Treasury,
        },
    ),
    (
        "0x2bf73491733c3b87d72b16d4f7151da294b55cb0",
        Label {
            name: "Sentrix DEX Router (Testnet)",
            kind: LabelKind::Token,
        },
    ),
    (
        "0x8565392086cba8d39cbba1f6f60ad1f1a17651c7",
        Label {
            name: "Sentrix DEX Factory (Testnet)",
            kind: LabelKind::Token,
        },
    ),
    (
        "0x85d5e7694af31c2edd0a7e66b7c6c92c59ff949a",
        Label {
            name: "WtSRX (Testnet)",
            kind: LabelKind::Token,
        },
    ),
    (
        "0x72730453f4080c6ad8def96c06f6074818fb95b5",
        Label {
            name: "SGC (Testnet)",
            kind: LabelKind::Token,
        },
    ),
];

/// Resolve an address to its static label, if any. Accepts either a
/// `0x`-prefixed string or bare hex; comparison is case-insensitive.
pub fn label_for(addr_hex: &str, network: Network) -> Option<Label> {
    let needle = normalise(addr_hex);
    if needle.len() != 42 {
        return None;
    }
    if let Some(l) = scan(SHARED, &needle) {
        return Some(l);
    }
    scan(
        match network {
            Network::Mainnet => MAINNET_ONLY,
            Network::Testnet => TESTNET_ONLY,
        },
        &needle,
    )
}

fn scan(table: &'static [(&'static str, Label)], needle: &str) -> Option<Label> {
    for (k, v) in table {
        if k.eq_ignore_ascii_case(needle) {
            return Some(*v);
        }
    }
    None
}

fn normalise(addr_hex: &str) -> String {
    let trimmed = addr_hex.trim();
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        trimmed.to_ascii_lowercase()
    } else {
        format!("0x{}", trimmed.to_ascii_lowercase())
    }
}

/// Tiny pill component — renders nothing when no label matches, so
/// callers can sprinkle it next to any address render without
/// guarding upstream.
#[component]
pub fn AddressLabel(
    /// Address as `0x…` hex (case-insensitive). Empty / malformed
    /// strings are silently dropped.
    #[prop(into)]
    addr: String,
) -> impl IntoView {
    let network = crate::state::network::use_network();
    let addr = StoredValue::new(addr);

    move || {
        let net = network.get();
        let resolved = label_for(&addr.read_value(), net)?;
        let classes = format!(
            "inline-flex items-center rounded-full border px-2 py-0.5 text-[10px] font-medium {}",
            resolved.pill_classes()
        );
        Some(view! {
            <span class=classes title=resolved.name>{resolved.name}</span>
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_shared_label() {
        let l = label_for(
            "0x5b5b06688dcdbe532353ac610aaff41af825279d",
            Network::Mainnet,
        )
        .expect("founder v3 should be labeled");
        assert_eq!(l.name, "Founder v3");
        assert_eq!(l.kind, LabelKind::Treasury);
    }

    #[test]
    fn resolves_per_network() {
        let mainnet = label_for(
            "0xab67e171c0de0cd6dd6fe87e5e399c091f9c9de8",
            Network::Mainnet,
        );
        let testnet = label_for(
            "0xab67e171c0de0cd6dd6fe87e5e399c091f9c9de8",
            Network::Testnet,
        );
        assert!(mainnet.is_some(), "mainnet router should resolve");
        assert!(
            testnet.is_none(),
            "mainnet router should not resolve on testnet"
        );
    }

    #[test]
    fn case_insensitive_and_prefix_optional() {
        let with_prefix = label_for(
            "0x5B5B06688DCDBE532353AC610AAFF41AF825279D",
            Network::Mainnet,
        );
        let bare = label_for("5b5b06688dcdbe532353ac610aaff41af825279d", Network::Mainnet);
        assert!(with_prefix.is_some());
        assert!(bare.is_some());
    }

    #[test]
    fn unknown_returns_none() {
        let l = label_for(
            "0xdeadbeef0000000000000000000000000000beef",
            Network::Mainnet,
        );
        assert!(l.is_none());
    }

    #[test]
    fn malformed_returns_none() {
        assert!(label_for("not-an-address", Network::Mainnet).is_none());
        assert!(label_for("0x1234", Network::Mainnet).is_none());
    }
}
