//! `/tx/:hash` detail screen.
//!
//! ## Lookup strategy
//!
//! Sentrix's gRPC service exposes `GetBlock` + `GetBalance` but not yet
//! `GetTransaction`. So this screen looks up the tx in the rolling
//! `BlockFeedState.tx_index` cache (last ~500 observed txs). Hits
//! show the full decoded body. Misses fall through to a clear
//! "outside the live window" affordance — better than fake stub
//! content, and the page header + share previews still work.

use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_params_map;

use crate::components::copy_cli::CopyCli;
use crate::components::identicon::Identicon;
use crate::i18n::{t, use_lang};
use crate::state::feed::BlockFeedState;

#[component]
pub fn TxDetailScreen() -> impl IntoView {
    let params = use_params_map();
    let raw_hash = params.read().get("hash").unwrap_or_default();
    let normalized = raw_hash
        .strip_prefix("0x")
        .or_else(|| raw_hash.strip_prefix("0X"))
        .unwrap_or(&raw_hash)
        .to_lowercase();

    let hash_seed = normalized.clone();
    let hash_display = format!("0x{normalized}");
    let hash_short = if hash_display.len() > 14 {
        format!(
            "{}…{}",
            &hash_display[..10],
            &hash_display[hash_display.len() - 4..]
        )
    } else {
        hash_display.clone()
    };

    let title = format!("Sentrix Transaction {hash_short} — Obsidian Engine");
    let description =
        format!("Transaction {hash_short} on the Sentrix L1 — sender, recipient, payload.");
    let cli = format!("srx-cli get tx --hash {hash_display}");
    let url = format!("https://scan.sentriscloud.com/tx/{hash_display}");

    let feed = use_context::<BlockFeedState>().expect("BlockFeedState context");
    let lookup_key = normalized.clone();
    let found = Memo::new(move |_| {
        feed.tx_index.with(|idx| {
            idx.iter()
                .find(|i| {
                    i.tx.txid
                        .as_ref()
                        .map(|h| hex::encode(&h.value) == lookup_key)
                        .unwrap_or(false)
                })
                .cloned()
        })
    });

    view! {
        <Title text=title.clone() />
        <Meta property="og:type" content="website" />
        <Meta property="og:title" content=title.clone() />
        <Meta property="og:description" content=description.clone() />
        <Meta property="og:url" content=url.clone() />
        <Meta property="og:image" content="https://scan.sentriscloud.com/icon.svg" />
        <Meta name="twitter:card" content="summary_large_image" />
        <Meta name="twitter:title" content=title />
        <Meta name="twitter:description" content=description />
        <Meta name="twitter:image" content="https://scan.sentriscloud.com/icon.svg" />

        <section class="glass-card space-y-4 rounded-2xl p-6">
            <header class="flex items-center gap-4">
                <div class="identicon-frame h-12 w-12 rounded-lg ring-1 ring-zinc-800/80">
                    <Identicon address_hex=hash_seed size=48 />
                </div>
                <div>
                    <div class="text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                        {move || t(use_lang().get(), "detail.transaction")}
                    </div>
                    <h1 class="hex break-all text-lg font-bold text-zinc-100">
                        {hash_display}
                    </h1>
                </div>
            </header>

            {move || match found.get() {
                Some(idx) => view! { <TxBody idx /> }.into_any(),
                None => view! { <NotInWindow /> }.into_any(),
            }}

            <footer class="border-t border-zinc-800/40 pt-4">
                <div class="mb-2 text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                    {move || t(use_lang().get(), "detail.cli_label")}
                </div>
                <CopyCli command=cli />
            </footer>
        </section>
    }
}

#[component]
fn TxBody(idx: crate::state::feed::IndexedTx) -> impl IntoView {
    let from = format!(
        "0x{}",
        idx.tx
            .from_address
            .as_ref()
            .map(|a| hex::encode(&a.value))
            .unwrap_or_default()
    );
    let to = format!(
        "0x{}",
        idx.tx
            .to_address
            .as_ref()
            .map(|a| hex::encode(&a.value))
            .unwrap_or_default()
    );
    let from_seed = idx
        .tx
        .from_address
        .as_ref()
        .map(|a| hex::encode(&a.value))
        .unwrap_or_default();
    let to_seed = idx
        .tx
        .to_address
        .as_ref()
        .map(|a| hex::encode(&a.value))
        .unwrap_or_default();
    let from_link = format!("/address/{from}");
    let to_link = format!("/address/{to}");
    let block_link = format!("/block/{}", idx.block_height);
    let amount = idx.tx.amount.as_ref().map(|a| a.sentri).unwrap_or(0);
    let fee = idx.tx.fee.as_ref().map(|a| a.sentri).unwrap_or(0);
    let kind = match idx.tx.tx_type {
        0 => "Transfer",
        1 => "Contract Call",
        2 => "Staking Operation",
        _ => "Other / RWA event",
    };

    view! {
        <dl class="space-y-3 text-sm">
            <Row label_key="detail.action">
                <span class="font-mono text-amber-300">{kind}</span>
            </Row>
            <Row label_key="detail.block">
                <a href=block_link class="font-mono text-zinc-200 hover:text-amber-300">
                    "#" {idx.block_height}
                </a>
            </Row>
            <Row label_key="detail.from">
                <a href=from_link class="flex items-center gap-2 hover:text-amber-300">
                    <span class="identicon-frame h-5 w-5 rounded">
                        <Identicon address_hex=from_seed size=20 />
                    </span>
                    <span class="hex break-all text-xs">{from}</span>
                </a>
            </Row>
            <Row label_key="detail.to">
                <a href=to_link class="flex items-center gap-2 hover:text-amber-300">
                    <span class="identicon-frame h-5 w-5 rounded">
                        <Identicon address_hex=to_seed size=20 />
                    </span>
                    <span class="hex break-all text-xs">{to}</span>
                </a>
            </Row>
            <Row label_key="detail.amount">
                <span class="font-mono text-zinc-200">{format_sentri(amount)} " SRX"</span>
            </Row>
            <Row label_key="detail.fee">
                <span class="font-mono text-zinc-400">{format_sentri(fee)} " SRX"</span>
            </Row>
            <Row label_key="detail.nonce">
                <span class="font-mono text-zinc-300">{idx.tx.nonce}</span>
            </Row>
            <Row label_key="detail.payload_size">
                <span class="font-mono text-zinc-400">{idx.tx.payload.len()} " bytes"</span>
            </Row>
        </dl>
    }
}

#[component]
fn NotInWindow() -> impl IntoView {
    let lang = use_lang();
    view! {
        <div class="rounded-xl border border-dashed border-zinc-800 bg-zinc-900/30 p-6 text-sm text-zinc-500">
            {move || t(lang.get(), "detail.tx_outside_window")}
        </div>
    }
}

#[component]
fn Row(label_key: &'static str, children: Children) -> impl IntoView {
    let lang = use_lang();
    view! {
        <div class="flex flex-col gap-1 border-b border-zinc-800/40 pb-2 last:border-b-0 last:pb-0 sm:flex-row sm:items-center sm:justify-between sm:gap-4">
            <dt class="text-xs uppercase tracking-wider text-zinc-500">
                {move || t(lang.get(), label_key)}
            </dt>
            <dd class="text-right">{children()}</dd>
        </div>
    }
}

fn format_sentri(sentri: u64) -> String {
    let whole = sentri / 100_000_000;
    let frac = (sentri % 100_000_000) / 10_000;
    format!("{whole}.{frac:04}")
}
