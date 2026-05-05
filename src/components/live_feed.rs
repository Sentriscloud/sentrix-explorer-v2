//! LiveBlockFeed — pure consumer of `BlockFeedState` from context.
//!
//! All gRPC subscription / poll-fallback logic lives in
//! `crate::state::feed` so other widgets (StatsPanel, future block
//! detail) read from the same signal without spinning up duplicate
//! streams.

use leptos::prelude::*;

use crate::components::skeleton::SkeletonRow;
use crate::i18n::{t, use_lang};
use crate::state::feed::{BlockFeedState, BlockRow};

#[component]
pub fn LiveBlockFeed() -> impl IntoView {
    let feed = use_context::<BlockFeedState>().expect("BlockFeedState context");
    let lang = use_lang();

    view! {
        <section class="glass-card rounded-2xl p-6">
            <header class="mb-5 flex items-center justify-between">
                <div>
                    <div class="eyebrow text-zinc-500">"Network · Live"</div>
                    <h2 class="mt-1 font-serif text-2xl font-bold tracking-tight text-zinc-100">
                        {move || t(lang.get(), "feed.latest_blocks")}
                    </h2>
                </div>
                <span class="inline-flex items-center gap-2 rounded-full border border-emerald-500/30 bg-emerald-500/10 px-2.5 py-1 text-[11px] font-medium tracking-wide text-emerald-500">
                    <span class="relative flex h-1.5 w-1.5">
                        <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-500 opacity-70"></span>
                        <span class="relative inline-flex h-1.5 w-1.5 rounded-full bg-emerald-500"></span>
                    </span>
                    {move || feed.status.get()}
                </span>
            </header>

            <div class="space-y-3">
                <Show
                    when=move || feed.blocks.with(|b| !b.is_empty())
                    fallback=|| view! { <EmptyState /> }
                >
                    <For
                        each=move || feed.blocks.get()
                        key=|row| row.hash_hex.clone()
                        children=|row: BlockRow| {
                            view! { <BlockTile row /> }
                        }
                    />
                </Show>
            </div>
        </section>
    }
}

#[component]
fn BlockTile(row: BlockRow) -> impl IntoView {
    // 4-4 hash truncation with 0x prefix — `0x3a93…874f` reads as
    // "address-shape" hex everywhere, no Solana-style identicon noise.
    let hash_short = if row.hash_hex.len() >= 8 {
        format!(
            "0x{}…{}",
            &row.hash_hex[..4],
            &row.hash_hex[row.hash_hex.len() - 4..]
        )
    } else {
        format!("0x{}", row.hash_hex)
    };
    let height_short_hex = format!("{:x}", row.height);
    let timestamp = row.timestamp;

    view! {
        <a
            href=format!("/block/{}", row.height)
            class="group flex items-center justify-between rounded-xl border border-zinc-800/40 bg-zinc-900/30 p-4 transition-colors hover:border-emerald-500/30 hover:bg-zinc-900/50"
        >
            <div class="flex items-center gap-4">
                <div class="flex h-10 w-12 items-center justify-center rounded-md border border-emerald-500/20 bg-emerald-500/5 font-mono text-[10px] font-semibold tabular-nums text-emerald-500/80">
                    {height_short_hex}
                </div>
                <div>
                    <div class="font-mono text-base font-bold tabular-nums text-zinc-100 group-hover:text-emerald-500">
                        "#" {row.height}
                    </div>
                    <div class="hex text-[11px] text-zinc-500">{hash_short}</div>
                </div>
            </div>
            <div class="text-right">
                <div class="font-mono text-sm font-semibold tabular-nums text-zinc-300">
                    {row.tx_count} " Txs"
                </div>
                <div class="font-mono text-[10px] tabular-nums text-zinc-600">
                    {move || format_relative(timestamp)}
                </div>
            </div>
        </a>
    }
}

/// Unix-seconds → "2s ago" / "5 min ago" / "1 hour ago". Bahasa-first
/// would read "2 detik lalu" but the block-feed cards stay in English
/// shorthand for table density. Bahasa relative form lives in
/// `screens/block_detail::format_relative_ts` for the detail page.
fn format_relative(ts: u64) -> String {
    #[cfg(target_arch = "wasm32")]
    let now = (js_sys::Date::now() / 1000.0) as u64;
    #[cfg(not(target_arch = "wasm32"))]
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(ts);

    let delta = now.saturating_sub(ts);
    if delta < 5 {
        "just now".into()
    } else if delta < 60 {
        format!("{delta}s ago")
    } else if delta < 3600 {
        format!("{} min ago", delta / 60)
    } else if delta < 86400 {
        format!("{} hr ago", delta / 3600)
    } else {
        format!("{} d ago", delta / 86400)
    }
}

#[component]
fn EmptyState() -> impl IntoView {
    let lang = use_lang();
    view! {
        <div class="space-y-3">
            <SkeletonRow />
            <SkeletonRow />
            <SkeletonRow />
            <div class="connecting-pulse pt-2 text-center text-xs text-zinc-500">
                {move || t(lang.get(), "feed.awaiting")}
            </div>
        </div>
    }
}
