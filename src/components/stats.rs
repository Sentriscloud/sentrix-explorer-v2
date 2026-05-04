//! Network Health header — TPS, finality, supply progress.
//!
//! Reads the shared `BlockFeedState` from context (no second gRPC
//! subscription) and derives:
//!   - TPS as Σtx_count / window seconds (10-block rolling window)
//!   - Avg finality from inter-block timestamp deltas
//!
//! Supply numbers come from `SUPPLY` constants — total is operator-
//! authoritative (315 M SRX). Circulating stays at "TBD" until the
//! tokenomics integration lands; we deliberately don't fabricate a
//! number here.

use leptos::prelude::*;

use crate::components::skeleton::Skeleton;
use crate::components::sparkline::Sparkline;
use crate::state::feed::{BlockFeedState, BlockRow};
use crate::state::gas::GasPriceState;

const TOTAL_SUPPLY_SRX: u64 = 315_000_000;

#[component]
pub fn StatsPanel() -> impl IntoView {
    let feed = use_context::<BlockFeedState>().expect("BlockFeedState context");
    let gas = use_context::<GasPriceState>().expect("GasPriceState context");

    // Derived signals: cheap closures, recomputed only when source ticks.
    let tps = Memo::new(move |_| feed.blocks.with(|b| compute_tps(b)));
    let finality_ms = Memo::new(move |_| feed.blocks.with(|b| compute_finality_ms(b)));
    let height = Memo::new(move |_| feed.blocks.with(|b| b.first().map(|r| r.height)));

    view! {
        <section class="glass-card grid grid-cols-2 gap-4 rounded-2xl p-6 md:grid-cols-5">
            <Stat
                label="Tip Height"
                value=Signal::derive(move || {
                    height.get().map(|h| format!("#{h}")).unwrap_or_else(|| "—".into())
                })
            />
            <Stat
                label="TPS · 10b window"
                value=Signal::derive(move || match tps.get() {
                    Some(t) => format!("{t:.2}"),
                    None => "—".into(),
                })
            />
            <Stat
                label="Avg Finality"
                value=Signal::derive(move || match finality_ms.get() {
                    Some(ms) => format!("{:.2}s", ms as f64 / 1000.0),
                    None => "—".into(),
                })
            />
            <Stat
                label="Gas · gwei"
                value=Signal::derive(move || match gas.gwei.get() {
                    Some(g) if g >= 1.0 => format!("{g:.2}"),
                    Some(g) => format!("{g:.4}"),
                    None => "—".into(),
                })
            />
            <Stat
                label="Total Supply"
                value=Signal::derive(move || format!("{} SRX", fmt_int(TOTAL_SUPPLY_SRX)))
            />

            <div class="col-span-2 md:col-span-5">
                <SupplyBar />
            </div>

            <SparkCard label="TPS · per block" col_span="md:col-span-3">
                {move || feed.blocks.with(|b| {
                    // Oldest first → newest last for the sparkline.
                    let mut pts: Vec<f64> =
                        b.iter().rev().map(|r| r.tx_count as f64).collect();
                    if pts.len() > 25 { pts = pts.split_off(pts.len() - 25); }
                    view! { <Sparkline points=pts stroke="#DBC17F" width=320 height=40 /> }
                })}
            </SparkCard>

            <SparkCard label="Gas history · gwei" col_span="md:col-span-2">
                {move || {
                    let pts = gas.history.get();
                    view! { <Sparkline points=pts stroke="#8A5A11" width=200 height=40 /> }
                }}
            </SparkCard>
        </section>
    }
}

#[component]
fn SparkCard(label: &'static str, col_span: &'static str, children: Children) -> impl IntoView {
    let class =
        format!("col-span-2 rounded-xl border border-zinc-800/40 bg-zinc-900/40 p-4 {col_span}");
    view! {
        <div class=class>
            <div class="mb-2 text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                {label}
            </div>
            {children()}
        </div>
    }
}

#[component]
fn Stat(label: &'static str, value: Signal<String>) -> impl IntoView {
    // While we're still in the no-data state (value == "—") swap the
    // text in for a small skeleton bar so the panel doesn't read as
    // "broken" before the first block lands.
    view! {
        <div class="rounded-xl border border-zinc-800/40 bg-zinc-900/40 p-4">
            <div class="text-[10px] uppercase tracking-[0.18em] text-zinc-500">
                {label}
            </div>
            <div class="mt-1">
                <Show
                    when=move || value.get() != "—"
                    fallback=|| view! { <Skeleton class="h-6 w-20" /> }
                >
                    <span class="font-mono text-lg font-bold text-zinc-100">
                        {move || value.get()}
                    </span>
                </Show>
            </div>
        </div>
    }
}

#[component]
fn SupplyBar() -> impl IntoView {
    // Circulating supply stays as a TBD slot until tokenomics data is
    // wired in. Keeping a pending state here is honest about the
    // current state rather than fabricating a number.
    view! {
        <div class="rounded-xl border border-zinc-800/40 bg-zinc-900/40 p-4">
            <div class="flex items-center justify-between text-xs">
                <span class="text-zinc-400">"Circulating · Locked"</span>
                <span class="font-mono text-zinc-500">"TBD · pending tokenomics integration"</span>
            </div>
            <div class="mt-3 h-2 overflow-hidden rounded-full bg-zinc-800">
                // Placeholder track. Replace `style:width` with a real
                // ratio once `circulating_supply()` returns a value.
                <div class="h-full w-0 rounded-full bg-gradient-to-r from-amber-400 to-amber-600 transition-all duration-700" />
            </div>
        </div>
    }
}

/// Σ(tx_count) / window-seconds. Returns None until we have ≥ 2 blocks
/// (need a span to divide over). 10-block window matches roughly 20 s
/// at the chain's 2 s block target — long enough to dampen single-
/// block spikes, short enough to reflect "current" load.
fn compute_tps(blocks: &[BlockRow]) -> Option<f64> {
    if blocks.len() < 2 {
        return None;
    }
    let window: Vec<&BlockRow> = blocks.iter().take(10).collect();
    let newest = window.first()?.timestamp;
    let oldest = window.last()?.timestamp;
    if newest <= oldest {
        return None;
    }
    let span_secs = (newest - oldest) as f64;
    let tx_total: usize = window.iter().map(|r| r.tx_count).sum();
    Some(tx_total as f64 / span_secs)
}

/// Mean inter-block delta in ms over the visible window. Sentrix block
/// timestamps are seconds; multiply to ms for the display.
fn compute_finality_ms(blocks: &[BlockRow]) -> Option<u64> {
    if blocks.len() < 2 {
        return None;
    }
    let window: Vec<&BlockRow> = blocks.iter().take(10).collect();
    let mut deltas = Vec::with_capacity(window.len());
    for pair in window.windows(2) {
        if pair[0].timestamp >= pair[1].timestamp {
            deltas.push(pair[0].timestamp - pair[1].timestamp);
        }
    }
    if deltas.is_empty() {
        return None;
    }
    let mean = deltas.iter().sum::<u64>() / deltas.len() as u64;
    Some(mean * 1000)
}

fn fmt_int(n: u64) -> String {
    // Thousands-separator formatter — std doesn't ship one and pulling
    // `num-format` for 8 lines isn't worth the WASM bytes.
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}
