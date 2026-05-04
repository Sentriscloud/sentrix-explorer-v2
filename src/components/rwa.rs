//! RWA — Real World Asset tracker layout.
//!
//! ## Why this is empty by default
//!
//! Sentrix positioning is "no RWA framing until signed partnerships."
//! This module ships the *layout* — column shape, table row component,
//! verification status badges — but the asset list is empty until real
//! tokenisations onboard. We deliberately don't fabricate sample
//! assets ("Gunung Sahilan", valuation figures) here; that ships when
//! the partnership data does.
//!
//! When real assets land, drop a populated `Vec<Asset>` into the
//! component prop and the rest of the chrome lights up.

use leptos::prelude::*;

use crate::components::identicon::Identicon;
use crate::components::verified_badge::{VerifiedBadge, VerifiedStatus};

#[derive(Clone, Debug)]
pub struct Asset {
    pub name: String,
    /// Free-form valuation string ("USD 1.2M", "12 BTC equiv", etc.).
    /// Kept as a String so a future tokenomics pipeline can format
    /// however it wants without a schema migration here.
    pub valuation: String,
    pub location: String,
    /// On-chain anchor — typically a contract address, transaction
    /// hash, or composite proof identifier. Rendered through
    /// `Identicon` so the row has a consistent visual signature.
    pub on_chain_proof: String,
    pub status: VerificationStatus,
    /// Optional legal/proof certificates — title deeds, registry
    /// extracts, audit attestations. Empty by default; populated
    /// when the asset's onboarding pipeline produces metadata
    /// references.
    pub certificates: Vec<Certificate>,
}

/// One legal/proof certificate referenced by an `Asset`.
///
/// `anchor` is the on-chain or content-addressed identifier (CID,
/// Sentrix tx hash, IPFS URL, etc.). The UI doesn't fetch or render
/// the underlying document — it surfaces the anchor so users can
/// verify out-of-band.
#[derive(Clone, Debug)]
pub struct Certificate {
    pub kind: String,
    pub issuer: String,
    pub anchor: String,
    pub status: VerifiedStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationStatus {
    Verified,
    Pending,
    Disputed,
}

impl VerificationStatus {
    fn classes(self) -> &'static str {
        match self {
            Self::Verified => "border-emerald-500/30 bg-emerald-500/10 text-emerald-300",
            Self::Pending => "border-amber-500/30 bg-amber-500/10 text-amber-300",
            Self::Disputed => "border-rose-500/30 bg-rose-500/10 text-rose-300",
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Verified => "Verified",
            Self::Pending => "Pending",
            Self::Disputed => "Disputed",
        }
    }
}

#[component]
pub fn AssetList(#[prop(default = vec![])] assets: Vec<Asset>) -> impl IntoView {
    // Show + For both want `Fn`, so a moved Vec falls afoul of the
    // FnOnce check. Stash it in a StoredValue so every closure can
    // pull a fresh reference cheaply.
    let count = assets.len();
    let has = count > 0;
    let assets = StoredValue::new(assets);

    view! {
        <section class="glass-card rounded-2xl p-6">
            <header class="mb-6 flex items-center justify-between">
                <div>
                    <h2 class="text-xl font-bold italic tracking-tighter text-zinc-100">
                        "TOKENISED ASSETS"
                    </h2>
                    <p class="mt-1 text-xs text-zinc-500">
                        "Sentrix RWA registry · on-chain provenance"
                    </p>
                </div>
                <span class="status-pill">{count} " listed"</span>
            </header>

            <Show when=move || has fallback=|| view! { <AssetEmpty /> }>
                <div class="hidden grid-cols-12 gap-3 border-b border-zinc-800/60 pb-2 text-[10px] uppercase tracking-[0.18em] text-zinc-500 md:grid">
                    <div class="col-span-4">"Asset"</div>
                    <div class="col-span-3">"Valuation"</div>
                    <div class="col-span-3">"Location"</div>
                    <div class="col-span-2 text-right">"Status"</div>
                </div>

                <div class="mt-3 space-y-2">
                    <For
                        each=move || assets.get_value()
                        key=|a| a.on_chain_proof.clone()
                        children=|a: Asset| {
                            view! { <AssetRow asset=a /> }
                        }
                    />
                </div>
            </Show>
        </section>
    }
}

#[component]
fn AssetRow(asset: Asset) -> impl IntoView {
    let proof_seed = asset.on_chain_proof.clone();
    let proof_short = if asset.on_chain_proof.len() > 14 {
        format!("{}…", &asset.on_chain_proof[..14])
    } else {
        asset.on_chain_proof.clone()
    };
    let status = asset.status;
    let certs = asset.certificates.clone();
    let has_certs = !certs.is_empty();

    view! {
        <div class="rounded-xl border border-zinc-800/30 bg-zinc-900/40 p-3 transition-all hover:border-zinc-700">
            <div class="grid grid-cols-12 items-center gap-3">
                <div class="col-span-4 flex items-center gap-3">
                    <div class="identicon-frame h-9 w-9 rounded-lg ring-1 ring-zinc-800/80">
                        <Identicon address_hex=proof_seed size=36 />
                    </div>
                    <div>
                        <div class="font-bold text-zinc-100">{asset.name}</div>
                        <div class="hex text-[10px] text-zinc-500">{proof_short}</div>
                    </div>
                </div>
                <div class="col-span-3 font-mono text-sm text-zinc-200">{asset.valuation}</div>
                <div class="col-span-3 text-sm text-zinc-400">{asset.location}</div>
                <div class="col-span-2 text-right">
                    <span class=format!(
                        "rounded-md border px-2 py-0.5 font-mono text-[10px] uppercase tracking-wider {}",
                        status.classes(),
                    )>{status.label()}</span>
                </div>
            </div>

            <Show when=move || has_certs fallback=|| ()>
                <CertList certs=certs.clone() />
            </Show>
        </div>
    }
}

#[component]
fn CertList(certs: Vec<Certificate>) -> impl IntoView {
    view! {
        <div class="mt-3 space-y-2 border-t border-zinc-800/40 pt-3">
            <div class="text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                "Legal · Proof"
            </div>
            <div class="space-y-1.5">
                {certs.into_iter().map(|c| view! { <CertRow c /> }).collect_view()}
            </div>
        </div>
    }
}

#[component]
fn CertRow(c: Certificate) -> impl IntoView {
    let anchor_short = if c.anchor.len() > 22 {
        format!("{}…{}", &c.anchor[..14], &c.anchor[c.anchor.len() - 4..])
    } else {
        c.anchor.clone()
    };
    view! {
        <div class="flex items-center justify-between rounded-lg border border-zinc-800/40 bg-zinc-900/30 p-2 text-xs">
            <div class="flex flex-col">
                <span class="text-zinc-200">{c.kind}</span>
                <span class="text-[10px] text-zinc-500">"issued by " {c.issuer}</span>
            </div>
            <div class="flex items-center gap-2">
                <span class="hex text-[10px] text-zinc-400">{anchor_short}</span>
                <VerifiedBadge status=c.status />
            </div>
        </div>
    }
}

#[component]
fn AssetEmpty() -> impl IntoView {
    view! {
        <div class="rounded-xl border border-dashed border-zinc-800 bg-zinc-900/30 p-10 text-center">
            <div class="mx-auto h-10 w-10 rounded-lg bg-gradient-to-br from-amber-300 via-amber-500 to-amber-700 opacity-30" />
            <h3 class="mt-4 text-sm font-semibold text-zinc-300">
                "Registry awaiting first onboarding"
            </h3>
            <p class="mx-auto mt-2 max-w-md text-xs text-zinc-500">
                "Real-world assets will appear here once partner registries publish their on-chain proofs. Each listing carries its own verification anchor."
            </p>
        </div>
    }
}
