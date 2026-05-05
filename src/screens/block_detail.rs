//! `/block/:height` detail screen.
//!
//! Body is fetched via a `LocalResource` against the native gRPC
//! `GetBlock(height)`. Loading state shows skeletons; errors fall
//! through to a `NodeReconnecting` panel rather than a blank slot.

use leptos::either::Either;
use leptos::prelude::*;
use leptos_meta::{Meta, Title};
use leptos_router::hooks::use_params_map;

use crate::api::{NativeError, NativeProvider, SentrixNativeProvider};
use crate::components::copy_cli::CopyCli;
use crate::components::error_boundary::NodeReconnecting;
use crate::components::identicon::Identicon;
use crate::components::skeleton::Skeleton;
use crate::grpc::pb::Transaction;
use crate::i18n::{t, use_lang};
use crate::labels::AddressLabel;

#[component]
pub fn BlockDetailScreen() -> impl IntoView {
    let params = use_params_map();
    let height_str = params.read().get("height").unwrap_or_default();
    let height: Option<u64> = height_str.parse().ok();

    let title = format!("Sentrix Block #{height_str} — Obsidian Engine");
    let description =
        format!("Block #{height_str} on the Sentrix L1 — proposer, transactions, finality.");
    let cli = format!("srx-cli get block --height {height_str}");
    let url = format!("https://scan.sentriscloud.com/block/{height_str}");

    let block_resource = LocalResource::new(move || {
        let h = height;
        async move {
            let provider = SentrixNativeProvider::default_for_network();
            match h {
                Some(h) => provider.get_block_by_height(h).await,
                None => Err(NativeError::NotFound),
            }
        }
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
            <BlockHeading height_str=height_str.clone() />

            <Suspense fallback=|| view! { <BlockBodySkeleton /> }>
                {move || Suspend::new(async move {
                    match block_resource.await {
                        Ok(block) => Either::Left(view! { <BlockBody block /> }),
                        Err(_) => Either::Right(view! { <NodeReconnecting /> }),
                    }
                })}
            </Suspense>

            <CliFooter command=cli />
        </section>
    }
}

#[component]
fn BlockHeading(height_str: String) -> impl IntoView {
    let lang = use_lang();
    view! {
        <header>
            <div class="eyebrow text-zinc-500">
                {move || t(lang.get(), "detail.block")}
            </div>
            <h1 class="font-serif text-5xl font-bold tabular-nums tracking-tight text-emerald-500">
                "#" {height_str}
            </h1>
        </header>
    }
}

#[component]
fn CliFooter(command: String) -> impl IntoView {
    let lang = use_lang();
    view! {
        <footer class="border-t border-zinc-800/40 pt-4">
            <div class="mb-2 text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                {move || t(lang.get(), "detail.cli_label")}
            </div>
            <CopyCli command=command />
        </footer>
    }
}

#[component]
fn BlockBody(block: crate::grpc::pb::Block) -> impl IntoView {
    let hash_hex = block
        .hash
        .as_ref()
        .map(|h| hex::encode(&h.value))
        .unwrap_or_else(|| "—".into());
    let parent_hex = block
        .parent_hash
        .as_ref()
        .map(|h| hex::encode(&h.value))
        .unwrap_or_else(|| "—".into());
    let state_root_hex = block
        .state_root
        .as_ref()
        .map(|h| hex::encode(&h.value))
        .unwrap_or_else(|| "—".into());
    let proposer_hex = block
        .proposer
        .as_ref()
        .map(|a| hex::encode(&a.value))
        .unwrap_or_else(|| "—".into());
    let proposer_seed = proposer_hex.clone();
    let tx_count = block.transactions.len();
    let timestamp = block.timestamp;
    let txs = block.transactions.clone();

    view! {
        <dl class="space-y-3 text-sm">
            <Field label_key="detail.hash">
                <span class="hex break-all text-xs">"0x" {hash_hex}</span>
            </Field>
            <Field label_key="detail.parent">
                <span class="hex break-all text-xs">"0x" {parent_hex}</span>
            </Field>
            <Field label_key="detail.state_root">
                <span class="hex break-all text-xs">"0x" {state_root_hex}</span>
            </Field>
            <Field label_key="detail.proposer">
                <div class="flex items-center gap-2">
                    <div class="identicon-frame h-5 w-5 rounded">
                        <Identicon address_hex=proposer_seed size=20 />
                    </div>
                    <span class="hex break-all text-xs">"0x" {proposer_hex.clone()}</span>
                    <AddressLabel addr=format!("0x{proposer_hex}") />
                </div>
            </Field>
            <Field label_key="detail.transactions">
                <span class="font-mono text-zinc-200">{tx_count}</span>
            </Field>
            <Field label_key="detail.timestamp">
                <div class="flex flex-col items-end gap-0.5 text-right">
                    <span class="font-mono text-xs text-zinc-300">
                        {format_unix_ts(timestamp)}
                    </span>
                    <span class="text-[10px] uppercase tracking-[0.16em] text-zinc-500">
                        {format_relative_ts(timestamp)}
                    </span>
                </div>
            </Field>
        </dl>

        <TxList txs />
    }
}

#[component]
fn TxList(txs: Vec<Transaction>) -> impl IntoView {
    let lang = use_lang();
    let count = txs.len();
    if count == 0 {
        return view! {
            <div class="mt-4 rounded-xl border border-dashed border-zinc-800 bg-zinc-900/30 p-4 text-center text-xs text-zinc-500">
                {move || t(lang.get(), "detail.no_txs")}
            </div>
        }
        .into_any();
    }

    view! {
        <section class="mt-4 space-y-2">
            <div class="flex items-center justify-between">
                <h3 class="text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                    {move || t(lang.get(), "detail.transactions")}
                </h3>
                <span class="status-pill">{count} " total"</span>
            </div>
            <div class="space-y-2">
                {txs.into_iter().enumerate().map(|(i, t)| view! {
                    <TxRow tx=t index=i />
                }).collect_view()}
            </div>
        </section>
    }
    .into_any()
}

#[component]
fn TxRow(tx: Transaction, index: usize) -> impl IntoView {
    let txid_hex = tx
        .txid
        .as_ref()
        .map(|h| hex::encode(&h.value))
        .unwrap_or_else(|| format!("idx-{index}"));
    let from_hex = tx
        .from_address
        .as_ref()
        .map(|a| hex::encode(&a.value))
        .unwrap_or_default();
    let to_hex = tx
        .to_address
        .as_ref()
        .map(|a| hex::encode(&a.value))
        .unwrap_or_default();
    let amount = tx.amount.as_ref().map(|a| a.sentri).unwrap_or(0);
    let kind = match tx.tx_type {
        0 => "transfer",
        1 => "contract",
        2 => "staking-op",
        _ => "other",
    };
    let from_short = if from_hex.len() >= 10 {
        format!("0x{}…", &from_hex[..6])
    } else {
        format!("0x{from_hex}")
    };
    let to_short = if to_hex.len() >= 10 {
        format!("0x{}…", &to_hex[..6])
    } else {
        format!("0x{to_hex}")
    };
    let txid_short = if txid_hex.len() >= 12 {
        format!("{}…", &txid_hex[..10])
    } else {
        txid_hex.clone()
    };
    let from_link = format!("/address/0x{from_hex}");
    let to_link = format!("/address/0x{to_hex}");
    let tx_link = format!("/tx/0x{txid_hex}");
    let from_full = format!("0x{from_hex}");
    let to_full = format!("0x{to_hex}");

    view! {
        <div class="flex items-center justify-between rounded-xl border border-zinc-800/30 bg-zinc-900/40 p-3 text-xs">
            <div class="flex items-center gap-3">
                <div class="identicon-frame h-7 w-7 rounded-md ring-1 ring-zinc-800/80">
                    <Identicon address_hex=from_hex.clone() size=28 />
                </div>
                <div class="flex flex-col gap-0.5">
                    <a href=tx_link class="hex text-zinc-300 hover:text-amber-300">
                        {txid_short}
                    </a>
                    <div class="flex flex-wrap items-center gap-1 text-[10px] text-zinc-500">
                        <a href=from_link class="font-mono hover:text-amber-300">
                            {from_short}
                        </a>
                        <AddressLabel addr=from_full />
                        <span class="text-zinc-700">"→"</span>
                        <a href=to_link class="font-mono hover:text-amber-300">
                            {to_short}
                        </a>
                        <AddressLabel addr=to_full />
                    </div>
                </div>
            </div>
            <div class="text-right">
                <div class="font-mono text-zinc-200">{format_sentri(amount)} " SRX"</div>
                <div class="text-[10px] uppercase tracking-wider text-zinc-500">{kind}</div>
            </div>
        </div>
    }
}

/// Sentri (10⁻⁸ SRX) → human SRX with 4 decimal places.
fn format_sentri(sentri: u64) -> String {
    let whole = sentri / 100_000_000;
    let frac = (sentri % 100_000_000) / 10_000;
    format!("{whole}.{frac:04}")
}

/// Unix-seconds → "YYYY-MM-DD HH:MM:SS UTC". Hand-rolled (no chrono on
/// the wasm side) — rough Gregorian conversion via days-since-epoch
/// and a leap-year accumulator. Good enough for block timestamps;
/// don't reach for it for anything calendar-correctness-critical.
fn format_unix_ts(ts: u64) -> String {
    let secs = ts % 60;
    let total_minutes = ts / 60;
    let mins = total_minutes % 60;
    let total_hours = total_minutes / 60;
    let hours = total_hours % 24;
    let mut days = total_hours / 24;

    let mut year: u64 = 1970;
    loop {
        let leap =
            (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
        let year_days = if leap { 366 } else { 365 };
        if days < year_days {
            break;
        }
        days -= year_days;
        year += 1;
    }

    let leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    let month_lengths = [
        31u64,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month: usize = 0;
    while month < 12 && days >= month_lengths[month] {
        days -= month_lengths[month];
        month += 1;
    }
    let day = days + 1;
    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02} UTC",
        year,
        month + 1,
        day,
        hours,
        mins,
        secs
    )
}

/// Relative-time form ("3 menit lalu", "2 jam lalu"). Bahasa first;
/// English short form fallback if we ever localise.
fn format_relative_ts(ts: u64) -> String {
    #[cfg(target_arch = "wasm32")]
    let now = (js_sys_now_ms() / 1000.0) as u64;
    #[cfg(not(target_arch = "wasm32"))]
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(ts);

    let delta = now.saturating_sub(ts);
    if delta < 5 {
        "baru saja".into()
    } else if delta < 60 {
        format!("{delta} detik lalu")
    } else if delta < 3600 {
        format!("{} menit lalu", delta / 60)
    } else if delta < 86400 {
        format!("{} jam lalu", delta / 3600)
    } else {
        format!("{} hari lalu", delta / 86400)
    }
}

#[cfg(target_arch = "wasm32")]
fn js_sys_now_ms() -> f64 {
    js_sys::Date::now()
}

#[component]
fn Field(label_key: &'static str, children: Children) -> impl IntoView {
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

#[component]
fn BlockBodySkeleton() -> impl IntoView {
    view! {
        <div class="space-y-3">
            <Skeleton class="h-4 w-full" />
            <Skeleton class="h-4 w-full" />
            <Skeleton class="h-4 w-3/4" />
            <Skeleton class="h-4 w-2/3" />
            <Skeleton class="h-4 w-1/2" />
        </div>
    }
}
