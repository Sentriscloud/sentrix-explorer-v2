//! StatsDashboard — 4-card live network stats with Suspense + error
//! boundary. Etherscan-style data, Linear-style polish.
//!
//! Data shape lives in `ChainStats`. The fetcher is mocked behind a
//! 500 ms delay; flip the TODO blocks to real RPC when ready.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use crate::state::network::{use_network, Network};

// ─────────────────────────────────────────────────────────────────
// Data layer
// ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChainStats {
    pub block_height: u64,
    pub avg_block_time_ms: u32,
    pub active_validators: u32,
    pub total_validators: u32,
    pub total_transactions: u64,
    pub network: Network,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FetchError {
    Rpc(String),
}

impl std::fmt::Display for FetchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Rpc(s) => write!(f, "rpc: {s}"),
        }
    }
}

impl std::error::Error for FetchError {}

/// Real chain-stats fetcher (wasm path) with SSR mock fallback.
///
/// ## Wired today
///   - `block_height`      → `eth_blockNumber` against `network.rpc_url()`
///   - `avg_block_time_ms` → mean delta from latest + (latest - 99)
///     timestamps via two `eth_getBlockByNumber` calls
///
/// ## Still mock (require new endpoints)
///   - `active_validators` / `total_validators` → needs native gRPC
///     `Sentrix.GetValidatorSet` (proto v0.3) or a curated registry
///   - `total_transactions` → no direct RPC; needs indexer aggregation
///
/// The two mock fields keep stable values so the UI doesn't pulse;
/// flip them as endpoints land.
#[cfg(target_arch = "wasm32")]
async fn fetch_chain_stats(network: Network) -> Result<ChainStats, FetchError> {
    use crate::api::evm::{EvmProvider, HttpEvmProvider};

    let provider = HttpEvmProvider::new(network.rpc_url());

    // Sequential rather than `futures::join!` — adding the macro pulls
    // a procmacro re-export and the three hits add maybe 200 ms over
    // a parallel version. Worth revisiting if/when the real
    // validator + indexer endpoints push us to 5+ hits.
    let block_height = provider
        .block_number()
        .await
        .map_err(|e| FetchError::Rpc(format!("block_number: {e:?}")))?;

    let avg_block_time_ms = compute_avg_block_time_ms(&provider, block_height).await;

    Ok(ChainStats {
        block_height,
        avg_block_time_ms,
        // TODO: replace with `Sentrix.GetValidatorSet` once chain
        // proto v0.3 lands.
        active_validators: 21,
        total_validators: 25,
        // TODO: cumulative tx count — needs an indexer aggregation
        // endpoint; chain RPC doesn't expose a direct query.
        total_transactions: match network {
            Network::Mainnet => 12_847_392,
            Network::Testnet => 2_103_847,
        },
        network,
    })
}

/// SSR-side fetcher — server pre-render returns mock so we don't fan
/// out external RPC from the prerender path. The hydrated bundle
/// runs the real `fetch_chain_stats` above on the client.
#[cfg(not(target_arch = "wasm32"))]
async fn fetch_chain_stats(network: Network) -> Result<ChainStats, FetchError> {
    sleep_500ms().await;
    Ok(ChainStats {
        block_height: 0,
        avg_block_time_ms: 0,
        active_validators: 21,
        total_validators: 25,
        total_transactions: 0,
        network,
    })
}

/// Two-point average — latest + (latest - 99) timestamps, divide by
/// the gap. Defaults to 1200 ms (chain target) on any RPC failure
/// or when there aren't enough blocks yet.
#[cfg(target_arch = "wasm32")]
async fn compute_avg_block_time_ms<P: crate::api::evm::EvmProvider>(provider: &P, tip: u64) -> u32 {
    const FALLBACK_MS: u32 = 1_200;
    if tip < 100 {
        return FALLBACK_MS;
    }
    let older = tip - 99;
    let (Ok(t_new), Ok(t_old)) = (
        provider.get_block_by_number(tip).await,
        provider.get_block_by_number(older).await,
    ) else {
        return FALLBACK_MS;
    };
    if t_new.timestamp <= t_old.timestamp {
        return FALLBACK_MS;
    }
    // (Δseconds * 1000) / 99 → millis per block, mean over the window.
    let span_ms = (t_new.timestamp - t_old.timestamp).saturating_mul(1_000);
    u32::try_from(span_ms / 99).unwrap_or(FALLBACK_MS)
}

#[cfg(not(target_arch = "wasm32"))]
async fn sleep_500ms() {
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
}

// ─────────────────────────────────────────────────────────────────
// Component
// ─────────────────────────────────────────────────────────────────

#[component]
pub fn StatsDashboard() -> impl IntoView {
    let network = use_network();

    // `LocalResource` rather than `Resource`: the wasm sleep uses
    // `JsFuture` which isn't `Send`, so the SSR-capable
    // `Resource::new` rejects the fetcher. SSR pre-renders the
    // skeleton; the hydrated bundle then runs the fetch and swaps
    // in the numbers — same UX pattern as Etherscan/Solscan first
    // paint. Switch to `Resource::new` once the real RPC fetcher
    // is split into a Send-friendly server-side branch.
    let stats = LocalResource::new(move || {
        let net = network.get();
        async move { fetch_chain_stats(net).await }
    });

    view! {
        <section class="space-y-4" aria-label="Network statistics">
            <NetworkBadge />

            <Suspense fallback=|| view! { <SkeletonGrid /> }>
                {move || Suspend::new(async move {
                    match stats.await {
                        Ok(s) => view! { <StatsGrid stats=s /> }.into_any(),
                        Err(e) => view! {
                            <ErrorState
                                message=e.to_string()
                                on_retry=move || stats.refetch()
                            />
                        }.into_any(),
                    }
                })}
            </Suspense>
        </section>
    }
}

#[component]
fn NetworkBadge() -> impl IntoView {
    let network = use_network();
    view! {
        <div class="flex items-center gap-2" role="status" aria-live="polite">
            <span class=move || {
                let base = "inline-flex items-center gap-1.5 rounded-full border px-2.5 py-0.5 text-xs font-medium";
                match network.get() {
                    Network::Mainnet => format!("{base} border-emerald-500/30 bg-emerald-500/10 text-emerald-300"),
                    Network::Testnet => format!("{base} border-amber-500/30 bg-amber-500/10 text-amber-300"),
                }
            }>
                <span class=move || match network.get() {
                    Network::Mainnet => "h-1.5 w-1.5 rounded-full bg-emerald-500",
                    Network::Testnet => "h-1.5 w-1.5 rounded-full bg-amber-500",
                } />
                {move || network.get().label()}
            </span>
            <span class="hex text-[10px] text-slate-500">
                "chain id · " {move || network.get().chain_id().to_string()}
            </span>
        </div>
    }
}

#[component]
fn StatsGrid(stats: ChainStats) -> impl IntoView {
    let block_time = format!("{:.1}s", f64::from(stats.avg_block_time_ms) / 1000.0);
    let validators = format!("{} / {}", stats.active_validators, stats.total_validators);

    view! {
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4">
            <StatCard
                label="Latest Block"
                value=format_int(stats.block_height)
                trend=Trend::Up
                icon=Icon::Block
            />
            <StatCard
                label="Avg Block Time"
                value=block_time
                trend=Trend::None
                icon=Icon::Clock
            />
            <StatCard
                label="Active Validators"
                value=validators
                trend=Trend::None
                icon=Icon::Validators
            />
            <StatCard
                label="Total Transactions"
                value=format_int(stats.total_transactions)
                trend=Trend::Up
                icon=Icon::Transactions
            />
        </div>
    }
}

// `Down` only fires once the real fetcher tracks negative deltas
// (currently the mock only emits Up/None); allow until then.
#[allow(dead_code)]
#[derive(Clone, Copy)]
enum Trend {
    Up,
    Down,
    None,
}

#[derive(Clone, Copy)]
enum Icon {
    Block,
    Clock,
    Validators,
    Transactions,
}

#[component]
fn StatCard(label: &'static str, value: String, trend: Trend, icon: Icon) -> impl IntoView {
    view! {
        <article
            class="group rounded-xl border border-slate-800 bg-slate-900/50 p-5 backdrop-blur-sm transition-all hover:scale-[1.02] hover:border-slate-700"
            aria-label=label
        >
            <header class="flex items-center justify-between">
                <span class="text-xs font-medium uppercase tracking-wider text-slate-400">
                    {label}
                </span>
                <IconSvg icon />
            </header>
            <div class="mt-3 font-mono text-2xl font-bold tabular-nums text-slate-100">
                {value}
            </div>
            <TrendIndicator trend />
        </article>
    }
}

#[component]
fn IconSvg(icon: Icon) -> impl IntoView {
    // Lucide-shape paths inlined so we don't pull a JSX-style icon
    // crate. Keep `stroke="currentColor"` so Solar Mode + hover
    // tints work via the parent's text color.
    let path = match icon {
        Icon::Block => {
            "M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16zM3.27 6.96L12 12.01l8.73-5.05M12 22.08V12"
        }
        Icon::Clock => "M12 6v6l4 2M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0z",
        Icon::Validators => {
            "M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2M9 11a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM23 21v-2a4 4 0 0 0-3-3.87M16 3.13a4 4 0 0 1 0 7.75"
        }
        Icon::Transactions => "M16 3l4 4-4 4M20 7H4M8 21l-4-4 4-4M4 17h16",
    };
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            class="h-4 w-4 text-slate-500 transition-colors group-hover:text-emerald-400"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
            aria-hidden="true"
        >
            <path d=path />
        </svg>
    }
}

#[component]
fn TrendIndicator(trend: Trend) -> impl IntoView {
    // TODO: real "vs 1h ago" diff once we keep a rolling sample.
    // For now the direction is a hint; values are mock.
    let (cls, label) = match trend {
        Trend::Up => ("text-emerald-400", "↑ 0.0% vs 1h ago"),
        Trend::Down => ("text-rose-400", "↓ 0.0% vs 1h ago"),
        Trend::None => ("text-slate-500", "— stable"),
    };
    view! {
        <div class=format!("mt-1 text-xs {cls}")>{label}</div>
    }
}

#[component]
fn SkeletonGrid() -> impl IntoView {
    view! {
        <div
            class="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-4"
            role="status"
            aria-label="Loading statistics"
        >
            {(0..4).map(|_| view! { <SkeletonCard /> }).collect_view()}
        </div>
    }
}

#[component]
fn SkeletonCard() -> impl IntoView {
    view! {
        <div class="rounded-xl border border-slate-800 bg-slate-900/50 p-5">
            <div class="skeleton-shimmer h-3 w-24 rounded bg-slate-800" />
            <div class="skeleton-shimmer mt-4 h-7 w-32 rounded bg-slate-800" />
            <div class="skeleton-shimmer mt-3 h-3 w-16 rounded bg-slate-800" />
        </div>
    }
}

#[component]
fn ErrorState<F>(message: String, on_retry: F) -> impl IntoView
where
    F: Fn() + Copy + Send + Sync + 'static,
{
    view! {
        <div
            role="alert"
            class="rounded-xl border border-rose-500/30 bg-rose-500/5 p-5"
        >
            <div class="text-sm font-semibold text-rose-200">
                "Failed to load network statistics"
            </div>
            <div class="mt-1 font-mono text-xs text-rose-300/80">{message}</div>
            <button
                type="button"
                on:click=move |_| on_retry()
                class="mt-3 rounded-md border border-rose-500/40 bg-rose-500/10 px-3 py-1 text-xs font-medium text-rose-200 transition hover:bg-rose-500/20"
            >
                "Retry"
            </button>
        </div>
    }
}

/// Thousands-separator formatter — std doesn't ship one and pulling
/// `num-format` for 8 lines isn't worth the WASM bytes.
fn format_int(n: u64) -> String {
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
