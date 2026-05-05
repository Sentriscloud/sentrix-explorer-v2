//! LiveBlockFeed — pure consumer of `BlockFeedState` from context.
//!
//! All gRPC subscription / poll-fallback logic lives in
//! `crate::state::feed` so other widgets (StatsPanel, future block
//! detail) read from the same signal without spinning up duplicate
//! streams.

use leptos::prelude::*;

use crate::components::identicon::Identicon;
use crate::components::skeleton::SkeletonRow;
use crate::i18n::{t, use_lang};
use crate::state::feed::{BlockFeedState, BlockRow};

#[component]
pub fn LiveBlockFeed() -> impl IntoView {
    let feed = use_context::<BlockFeedState>().expect("BlockFeedState context");
    let lang = use_lang();

    view! {
        <section class="glass-card rounded-2xl p-6">
            <header class="mb-6 flex items-center justify-between">
                <h2 class="text-xl font-bold italic tracking-tighter text-zinc-100">
                    {move || t(lang.get(), "feed.latest_blocks")}
                </h2>
                <span class="flex items-center gap-2 rounded-md border border-amber-500/20 bg-amber-500/10 px-2 py-1 text-xs text-amber-400">
                    <span class="relative flex h-2 w-2">
                        <span class="absolute inline-flex h-full w-full animate-ping rounded-full bg-amber-400 opacity-75"></span>
                        <span class="relative inline-flex h-2 w-2 rounded-full bg-amber-500"></span>
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
    let hash_preview = row.hash_hex[..12].to_string();
    let avatar_seed = row.hash_hex.clone();

    view! {
        <div class="group flex items-center justify-between rounded-xl border border-zinc-800/30 bg-zinc-900/40 p-4 transition-all hover:border-zinc-700">
            <div class="flex items-center gap-4">
                <div class="identicon-frame h-10 w-10 rounded-lg ring-1 ring-zinc-800/80 transition-shadow group-hover:ring-amber-500/40">
                    <Identicon address_hex=avatar_seed size=40 />
                </div>
                <div>
                    <div class="font-mono font-bold text-zinc-100">"#" {row.height}</div>
                    <div class="hex w-32 truncate text-xs text-zinc-500">
                        {hash_preview} "…"
                    </div>
                </div>
            </div>
            <div class="text-right">
                <div class="text-sm font-bold text-zinc-300">{row.tx_count} " Txs"</div>
                <div class="text-[10px] text-zinc-600">{row.timestamp}</div>
            </div>
        </div>
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
