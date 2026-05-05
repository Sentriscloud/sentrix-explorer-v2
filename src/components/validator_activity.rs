//! Validator activity — proposer distribution from the feed window.
//!
//! Pure derivation: counts how many of the last N blocks each unique
//! proposer signed. No new RPC, no geo data; surfaces the same
//! diagnostic Solscan-style explorers do for "active validators in
//! the last X blocks".
//!
//! When the chain ships a `ValidatorSetChange` event filter (proto
//! v0.3) we'll switch to that for the active set; for now the feed
//! window is the source of truth.
//!
//! Per-proposer geographic plotting (the validator map) is deferred —
//! needs a coordinate registry that doesn't exist yet.

use leptos::prelude::*;
use std::collections::HashMap;

use crate::components::identicon::Identicon;
use crate::state::feed::BlockFeedState;

#[component]
pub fn ValidatorActivity() -> impl IntoView {
    let feed = use_context::<BlockFeedState>().expect("BlockFeedState context");

    // Real proposer counts from the live feed window. Blocks
    // without a proposer field (older proto, dev fixtures) bucket
    // into "—" rather than skewing the display.
    let stats = Memo::new(move |_| {
        feed.blocks.with(|blocks| {
            let mut counts: HashMap<String, usize> = HashMap::new();
            for b in blocks.iter() {
                let key = if b.proposer_hex.is_empty() {
                    "—".to_string()
                } else {
                    b.proposer_hex.clone()
                };
                *counts.entry(key).or_insert(0) += 1;
            }
            let mut v: Vec<(String, usize)> = counts.into_iter().collect();
            v.sort_by_key(|entry| std::cmp::Reverse(entry.1));
            v.truncate(8);
            (v, blocks.len())
        })
    });

    view! {
        <section class="glass-card rounded-2xl p-6">
            <header class="mb-4 flex items-center justify-between">
                <div>
                    <div class="eyebrow text-zinc-500">"BFT · Producers"</div>
                    <h3 class="mt-1 font-mono text-base font-bold tracking-tight text-zinc-100">
                        "Validator activity"
                    </h3>
                </div>
                <span class="status-pill">
                    {move || format!("{} blocks", stats.with(|s| s.1))}
                </span>
            </header>

            <Show
                when=move || stats.with(|s| !s.0.is_empty())
                fallback=|| view! { <ActivityEmpty /> }
            >
                <div class="space-y-2">
                    {move || {
                        let max = stats.with(|s| s.0.first().map(|(_, n)| *n).unwrap_or(1));
                        stats.with(|s| {
                            s.0.iter()
                                .map(|(seed, count)| {
                                    let pct = (*count * 100).checked_div(max).unwrap_or(0);
                                    view! {
                                        <ActivityRow
                                            seed=seed.clone()
                                            count=*count
                                            pct=pct as u32
                                        />
                                    }
                                })
                                .collect_view()
                        })
                    }}
                </div>
            </Show>
        </section>
    }
}

#[component]
fn ActivityRow(seed: String, count: usize, pct: u32) -> impl IntoView {
    let display = if seed == "—" {
        "unknown".to_string()
    } else if seed.len() >= 12 {
        format!("0x{}…{}", &seed[..6], &seed[seed.len() - 4..])
    } else {
        format!("0x{seed}")
    };
    let link = if seed != "—" {
        Some(format!("/address/0x{seed}"))
    } else {
        None
    };

    view! {
        <div class="flex items-center gap-3">
            <div class="identicon-frame h-7 w-7 rounded-md ring-1 ring-zinc-800/80">
                <Identicon address_hex=seed size=28 />
            </div>
            <div class="flex-1">
                <div class="flex items-center justify-between text-[10px]">
                    {match link {
                        Some(l) => view! {
                            <a href=l class="hex text-zinc-400 hover:text-amber-300">
                                {display}
                            </a>
                        }.into_any(),
                        None => view! {
                            <span class="hex text-zinc-500">{display}</span>
                        }.into_any(),
                    }}
                    <span class="font-mono text-zinc-300">{count} " blocks"</span>
                </div>
                <div class="mt-1 h-1.5 overflow-hidden rounded-full bg-zinc-800">
                    <div
                        class="h-full bg-gradient-to-r from-amber-400 to-amber-600 transition-all duration-700"
                        style=format!("width: {pct}%")
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
fn ActivityEmpty() -> impl IntoView {
    view! {
        <div class="rounded-xl border border-dashed border-zinc-800 bg-zinc-900/30 p-6 text-center text-xs text-zinc-500">
            "No blocks observed yet."
        </div>
    }
}
